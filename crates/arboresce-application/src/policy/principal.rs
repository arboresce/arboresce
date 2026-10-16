//! Exact checked identity lookup through a bounded, trusted directory port.
//!
//! These values establish representation and mapping only. Authentication,
//! current authority, I/O deadlines and real adapters require separate gates.

use std::{fmt, future::Future};

use arboresce_domain::Identifier;

const PROFILE: &str = "arboresce.nostr-identity.v1";
const VERSION: &str = "nip01-v1";

/// A finite lookup failure without keys, supplied values or backend causes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrincipalFailure {
    UnsupportedIdentityProfile,
    InvalidPublicKey,
    UnknownCanonicalMapping,
    CanonicalUniquenessViolation,
    Unavailable,
    InvalidStoredData,
}

impl fmt::Display for PrincipalFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, formatter)
    }
}

impl std::error::Error for PrincipalFailure {}

/// Absence is a successful empty directory result, never a backend failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectoryFailure {
    Unavailable,
    InvalidStoredData,
}

impl fmt::Display for DirectoryFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, formatter)
    }
}

impl std::error::Error for DirectoryFailure {}

/// The selected profile/version and exact 32-byte key, without key-control proof.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IdentityKey {
    public_key: [u8; 32],
}

impl IdentityKey {
    /// Parse only the selected canonical lookup representation.
    ///
    /// This does not admit a complete wire identity profile or verify a curve,
    /// signature, request, principal state or human presence.
    pub fn parse(profile: &str, version: &str, key: &str) -> Result<Self, PrincipalFailure> {
        if profile != PROFILE || version != VERSION {
            return Err(PrincipalFailure::UnsupportedIdentityProfile);
        }
        let bytes = key.as_bytes();
        if bytes.len() != 64 {
            return Err(PrincipalFailure::InvalidPublicKey);
        }
        let mut public_key = [0; 32];
        for (value, pair) in public_key.iter_mut().zip(bytes.as_chunks::<2>().0) {
            let high = nibble(pair[0]).ok_or(PrincipalFailure::InvalidPublicKey)?;
            let low = nibble(pair[1]).ok_or(PrincipalFailure::InvalidPublicKey)?;
            *value = high * 16 + low;
        }
        Ok(Self { public_key })
    }

    pub const fn profile(self) -> &'static str {
        PROFILE
    }

    pub const fn version(self) -> &'static str {
        VERSION
    }

    pub const fn public_key(self) -> [u8; 32] {
        self.public_key
    }
}

fn nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        _ => None,
    }
}

/// A checked stored observation, without a membership or permission receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StoredPrincipal {
    identity: IdentityKey,
    principal: Identifier,
}

impl StoredPrincipal {
    pub fn decode(
        profile: &str,
        version: &str,
        key: &str,
        principal: &str,
    ) -> Result<Self, DirectoryFailure> {
        let identity = IdentityKey::parse(profile, version, key)
            .map_err(|_| DirectoryFailure::InvalidStoredData)?;
        let principal = principal
            .parse()
            .map_err(|_| DirectoryFailure::InvalidStoredData)?;
        Ok(Self {
            identity,
            principal,
        })
    }

    pub const fn identity(self) -> IdentityKey {
        self.identity
    }

    pub const fn principal(self) -> Identifier {
        self.principal
    }
}

/// A backend stops after two observations and never collapses duplicate IDs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrincipalMatches {
    Missing,
    One(StoredPrincipal),
    Multiple,
}

/// A trusted exact-identity read boundary; it exposes no creation or alias API.
///
/// The actual adapter must qualify bounded I/O, canonical storage, current state
/// and its real authentication consumer before operational use. In particular,
/// this port does not select a privileged SQL/RLS lookup mode.
pub trait PrincipalDirectory: Send + Sync {
    fn lookup(
        &self,
        identity: IdentityKey,
    ) -> impl Future<Output = Result<PrincipalMatches, DirectoryFailure>> + Send;
}

/// Resolve one exact checked identity, without inferring any action authority.
pub async fn map_principal(
    directory: &impl PrincipalDirectory,
    identity: IdentityKey,
) -> Result<Identifier, PrincipalFailure> {
    let matches = directory
        .lookup(identity)
        .await
        .map_err(|failure| match failure {
            DirectoryFailure::Unavailable => PrincipalFailure::Unavailable,
            DirectoryFailure::InvalidStoredData => PrincipalFailure::InvalidStoredData,
        })?;
    match matches {
        PrincipalMatches::Missing => Err(PrincipalFailure::UnknownCanonicalMapping),
        PrincipalMatches::Multiple => Err(PrincipalFailure::CanonicalUniquenessViolation),
        PrincipalMatches::One(record) if record.identity() == identity => Ok(record.principal()),
        PrincipalMatches::One(_) => Err(PrincipalFailure::InvalidStoredData),
    }
}
