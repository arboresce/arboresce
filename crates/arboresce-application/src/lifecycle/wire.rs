//! Fixed journal structure and bounded raw JSON. No authenticity or writer grant.

use std::{error::Error, fmt, sync::OnceLock, vec};

use arboresce_domain::{
    ContentRevision, Identifier, Sha256Digest, UnsignedJsonInteger, UtcTimestamp,
};
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};

use super::{CheckpointScope, EntryScope, Family, Namespace, Ordinal};

pub const MAX_CONTROL_BYTES: usize = 1_048_576;
const MAX_CONTAINERS: usize = 16;
const MAX_MEMBERS: usize = 128;
const MAX_ELEMENTS: usize = 256;
const MAX_STRING_SCALARS: usize = 65_536;
const MAX_SCHEMA_DEPTH: usize = 128;

const DEFINITIONS: [&str; 30] = [
    "recognition_scope",
    "recognition_identity",
    "recognition_reference",
    "continuity_intent_reference",
    "recognition_entry",
    "origin_candidate",
    "command_origin_candidate",
    "command_admission_reference",
    "continuity_abort_reference",
    "recognition_intent",
    "recognition_resolution",
    "retirement_scope",
    "scope_retirement_reference",
    "scope_retirement_core",
    "scope_retirement_record",
    "scope_retirement_intent",
    "scope_retirement_resolution",
    "upload_session_journal_subject",
    "upload_session_event_core",
    "upload_session_event_record",
    "continuity_binding_reference",
    "initial_propagation_binding",
    "journal_prepared_payload",
    "journal_committed_payload",
    "journal_intent",
    "journal_resolution",
    "continuity_record",
    "continuity_entry",
    "journal_checkpoint",
    "continuity_batch",
];

/// One exact named internal definition. A library root is never a target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Definition(&'static str);

impl Definition {
    pub fn from_name(name: &str) -> Option<Self> {
        DEFINITIONS
            .iter()
            .copied()
            .find(|value| *value == name)
            .map(Self)
    }

    pub const fn name(self) -> &'static str {
        self.0
    }

    /// Structural diagnostic over parsed values; no raw-token or admission proof.
    pub fn matches_parsed(self, value: &Value) -> bool {
        schemas().is_some_and(|schemas| schemas.selected(self, value, Mode::Parsed))
    }
}

/// A rejected raw document, without supplied text or parser diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidControlJson;

impl fmt::Display for InvalidControlJson {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid bounded control JSON")
    }
}

impl Error for InvalidControlJson {}

/// Immutable exact raw bytes plus a private bounded decoded value.
pub struct ControlDocument<'a> {
    bytes: &'a [u8],
    value: Value,
}

impl fmt::Debug for ControlDocument<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ControlDocument(<redacted>)")
    }
}

impl<'a> ControlDocument<'a> {
    /// Slice intake only; an HTTP reader must enforce the byte cap while reading.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, InvalidControlJson> {
        if bytes.len() > MAX_CONTROL_BYTES {
            return Err(InvalidControlJson);
        }
        let text = std::str::from_utf8(bytes).map_err(|_| InvalidControlJson)?;
        let mut numbers = number_tokens(text).ok_or(InvalidControlJson)?.into_iter();
        let mut decoder = serde_json::Deserializer::from_str(text);
        let value = Seed {
            depth: 0,
            numbers: &mut numbers,
            allowed: true,
        }
        .deserialize(&mut decoder)
        .map_err(|_| InvalidControlJson)?;
        decoder.end().map_err(|_| InvalidControlJson)?;
        if numbers.next().is_some() {
            return Err(InvalidControlJson);
        }
        Ok(Self { bytes, value })
    }

    pub fn matches_shape(&self, definition: Definition) -> bool {
        definition.matches_parsed(&self.value)
    }

    /// Enforce shape and unsigned protocol tokens, retaining no mutable value API.
    pub fn check(self, definition: Definition) -> Option<WireRecord<'a>> {
        let schemas = schemas()?;
        if !schemas.selected(definition, &self.value, Mode::Parsed)
            || !schemas.selected(definition, &self.value, Mode::Wire)
        {
            return None;
        }
        Some(WireRecord {
            document: self,
            definition,
        })
    }
}

/// Checked wire structure only. Digests and references are not authenticated.
pub struct WireRecord<'a> {
    document: ControlDocument<'a>,
    definition: Definition,
}

