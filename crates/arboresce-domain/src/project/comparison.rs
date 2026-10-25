//! Conditional pure comparison over checked values and explicitly supplied facts.

use serde_json::Value;

use crate::{ContentRevision, Identifier, Sha256Digest};

use super::{
    InvalidProjectValue, MAX_PROJECT_COMPARISON_BYTES, PartyRecord, ProjectChangeContent,
    ProjectChangeItem, ProjectChangeOrigin, ProjectContentView, ProjectDate, ProjectDependency,
    ProjectFieldPath, ProjectLineageView, ProjectObservationKind, ProjectRecordContent,
    ProjectRecordReference, ProjectReference, WorkspaceReference, parsed,
    provenance::{self, ProjectCaptureReference, ProjectObservationReference},
};

macro_rules! codes {
    ($name:ident { $($variant:ident => $wire:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub enum $name { $($variant),+ }
        impl $name { pub const fn as_str(self) -> &'static str {
            match self { $(Self::$variant => $wire),+ }
        } }
    };
}
codes!(ProjectLineageStatus { Valid => "valid", Invalid => "invalid",
    Incomplete => "incomplete", NotChecked => "not_checked" });
codes!(ProjectGraphStatus { Acyclic => "acyclic", Cycle => "cycle", SelfEdge => "self_edge",
    Foreign => "foreign", Incomplete => "incomplete", NotChecked => "not_checked" });
codes!(ProjectDependencyReadiness { Ready => "ready", Pending => "pending", Rejected => "rejected",
    Contested => "contested", Stale => "stale", Missing => "missing", Unresolved => "unresolved",
    NotChecked => "not_checked" });
codes!(ProjectComparisonReason {
    AmbiguousField => "ambiguous_field", ComparisonOriginMismatch => "comparison_origin_mismatch",
    ComparisonScopeMismatch => "comparison_scope_mismatch", ContestedDependency => "contested_dependency",
    DefaultApplicabilityMismatch => "default_applicability_mismatch", DefaultEventPrecision => "default_event_precision",
    DefaultFieldMismatch => "default_field_mismatch", DefaultValueMismatch => "default_value_mismatch",
    DependencyCycle => "dependency_cycle", DependencySelfEdge => "dependency_self_edge",
    ForeignDependency => "foreign_dependency", ForeignParty => "foreign_party",
    InferenceContentMismatch => "inference_content_mismatch", InferenceProfileMismatch => "inference_profile_mismatch",
    InferenceScopeMismatch => "inference_scope_mismatch", InferenceValueMismatch => "inference_value_mismatch",
    InvalidChange => "invalid_change", InvalidComparison => "invalid_comparison",
    MissingChangedField => "missing_changed_field", MissingDefaultBinding => "missing_default_binding",
    MissingDependencyFact => "missing_dependency_fact", MissingDependencyFence => "missing_dependency_fence",
    MissingFieldLineage => "missing_field_lineage", MissingInferenceFact => "missing_inference_fact",
    MissingObservationFact => "missing_observation_fact", MissingObservedField => "missing_observed_field",
    MissingPartyFact => "missing_party_fact", MissingRecordFact => "missing_record_fact",
    MissingRetainedBinding => "missing_retained_binding", MissingSelectedNode => "missing_selected_node",
    RejectedDependency => "rejected_dependency", RetainedValueMismatch => "retained_value_mismatch",
    SelectedNodeMismatch => "selected_node_mismatch", SelectedRevisionMismatch => "selected_revision_mismatch",
    SourceScopeMismatch => "source_scope_mismatch", SourceValueMismatch => "source_value_mismatch",
    StaleDependency => "stale_dependency", StaleParty => "stale_party", StaleTarget => "stale_target",
    UnacceptedDependency => "unaccepted_dependency", UnchangedDeclaredField => "unchanged_declared_field",
    UnknownField => "unknown_field", UnknownRetainedField => "unknown_retained_field",
    UnresolvedFields => "unresolved_fields", UnselectedProjectDependency => "unselected_project_dependency",
    UnsupportedDefaultRule => "unsupported_default_rule"
});

/// A supplied-fact diagnostic, never permission to apply an effect.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ProjectComparisonReport {
    lineage: ProjectLineageStatus,
    graph: ProjectGraphStatus,
    dependency_readiness: ProjectDependencyReadiness,
    reason: Option<ProjectComparisonReason>,
}
impl ProjectComparisonReport {
    pub const fn lineage(self) -> ProjectLineageStatus {
        self.lineage
    }
    pub const fn graph(self) -> ProjectGraphStatus {
        self.graph
    }
    pub const fn dependency_readiness(self) -> ProjectDependencyReadiness {
        self.dependency_readiness
    }
    pub const fn reason(self) -> Option<ProjectComparisonReason> {
        self.reason
    }
    fn unchecked(reason: ProjectComparisonReason) -> Self {
        Self {
            lineage: ProjectLineageStatus::NotChecked,
            graph: ProjectGraphStatus::NotChecked,
            dependency_readiness: ProjectDependencyReadiness::NotChecked,
            reason: Some(reason),
        }
    }
    fn lineage_failure(reason: ProjectComparisonReason, incomplete: bool) -> Self {
        Self {
            lineage: if incomplete {
                ProjectLineageStatus::Incomplete
            } else {
                ProjectLineageStatus::Invalid
            },
            graph: ProjectGraphStatus::NotChecked,
            dependency_readiness: ProjectDependencyReadiness::NotChecked,
            reason: Some(reason),
        }
    }
    fn after_lineage(
        graph: ProjectGraphStatus,
        readiness: ProjectDependencyReadiness,
        reason: Option<ProjectComparisonReason>,
    ) -> Self {
        Self {
            lineage: ProjectLineageStatus::Valid,
            graph,
            dependency_readiness: readiness,
            reason,
        }
    }
}

