use std::{
    collections::hash_map::DefaultHasher,
    fmt,
    hash::{Hash, Hasher},
};

use arboresce_domain::{ContentRevision, project::*};
use serde_json::{Value, json};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum CheckedValue {
    Party(PartyRecord),
    Content(ProjectRecordContent),
}

pub fn inputs() -> Value {
    serde_json::from_str(include_str!("../fixtures/project-values-v1/inputs.json")).unwrap()
}

pub fn oracle() -> Value {
    serde_json::from_str(include_str!("../fixtures/project-values-v1/oracle.json")).unwrap()
}

pub fn admit(row: &Value) -> Result<CheckedValue, InvalidProjectValue> {
    match row["api"].as_str().unwrap() {
        "party" => PartyRecord::from_value(&row["value"]).map(CheckedValue::Party),
        "content" => ProjectRecordContent::from_value(&row["value"]).map(CheckedValue::Content),
        _ => panic!("unknown frozen value API"),
    }
}

fn revision(value: ContentRevision) -> Value {
    json!({"kind":"content","value":value.to_string()})
}

fn party_reference(value: PartyReference) -> Value {
    json!({"party_id":value.party_id().to_string(),"content_revision":revision(value.content_revision())})
}

fn subject(value: RelationshipSubject) -> Value {
    match value {
        RelationshipSubject::Project => json!({"scope":"project"}),
        RelationshipSubject::Record(value) => json!({"record":{
            "kind":value.kind().to_string(),"record_id":value.record_id().to_string(),
            "content_revision":revision(value.content_revision())}}),
    }
}

pub fn projection(value: &CheckedValue) -> Value {
    match value {
        CheckedValue::Party(value) => json!({
            "workspace":{"organization_id":value.workspace().organization_id().to_string(),"domain_id":value.workspace().domain_id().to_string()},
            "party_id":value.party_id().to_string(),"content_revision":revision(value.content_revision()),"kind":value.kind().to_string(),
            "aliases":value.aliases().iter().map(|v|json!({"value":v.value(),"status":v.status().to_string()})).collect::<Vec<_>>(),
            "details":value.details().iter().map(|v|json!({"kind":v.kind(),"value":v.value(),"status":v.status().to_string()})).collect::<Vec<_>>()
        }),
        CheckedValue::Content(value) => {
            let kind = value.kind().to_string();
            match value.view() {
                ProjectContentView::Description(statement) => {
                    json!({"kind":kind,"statement":statement})
                }
                ProjectContentView::Context {
                    summary,
                    established,
                } => json!({"kind":kind,"summary":summary,"established":established}),
                ProjectContentView::Participant {
                    party,
                    role,
                    assent_reported,
                } => {
                    json!({"kind":kind,"party":party_reference(party),"role":role.to_string(),"assent_reported":assent_reported})
                }
                ProjectContentView::Decision { statement, state } => {
                    json!({"kind":kind,"statement":statement,"state":state.to_string()})
                }
                ProjectContentView::WorkItem { statement, state } => {
                    json!({"kind":kind,"statement":statement,"state":state.to_string()})
                }
                ProjectContentView::Responsibility {
                    party,
                    role,
                    state,
                    subject: target,
                } => {
                    json!({"kind":kind,"party":party_reference(party),"role":role.to_string(),"state":state.to_string(),"subject":subject(target)})
                }
                ProjectContentView::DueDate {
                    due,
                    subject: target,
                } => {
                    let due = match due {
                        ProjectDate::Date(date) => {
                            json!({"precision":"date","value":date.to_string()})
                        }
                        ProjectDate::Instant(instant) => {
                            json!({"precision":"instant","value":instant.to_string()})
                        }
                    };
                    json!({"kind":kind,"due":due,"subject":subject(target)})
                }
                ProjectContentView::Status(status) => json!({"kind":kind,"value":status}),
                ProjectContentView::Fact { name, value } => {
                    json!({"kind":kind,"name":name,"value":value})
                }
            }
        }
    }
}

pub fn output(value: &CheckedValue) -> Value {
    let (bytes, length, digest, debug) = match value {
        CheckedValue::Party(v) => (v.as_bytes(), v.byte_length(), v.sha256(), format!("{v:?}")),
        CheckedValue::Content(v) => (v.as_bytes(), v.byte_length(), v.sha256(), format!("{v:?}")),
    };
    json!({"typed_projection":projection(value),"canonical_utf8":std::str::from_utf8(bytes).unwrap(),"canonical_bytes":length,"sha256":digest.to_string(),"debug":debug})
}

pub fn hash(value: &CheckedValue) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

pub fn case_event(id: &str) {
    println!("collector case={id} pid={}", std::process::id());
}

struct RejectWrite;
impl fmt::Write for RejectWrite {
    fn write_str(&mut self, _: &str) -> fmt::Result {
        Err(fmt::Error)
    }
}

pub fn formatting_fails(value: impl fmt::Display) -> bool {
    use fmt::Write;
    write!(RejectWrite, "{value}").is_err()
}
