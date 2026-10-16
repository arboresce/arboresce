//! Application-owned typed ports, independent of concrete effect adapters.

use std::future::Future;

use arboresce_domain::{CalendarDate, ContentRevision, Identifier};

pub mod policy;

/// An immutable synthetic record for isolated migration qualification only.
///
/// Checked identifiers establish format, not product identity or authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MigrationRecord {
    organization: Identifier,
    record: Identifier,
    revision: ContentRevision,
    date: CalendarDate,
}

impl MigrationRecord {
    /// Retain complete already checked values without ambient input or coercion.
    pub const fn new(
        organization: Identifier,
        record: Identifier,
        revision: ContentRevision,
        date: CalendarDate,
    ) -> Self {
        Self {
            organization,
            record,
            revision,
            date,
        }
    }

    pub const fn organization(self) -> Identifier {
        self.organization
    }

    pub const fn record(self) -> Identifier {
        self.record
    }

    pub const fn revision(self) -> ContentRevision {
        self.revision
    }

    pub const fn date(self) -> CalendarDate {
        self.date
    }
}

/// A write known not to have accepted the requested effect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WriteFailure {
    Unavailable,
    Conflict,
    Rejected,
}

/// A result at the actual database transaction boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WriteOutcome {
    /// The complete typed record after positive COMMIT completion.
    Committed(MigrationRecord),
    /// No write was sent, or a failed write received positive ROLLBACK completion.
    NotCommitted(WriteFailure),
    /// Completion was lost; no positive receipt or automatic retry is justified.
    Unknown,
}

/// Input-independent exact-read failure; it contains no SQL or supplied values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadFailure {
    Unavailable,
    InvalidStoredData,
}

/// Effects for the isolated synthetic harness, without infrastructure types.
pub trait MigrationStore {
    fn insert(&self, record: MigrationRecord) -> impl Future<Output = WriteOutcome> + Send;

    fn read(
        &self,
        organization: Identifier,
        record: Identifier,
    ) -> impl Future<Output = Result<Option<MigrationRecord>, ReadFailure>> + Send;
}