struct RecordFact<'a> {
    raw: &'a Value,
    workspace: WorkspaceReference,
    project_id: Identifier,
    reference: ProjectRecordReference,
    current: ContentRevision,
}
struct PartyFact<'a> {
    raw: &'a Value,
    record: PartyRecord,
    current: ContentRevision,
}
struct ObservedField<'a> {
    field: ProjectFieldPath,
    value: &'a Value,
}
struct ObservationFact<'a> {
    raw: &'a Value,
    reference: ProjectObservationReference,
    capture: ProjectCaptureReference,
    fields: Vec<ObservedField<'a>>,
}
#[derive(Clone, Copy, Eq, PartialEq)]
enum NodeState {
    Pending,
    Accepted,
    Rejected,
    Contested,
    Superseded,
}
struct Node<'a> {
    raw: &'a Value,
    item_id: Identifier,
    revision: ContentRevision,
    workspace: WorkspaceReference,
    project_id: Identifier,
    dependencies: Vec<ProjectDependency>,
    state: NodeState,
    current: ContentRevision,
}
struct RetainedBinding<'a> {
    raw: &'a Value,
    field: ProjectFieldPath,
    group: usize,
    entry: usize,
    record: ProjectRecordReference,
    source_field: ProjectFieldPath,
}
struct DefaultBinding<'a> {
    raw: &'a Value,
    rule: &'a str,
    field: ProjectFieldPath,
    applies_to: &'a str,
}
struct InferenceFact<'a> {
    raw: &'a Value,
    workspace: WorkspaceReference,
    project_id: Identifier,
    field: ProjectFieldPath,
    group: usize,
    entry: usize,
    profile: &'a str,
    digest: Sha256Digest,
    basis: &'a str,
    alternatives: Vec<String>,
    unresolved: Vec<String>,
    value: &'a Value,
}
struct Context<'a> {
    workspace: WorkspaceReference,
    project: ProjectReference,
    origin: ProjectChangeOrigin,
    selected_id: Identifier,
    selected_revision: ContentRevision,
    projects: Vec<ProjectReference>,
    records: Vec<RecordFact<'a>>,
    parties: Vec<PartyFact<'a>>,
    observations: Vec<ObservationFact<'a>>,
    nodes: Vec<Node<'a>>,
    retained: Vec<RetainedBinding<'a>>,
    defaults: Vec<DefaultBinding<'a>>,
    inferences: Vec<InferenceFact<'a>>,
}

