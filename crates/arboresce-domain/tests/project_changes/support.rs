use std::{
    collections::hash_map::DefaultHasher,
    fmt,
    hash::{Hash, Hasher},
};

use arboresce_domain::{ContentRevision, project::*};
use serde_json::{Value, json};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Checked {
    Content(ProjectChangeContent),
    Item(ProjectChangeItem),
    Field(ProjectFieldPath),
}
pub fn inputs() -> Value {
    serde_json::from_str(include_str!("../fixtures/project-changes-v1/inputs.json")).unwrap()
}
pub fn oracle() -> Value {
    serde_json::from_str(include_str!("../fixtures/project-changes-v1/oracle.json")).unwrap()
}
pub fn semantic_inputs() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/project-changes-v1/semantic-inputs.json"
    ))
    .unwrap()
}
pub fn semantic_oracle() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/project-changes-v1/semantic-oracle.json"
    ))
    .unwrap()
}
pub fn admit(row: &Value) -> Result<Checked, InvalidProjectValue> {
    match row["api"].as_str().unwrap() {
        "change_content" => ProjectChangeContent::from_value(&row["value"]).map(Checked::Content),
        "change_item" => ProjectChangeItem::from_value(&row["value"]).map(Checked::Item),
        "field_path" => ProjectFieldPath::from_value(&row["value"]).map(Checked::Field),
        _ => panic!("unknown admitted parsed API"),
    }
}
pub fn compare(row: &Value) -> Value {
    let value = &row["value"];
    let result = match row["api"].as_str().unwrap() {
        "compare_content" => compare_content(
            &value["content"],
            &value["item_revision"],
            &value["comparison"],
        ),
        "compare_item" => compare_item(&value["item"], &value["comparison"]),
        _ => panic!("unknown admitted comparison API"),
    };
    json!({"lineage":result.lineage().as_str(),"graph":result.graph().as_str(),
        "dependency_readiness":result.dependency_readiness().as_str(),
        "reason":result.reason().map(ProjectComparisonReason::as_str)})
}
fn revision(v: ContentRevision) -> Value {
    json!({"kind":"content","value":v.to_string()})
}
fn workspace(v: WorkspaceReference) -> Value {
    json!({"organization_id":v.organization_id().to_string(),"domain_id":v.domain_id().to_string()})
}
fn project(v: ProjectReference) -> Value {
    json!({"project_id":v.project_id().to_string(),"content_revision":revision(v.content_revision())})
}
fn record(v: ProjectRecordReference) -> Value {
    json!({"kind":v.kind().as_str(),"record_id":v.record_id().to_string(),"content_revision":revision(v.content_revision())})
}
fn observation(v: ProjectObservationReference) -> Value {
    json!({"organization_id":v.organization_id().to_string(),"observation_id":v.observation_id().to_string()})
}
fn date(v: ProjectDate) -> Value {
    match v {
        ProjectDate::Date(v) => json!({"precision":"date","value":v.to_string()}),
        ProjectDate::Instant(v) => json!({"precision":"instant","value":v.to_string()}),
    }
}
fn origin(v: &ProjectChangeOrigin) -> Value {
    let capture = v.capture();
    json!({"capture":{"organization_id":capture.workspace().organization_id().to_string(),
        "domain_id":capture.workspace().domain_id().to_string(),"capture_id":capture.capture_id().to_string(),
        "version":revision(capture.version())},"analysis_id":v.analysis_id().to_string(),
        "analysis_revision":revision(v.analysis_revision()),"processing_profile_digest":v.processing_profile_digest().to_string(),
        "temporal_basis":{"event":date(v.temporal_basis().event()),"timezone":v.temporal_basis().timezone(),
            "relative_date_rule_version":v.temporal_basis().relative_date_rule_version()}})
}
fn target(v: ProjectChangeTarget) -> Value {
    let r = json!({"kind":v.kind().as_str(),"record_id":v.record_id().to_string()});
    match v.expected() {
        Some(reference) => json!({"record":r,"expected":revision(reference.content_revision())}),
        None => json!({"record":r,"absent":true}),
    }
}
fn lineage(v: &ProjectLineage) -> Value {
    match v.view() {
        ProjectLineageView::SourceObservation(v) => {
            json!({"category":"source_observation","observation":observation(v)})
        }
        ProjectLineageView::RetainedContext(v) => {
            json!({"category":"retained_context","record":record(v)})
        }
        ProjectLineageView::DeterministicDefault {
            rule_version,
            applies_to,
        } => {
            json!({"category":"deterministic_default","rule_version":rule_version,"applies_to":applies_to})
        }
        ProjectLineageView::ModelInference {
            profile,
            profile_digest,
            basis,
            alternatives,
            unresolved,
        } => {
            json!({"category":"model_inference","profile":profile,"profile_digest":profile_digest.to_string(),
                "basis":basis,"alternatives":alternatives,"unresolved":unresolved})
        }
    }
}
fn content(v: &ProjectChangeContent) -> Value {
    // The unchanged checked record-content codec owns this subprojection.
    // Every new wrapper/provenance member is read through its typed getter.
    let proposed: Value = serde_json::from_slice(v.content().as_bytes()).unwrap();
    let evidence = v.evidence();
    json!({"kind":"arboresce.project-change-content.v1","item_id":v.item_id().to_string(),
        "operation":v.operation().as_str(),"record_kind":v.record_kind().as_str(),"workspace":workspace(v.workspace()),
        "project":project(v.project()),"origin":origin(v.origin()),"target":target(v.target()),"content":proposed,
        "changed_fields":v.changed_fields().iter().map(ProjectFieldPath::as_str).collect::<Vec<_>>(),
        "statement":v.statement(),"scope":{"summary":v.scope().summary(),"applies_to":v.scope().applies_to(),"excludes":v.scope().excludes()},
        "qualifications":v.qualifications(),"evidence":{"supporting":evidence.supporting().iter().copied().map(observation).collect::<Vec<_>>(),
            "refuting":evidence.refuting().iter().copied().map(observation).collect::<Vec<_>>(),
            "qualifying":evidence.qualifying().iter().copied().map(observation).collect::<Vec<_>>()},
        "field_lineage":v.field_lineage().iter().map(|g|json!({"field":g.field().as_str(),
            "entries":g.entries().iter().map(lineage).collect::<Vec<_>>()})).collect::<Vec<_>>(),
        "unresolved":v.unresolved().iter().map(|v|json!({"field":v.field().as_str(),"requirement":v.requirement().as_str()})).collect::<Vec<_>>(),
        "dependencies":v.dependencies().iter().map(|v|match *v {
            ProjectDependency::Record(v)=>json!({"record":record(v)}),
            ProjectDependency::Item { item_id, content_revision }=>json!({"item_id":item_id.to_string(),"content_revision":revision(content_revision)})
        }).collect::<Vec<_>>()})
}
pub fn projection(value: &Checked) -> Value {
    match value {
        Checked::Content(v) => content(v),
        Checked::Field(v) => json!(v.as_str()),
        Checked::Item(v) => {
            let mut projection = content(v.content());
            projection.as_object_mut().unwrap().remove("kind");
            let candidate = v.candidate();
            projection["candidate"] = json!({"context":{"organization_id":candidate.workspace().organization_id().to_string(),
                "domain_id":candidate.workspace().domain_id().to_string(),"purpose":candidate.purpose()},
                "candidate_id":candidate.candidate_id().to_string(),"version":revision(candidate.version()),
                "content_sha256":candidate.content_sha256().to_string()});
            projection
        }
    }
}
pub fn output(value: &Checked) -> Value {
    let mut result = json!({"projection":projection(value),"debug":format!("{value:?}")});
    let (bytes, length, digest, debug) = match value {
        Checked::Content(v) => (v.as_bytes(), v.byte_length(), v.sha256(), format!("{v:?}")),
        Checked::Item(v) => (v.as_bytes(), v.byte_length(), v.sha256(), format!("{v:?}")),
        Checked::Field(v) => {
            result["debug"] = json!(format!("{v:?}"));
            return result;
        }
    };
    result["debug"] = json!(debug);
    result["canonical_utf8"] = json!(std::str::from_utf8(bytes).unwrap());
    result["canonical_bytes"] = json!(length);
    result["canonical_sha256"] = json!(digest.to_string());
    result
}
pub fn hash(v: &Checked) -> u64 {
    let mut h = DefaultHasher::new();
    v.hash(&mut h);
    h.finish()
}
pub fn event(id: &str) {
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
pub fn debug_fails(value: impl fmt::Debug) -> bool {
    use fmt::Write;
    write!(RejectWrite, "{value:?}").is_err()
}
pub fn check_debug_failure(value: &Checked) {
    let content = match value {
        Checked::Field(v) => {
            assert!(debug_fails(v));
            return;
        }
        Checked::Content(v) => {
            assert!(debug_fails(v));
            v
        }
        Checked::Item(v) => {
            assert!(debug_fails(v));
            assert!(debug_fails(v.candidate()));
            v.content()
        }
    };
    assert!(debug_fails(content.origin()));
    assert!(debug_fails(content.origin().capture()));
    assert!(debug_fails(content.origin().temporal_basis()));
    assert!(debug_fails(content.scope()));
    assert!(debug_fails(content.evidence()));
    for reference in content
        .evidence()
        .supporting()
        .iter()
        .chain(content.evidence().refuting())
        .chain(content.evidence().qualifying())
    {
        assert!(debug_fails(reference));
    }
    for group in content.field_lineage() {
        assert!(debug_fails(group));
        assert!(debug_fails(group.field()));
        for entry in group.entries() {
            assert!(debug_fails(entry));
            assert!(debug_fails(entry.view()));
        }
    }
    for unresolved in content.unresolved() {
        assert!(debug_fails(unresolved));
    }
}
