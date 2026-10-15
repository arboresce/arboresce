//! Complete bounded source admission followed by immutable canonical selection.

use std::{
    error::Error,
    fmt,
    io::{ErrorKind, Result as OutputResult, Write},
};

use serde::{
    Serialize, Serializer,
    ser::{SerializeMap, SerializeSeq},
};
use sha2::{Digest, Sha256};

use crate::{CanonicalJsonNumber, JsonPointer, MAX_JSON_NUMBER_TOKEN_BYTES, Sha256Digest};

pub const MAX_JSON_SOURCE_BYTES: usize = 10_485_760;
pub const MAX_JSON_SOURCE_DEPTH: usize = 128;
pub const MAX_JSON_SOURCE_NODES: usize = 262_144;
pub const MAX_JSON_SOURCE_MEMBERS: usize = 131_072;
pub const MAX_JSON_OBJECT_MEMBERS: usize = 65_536;
pub const MAX_JSON_ARRAY_ELEMENTS: usize = 131_072;
pub const MAX_SELECTED_JSON_DEPTH: usize = 64;
pub const MAX_SELECTED_JSON_NODES: usize = 65_536;
pub const MAX_SELECTED_JSON_BYTES: usize = 10_485_760;

/// A completely checked source. Child ownership is flat, including on rejection.
pub struct SourceJson {
    nodes: Vec<Node>,
    root: usize,
}

/// Complete canonical bytes and their digest, never a partial selection.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct CanonicalJsonSelection {
    bytes: Vec<u8>,
    sha256: Sha256Digest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidSourceJson;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidJsonSelection;

struct Node {
    value: Value,
    depth: usize,
    count: usize,
    canonical_bytes: usize,
}
enum Value {
    Null,
    Bool(bool),
    Number(CanonicalJsonNumber),
    String(String),
    Array(Vec<usize>),
    Object(Vec<Member>),
}
struct Member {
    key: String,
    child: usize,
}

// These actual layouts are part of the selected resource derivation.
const _: () = assert!(size_of::<Node>() <= 128);
const _: () = assert!(size_of::<Member>() <= 32);

impl SourceJson {
    /// Check the entire original, including every value outside a later path.
    /// The caller's bytes are borrowed only for this call and never modified.
    pub fn parse(input: &[u8]) -> Result<Self, InvalidSourceJson> {
        if input.len() > MAX_JSON_SOURCE_BYTES || std::str::from_utf8(input).is_err() {
            return Err(InvalidSourceJson);
        }
        let mut parser = Parser {
            input,
            position: 0,
            nodes: Vec::new(),
            count: 0,
            members: 0,
        };
        let root = parser.value(1)?;
        parser.whitespace();
        if parser.position != input.len() {
            return Err(InvalidSourceJson);
        }
        Ok(Self {
            nodes: parser.nodes,
            root,
        })
    }

