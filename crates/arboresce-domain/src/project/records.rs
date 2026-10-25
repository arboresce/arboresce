use std::fmt;

use serde_json::Value;

use crate::{CanonicalJsonSelection, Sha256Digest};

use super::{
    DecisionState, InvalidProjectValue, MAX_PROJECT_CONTENT_BYTES, PartyReference, PartyRole,
    ProjectDate, ProjectRecordKind, RelationshipSubject, ResponsibilityState, WorkItemState,
    parsed,
};

/// An immutable projection of a checked value, not a value-admission constructor.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum ProjectContentView<'a> {
    Description(&'a str),
    Context {
        summary: &'a str,
        established: bool,
    },
    Participant {
        party: PartyReference,
        role: PartyRole,
        assent_reported: bool,
    },
    Decision {
        statement: &'a str,
        state: DecisionState,
    },
    WorkItem {
        statement: &'a str,
        state: WorkItemState,
    },
    Responsibility {
        party: PartyReference,
        role: PartyRole,
        state: ResponsibilityState,
        subject: RelationshipSubject,
    },
    DueDate {
        due: ProjectDate,
        subject: RelationshipSubject,
    },
    Status(&'a str),
    Fact {
        name: &'a str,
        value: &'a str,
    },
}

impl ProjectContentView<'_> {
    pub const fn kind(self) -> ProjectRecordKind {
        match self {
            Self::Description(_) => ProjectRecordKind::Description,
            Self::Context { .. } => ProjectRecordKind::Context,
            Self::Participant { .. } => ProjectRecordKind::Participant,
            Self::Decision { .. } => ProjectRecordKind::Decision,
            Self::WorkItem { .. } => ProjectRecordKind::WorkItem,
            Self::Responsibility { .. } => ProjectRecordKind::Responsibility,
            Self::DueDate { .. } => ProjectRecordKind::DueDate,
            Self::Status(_) => ProjectRecordKind::Status,
            Self::Fact { .. } => ProjectRecordKind::Fact,
        }
    }
}

impl fmt::Debug for ProjectContentView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProjectContentView(..)")
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
enum ContentData {
    Description(String),
    Context {
        summary: String,
        established: bool,
    },
    Participant {
        party: PartyReference,
        role: PartyRole,
        assent_reported: bool,
    },
    Decision {
        statement: String,
        state: DecisionState,
    },
    WorkItem {
        statement: String,
        state: WorkItemState,
    },
    Responsibility {
        party: PartyReference,
        role: PartyRole,
        state: ResponsibilityState,
        subject: RelationshipSubject,
    },
    DueDate {
        due: ProjectDate,
        subject: RelationshipSubject,
    },
    Status(String),
    Fact {
        name: String,
        value: String,
    },
}

/// Complete typed content with owned text and complete canonical identity.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ProjectRecordContent {
    data: ContentData,
    canonical: CanonicalJsonSelection,
}

impl ProjectRecordContent {
    /// Admit the complete closed parsed shape. This establishes no state truth,
    /// Party agreement, canonical relationship, current permission or persistence.
    pub fn from_value(value: &Value) -> Result<Self, InvalidProjectValue> {
        let raw = value.as_object().ok_or(InvalidProjectValue)?;
        let kind = parsed::enumeration(raw.get("kind").ok_or(InvalidProjectValue)?)?;
        let data = match kind {
            ProjectRecordKind::Description => {
                let r = parsed::object(value, &["kind", "statement"])?;
                ContentData::Description(parsed::text(&r["statement"], 4096)?.to_owned())
            }
            ProjectRecordKind::Context => {
                let r = parsed::object(value, &["kind", "summary", "established"])?;
                ContentData::Context {
                    summary: parsed::text(&r["summary"], 4096)?.to_owned(),
                    established: parsed::boolean(&r["established"])?,
                }
            }
            ProjectRecordKind::Participant => {
                let r = parsed::object(value, &["kind", "party", "role", "assent_reported"])?;
                ContentData::Participant {
                    party: parsed::party_reference(&r["party"])?,
                    role: parsed::enumeration(&r["role"])?,
                    assent_reported: parsed::boolean(&r["assent_reported"])?,
                }
            }
            ProjectRecordKind::Decision => {
                let r = parsed::object(value, &["kind", "statement", "state"])?;
                ContentData::Decision {
                    statement: parsed::text(&r["statement"], 4096)?.to_owned(),
                    state: parsed::enumeration(&r["state"])?,
                }
            }
            ProjectRecordKind::WorkItem => {
                let r = parsed::object(value, &["kind", "statement", "state"])?;
                ContentData::WorkItem {
                    statement: parsed::text(&r["statement"], 4096)?.to_owned(),
                    state: parsed::enumeration(&r["state"])?,
                }
            }
            ProjectRecordKind::Responsibility => {
                let r = parsed::object(value, &["kind", "party", "role", "state", "subject"])?;
                ContentData::Responsibility {
                    party: parsed::party_reference(&r["party"])?,
                    role: parsed::enumeration(&r["role"])?,
                    state: parsed::enumeration(&r["state"])?,
                    subject: parsed::subject(&r["subject"])?,
                }
            }
            ProjectRecordKind::DueDate => {
                let r = parsed::object(value, &["kind", "due", "subject"])?;
                ContentData::DueDate {
                    due: parsed::date(&r["due"])?,
                    subject: parsed::subject(&r["subject"])?,
                }
            }
            ProjectRecordKind::Status => {
                let r = parsed::object(value, &["kind", "value"])?;
                ContentData::Status(parsed::slug(&r["value"])?.to_owned())
            }
            ProjectRecordKind::Fact => {
                let r = parsed::object(value, &["kind", "name", "value"])?;
                ContentData::Fact {
                    name: parsed::slug(&r["name"])?.to_owned(),
                    value: parsed::text(&r["value"], 2048)?.to_owned(),
                }
            }
        };
        let canonical = parsed::canonical(value, MAX_PROJECT_CONTENT_BYTES)?;
        Ok(Self { data, canonical })
    }

    pub fn view(&self) -> ProjectContentView<'_> {
        match &self.data {
            ContentData::Description(value) => ProjectContentView::Description(value),
            ContentData::Context {
                summary,
                established,
            } => ProjectContentView::Context {
                summary,
                established: *established,
            },
            ContentData::Participant {
                party,
                role,
                assent_reported,
            } => ProjectContentView::Participant {
                party: *party,
                role: *role,
                assent_reported: *assent_reported,
            },
            ContentData::Decision { statement, state } => ProjectContentView::Decision {
                statement,
                state: *state,
            },
            ContentData::WorkItem { statement, state } => ProjectContentView::WorkItem {
                statement,
                state: *state,
            },
            ContentData::Responsibility {
                party,
                role,
                state,
                subject,
            } => ProjectContentView::Responsibility {
                party: *party,
                role: *role,
                state: *state,
                subject: *subject,
            },
            ContentData::DueDate { due, subject } => ProjectContentView::DueDate {
                due: *due,
                subject: *subject,
            },
            ContentData::Status(value) => ProjectContentView::Status(value),
            ContentData::Fact { name, value } => ProjectContentView::Fact { name, value },
        }
    }

    pub fn kind(&self) -> ProjectRecordKind {
        self.view().kind()
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.canonical.as_bytes()
    }
    pub fn byte_length(&self) -> usize {
        self.canonical.byte_length()
    }
    pub const fn sha256(&self) -> Sha256Digest {
        self.canonical.sha256()
    }
}

impl fmt::Debug for ProjectRecordContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProjectRecordContent(..)")
    }
}