fn rows<'a, T>(
    value: &'a Value,
    maximum: usize,
    parse: impl FnMut(&'a Value) -> Result<T, InvalidProjectValue>,
) -> Result<Vec<T>, InvalidProjectValue> {
    parsed::array(value, 0, maximum)?
        .iter()
        .map(parse)
        .collect()
}
fn consistent<'a, K: Eq>(
    values: impl Iterator<Item = (K, &'a Value)>,
) -> Result<(), InvalidProjectValue> {
    let mut seen: Vec<(K, &'a Value)> = Vec::new();
    for (key, value) in values {
        if seen
            .iter()
            .any(|(k, v)| *k == key && !provenance::equal_value(v, value))
        {
            return Err(InvalidProjectValue);
        }
        seen.push((key, value));
    }
    Ok(())
}

impl<'a> Context<'a> {
    fn from_value(value: &'a Value, preimage: &Value) -> Result<Self, InvalidProjectValue> {
        provenance::encoded_size(value, MAX_PROJECT_COMPARISON_BYTES, 16)?;
        let r = parsed::object(
            value,
            &[
                "version",
                "workspace",
                "project",
                "observed_project_head",
                "origin",
                "selected_item",
                "context_projects",
                "records",
                "parties",
                "observations",
                "nodes",
                "retained_bindings",
                "default_rules",
                "inference_facts",
                "previous_candidate_content",
            ],
        )?;
        if r["version"].as_str() != Some("arboresce.project-change-comparison.v1")
            || !r["previous_candidate_content"].is_null()
        {
            return Err(InvalidProjectValue);
        }
        parsed::revision(&r["observed_project_head"])?;
        let workspace = parsed::workspace(&r["workspace"])?;
        let project = provenance::project_reference(&r["project"])?;
        let origin = ProjectChangeOrigin::from_value(&r["origin"])?;
        let selected = parsed::object(&r["selected_item"], &["item_id", "content_revision"])?;
        let selected_id = parsed::identifier(&selected["item_id"])?;
        let selected_revision = parsed::revision(&selected["content_revision"])?;
        let projects = parsed::array(&r["context_projects"], 1, 8)?
            .iter()
            .map(provenance::project_reference)
            .collect::<Result<Vec<_>, _>>()?;
        if !projects.contains(&project) {
            return Err(InvalidProjectValue);
        }
        let records = rows(&r["records"], 128, |v| {
            let r = parsed::object(
                v,
                &[
                    "workspace",
                    "project_id",
                    "record",
                    "content",
                    "current_content_revision",
                ],
            )?;
            let reference = parsed::record_reference(&r["record"])?;
            let content = ProjectRecordContent::from_value(&r["content"])?;
            if content.kind() != reference.kind() {
                return Err(InvalidProjectValue);
            }
            Ok(RecordFact {
                raw: v,
                workspace: parsed::workspace(&r["workspace"])?,
                project_id: parsed::identifier(&r["project_id"])?,
                reference,
                current: parsed::revision(&r["current_content_revision"])?,
            })
        })?;
        let parties = rows(&r["parties"], 64, |v| {
            let r = parsed::object(v, &["record", "current_content_revision"])?;
            Ok(PartyFact {
                raw: v,
                record: PartyRecord::from_value(&r["record"])?,
                current: parsed::revision(&r["current_content_revision"])?,
            })
        })?;
        let observations = rows(&r["observations"], 192, |v| {
            let r = parsed::object(v, &["reference", "capture", "kind", "observed_fields"])?;
            let reference = ProjectObservationReference::from_value(&r["reference"])?;
            let capture = ProjectCaptureReference::from_value(&r["capture"])?;
            if reference.organization_id() != capture.workspace().organization_id() {
                return Err(InvalidProjectValue);
            }
            let fields = rows(&r["observed_fields"], 64, |f| {
                let r = parsed::object(f, &["field", "value"])?;
                Ok(ObservedField {
                    field: ProjectFieldPath::from_value(&r["field"])?,
                    value: &r["value"],
                })
            })?;
            consistent(fields.iter().map(|f| (&f.field, f.value)))?;
            let _: ProjectObservationKind = parsed::enumeration(&r["kind"])?;
            Ok(ObservationFact {
                raw: v,
                reference,
                capture,
                fields,
            })
        })?;
        let nodes = rows(&r["nodes"], 64, |v| {
            let r = parsed::object(
                v,
                &[
                    "item_id",
                    "content_revision",
                    "workspace",
                    "project_id",
                    "dependencies",
                    "state",
                    "current_content_revision",
                ],
            )?;
            let state = match r["state"].as_str() {
                Some("pending") => NodeState::Pending,
                Some("accepted") => NodeState::Accepted,
                Some("rejected") => NodeState::Rejected,
                Some("contested") => NodeState::Contested,
                Some("superseded") => NodeState::Superseded,
                _ => return Err(InvalidProjectValue),
            };
            Ok(Node {
                raw: v,
                item_id: parsed::identifier(&r["item_id"])?,
                revision: parsed::revision(&r["content_revision"])?,
                workspace: parsed::workspace(&r["workspace"])?,
                project_id: parsed::identifier(&r["project_id"])?,
                state,
                current: parsed::revision(&r["current_content_revision"])?,
                dependencies: rows(&r["dependencies"], 32, ProjectDependency::from_value)?,
            })
        })?;
        let retained = rows(&r["retained_bindings"], 64, |v| {
            let r = parsed::object(
                v,
                &[
                    "field",
                    "group_index",
                    "entry_index",
                    "record",
                    "source_field",
                ],
            )?;
            Ok(RetainedBinding {
                raw: v,
                field: ProjectFieldPath::from_value(&r["field"])?,
                group: provenance::index(&r["group_index"], 63)?,
                entry: provenance::index(&r["entry_index"], 15)?,
                record: parsed::record_reference(&r["record"])?,
                source_field: ProjectFieldPath::from_value(&r["source_field"])?,
            })
        })?;
        let defaults = rows(&r["default_rules"], 16, |v| {
            let r = parsed::object(v, &["rule_version", "field", "applies_to"])?;
            Ok(DefaultBinding {
                raw: v,
                rule: parsed::slug(&r["rule_version"])?,
                field: ProjectFieldPath::from_value(&r["field"])?,
                applies_to: parsed::text(&r["applies_to"], 1024)?,
            })
        })?;
        let inferences = rows(&r["inference_facts"], 32, |v| {
            let r = parsed::object(
                v,
                &[
                    "workspace",
                    "project_id",
                    "field",
                    "group_index",
                    "entry_index",
                    "profile",
                    "profile_digest",
                    "analysis_id",
                    "analysis_revision",
                    "basis",
                    "alternatives",
                    "unresolved",
                    "value",
                ],
            )?;
            parsed::identifier(&r["analysis_id"])?;
            parsed::revision(&r["analysis_revision"])?;
            Ok(InferenceFact {
                raw: v,
                workspace: parsed::workspace(&r["workspace"])?,
                project_id: parsed::identifier(&r["project_id"])?,
                field: ProjectFieldPath::from_value(&r["field"])?,
                group: provenance::index(&r["group_index"], 63)?,
                entry: provenance::index(&r["entry_index"], 15)?,
                profile: parsed::slug(&r["profile"])?,
                digest: provenance::digest(&r["profile_digest"])?,
                basis: parsed::text(&r["basis"], 2048)?,
                alternatives: provenance::strings(&r["alternatives"], 0, 16, 1024, true)?,
                unresolved: provenance::strings(&r["unresolved"], 0, 16, 1024, true)?,
                value: &r["value"],
            })
        })?;
        consistent(records.iter().map(|v| ((v.workspace, v.reference), v.raw)))?;
        consistent(parties.iter().map(|v| {
            (
                (
                    v.record.workspace(),
                    v.record.party_id(),
                    v.record.content_revision(),
                ),
                v.raw,
            )
        }))?;
        consistent(observations.iter().map(|v| (v.reference, v.raw)))?;
        consistent(
            nodes
                .iter()
                .map(|v| ((v.workspace, v.item_id, v.revision), v.raw)),
        )?;
        consistent(retained.iter().map(|v| ((v.group, v.entry), v.raw)))?;
        consistent(defaults.iter().map(|v| {
            // Resolve binding meaning without changing its supplied spelling.
            // Unresolved unused paths retain their literal identity.
            let field = match resolve(preimage, &v.field) {
                Ok(field) => (true, field.key),
                Err(_) => (false, v.field.as_str().to_owned()),
            };
            ((field, v.rule), &v.raw["applies_to"])
        }))?;
        consistent(inferences.iter().map(|v| ((v.group, v.entry), v.raw)))?;
        Ok(Self {
            workspace,
            project,
            origin,
            selected_id,
            selected_revision,
            projects,
            records,
            parties,
            observations,
            nodes,
            retained,
            defaults,
            inferences,
        })
    }
    fn project_selected(&self, id: Identifier) -> bool {
        self.projects.iter().any(|p| p.project_id() == id)
    }
    fn record(&self, reference: ProjectRecordReference) -> Option<&RecordFact<'a>> {
        self.records
            .iter()
            .find(|r| r.reference == reference && r.workspace == self.workspace)
            .or_else(|| self.records.iter().find(|r| r.reference == reference))
    }
    fn node(&self, id: Identifier, revision: ContentRevision) -> Option<&Node<'a>> {
        self.nodes
            .iter()
            .find(|r| r.item_id == id && r.revision == revision && r.workspace == self.workspace)
            .or_else(|| {
                self.nodes
                    .iter()
                    .find(|r| r.item_id == id && r.revision == revision)
            })
    }
}