impl fmt::Debug for WireRecord<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("WireRecord(<redacted>)")
    }
}

/// Complete typed scope projection, without historical payload/binding joins.
pub struct BatchScope {
    pub checkpoint: CheckpointScope,
    pub entries: Vec<EntryScope>,
}

impl fmt::Debug for BatchScope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BatchScope(<redacted>)")
    }
}

impl WireRecord<'_> {
    pub fn as_bytes(&self) -> &[u8] {
        self.document.bytes
    }

    pub const fn definition(&self) -> Definition {
        self.definition
    }

    pub fn project_batch(&self) -> Option<BatchScope> {
        if self.definition.name() != "continuity_batch" {
            return None;
        }
        let value = &self.document.value;
        let checkpoint = value.get("checkpoint")?;
        let organization = checkpoint.get("organization_id")?.as_str()?.parse().ok()?;
        let stream = checkpoint.get("stream_id")?.as_str()?.parse().ok()?;
        let ordinal = |value: &Value| Ordinal::new(value.as_str()?.parse().ok()?).ok();
        let previous = match checkpoint.get("previous_checkpoint_sha256")? {
            Value::Null => None,
            value => Some(value.as_str()?.parse().ok()?),
        };
        let checkpoint = CheckpointScope {
            namespace: Namespace::new(organization, stream),
            first: ordinal(checkpoint.get("first_ordinal")?)?,
            last: ordinal(checkpoint.get("last_ordinal")?)?,
            previous_checkpoint_sha256: previous,
        };
        let entries = value
            .get("entries")?
            .as_array()?
            .iter()
            .map(|entry| {
                let record = entry.get("record")?;
                Some(EntryScope {
                    organization: record
                        .get("value")?
                        .get("organization_id")?
                        .as_str()?
                        .parse()
                        .ok()?,
                    ordinal: ordinal(entry.get("ordinal")?)?,
                    family: Family::from_name(record.get("kind")?.as_str()?)?,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        Some(BatchScope {
            checkpoint,
            entries,
        })
    }
}

/// Inspect original tokens before serde can round away underflow or lexical kind.
/// Strings are skipped without interpreting them; the decoder validates grammar.
fn number_tokens(text: &str) -> Option<Vec<Number>> {
    let bytes = text.as_bytes();
    let mut position = 0;
    let mut numbers = Vec::new();
    while position < bytes.len() {
        match bytes[position] {
            b'"' => {
                position += 1;
                while position < bytes.len() {
                    match bytes[position] {
                        b'\\' => position += 2,
                        b'"' => {
                            position += 1;
                            break;
                        }
                        _ => position += 1,
                    }
                }
            }
            b'-' | b'0'..=b'9' => {
                let start = position;
                while position < bytes.len()
                    && !matches!(
                        bytes[position],
                        b' ' | b'\t' | b'\r' | b'\n' | b',' | b']' | b'}'
                    )
                {
                    position += 1;
                }
                let token = text.get(start..position)?;
                let value = token.parse::<f64>().ok()?;
                let nonzero = token
                    .split(['e', 'E'])
                    .next()?
                    .bytes()
                    .any(|b| matches!(b, b'1'..=b'9'));
                if !value.is_finite()
                    || value.abs() > 9_007_199_254_740_991.0
                    || (value == 0.0 && nonzero)
                {
                    return None;
                }
                numbers.push(match UnsignedJsonInteger::from_token(token.as_bytes()) {
                    Ok(value) => Number::from(value.value()),
                    Err(_) => Number::from_f64(value)?,
                });
            }
            _ => position += 1,
        }
    }
    Some(numbers)
}

struct Seed<'a> {
    depth: usize,
    numbers: &'a mut vec::IntoIter<Number>,
    allowed: bool,
}

impl Seed<'_> {
    fn number<E: de::Error>(self) -> Result<Value, E> {
        self.numbers
            .next()
            .map(Value::Number)
            .ok_or_else(|| E::custom("invalid number"))
    }
}

impl<'de> DeserializeSeed<'de> for Seed<'_> {
    type Value = Value;

    fn deserialize<D: de::Deserializer<'de>>(self, decoder: D) -> Result<Value, D::Error> {
        if !self.allowed {
            return Err(de::Error::custom("control array limit"));
        }
        decoder.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Seed<'_> {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("bounded control JSON")
    }

    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_u64<E: de::Error>(self, _value: u64) -> Result<Value, E> {
        self.number()
    }

    fn visit_i64<E: de::Error>(self, _value: i64) -> Result<Value, E> {
        self.number()
    }

    fn visit_f64<E: de::Error>(self, _value: f64) -> Result<Value, E> {
        self.number()
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Value, E> {
        if value.chars().count() > MAX_STRING_SCALARS {
            return Err(E::custom("control string limit"));
        }
        Ok(Value::String(value.to_owned()))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Value, A::Error> {
        if self.depth >= MAX_CONTAINERS {
            return Err(de::Error::custom("control depth limit"));
        }
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(Seed {
            depth: self.depth + 1,
            numbers: &mut *self.numbers,
            allowed: values.len() < MAX_ELEMENTS,
        })? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut entries: A) -> Result<Value, A::Error> {
        if self.depth >= MAX_CONTAINERS {
            return Err(de::Error::custom("control depth limit"));
        }
        let mut values = Map::new();
        while let Some(key) = entries.next_key::<String>()? {
            if values.len() == MAX_MEMBERS || !valid_key(&key) || values.contains_key(&key) {
                return Err(de::Error::custom("invalid control members"));
            }
            let value = entries.next_value_seed(Seed {
                depth: self.depth + 1,
                numbers: &mut *self.numbers,
                allowed: true,
            })?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}

fn valid_key(value: &str) -> bool {
    (1..=128).contains(&value.chars().count())
        && !value.chars().any(|c| c <= '\u{1f}' || c == '\u{7f}')
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Mode {
    Parsed,
    Wire,
}

#[derive(Clone, Copy)]
enum Library {
    Continuity,
    Common,
    Lifecycle,
    Context,
    Acquisition,
    Review,
    Financial,
}

struct Schemas([Value; 7]);

fn schemas() -> Option<&'static Schemas> {
    static SCHEMAS: OnceLock<Option<Schemas>> = OnceLock::new();
    SCHEMAS
        .get_or_init(|| {
            Some(Schemas([
                serde_json::from_str(include_str!(
                    "../../../../contracts/internal/v1/lifecycle-continuity.schema.json"
                ))
                .ok()?,
                serde_json::from_str(include_str!(
                    "../../../../contracts/public/v1/common.schema.json"
                ))
                .ok()?,
                serde_json::from_str(include_str!(
                    "../../../../contracts/public/v1/lifecycle.schema.json"
                ))
                .ok()?,
                serde_json::from_str(include_str!(
                    "../../../../contracts/public/v1/context.schema.json"
                ))
                .ok()?,
                serde_json::from_str(include_str!(
                    "../../../../contracts/public/v1/acquisition.schema.json"
                ))
                .ok()?,
                serde_json::from_str(include_str!(
                    "../../../../contracts/public/v1/candidate-review.schema.json"
                ))
                .ok()?,
                serde_json::from_str(include_str!(
                    "../../../../contracts/public/v1/financial-records.schema.json"
                ))
                .ok()?,
            ]))
        })
        .as_ref()
}

impl Schemas {
    fn library(&self, library: Library) -> &Value {
        &self.0[library as usize]
    }

    fn selected(&self, definition: Definition, value: &Value, mode: Mode) -> bool {
        self.library(Library::Continuity)
            .get("$defs")
            .and_then(|d| d.get(definition.name()))
            .is_some_and(|schema| {
                self.matches(Library::Continuity, schema, value, mode, MAX_SCHEMA_DEPTH)
            })
    }

    /// Closed local references only. The supplied instance can never choose a URI.
    fn reference(&self, current: Library, reference: &str) -> Option<(Library, &Value)> {
        let (file, definition) = reference.split_once("#/$defs/")?;
        let library = match file {
            "" => current,
            "common.schema.json" | "../../public/v1/common.schema.json" => Library::Common,
            "lifecycle.schema.json" | "../../public/v1/lifecycle.schema.json" => Library::Lifecycle,
            "context.schema.json" | "../../public/v1/context.schema.json" => Library::Context,
            "acquisition.schema.json" | "../../public/v1/acquisition.schema.json" => {
                Library::Acquisition
            }
            "candidate-review.schema.json" | "../../public/v1/candidate-review.schema.json" => {
                Library::Review
            }
            "financial-records.schema.json" | "../../public/v1/financial-records.schema.json" => {
                Library::Financial
            }
            _ => return None,
        };
        Some((
            library,
            self.library(library).get("$defs")?.get(definition)?,
        ))
    }

    /// Interpret only this frozen schema vocabulary, with finite recursion.
    fn matches(
        &self,
        library: Library,
        schema: &Value,
        value: &Value,
        mode: Mode,
        depth: usize,
    ) -> bool {
        if depth == 0 {
            return false;
        }
        let Some(fields) = schema.as_object() else {
            return schema.as_bool() == Some(true);
        };
        if fields.keys().any(|key| {
            !matches!(
                key.as_str(),
                "$ref"
                    | "type"
                    | "required"
                    | "properties"
                    | "additionalProperties"
                    | "description"
                    | "minLength"
                    | "maxLength"
                    | "pattern"
                    | "allOf"
                    | "not"
                    | "const"
                    | "format"
                    | "oneOf"
                    | "enum"
                    | "minimum"
                    | "maximum"
                    | "if"
                    | "then"
                    | "else"
                    | "maxProperties"
                    | "propertyNames"
                    | "maxItems"
                    | "items"
                    | "minItems"
                    | "uniqueItems"
                    | "prefixItems"
            )
        }) {
            return false;
        }
        let recur =
            |schema: &Value, value: &Value| self.matches(library, schema, value, mode, depth - 1);
        if let Some(reference) = fields.get("$ref")
            && !reference
                .as_str()
                .and_then(|r| self.reference(library, r))
                .is_some_and(|(owner, schema)| self.matches(owner, schema, value, mode, depth - 1))
        {
            return false;
        }
        if let Some(kind) = fields.get("type")
            && !kind
                .as_str()
                .is_some_and(|kind| kind_matches(kind, value, mode))
        {
            return false;
        }
        if fields
            .get("const")
            .is_some_and(|expected| !equal(value, expected, depth))
            || fields.get("enum").is_some_and(|values| {
                !values.as_array().is_some_and(|values| {
                    values.iter().any(|expected| equal(value, expected, depth))
                })
            })
        {
            return false;
        }
        if let Some(schemas) = fields.get("allOf")
            && !schemas
                .as_array()
                .is_some_and(|schemas| schemas.iter().all(|schema| recur(schema, value)))
        {
            return false;
        }
        if let Some(schemas) = fields.get("oneOf")
            && !schemas.as_array().is_some_and(|schemas| {
                schemas.iter().filter(|schema| recur(schema, value)).count() == 1
            })
        {
            return false;
        }
        // Negation and branch selection are structural predicates. They cannot
        // turn a rejected unsigned token into a different accepted branch.
        if fields
            .get("not")
            .is_some_and(|schema| self.matches(library, schema, value, Mode::Parsed, depth - 1))
        {
            return false;
        }
        if let Some(condition) = fields.get("if") {
            let branch = if self.matches(library, condition, value, Mode::Parsed, depth - 1) {
                "then"
            } else {
                "else"
            };
            if fields
                .get(branch)
                .is_some_and(|schema| !recur(schema, value))
            {
                return false;
            }
        }
        if let Some(text) = value.as_str()
            && (!minimum(fields.get("minLength"), text.chars().count())
                || !maximum(fields.get("maxLength"), text.chars().count())
                || fields.get("pattern").is_some_and(|pattern| {
                    !pattern
                        .as_str()
                        .is_some_and(|pattern| pattern_matches(pattern, text))
                })
                || fields.get("format").is_some_and(|format| {
                    format != "date-time" || text.parse::<UtcTimestamp>().is_err()
                }))
        {
            return false;
        }
        if let Some(number) = value.as_number() {
            let Some(number) = number.as_f64() else {
                return false;
            };
            if fields
                .get("minimum")
                .is_some_and(|bound| !bound.as_f64().is_some_and(|bound| number >= bound))
                || fields
                    .get("maximum")
                    .is_some_and(|bound| !bound.as_f64().is_some_and(|bound| number <= bound))
            {
                return false;
            }
        }
        if let Some(object) = value.as_object() {
            if !maximum(fields.get("maxProperties"), object.len())
                || fields.get("required").is_some_and(|names| {
                    !names.as_array().is_some_and(|names| {
                        names
                            .iter()
                            .all(|name| name.as_str().is_some_and(|name| object.contains_key(name)))
                    })
                })
            {
                return false;
            }
            for (key, member) in object {
                if fields
                    .get("propertyNames")
                    .is_some_and(|schema| !recur(schema, &Value::String(key.clone())))
                {
                    return false;
                }
                if let Some(schema) = fields
                    .get("properties")
                    .and_then(|properties| properties.get(key))
                {
                    if !recur(schema, member) {
                        return false;
                    }
                } else if fields
                    .get("additionalProperties")
                    .is_some_and(|schema| !recur(schema, member))
                {
                    return false;
                }
            }
        }
        if let Some(array) = value.as_array() {
            if !minimum(fields.get("minItems"), array.len())
                || !maximum(fields.get("maxItems"), array.len())
            {
                return false;
            }
            let prefix = fields.get("prefixItems").and_then(Value::as_array);
            for (index, member) in array.iter().enumerate() {
                let schema = prefix
                    .and_then(|prefix| prefix.get(index))
                    .or_else(|| fields.get("items"));
                if schema.is_some_and(|schema| !recur(schema, member)) {
                    return false;
                }
            }
            if fields.get("uniqueItems") == Some(&Value::Bool(true))
                && array
                    .iter()
                    .enumerate()
                    .any(|(i, left)| array[i + 1..].iter().any(|right| equal(left, right, depth)))
            {
                return false;
            }
        }
        true
    }
}

fn minimum(bound: Option<&Value>, count: usize) -> bool {
    bound.is_none_or(|bound| bound.as_u64().is_some_and(|bound| count as u64 >= bound))
}

fn maximum(bound: Option<&Value>, count: usize) -> bool {
    bound.is_none_or(|bound| bound.as_u64().is_some_and(|bound| count as u64 <= bound))
}

fn kind_matches(kind: &str, value: &Value, mode: Mode) -> bool {
    match kind {
        "null" => value.is_null(),
        "boolean" => value.is_boolean(),
        "string" => value.is_string(),
        "number" => value.is_number(),
        "object" => value.is_object(),
        "array" => value.is_array(),
        "integer" if mode == Mode::Wire => value.as_number().is_some_and(Number::is_u64),
        "integer" => value
            .as_f64()
            .is_some_and(|number| number.is_finite() && number.fract() == 0.0),
        _ => false,
    }
}

fn equal(left: &Value, right: &Value, depth: usize) -> bool {
    if depth == 0 {
        return false;
    }
    match (left, right) {
        (Value::Number(left), Value::Number(right)) => left
            .as_f64()
            .zip(right.as_f64())
            .is_some_and(|(left, right)| left == right),
        (Value::Array(left), Value::Array(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| equal(left, right, depth - 1))
        }
        (Value::Object(left), Value::Object(right)) => {
            left.len() == right.len()
                && left.iter().all(|(key, left)| {
                    right
                        .get(key)
                        .is_some_and(|right| equal(left, right, depth - 1))
                })
        }
        _ => left == right,
    }
}

fn slug_component(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes.next().is_some_and(|b| b.is_ascii_lowercase())
        && bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

fn pattern_matches(pattern: &str, value: &str) -> bool {
    match pattern {
        r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}(?![\s\S])" => {
            value.parse::<Identifier>().is_ok()
        }
        r"^[0-9a-f]{64}(?![\s\S])" => value.parse::<Sha256Digest>().is_ok(),
        r"^[1-9][0-9]{0,17}(?![\s\S])" => value.parse::<ContentRevision>().is_ok(),
        r"^[a-z][a-z0-9_]*(?:\.[a-z][a-z0-9_]*)+(?![\s\S])" => {
            value.split('.').count() >= 2 && value.split('.').all(slug_component)
        }
        r"^[a-z][a-z0-9_]{0,63}(?![\s\S])" => value.len() <= 64 && slug_component(value),
        r"^[^\u0000-\u001f\u007f]+(?![\s\S])" => valid_key(value),
        r"^urn:uuid:[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}(?![\s\S])" => {
            value
                .strip_prefix("urn:uuid:")
                .is_some_and(|value| value.parse::<Identifier>().is_ok())
        }
        r"^(?!0000)[0-9]{4}-(?:0[1-9]|1[0-2])-(?:0[1-9]|[12][0-9]|3[01])T(?:[01][0-9]|2[0-3]):[0-5][0-9]:[0-5][0-9]\.[0-9]{3}Z(?![\s\S])" => {
            value.parse::<UtcTimestamp>().is_ok()
        }
        _ => false,
    }
}
