//! Pure checked continuity projections, without retained-byte or authenticity proof.

use arboresce_domain::{ContentRevision, Identifier, RevisionError, Sha256Digest};

pub const MAX_BATCH_ENTRIES: usize = 256;

/// The closed continuity family vocabulary; payload validation is separate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Family {
    LifecycleIntent,
    LifecycleResolution,
    RecognitionIntent,
    RecognitionResolution,
    ScopeRetirementIntent,
    ScopeRetirementResolution,
}

impl Family {
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "lifecycle_intent" => Self::LifecycleIntent,
            "lifecycle_resolution" => Self::LifecycleResolution,
            "recognition_intent" => Self::RecognitionIntent,
            "recognition_resolution" => Self::RecognitionResolution,
            "scope_retirement_intent" => Self::ScopeRetirementIntent,
            "scope_retirement_resolution" => Self::ScopeRetirementResolution,
            _ => return None,
        })
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LifecycleIntent => "lifecycle_intent",
            Self::LifecycleResolution => "lifecycle_resolution",
            Self::RecognitionIntent => "recognition_intent",
            Self::RecognitionResolution => "recognition_resolution",
            Self::ScopeRetirementIntent => "scope_retirement_intent",
            Self::ScopeRetirementResolution => "scope_retirement_resolution",
        }
    }
}

/// An independent positive continuity ordinal with the common 18-digit bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ordinal(ContentRevision);

impl Ordinal {
    pub fn new(value: u64) -> Result<Self, RevisionError> {
        ContentRevision::new(value).map(Self)
    }

    pub const fn value(self) -> u64 {
        self.0.value()
    }
}

/// Checked identifier syntax does not establish authority over this namespace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Namespace {
    organization: Identifier,
    stream: Identifier,
}

impl Namespace {
    pub const fn new(organization: Identifier, stream: Identifier) -> Self {
        Self {
            organization,
            stream,
        }
    }

    pub const fn organization(self) -> Identifier {
        self.organization
    }

    pub const fn stream(self) -> Identifier {
        self.stream
    }
}

/// Only the range/link fields of a checkpoint, without encoded or signed bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckpointScope {
    pub namespace: Namespace,
    pub first: Ordinal,
    pub last: Ordinal,
    pub previous_checkpoint_sha256: Option<Sha256Digest>,
}

/// Only entry scope fields; complete identities, payloads and bindings are separate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntryScope {
    pub organization: Identifier,
    pub ordinal: Ordinal,
    pub family: Family,
}

/// Explicit caller context; neither variant authenticates a beginning or tail.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Predecessor {
    Beginning,
    After {
        namespace: Namespace,
        last_ordinal: Ordinal,
        checkpoint_sha256: Sha256Digest,
    },
}

/// A projected match grants no serving or writing capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MatchedRange {
    pub first: Ordinal,
    pub last: Ordinal,
    pub count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeFailure {
    EmptyBatch,
    BatchTooLarge,
    InvalidRange,
    CheckpointScopeMismatch,
    PredecessorScopeMismatch,
    PredecessorLinkMismatch,
    OrdinalExhausted,
    EntryScopeMismatch,
    OrdinalMismatch,
}

/// Match immutable, already typed projections without I/O or allocation.
///
/// Equal supplied digests do not prove retained bytes, trust, a complete tail,
/// binding, abort, inventory, recovery or permission to serve. Complete record
/// validation and independently authenticated continuity remain separate gates.
pub fn match_scope(
    expected_namespace: Namespace,
    predecessor: Predecessor,
    checkpoint: &CheckpointScope,
    entries: &[EntryScope],
) -> Result<MatchedRange, ScopeFailure> {
    if entries.is_empty() {
        return Err(ScopeFailure::EmptyBatch);
    }
    if entries.len() > MAX_BATCH_ENTRIES {
        return Err(ScopeFailure::BatchTooLarge);
    }
    if checkpoint.namespace != expected_namespace {
        return Err(ScopeFailure::CheckpointScopeMismatch);
    }
    let first = checkpoint.first.value();
    let last = checkpoint.last.value();
    if first > last || last - first + 1 != entries.len() as u64 {
        return Err(ScopeFailure::InvalidRange);
    }
    match predecessor {
        Predecessor::Beginning => {
            if first != 1 || checkpoint.previous_checkpoint_sha256.is_some() {
                return Err(ScopeFailure::PredecessorLinkMismatch);
            }
        }
        Predecessor::After {
            namespace,
            last_ordinal,
            checkpoint_sha256,
        } => {
            if namespace != expected_namespace {
                return Err(ScopeFailure::PredecessorScopeMismatch);
            }
            let next = last_ordinal
                .0
                .checked_next()
                .map_err(|_| ScopeFailure::OrdinalExhausted)?;
            if first != next.value()
                || checkpoint.previous_checkpoint_sha256 != Some(checkpoint_sha256)
            {
                return Err(ScopeFailure::PredecessorLinkMismatch);
            }
        }
    }
    for (index, entry) in entries.iter().enumerate() {
        if entry.organization != expected_namespace.organization {
            return Err(ScopeFailure::EntryScopeMismatch);
        }
        // The already checked inclusive range bounds every expected successor.
        if entry.ordinal.value() != first + index as u64 {
            return Err(ScopeFailure::OrdinalMismatch);
        }
    }
    Ok(MatchedRange {
        first: checkpoint.first,
        last: checkpoint.last,
        count: entries.len(),
    })
}