struct ResolvedField<'a> {
    key: String,
    value: &'a Value,
}
fn member<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.')
        .try_fold(value, |value, part| value.as_object()?.get(part))
}
fn review_member(path: &str) -> bool {
    matches!(
        path.split('.').next(),
        Some("statement" | "scope" | "qualifications" | "evidence")
    )
}
fn resolve<'a>(
    preimage: &'a Value,
    field: &ProjectFieldPath,
) -> Result<ResolvedField<'a>, ProjectComparisonReason> {
    use ProjectComparisonReason::{AmbiguousField, UnknownField};
    let path = field.as_str();
    if let Some(path) = path.strip_prefix("content.") {
        return member(&preimage["content"], path)
            .map(|value| ResolvedField {
                key: format!("content.{path}"),
                value,
            })
            .ok_or(UnknownField);
    }
    if let Some(path) = path.strip_prefix("candidate.") {
        return if review_member(path) {
            member(preimage, path)
                .map(|value| ResolvedField {
                    key: format!("candidate.{path}"),
                    value,
                })
                .ok_or(UnknownField)
        } else {
            Err(UnknownField)
        };
    }
    let content = member(&preimage["content"], path);
    let candidate = if review_member(path) {
        member(preimage, path)
    } else {
        None
    };
    match (content, candidate) {
        (Some(_), Some(_)) => Err(AmbiguousField),
        (Some(value), None) => Ok(ResolvedField {
            key: format!("content.{path}"),
            value,
        }),
        (None, Some(value)) => Ok(ResolvedField {
            key: format!("candidate.{path}"),
            value,
        }),
        (None, None) => Err(UnknownField),
    }
}
fn covers(parent: &str, child: &str) -> bool {
    parent == child
        || child
            .strip_prefix(parent)
            .is_some_and(|rest| rest.starts_with('.'))
}
fn leaves(value: &Value, path: &str, result: &mut Vec<String>) {
    if let Value::Object(object) = value {
        for (key, value) in object {
            if path == "content" && key == "kind" {
                continue;
            }
            leaves(value, &format!("{path}.{key}"), result);
        }
    } else {
        result.push(path.to_owned());
    }
}
fn changed_leaves(new: &Value, old: &Value, path: &str, result: &mut Vec<String>) {
    if provenance::equal_value(new, old) {
        return;
    }
    if let (Value::Object(new), Value::Object(old)) = (new, old) {
        for (key, new) in new {
            if path == "content" && key == "kind" {
                continue;
            }
            let path = format!("{path}.{key}");
            match old.get(key) {
                Some(old) => changed_leaves(new, old, &path, result),
                None => leaves(new, &path, result),
            }
        }
        for key in old.keys() {
            if !new.contains_key(key) && (path != "content" || key != "kind") {
                result.push(format!("{path}.{key}"));
            }
        }
    } else {
        result.push(path.to_owned());
    }
}

