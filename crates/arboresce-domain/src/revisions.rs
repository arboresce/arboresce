//! Independent checked revision values. These APIs parse decimal payloads,
//! not complete tagged wire objects or original JSON input.

use std::{error::Error, fmt, str::FromStr};

/// The inclusive v1 maximum for content, control and generation values.
pub const MAX_REVISION_VALUE: u64 = 999_999_999_999_999_999;

/// An input-independent checked-value failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RevisionError {
    InvalidValue,
    Exhausted,
}

impl fmt::Display for RevisionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidValue => "invalid canonical revision value",
            Self::Exhausted => "revision cannot advance",
        })
    }
}

impl Error for RevisionError {}

fn decimal(input: &str) -> Result<u64, RevisionError> {
    if input.is_empty() || input.len() > 18 || (input.len() > 1 && input.starts_with('0')) {
        return Err(RevisionError::InvalidValue);
    }
    input.bytes().try_fold(0u64, |value, byte| {
        if !byte.is_ascii_digit() {
            return Err(RevisionError::InvalidValue);
        }
        // At most eighteen ASCII digits fit exactly within u64.
        Ok(value * 10 + u64::from(byte - b'0'))
    })
}

macro_rules! revision {
    ($name:ident, $minimum:literal, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u64);

        impl $name {
            /// Construct a checked value from an already typed integer.
            pub fn new(value: u64) -> Result<Self, RevisionError> {
                if !($minimum..=MAX_REVISION_VALUE).contains(&value) {
                    return Err(RevisionError::InvalidValue);
                }
                Ok(Self(value))
            }

            /// Return the exact integer without permitting mutation.
            pub const fn value(self) -> u64 {
                self.0
            }

            /// Return an advanced copy, preserving the original even at exhaustion.
            pub fn checked_next(self) -> Result<Self, RevisionError> {
                if self.0 == MAX_REVISION_VALUE {
                    return Err(RevisionError::Exhausted);
                }
                Ok(Self(self.0 + 1))
            }
        }

        impl FromStr for $name {
            type Err = RevisionError;

            /// Parse only the canonical decimal value payload, without a wire tag.
            fn from_str(input: &str) -> Result<Self, Self::Err> {
                Self::new(decimal(input)?)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "{}", self.0)
            }
        }
    };
}

revision!(ContentRevision, 1, "An immutable checked content revision.");
revision!(
    ControlRevision,
    1,
    "An independent immutable control revision."
);
revision!(
    Generation,
    0,
    "An independent checked generation, including zero."
);
