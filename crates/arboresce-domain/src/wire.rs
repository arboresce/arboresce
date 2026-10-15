//! Bounded JSON admission for the three closed revision objects only.
//! This is not a general JSON, source, locator or transport parser.

use std::{error::Error, fmt};

use crate::{ContentRevision, ControlRevision, Generation};

/// The existing common control envelope, checked before traversing input.
pub const MAX_REVISION_WIRE_BYTES: usize = 1_048_576;

/// A rejected closed revision object, without retaining any input bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidRevisionWire;

impl fmt::Display for InvalidRevisionWire {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid revision wire object")
    }
}

impl Error for InvalidRevisionWire {}

/// An immutable tagged value whose variants retain their distinct scalar types.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WireRevision {
    Content(ContentRevision),
    Control(ControlRevision),
    Generation(Generation),
}

impl WireRevision {
    /// Admit exactly two named string fields, with no duplicate or extra member.
    /// Every admissible decoded key, tag and value is ASCII. Valid ASCII JSON
    /// escapes preserve that meaning; other decoded characters cannot match it.
    pub fn from_json(input: &[u8]) -> Result<Self, InvalidRevisionWire> {
        if input.len() > MAX_REVISION_WIRE_BYTES {
            return Err(InvalidRevisionWire);
        }
        let mut cursor = Cursor { input, position: 0 };
        cursor.token(b'{')?;
        let mut kind = None;
        let mut value = None;
        for index in 0..2 {
            if index != 0 {
                cursor.token(b',')?;
            }
            let key = cursor.string(5)?;
            cursor.token(b':')?;
            match key.as_str() {
                "kind" if kind.is_none() => kind = Some(cursor.string(10)?),
                "value" if value.is_none() => value = Some(cursor.string(18)?),
                _ => return Err(InvalidRevisionWire),
            }
        }
        cursor.token(b'}')?;
        cursor.whitespace();
        if cursor.position != input.len() {
            return Err(InvalidRevisionWire);
        }
        let kind = kind.ok_or(InvalidRevisionWire)?;
        let value = value.ok_or(InvalidRevisionWire)?;
        match kind.as_str() {
            "content" => value.parse().map(Self::Content),
            "control" => value.parse().map(Self::Control),
            "generation" => value.parse().map(Self::Generation),
            _ => return Err(InvalidRevisionWire),
        }
        .map_err(|_| InvalidRevisionWire)
    }

    pub const fn kind(self) -> &'static str {
        match self {
            Self::Content(_) => "content",
            Self::Control(_) => "control",
            Self::Generation(_) => "generation",
        }
    }

    /// Return the exact integer; this grants no cross-kind comparison authority.
    pub const fn value(self) -> u64 {
        match self {
            Self::Content(value) => value.value(),
            Self::Control(value) => value.value(),
            Self::Generation(value) => value.value(),
        }
    }
}

impl fmt::Display for WireRevision {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{{\"kind\":\"{}\",\"value\":\"{}\"}}",
            self.kind(),
            self.value()
        )
    }
}

macro_rules! wire_value {
    ($scalar:ident, $variant:ident) => {
        impl From<$scalar> for WireRevision {
            fn from(value: $scalar) -> Self {
                Self::$variant(value)
            }
        }

        impl $scalar {
            /// Admit the complete closed object only with this scalar's own tag.
            pub fn from_wire_json(input: &[u8]) -> Result<Self, InvalidRevisionWire> {
                match WireRevision::from_json(input)? {
                    WireRevision::$variant(value) => Ok(value),
                    _ => Err(InvalidRevisionWire),
                }
            }
        }
    };
}

wire_value!(ContentRevision, Content);
wire_value!(ControlRevision, Control);
wire_value!(Generation, Generation);

/// A cursor over a bounded flat object. No recursion or input-sized allocation.
struct Cursor<'a> {
    input: &'a [u8],
    position: usize,
}

impl Cursor<'_> {
    fn whitespace(&mut self) {
        while matches!(
            self.input.get(self.position),
            Some(b' ' | b'\t' | b'\r' | b'\n')
        ) {
            self.position += 1;
        }
    }

    fn take(&mut self) -> Result<u8, InvalidRevisionWire> {
        let byte = self
            .input
            .get(self.position)
            .copied()
            .ok_or(InvalidRevisionWire)?;
        // The cursor is within the one-MiB input before advancing by one.
        self.position += 1;
        Ok(byte)
    }

    fn token(&mut self, expected: u8) -> Result<(), InvalidRevisionWire> {
        self.whitespace();
        if self.take()? == expected {
            Ok(())
        } else {
            Err(InvalidRevisionWire)
        }
    }

    fn escaped(&mut self) -> Result<u8, InvalidRevisionWire> {
        Ok(match self.take()? {
            b'"' => b'"',
            b'\\' => b'\\',
            b'/' => b'/',
            b'b' => 8,
            b'f' => 12,
            b'n' => b'\n',
            b'r' => b'\r',
            b't' => b'\t',
            b'u' => {
                let mut value = 0u16;
                for _ in 0..4 {
                    let digit = match self.take()? {
                        byte @ b'0'..=b'9' => byte - b'0',
                        byte @ b'a'..=b'f' => byte - b'a' + 10,
                        byte @ b'A'..=b'F' => byte - b'A' + 10,
                        _ => return Err(InvalidRevisionWire),
                    };
                    // Four hexadecimal digits fit exactly in u16.
                    value = (value << 4) | u16::from(digit);
                }
                // Non-ASCII decoded strings cannot be a known field/tag/payload.
                if value > 127 {
                    return Err(InvalidRevisionWire);
                }
                value as u8
            }
            _ => return Err(InvalidRevisionWire),
        })
    }

    fn string(&mut self, maximum: usize) -> Result<String, InvalidRevisionWire> {
        self.token(b'"')?;
        let mut value = String::with_capacity(maximum);
        loop {
            let byte = match self.take()? {
                b'"' => return Ok(value),
                b'\\' => self.escaped()?,
                byte @ b' '..=127 => byte,
                _ => return Err(InvalidRevisionWire),
            };
            if value.len() == maximum {
                return Err(InvalidRevisionWire);
            }
            value.push(char::from(byte));
        }
    }
}