type LineageFailure = (ProjectComparisonReason, bool);
fn invalid(reason: ProjectComparisonReason) -> LineageFailure {
    (reason, false)
}
fn incomplete(reason: ProjectComparisonReason) -> LineageFailure {
    (reason, true)
}

fn check_fields<'a>(
    content: &ProjectChangeContent,
    preimage: &'a Value,
    base: Option<&Value>,
) -> Result<Vec<ResolvedField<'a>>, LineageFailure> {
    use ProjectComparisonReason as R;
    let changed = content
        .changed_fields()
        .iter()
        .map(|f| resolve(preimage, f).map_err(invalid))
        .collect::<Result<Vec<_>, _>>()?;
    let mut required_changes = Vec::new();
    if let Some(base) = base {
        changed_leaves(&preimage["content"], base, "content", &mut required_changes);
        for field in &changed {
            if let Some(path) = field.key.strip_prefix("content.")
                && member(base, path).is_some_and(|old| provenance::equal_value(old, field.value))
            {
                return Err(invalid(R::UnchangedDeclaredField));
            }
        }
    } else {
        leaves(&preimage["content"], "content", &mut required_changes);
    }
    if required_changes
        .iter()
        .any(|path| !changed.iter().any(|f| covers(&f.key, path)))
    {
        return Err(invalid(R::MissingChangedField));
    }
    let groups = content
        .field_lineage()
        .iter()
        .map(|f| resolve(preimage, f.field()).map_err(invalid))
        .collect::<Result<Vec<_>, _>>()?;
    let mut required_lineage = Vec::new();
    leaves(&preimage["content"], "content", &mut required_lineage);
    required_lineage.extend(
        changed
            .iter()
            .filter(|f| f.key.starts_with("candidate."))
            .map(|f| f.key.clone()),
    );
    if required_lineage
        .iter()
        .any(|path| !groups.iter().any(|f| covers(&f.key, path)))
    {
        return Err(invalid(R::MissingFieldLineage));
    }
    for unresolved in content.unresolved() {
        resolve(preimage, unresolved.field()).map_err(invalid)?;
    }
    Ok(groups)
}

