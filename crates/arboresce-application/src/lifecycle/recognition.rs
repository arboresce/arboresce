//! Bounded recognition mathematics, without command admission or authority.

use std::{error::Error, fmt};

use arboresce_domain::{Identifier, Sha256Digest};
use sha2::{Digest, Sha256};

/// A rejected input; no supplied key or identity is retained.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidRecognitionInput;

impl fmt::Display for InvalidRecognitionInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid recognition input")
    }
}

impl Error for InvalidRecognitionInput {}

/// A mathematical scoped reference, with no retained key or identity preimage.
///
/// This value proves neither a registered command, its admitted key relation,
/// current authorization, durable recognition nor permission to perform effects.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct RecognitionReference {
    organization_id: Identifier,
    principal_id: Identifier,
    recognition_sha256: Sha256Digest,
}

impl RecognitionReference {
    pub const fn organization_id(self) -> Identifier {
        self.organization_id
    }

    pub const fn principal_id(self) -> Identifier {
        self.principal_id
    }

    pub const fn recognition_sha256(self) -> Sha256Digest {
        self.recognition_sha256
    }
}

impl fmt::Debug for RecognitionReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RecognitionReference(<redacted>)")
    }
}

/// Hash exactly a 32..128-byte ASCII key from A-Z, a-z, 0-9, '_' and '-'.
///
/// There is no normalization, decoding, terminator or domain prefix. Borrowing
/// and finite diagnostics do not establish secure erasure of caller memory.
pub fn key_digest(key: &str) -> Result<Sha256Digest, InvalidRecognitionInput> {
    if !(32..=128).contains(&key.len())
        || !key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(InvalidRecognitionInput);
    }
    Ok(Sha256Digest::from_bytes(
        Sha256::digest(key.as_bytes()).into(),
    ))
}

/// Compute the domain-separated hash of the four checked identity strings.
///
/// Command checking establishes only common command-name syntax. Registered
/// admission, the actual key relation and all effect fences remain mandatory
/// at the first consuming operation. A typed digest alone establishes no key.
pub fn recognition_reference(
    organization_id: Identifier,
    principal_id: Identifier,
    command: &str,
    key_digest: Sha256Digest,
) -> Result<RecognitionReference, InvalidRecognitionInput> {
    if !(3..=64).contains(&command.len())
        || !command.contains('.')
        || !command.split('.').all(|component| {
            let mut bytes = component.bytes();
            bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
                && bytes
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        })
    {
        return Err(InvalidRecognitionInput);
    }
    // All four values are checked ASCII strings without escaping characters.
    // These sorted keys and their exact string values are complete JCS for
    // this fixed type; this serializer does not implement general JSON JCS.
    let identity = format!(
        "{{\"command\":\"{command}\",\"key_digest\":\"{key_digest}\",\"organization_id\":\"{organization_id}\",\"principal_id\":\"{principal_id}\"}}"
    );
    let mut hash = Sha256::new();
    hash.update(b"arboresce.command-recognition.v1\0");
    hash.update(identity.as_bytes());
    Ok(RecognitionReference {
        organization_id,
        principal_id,
        recognition_sha256: Sha256Digest::from_bytes(hash.finalize().into()),
    })
}