    /// Resolve already checked RFC 6901 tokens and hash complete JCS bytes.
    pub fn select(
        &self,
        pointer: &JsonPointer,
    ) -> Result<CanonicalJsonSelection, InvalidJsonSelection> {
        let mut index = self.root;
        for token in pointer.tokens() {
            index = match &self.nodes[index].value {
                Value::Object(members) => members
                    .iter()
                    .find(|m| m.key == token)
                    .map(|m| m.child)
                    .ok_or(InvalidJsonSelection)?,
                Value::Array(children) => {
                    if token.is_empty()
                        || (token.len() > 1 && token.starts_with('0'))
                        || !token.bytes().all(|b| b.is_ascii_digit())
                    {
                        return Err(InvalidJsonSelection);
                    }
                    let offset: usize = token.parse().map_err(|_| InvalidJsonSelection)?;
                    *children.get(offset).ok_or(InvalidJsonSelection)?
                }
                _ => return Err(InvalidJsonSelection),
            };
        }
        let node = &self.nodes[index];
        if node.depth > MAX_SELECTED_JSON_DEPTH
            || node.count > MAX_SELECTED_JSON_NODES
            || node.canonical_bytes > MAX_SELECTED_JSON_BYTES
        {
            return Err(InvalidJsonSelection);
        }
        // Exact escaped output size is known before any opaque object buffers.
        let mut output = BoundedOutput {
            bytes: Vec::new(),
            limit: node.canonical_bytes,
        };
        output
            .bytes
            .try_reserve_exact(node.canonical_bytes)
            .map_err(|_| InvalidJsonSelection)?;
        serde_json_canonicalizer::to_writer(
            &View {
                nodes: &self.nodes,
                index,
            },
            &mut output,
        )
        .map_err(|_| InvalidJsonSelection)?;
        if output.bytes.len() != node.canonical_bytes {
            return Err(InvalidJsonSelection);
        }
        let sha256 = Sha256Digest::from_bytes(Sha256::digest(&output.bytes).into());
        Ok(CanonicalJsonSelection {
            bytes: output.bytes,
            sha256,
        })
    }
}

impl CanonicalJsonSelection {
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn byte_length(&self) -> usize {
        self.bytes.len()
    }
    pub const fn sha256(&self) -> Sha256Digest {
        self.sha256
    }
}
impl fmt::Debug for SourceJson {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SourceJson(<redacted>)")
    }
}
impl fmt::Debug for CanonicalJsonSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CanonicalJsonSelection(<redacted>)")
    }
}
impl fmt::Display for InvalidSourceJson {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid source JSON")
    }
}
impl Error for InvalidSourceJson {}
impl fmt::Display for InvalidJsonSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid JSON selection")
    }
}
impl Error for InvalidJsonSelection {}

struct Parser<'a> {
    input: &'a [u8],
    position: usize,
    nodes: Vec<Node>,
    count: usize,
    members: usize,
}

/// Explicit requested capacities: minimum four, then doubling, capped before
/// reservation. Push itself cannot allocate. All callers check logical quotas.
fn push<T>(values: &mut Vec<T>, value: T, maximum: usize) -> Result<(), InvalidSourceJson> {
    if values.len() == values.capacity() {
        let capacity = (values.capacity().max(2) * 2).min(maximum);
        values
            .try_reserve_exact(capacity - values.len())
            .map_err(|_| InvalidSourceJson)?;
    }
    values.push(value);
    Ok(())
}