fn same_field(preimage: &Value, field: &ProjectFieldPath, key: &str) -> bool {
    resolve(preimage, field).is_ok_and(|f| f.key == key)
}
fn check_lineage(
    content: &ProjectChangeContent,
    preimage: &Value,
    context: &Context<'_>,
    groups: &[ResolvedField<'_>],
) -> Result<(), LineageFailure> {
    use ProjectComparisonReason as R;
    for (group_index, (group, selected)) in content.field_lineage().iter().zip(groups).enumerate() {
        for (entry_index, entry) in group.entries().iter().enumerate() {
            match entry.view() {
                ProjectLineageView::SourceObservation(reference) => {
                    let fact = context
                        .observations
                        .iter()
                        .find(|f| f.reference == reference)
                        .ok_or_else(|| incomplete(R::MissingObservationFact))?;
                    if fact.capture.workspace() != content.workspace() {
                        return Err(invalid(R::SourceScopeMismatch));
                    }
                    let mut found = false;
                    for source in &fact.fields {
                        let Ok(field) = resolve(preimage, &source.field) else {
                            continue;
                        };
                        if !covers(&field.key, &selected.key) {
                            continue;
                        }
                        let value = if field.key == selected.key {
                            Some(source.value)
                        } else {
                            member(source.value, &selected.key[field.key.len() + 1..])
                        };
                        if let Some(value) = value {
                            found = true;
                            if !provenance::equal_value(value, selected.value) {
                                return Err(invalid(R::SourceValueMismatch));
                            }
                        }
                    }
                    if !found {
                        return Err(incomplete(R::MissingObservedField));
                    }
                }
                ProjectLineageView::RetainedContext(reference) => {
                    let binding = context
                        .retained
                        .iter()
                        .find(|f| {
                            f.group == group_index
                                && f.entry == entry_index
                                && f.record == reference
                                && same_field(preimage, &f.field, &selected.key)
                        })
                        .ok_or_else(|| incomplete(R::MissingRetainedBinding))?;
                    let fact = context
                        .record(reference)
                        .ok_or_else(|| incomplete(R::MissingRecordFact))?;
                    if fact.workspace != content.workspace()
                        || !context.project_selected(fact.project_id)
                    {
                        return Err(invalid(R::ForeignDependency));
                    }
                    let path = binding
                        .source_field
                        .as_str()
                        .strip_prefix("content.")
                        .unwrap_or(binding.source_field.as_str());
                    let value = member(&fact.raw["content"], path)
                        .ok_or_else(|| invalid(R::UnknownRetainedField))?;
                    if !provenance::equal_value(value, selected.value) {
                        return Err(invalid(R::RetainedValueMismatch));
                    }
                }
                ProjectLineageView::DeterministicDefault {
                    rule_version,
                    applies_to,
                } => {
                    if rule_version != "event_date_v1" {
                        return Err(invalid(R::UnsupportedDefaultRule));
                    }
                    if !matches!(
                        selected.key.as_str(),
                        "content.due" | "content.due.value" | "content.due.precision"
                    ) {
                        return Err(invalid(R::DefaultFieldMismatch));
                    }
                    let binding = context
                        .defaults
                        .iter()
                        .find(|f| {
                            f.rule == rule_version && same_field(preimage, &f.field, &selected.key)
                        })
                        .ok_or_else(|| incomplete(R::MissingDefaultBinding))?;
                    if binding.applies_to != applies_to {
                        return Err(invalid(R::DefaultApplicabilityMismatch));
                    }
                    let ProjectDate::Date(event) = content.origin().temporal_basis().event() else {
                        return Err(invalid(R::DefaultEventPrecision));
                    };
                    if !matches!(content.content().view(), ProjectContentView::DueDate { due: ProjectDate::Date(date), .. } if date == event)
                    {
                        return Err(invalid(R::DefaultValueMismatch));
                    }
                }
                ProjectLineageView::ModelInference {
                    profile,
                    profile_digest,
                    basis,
                    alternatives,
                    unresolved,
                } => {
                    let fact = context
                        .inferences
                        .iter()
                        .find(|f| {
                            f.group == group_index
                                && f.entry == entry_index
                                && same_field(preimage, &f.field, &selected.key)
                        })
                        .ok_or_else(|| incomplete(R::MissingInferenceFact))?;
                    if fact.workspace != content.workspace()
                        || fact.project_id != content.project().project_id()
                    {
                        return Err(invalid(R::InferenceScopeMismatch));
                    }
                    if fact.profile != profile || fact.digest != profile_digest {
                        return Err(invalid(R::InferenceProfileMismatch));
                    }
                    if fact.basis != basis
                        || fact.alternatives != alternatives
                        || fact.unresolved != unresolved
                    {
                        return Err(invalid(R::InferenceContentMismatch));
                    }
                    if !provenance::equal_value(fact.value, selected.value) {
                        return Err(invalid(R::InferenceValueMismatch));
                    }
                }
            }
        }
    }
    Ok(())
}

fn party_stale(
    content: &ProjectChangeContent,
    context: &Context<'_>,
) -> Result<bool, LineageFailure> {
    use ProjectComparisonReason as R;
    let reference = match content.content().view() {
        ProjectContentView::Participant { party, .. }
        | ProjectContentView::Responsibility { party, .. } => party,
        _ => return Ok(false),
    };
    let matches = |f: &&PartyFact<'_>| {
        f.record.party_id() == reference.party_id()
            && f.record.content_revision() == reference.content_revision()
    };
    let fact = context
        .parties
        .iter()
        .filter(matches)
        .find(|f| f.record.workspace() == content.workspace())
        .or_else(|| context.parties.iter().find(matches))
        .ok_or_else(|| incomplete(R::MissingPartyFact))?;
    if fact.record.workspace() != content.workspace() {
        return Err(invalid(R::ForeignParty));
    }
    Ok(fact.current != reference.content_revision())
}

