//! Checked, owned Party and project-record values from parsed public values.

use std::{error::Error, fmt, str::FromStr};

use crate::{CalendarDate, ContentRevision, Identifier, UtcTimestamp};

mod parsed;
mod party;
mod records;

pub use party::{PartyAlias, PartyDetail, PartyRecord};
pub use records::{ProjectContentView, ProjectRecordContent};

/// Conservative complete encoded bounds for the closed supported shapes.
pub const MAX_PARTY_VALUE_BYTES: usize = 262_144;
pub const MAX_PROJECT_CONTENT_BYTES: usize = 32_768;

/// A rejected parsed value. No supplied data is retained.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidProjectValue;

impl fmt::Display for InvalidProjectValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid project value")
    }
}

impl Error for InvalidProjectValue {}

macro_rules! value_enum {
    ($name:ident { $($variant:ident => $wire:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub enum $name { $($variant),+ }

        impl $name {
            pub const fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $wire),+ }
            }
        }

        impl FromStr for $name {
            type Err = InvalidProjectValue;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value { $($wire => Ok(Self::$variant),)+ _ => Err(InvalidProjectValue) }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

value_enum!(PartyKind { Person => "person", ExternalBusiness => "external_business" });
value_enum!(PartyAliasStatus { Known => "known", Ambiguous => "ambiguous", Unverified => "unverified" });
value_enum!(PartyRole {
    Supplier => "supplier", Speaker => "speaker", Participant => "participant",
    Assignee => "assignee", Reviewer => "reviewer", Executor => "executor"
});
value_enum!(ProjectRecordKind {
    Description => "description", Context => "context", Participant => "participant",
    Decision => "decision", WorkItem => "work_item", Responsibility => "responsibility",
    DueDate => "due_date", Status => "status", Fact => "fact"
});
value_enum!(DecisionState {
    Proposed => "proposed", Conditional => "conditional", Agreed => "agreed",
    Rejected => "rejected", Unknown => "unknown"
});
value_enum!(WorkItemState {
    Planned => "planned", InProgress => "in_progress", Completed => "completed",
    Cancelled => "cancelled", Unknown => "unknown"
});
value_enum!(ResponsibilityState {
    Reported => "reported", ProductUpdate => "product_update", Agreed => "agreed"
});

/// A supplied reference tuple, without an existence or authorization claim.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WorkspaceReference {
    organization_id: Identifier,
    domain_id: Identifier,
}

impl WorkspaceReference {
    pub const fn new(organization_id: Identifier, domain_id: Identifier) -> Self {
        Self {
            organization_id,
            domain_id,
        }
    }
    pub const fn organization_id(self) -> Identifier {
        self.organization_id
    }
    pub const fn domain_id(self) -> Identifier {
        self.domain_id
    }
}

macro_rules! versioned_reference {
    ($name:ident, $id:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub struct $name {
            $id: Identifier,
            content_revision: ContentRevision,
        }
        impl $name {
            pub const fn new($id: Identifier, content_revision: ContentRevision) -> Self {
                Self {
                    $id,
                    content_revision,
                }
            }
            pub const fn $id(self) -> Identifier {
                self.$id
            }
            pub const fn content_revision(self) -> ContentRevision {
                self.content_revision
            }
        }
    };
}

versioned_reference!(PartyReference, party_id);
versioned_reference!(ProjectReference, project_id);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ProjectRecordReference {
    kind: ProjectRecordKind,
    record_id: Identifier,
    content_revision: ContentRevision,
}

impl ProjectRecordReference {
    pub const fn new(
        kind: ProjectRecordKind,
        record_id: Identifier,
        content_revision: ContentRevision,
    ) -> Self {
        Self {
            kind,
            record_id,
            content_revision,
        }
    }
    pub const fn kind(self) -> ProjectRecordKind {
        self.kind
    }
    pub const fn record_id(self) -> Identifier {
        self.record_id
    }
    pub const fn content_revision(self) -> ContentRevision {
        self.content_revision
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RelationshipSubject {
    Project,
    Record(ProjectRecordReference),
}

/// Exact date precision is retained; this type interprets no timezone.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ProjectDate {
    Date(CalendarDate),
    Instant(UtcTimestamp),
}
