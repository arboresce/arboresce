//! Checked opaque identifier values, independent of transport and storage.

use std::{error::Error, fmt, str::FromStr};

/// A canonical lower-case UUIDv4 value.
///
/// Validation establishes format, not resource existence, authorization,
/// authenticated principal identity, ordering or random generation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Identifier([u8; 16]);

/// A rejected identifier. It retains none of the supplied input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidIdentifier;

impl Identifier {
    /// Check the UUID version and RFC variant before constructing a value.
    pub fn from_bytes(bytes: [u8; 16]) -> Result<Self, InvalidIdentifier> {
        if bytes[6] >> 4 != 4 || bytes[8] >> 6 != 2 {
            return Err(InvalidIdentifier);
        }
        Ok(Self(bytes))
    }

    /// Borrow the exact validated bytes without permitting mutation.
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl FromStr for Identifier {
    type Err = InvalidIdentifier;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.len() != 36 {
            return Err(InvalidIdentifier);
        }
        let mut bytes = [0u8; 16];
        let mut nibble = 0usize;
        for (index, byte) in input.bytes().enumerate() {
            if matches!(index, 8 | 13 | 18 | 23) {
                if byte != b'-' {
                    return Err(InvalidIdentifier);
                }
                continue;
            }
            let value = match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                _ => return Err(InvalidIdentifier),
            };
            // Exact length and the four fixed separators admit 32 nibbles.
            if nibble & 1 == 0 {
                bytes[nibble / 2] = value << 4;
            } else {
                bytes[nibble / 2] |= value;
            }
            nibble += 1;
        }
        Self::from_bytes(bytes)
    }
}

impl TryFrom<&str> for Identifier {
    type Error = InvalidIdentifier;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, byte) in self.0.iter().enumerate() {
            if matches!(index, 4 | 6 | 8 | 10) {
                formatter.write_str("-")?;
            }
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl fmt::Display for InvalidIdentifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid canonical UUIDv4 identifier")
    }
}

impl Error for InvalidIdentifier {}