fn complete_fences(content: &ProjectChangeContent) -> bool {
    let has = |reference| {
        content
            .dependencies()
            .contains(&ProjectDependency::Record(reference))
    };
    for group in content.field_lineage() {
        for entry in group.entries() {
            if let ProjectLineageView::RetainedContext(reference) = entry.view()
                && !has(reference)
            {
                return false;
            }
        }
    }
    let subject = match content.content().view() {
        ProjectContentView::Responsibility { subject, .. }
        | ProjectContentView::DueDate { subject, .. } => subject,
        _ => return true,
    };
    match subject {
        super::RelationshipSubject::Project => true,
        super::RelationshipSubject::Record(reference) => has(reference),
    }
}

fn graph_failure(
    graph: ProjectGraphStatus,
    reason: ProjectComparisonReason,
    missing: bool,
) -> ProjectComparisonReport {
    ProjectComparisonReport::after_lineage(
        graph,
        if missing {
            ProjectDependencyReadiness::Missing
        } else {
            ProjectDependencyReadiness::NotChecked
        },
        Some(reason),
    )
}
type NodeKey = (Identifier, ContentRevision);
fn graph(
    context: &Context<'_>,
    selected: NodeKey,
) -> Result<Vec<ProjectDependency>, ProjectComparisonReport> {
    use ProjectComparisonReason as R;
    use ProjectGraphStatus as G;
    fn visit(
        context: &Context<'_>,
        key: NodeKey,
        colors: &mut Vec<(NodeKey, u8)>,
        closure: &mut Vec<ProjectDependency>,
    ) -> Result<(), ProjectComparisonReport> {
        use ProjectComparisonReason as R;
        use ProjectGraphStatus as G;
        match colors
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, color)| *color)
        {
            Some(1) => return Err(graph_failure(G::Cycle, R::DependencyCycle, false)),
            Some(2) => return Ok(()),
            _ => {}
        }
        let node = context
            .node(key.0, key.1)
            .ok_or_else(|| graph_failure(G::Incomplete, R::MissingDependencyFact, true))?;
        if node.workspace != context.workspace {
            return Err(graph_failure(G::Foreign, R::ForeignDependency, false));
        }
        if !context.project_selected(node.project_id) {
            return Err(graph_failure(
                G::Foreign,
                R::UnselectedProjectDependency,
                false,
            ));
        }
        let slot = colors.len();
        colors.push((key, 1));
        for dependency in &node.dependencies {
            closure.push(*dependency);
            match *dependency {
                ProjectDependency::Item {
                    item_id,
                    content_revision,
                } => {
                    if (item_id, content_revision) == key {
                        return Err(graph_failure(G::SelfEdge, R::DependencySelfEdge, false));
                    }
                    visit(context, (item_id, content_revision), colors, closure)?;
                }
                ProjectDependency::Record(reference) => {
                    let fact = context.record(reference).ok_or_else(|| {
                        graph_failure(G::Incomplete, R::MissingDependencyFact, true)
                    })?;
                    if fact.workspace != context.workspace {
                        return Err(graph_failure(G::Foreign, R::ForeignDependency, false));
                    }
                    if !context.project_selected(fact.project_id) {
                        return Err(graph_failure(
                            G::Foreign,
                            R::UnselectedProjectDependency,
                            false,
                        ));
                    }
                }
            }
        }
        colors[slot].1 = 2;
        Ok(())
    }
    if context.node(selected.0, selected.1).is_none() {
        return Err(graph_failure(G::Incomplete, R::MissingSelectedNode, true));
    }
    let mut closure = Vec::new();
    visit(context, selected, &mut Vec::new(), &mut closure)?;
    Ok(closure)
}

fn readiness(
    content: &ProjectChangeContent,
    context: &Context<'_>,
    closure: &[ProjectDependency],
    stale_party: bool,
) -> ProjectComparisonReport {
    use ProjectComparisonReason as R;
    use ProjectDependencyReadiness as D;
    let report = |readiness, reason| {
        ProjectComparisonReport::after_lineage(ProjectGraphStatus::Acyclic, readiness, reason)
    };
    if let Some(target) = content.target().expected()
        && context
            .record(target)
            .is_some_and(|f| f.current != target.content_revision())
    {
        return report(D::Stale, Some(R::StaleTarget));
    }
    if stale_party {
        return report(D::Stale, Some(R::StaleParty));
    }
    for dependency in closure {
        match *dependency {
            ProjectDependency::Record(reference) => {
                if context
                    .record(reference)
                    .is_some_and(|f| f.current != reference.content_revision())
                {
                    return report(D::Stale, Some(R::StaleDependency));
                }
            }
            ProjectDependency::Item {
                item_id,
                content_revision,
            } => {
                if let Some(node) = context.node(item_id, content_revision) {
                    if node.current != content_revision {
                        return report(D::Stale, Some(R::StaleDependency));
                    }
                    match node.state {
                        NodeState::Pending => {
                            return report(D::Pending, Some(R::UnacceptedDependency));
                        }
                        NodeState::Rejected => {
                            return report(D::Rejected, Some(R::RejectedDependency));
                        }
                        NodeState::Contested => {
                            return report(D::Contested, Some(R::ContestedDependency));
                        }
                        NodeState::Superseded => return report(D::Stale, Some(R::StaleDependency)),
                        NodeState::Accepted => {}
                    }
                }
            }
        }
    }
    if !content.unresolved().is_empty() {
        return report(D::Unresolved, Some(R::UnresolvedFields));
    }
    report(D::Ready, None)
}