impl Parser<'_> {
    fn whitespace(&mut self) {
        while matches!(
            self.input.get(self.position),
            Some(b' ' | b'\t' | b'\r' | b'\n')
        ) {
            self.position += 1;
        }
    }
    fn take(&mut self, byte: u8) -> bool {
        if self.input.get(self.position) == Some(&byte) {
            self.position += 1;
            true
        } else {
            false
        }
    }
    fn value(&mut self, depth: usize) -> Result<usize, InvalidSourceJson> {
        if depth > MAX_JSON_SOURCE_DEPTH || self.count == MAX_JSON_SOURCE_NODES {
            return Err(InvalidSourceJson);
        }
        self.count += 1;
        self.whitespace();
        let value = match self.input.get(self.position) {
            Some(b'n') => {
                self.literal(b"null")?;
                Value::Null
            }
            Some(b't') => {
                self.literal(b"true")?;
                Value::Bool(true)
            }
            Some(b'f') => {
                self.literal(b"false")?;
                Value::Bool(false)
            }
            Some(b'"') => Value::String(self.string()?),
            Some(b'[') => Value::Array(self.array(depth)?),
            Some(b'{') => Value::Object(self.object(depth)?),
            Some(b'-' | b'0'..=b'9') => Value::Number(self.number()?),
            _ => return Err(InvalidSourceJson),
        };
        let mut node = Node {
            value,
            depth: 1,
            count: 1,
            canonical_bytes: 0,
        };
        node.canonical_bytes = match &node.value {
            Value::Null => 4,
            Value::Bool(true) => 4,
            Value::Bool(false) => 5,
            Value::Number(value) => value.byte_length(),
            Value::String(text) => string_bytes(text),
            Value::Array(children) => {
                let mut bytes = 2 + children.len().saturating_sub(1);
                for child in children {
                    let child = &self.nodes[*child];
                    node.depth = node.depth.max(child.depth + 1);
                    node.count += child.count;
                    bytes += child.canonical_bytes;
                }
                bytes
            }
            Value::Object(members) => {
                let mut bytes = 2 + members.len().saturating_sub(1);
                for member in members {
                    let child = &self.nodes[member.child];
                    node.depth = node.depth.max(child.depth + 1);
                    node.count += child.count;
                    bytes += string_bytes(&member.key) + 1 + child.canonical_bytes;
                }
                bytes
            }
        };
        let index = self.nodes.len();
        push(&mut self.nodes, node, MAX_JSON_SOURCE_NODES)?;
        Ok(index)
    }
    fn literal(&mut self, literal: &[u8]) -> Result<(), InvalidSourceJson> {
        if !self.input[self.position..].starts_with(literal) {
            return Err(InvalidSourceJson);
        }
        self.position += literal.len();
        Ok(())
    }
    fn number(&mut self) -> Result<CanonicalJsonNumber, InvalidSourceJson> {
        let start = self.position;
        while matches!(
            self.input.get(self.position),
            Some(b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
        ) {
            if self.position - start == MAX_JSON_NUMBER_TOKEN_BYTES {
                return Err(InvalidSourceJson);
            }
            self.position += 1;
        }
        // Only ASCII bytes have been consumed, after complete input UTF8 check.
        CanonicalJsonNumber::from_token(
            std::str::from_utf8(&self.input[start..self.position])
                .map_err(|_| InvalidSourceJson)?,
        )
        .map_err(|_| InvalidSourceJson)
    }
    fn string(&mut self) -> Result<String, InvalidSourceJson> {
        if !self.take(b'"') {
            return Err(InvalidSourceJson);
        }
        let start = self.position - 1;
        loop {
            match self.input.get(self.position) {
                Some(b'"') => {
                    self.position += 1;
                    // A single delimited token, never a generic source Value.
                    return serde_json::from_slice::<String>(&self.input[start..self.position])
                        .map_err(|_| InvalidSourceJson);
                }
                Some(b'\\') => {
                    self.position += 1;
                    if self.input.get(self.position).is_none() {
                        return Err(InvalidSourceJson);
                    }
                    self.position += 1;
                }
                Some(0..=31) | None => return Err(InvalidSourceJson),
                Some(_) => self.position += 1,
            }
        }
    }
    fn array(&mut self, depth: usize) -> Result<Vec<usize>, InvalidSourceJson> {
        self.position += 1;
        self.whitespace();
        let mut children = Vec::new();
        if self.take(b']') {
            return Ok(children);
        }
        loop {
            if children.len() == MAX_JSON_ARRAY_ELEMENTS {
                return Err(InvalidSourceJson);
            }
            let child = self.value(depth + 1)?;
            push(&mut children, child, MAX_JSON_ARRAY_ELEMENTS)?;
            self.whitespace();
            if self.take(b']') {
                return Ok(children);
            }
            if !self.take(b',') {
                return Err(InvalidSourceJson);
            }
        }
    }
    fn object(&mut self, depth: usize) -> Result<Vec<Member>, InvalidSourceJson> {
        self.position += 1;
        self.whitespace();
        let mut members = Vec::new();
        if self.take(b'}') {
            return Ok(members);
        }
        loop {
            if members.len() == MAX_JSON_OBJECT_MEMBERS || self.members == MAX_JSON_SOURCE_MEMBERS {
                return Err(InvalidSourceJson);
            }
            self.members += 1;
            self.whitespace();
            let key = self.string()?;
            self.whitespace();
            if !self.take(b':') {
                return Err(InvalidSourceJson);
            }
            let child = self.value(depth + 1)?;
            push(&mut members, Member { key, child }, MAX_JSON_OBJECT_MEMBERS)?;
            self.whitespace();
            if self.take(b'}') {
                break;
            }
            if !self.take(b',') {
                return Err(InvalidSourceJson);
            }
        }
        unique_keys(&members)?;
        Ok(members)
    }
}

