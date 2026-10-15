use std::{error::Error, fmt, str::FromStr};

use sha2::{Digest, Sha256};

use crate::Sha256Digest;

pub const MAX_JSON_NUMBER_TOKEN_BYTES: usize = 4096;
pub const MAX_CANONICAL_NUMBER_BYTES: usize = 32;
const MAX_EXPONENT_MAGNITUDE: u32 = 8192;
const MAX_SAFE_NUMBER: f64 = 9_007_199_254_740_991.0;

/// One admitted scalar number's canonical bytes and their SHA-256.
///
/// This does not parse a document or establish an original's byte identity.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct CanonicalJsonNumber {
    bytes: [u8; MAX_CANONICAL_NUMBER_BYTES],
    length: u8,
    sha256: Sha256Digest,
}

/// A static rejection without the supplied token or an approximate value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidCanonicalNumber;

impl CanonicalJsonNumber {
    /// Admit complete JSON numeric grammar without whitespace or coercion.
    ///
    /// Decimal meaning must survive shortest JCS serialization after finite
    /// binary64 conversion. This is distinct from unsigned wire integers.
    pub fn from_token(input: &str) -> Result<Self, InvalidCanonicalNumber> {
        let original = Decimal::parse(input)?;
        // Never pass a long, untrusted mantissa to the platform float parser.
        let short = format!(
            "{}{}e{}",
            if original.negative { "-" } else { "" },
            original.coefficient,
            original.exponent
        );
        let value = short.parse::<f64>().map_err(|_| InvalidCanonicalNumber)?;
        if !value.is_finite()
            || value.abs() > MAX_SAFE_NUMBER
            || (value == 0.0 && original.coefficient != 0)
        {
            return Err(InvalidCanonicalNumber);
        }
        // The only Serialize implementation entering this dependency is f64.
        // Its reviewed finite path emits <=25 bytes into the initial 128-byte
        // vector, with one boxed writer reference and no object/key buffers.
        let canonical =
            serde_json_canonicalizer::to_vec(&value).map_err(|_| InvalidCanonicalNumber)?;
        if canonical.len() > MAX_CANONICAL_NUMBER_BYTES {
            return Err(InvalidCanonicalNumber);
        }
        let text = std::str::from_utf8(&canonical).map_err(|_| InvalidCanonicalNumber)?;
        if Decimal::parse(text)? != original {
            return Err(InvalidCanonicalNumber);
        }
        let mut bytes = [0; MAX_CANONICAL_NUMBER_BYTES];
        bytes[..canonical.len()].copy_from_slice(&canonical);
        Ok(Self {
            bytes,
            length: canonical.len() as u8,
            sha256: Sha256Digest::from_bytes(Sha256::digest(&canonical).into()),
        })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.byte_length()]
    }

    pub const fn byte_length(&self) -> usize {
        self.length as usize
    }

    pub const fn sha256(&self) -> Sha256Digest {
        self.sha256
    }
}

impl FromStr for CanonicalJsonNumber {
    type Err = InvalidCanonicalNumber;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::from_token(input)
    }
}

impl fmt::Display for CanonicalJsonNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = std::str::from_utf8(self.as_bytes()).map_err(|_| fmt::Error)?;
        f.write_str(text)
    }
}

impl fmt::Debug for CanonicalJsonNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CanonicalJsonNumber(<redacted>)")
    }
}

impl fmt::Display for InvalidCanonicalNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid canonical JSON number")
    }
}

impl Error for InvalidCanonicalNumber {}

#[derive(Eq, PartialEq)]
struct Decimal {
    negative: bool,
    coefficient: u64,
    exponent: i32,
}

impl Decimal {
    fn parse(input: &str) -> Result<Self, InvalidCanonicalNumber> {
        if input.is_empty() || input.len() > MAX_JSON_NUMBER_TOKEN_BYTES {
            return Err(InvalidCanonicalNumber);
        }
        let bytes = input.as_bytes();
        let negative = bytes[0] == b'-';
        let start = usize::from(negative);
        let mut position = start;
        match bytes.get(position) {
            Some(b'0') => position += 1,
            Some(b'1'..=b'9') => {
                while bytes.get(position).is_some_and(u8::is_ascii_digit) {
                    position += 1;
                }
            }
            _ => return Err(InvalidCanonicalNumber),
        }
        let mut fraction_digits = 0i32;
        if bytes.get(position) == Some(&b'.') {
            position += 1;
            while bytes.get(position).is_some_and(u8::is_ascii_digit) {
                fraction_digits += 1;
                position += 1;
            }
            if fraction_digits == 0 {
                return Err(InvalidCanonicalNumber);
            }
        }
        let mantissa_end = position;
        let mut exponent_negative = false;
        let mut exponent_start = position;
        if matches!(bytes.get(position), Some(b'e' | b'E')) {
            position += 1;
            exponent_negative = bytes.get(position) == Some(&b'-');
            if matches!(bytes.get(position), Some(b'+' | b'-')) {
                position += 1;
            }
            exponent_start = position;
            while bytes.get(position).is_some_and(u8::is_ascii_digit) {
                position += 1;
            }
            if position == exponent_start {
                return Err(InvalidCanonicalNumber);
            }
        }
        if position != bytes.len() {
            return Err(InvalidCanonicalNumber);
        }

        let mut coefficient = 0u64;
        let mut significant_digits = 0;
        let mut trailing_zeros = 0i32;
        for byte in &bytes[start..mantissa_end] {
            match byte {
                b'.' => {}
                b'0' => {
                    if coefficient != 0 {
                        trailing_zeros += 1;
                    }
                }
                digit => {
                    significant_digits += trailing_zeros + 1;
                    if significant_digits > 17 {
                        return Err(InvalidCanonicalNumber);
                    }
                    for _ in 0..trailing_zeros {
                        coefficient *= 10;
                    }
                    coefficient = coefficient * 10 + u64::from(digit - b'0');
                    trailing_zeros = 0;
                }
            }
        }
        // Grammar is complete. A mathematical zero stays zero at any exponent.
        if coefficient == 0 {
            return Ok(Self {
                negative: false,
                coefficient: 0,
                exponent: 0,
            });
        }
        let mut magnitude = 0u32;
        // With no exponent, this range is empty. Leading zeros cost no range.
        for byte in &bytes[exponent_start..position] {
            magnitude = (magnitude * 10 + u32::from(byte - b'0')).min(MAX_EXPONENT_MAGNITUDE + 1);
        }
        // A nonzero exponent beyond this magnitude cannot be compensated by
        // <=4096 mantissa bytes into a finite supported nonzero number.
        if magnitude > MAX_EXPONENT_MAGNITUDE {
            return Err(InvalidCanonicalNumber);
        }
        let exponent = if exponent_negative {
            -(magnitude as i32)
        } else {
            magnitude as i32
        };
        Ok(Self {
            negative,
            coefficient,
            exponent: exponent - fraction_digits + trailing_zeros,
        })
    }
}