fn compare_checked(
    content: &ProjectChangeContent,
    revision: ContentRevision,
    supplied: &Value,
) -> ProjectComparisonReport {
    use ProjectComparisonReason as R;
    let preimage: Value = match serde_json::from_slice(content.as_bytes()) {
        Ok(value) => value,
        Err(_) => return ProjectComparisonReport::unchecked(R::InvalidChange),
    };
    let context = match Context::from_value(supplied, &preimage) {
        Ok(context) => context,
        Err(_) => return ProjectComparisonReport::unchecked(R::InvalidComparison),
    };
    if context.workspace != content.workspace() || context.project != content.project() {
        return ProjectComparisonReport::unchecked(R::ComparisonScopeMismatch);
    }
    if context.origin != *content.origin() {
        return ProjectComparisonReport::unchecked(R::ComparisonOriginMismatch);
    }
    if context.selected_revision != revision {
        return ProjectComparisonReport::unchecked(R::SelectedRevisionMismatch);
    }
    if context.selected_id != content.item_id() {
        return ProjectComparisonReport::unchecked(R::SelectedNodeMismatch);
    }
    if let Some(node) = context.node(content.item_id(), revision)
        && (node.workspace != content.workspace()
            || node.project_id != content.project().project_id()
            || node.dependencies != content.dependencies())
    {
        return ProjectComparisonReport::unchecked(R::SelectedNodeMismatch);
    }
    // Bindings name occurrences in this selected immutable proposal, not hidden
    // records or a guessed other candidate. General counter bounds were checked.
    let occurrence = |group: usize, entry: usize| {
        content
            .field_lineage()
            .get(group)
            .is_some_and(|g| entry < g.entries().len())
    };
    if context
        .retained
        .iter()
        .any(|f| !occurrence(f.group, f.entry))
        || context
            .inferences
            .iter()
            .any(|f| !occurrence(f.group, f.entry))
    {
        return ProjectComparisonReport::unchecked(R::InvalidComparison);
    }
    let base = if let Some(target) = content.target().expected() {
        let Some(fact) = context.record(target) else {
            return ProjectComparisonReport::lineage_failure(R::MissingRecordFact, true);
        };
        if fact.workspace != content.workspace()
            || fact.project_id != content.project().project_id()
        {
            return ProjectComparisonReport::lineage_failure(R::ForeignDependency, false);
        }
        Some(&fact.raw["content"])
    } else {
        None
    };
    let lineage = || -> Result<bool, LineageFailure> {
        let groups = check_fields(content, &preimage, base)?;
        check_lineage(content, &preimage, &context, &groups)?;
        party_stale(content, &context)
    };
    let stale_party = match lineage() {
        Ok(stale) => stale,
        Err((reason, missing)) => return ProjectComparisonReport::lineage_failure(reason, missing),
    };
    if !complete_fences(content) {
        return graph_failure(
            ProjectGraphStatus::NotChecked,
            R::MissingDependencyFence,
            true,
        );
    }
    match graph(&context, (content.item_id(), revision)) {
        Ok(closure) => readiness(content, &context, &closure, stale_party),
        Err(report) => report,
    }
}

/// Compare one selected parsed content value and explicit tagged revision against
/// supplied facts. This does not read canonical state or authorize an effect.
pub fn compare_content(
    value: &Value,
    item_revision: &Value,
    comparison: &Value,
) -> ProjectComparisonReport {
    let content = match ProjectChangeContent::from_value(value) {
        Ok(value) => value,
        Err(_) => {
            return ProjectComparisonReport::unchecked(ProjectComparisonReason::InvalidChange);
        }
    };
    let revision = match parsed::revision(item_revision) {
        Ok(value) => value,
        Err(_) => {
            return ProjectComparisonReport::unchecked(ProjectComparisonReason::InvalidComparison);
        }
    };
    compare_checked(&content, revision, comparison)
}

/// Compare one selected exact item using its candidate content revision.
pub fn compare_item(value: &Value, comparison: &Value) -> ProjectComparisonReport {
    match ProjectChangeItem::from_value(value) {
        Ok(item) => compare_checked(item.content(), item.candidate().version(), comparison),
        Err(_) => ProjectComparisonReport::unchecked(ProjectComparisonReason::InvalidChange),
    }
}