/// Equality detection only; canonical ordering belongs to the pinned JCS path.
fn unique_keys(members: &[Member]) -> Result<(), InvalidSourceJson> {
    let count = members.len();
    if count < 2 {
        return Ok(());
    }
    let mut order = Vec::new();
    let mut scratch = Vec::new();
    order
        .try_reserve_exact(count)
        .map_err(|_| InvalidSourceJson)?;
    scratch
        .try_reserve_exact(count)
        .map_err(|_| InvalidSourceJson)?;
    order.extend(0..count);
    scratch.resize(count, 0);
    let mut width = 1;
    while width < count {
        for start in (0..count).step_by(width * 2) {
            let middle = (start + width).min(count);
            let end = (start + width * 2).min(count);
            let (mut left, mut right) = (start, middle);
            for slot in &mut scratch[start..end] {
                if left < middle
                    && (right == end || members[order[left]].key <= members[order[right]].key)
                {
                    *slot = order[left];
                    left += 1;
                } else {
                    *slot = order[right];
                    right += 1;
                }
            }
        }
        std::mem::swap(&mut order, &mut scratch);
        width *= 2;
    }
    if order
        .windows(2)
        .any(|pair| members[pair[0]].key == members[pair[1]].key)
    {
        return Err(InvalidSourceJson);
    }
    Ok(())
}

fn string_bytes(text: &str) -> usize {
    2 + text
        .bytes()
        .map(|byte| match byte {
            b'"' | b'\\' | b'\x08' | b'\t' | b'\n' | b'\x0c' | b'\r' => 2,
            0..=31 => 6,
            _ => 1,
        })
        .sum::<usize>()
}

/// Only checked nodes enter Serialize. No public serializer or raw fragment API.
struct View<'a> {
    nodes: &'a [Node],
    index: usize,
}
impl Serialize for View<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match &self.nodes[self.index].value {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(value) => serializer.serialize_bool(*value),
            Value::Number(value) => {
                // Reparse the admitted short canonical spelling, not original
                // decimal input. The scalar's finite/equality check is retained.
                let text =
                    std::str::from_utf8(value.as_bytes()).map_err(serde::ser::Error::custom)?;
                let finite = text.parse::<f64>().map_err(serde::ser::Error::custom)?;
                serializer.serialize_f64(finite)
            }
            Value::String(text) => serializer.serialize_str(text),
            Value::Array(children) => {
                let mut output = serializer.serialize_seq(Some(children.len()))?;
                for child in children {
                    output.serialize_element(&View {
                        nodes: self.nodes,
                        index: *child,
                    })?;
                }
                output.end()
            }
            Value::Object(members) => {
                let mut output = serializer.serialize_map(Some(members.len()))?;
                for member in members {
                    output.serialize_entry(
                        &member.key,
                        &View {
                            nodes: self.nodes,
                            index: member.child,
                        },
                    )?;
                }
                output.end()
            }
        }
    }
}

struct BoundedOutput {
    bytes: Vec<u8>,
    limit: usize,
}
impl Write for BoundedOutput {
    fn write(&mut self, bytes: &[u8]) -> OutputResult<usize> {
        if bytes.len() > self.limit - self.bytes.len() {
            return Err(ErrorKind::FileTooLarge.into());
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> OutputResult<()> {
        Ok(())
    }
}
