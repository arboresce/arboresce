use std::fmt;

use serde_json::Value;

use crate::{CanonicalJsonSelection, ContentRevision, Identifier, Sha256Digest};

use super::{
    InvalidProjectValue, MAX_PROJECT_CHANGE_BYTES, MAX_PROJECT_CHANGE_ITEM_BYTES,
    ProjectChangeOperation, ProjectChangeOrigin, ProjectDependency, ProjectEvidence,
    ProjectFieldLineage, ProjectFieldPath, ProjectRecordContent, ProjectRecordKind,
    ProjectRecordReference, ProjectReference, ProjectReviewScope, ProjectUnresolved,
    WorkspaceReference, parsed, provenance,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ProjectChangeTarget {
    Absent {
        kind: ProjectRecordKind,
        record_id: Identifier,
    },
    Expected(ProjectRecordReference),
}

impl ProjectChangeTarget {
    fn from_value(value: &Value) -> Result<Self, InvalidProjectValue> {
        let raw = value.as_object().ok_or(InvalidProjectValue)?;
        let absent = raw.contains_key("absent");
        let r = parsed::object(
            value,
            if absent {
                &["record", "absent"]
            } else {
                &["record", "expected"]
            },
        )?;
        let target = parsed::object(&r["record"], &["kind", "record_id"])?;
        let kind = parsed::enumeration(&target["kind"])?;
        let record_id = parsed::identifier(&target["record_id"])?;
        if absent {
            if !parsed::boolean(&r["absent"])? {
                return Err(InvalidProjectValue);
            }
            Ok(Self::Absent { kind, record_id })
        } else {
            Ok(Self::Expected(ProjectRecordReference::new(
                kind,
                record_id,
                parsed::revision(&r["expected"])?,
            )))
        }
    }
    pub const fn kind(self) -> ProjectRecordKind {
        match self {
            Self::Absent { kind, .. } => kind,
            Self::Expected(v) => v.kind(),
        }
    }
    pub const fn record_id(self) -> Identifier {
        match self {
            Self::Absent { record_id, .. } => record_id,
            Self::Expected(v) => v.record_id(),
        }
    }
    pub const fn expected(self) -> Option<ProjectRecordReference> {
        match self {
            Self::Absent { .. } => None,
            Self::Expected(v) => Some(v),
        }
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ProjectCandidateReference {
    workspace: WorkspaceReference,
    purpose: String,
    candidate_id: Identifier,
    version: ContentRevision,
    content_sha256: Sha256Digest,
}

impl ProjectCandidateReference {
    fn from_value(value: &Value) -> Result<Self, InvalidProjectValue> {
        let r = parsed::object(
            value,
            &["context", "candidate_id", "version", "content_sha256"],
        )?;
        let context = parsed::object(&r["context"], &["organization_id", "domain_id", "purpose"])?;
        Ok(Self {
            workspace: WorkspaceReference::new(
                parsed::identifier(&context["organization_id"])?,
                parsed::identifier(&context["domain_id"])?,
            ),
            purpose: parsed::slug(&context["purpose"])?.to_owned(),
            candidate_id: parsed::identifier(&r["candidate_id"])?,
            version: parsed::revision(&r["version"])?,
            content_sha256: provenance::digest(&r["content_sha256"])?,
        })
    }
    pub const fn workspace(&self) -> WorkspaceReference {
        self.workspace
    }
    pub fn purpose(&self) -> &str {
        &self.purpose
    }
    pub const fn candidate_id(&self) -> Identifier {
        self.candidate_id
    }
    pub const fn version(&self) -> ContentRevision {
        self.version
    }
    pub const fn content_sha256(&self) -> Sha256Digest {
        self.content_sha256
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ProjectChangeContent {
    item_id: Identifier,
    operation: ProjectChangeOperation,
    workspace: WorkspaceReference,
    project: ProjectReference,
    origin: ProjectChangeOrigin,
    target: ProjectChangeTarget,
    content: ProjectRecordContent,
    changed_fields: Vec<ProjectFieldPath>,
    statement: String,
    scope: ProjectReviewScope,
    qualifications: Vec<String>,
    evidence: ProjectEvidence,
    field_lineage: Vec<ProjectFieldLineage>,
    unresolved: Vec<ProjectUnresolved>,
    dependencies: Vec<ProjectDependency>,
    canonical: CanonicalJsonSelection,
}

const CONTENT_KEYS: &[&str] = &[
    "kind",
    "item_id",
    "operation",
    "record_kind",
    "workspace",
    "project",
    "origin",
    "target",
    "content",
    "changed_fields",
    "statement",
    "scope",
    "qualifications",
    "evidence",
    "field_lineage",
    "unresolved",
    "dependencies",
];

impl ProjectChangeContent {
    /// Check an owned parsed representation. This does not establish semantic
    /// eligibility, current authority, target absence or a committed effect.
    pub fn from_value(value: &Value) -> Result<Self, InvalidProjectValue> {
        provenance::encoded_size(value, MAX_PROJECT_CHANGE_BYTES, 16)?;
        let r = parsed::object(value, CONTENT_KEYS)?;
        if r["kind"].as_str() != Some("arboresce.project-change-content.v1") {
            return Err(InvalidProjectValue);
        }
        let item_id = parsed::identifier(&r["item_id"])?;
        let operation = parsed::enumeration(&r["operation"])?;
        let workspace = parsed::workspace(&r["workspace"])?;
        let project = provenance::project_reference(&r["project"])?;
        let origin = ProjectChangeOrigin::from_value(&r["origin"])?;
        let target = ProjectChangeTarget::from_value(&r["target"])?;
        let content = ProjectRecordContent::from_value(&r["content"])?;
        let record_kind: ProjectRecordKind = parsed::enumeration(&r["record_kind"])?;
        if origin.capture().workspace() != workspace
            || target.kind() != record_kind
            || content.kind() != record_kind
            || (operation == ProjectChangeOperation::Create)
                != matches!(target, ProjectChangeTarget::Absent { .. })
        {
            return Err(InvalidProjectValue);
        }
        let mut changed_fields = Vec::new();
        for value in parsed::array(&r["changed_fields"], 1, 64)? {
            let field = ProjectFieldPath::from_value(value)?;
            if changed_fields.contains(&field) {
                return Err(InvalidProjectValue);
            }
            changed_fields.push(field);
        }
        let statement = provenance::free_text(&r["statement"], 16384)?.to_owned();
        let scope = ProjectReviewScope::from_value(&r["scope"])?;
        let qualifications = provenance::strings(&r["qualifications"], 0, 32, 2048, false)?;
        let evidence = ProjectEvidence::from_value(&r["evidence"], workspace.organization_id())?;
        let field_lineage = parsed::array(&r["field_lineage"], 1, 64)?
            .iter()
            .map(|v| ProjectFieldLineage::from_value(v, workspace.organization_id()))
            .collect::<Result<Vec<_>, _>>()?;
        let unresolved = parsed::array(&r["unresolved"], 0, 32)?
            .iter()
            .map(ProjectUnresolved::from_value)
            .collect::<Result<_, _>>()?;
        let dependencies = parsed::array(&r["dependencies"], 0, 32)?
            .iter()
            .map(ProjectDependency::from_value)
            .collect::<Result<_, _>>()?;
        let mut normalized = r.clone();
        normalized.insert("evidence".to_owned(), evidence.json());
        let canonical = parsed::canonical(&Value::Object(normalized), MAX_PROJECT_CHANGE_BYTES)?;
        Ok(Self {
            item_id,
            operation,
            workspace,
            project,
            origin,
            target,
            content,
            changed_fields,
            statement,
            scope,
            qualifications,
            evidence,
            field_lineage,
            unresolved,
            dependencies,
            canonical,
        })
    }
    pub const fn item_id(&self) -> Identifier {
        self.item_id
    }
    pub const fn operation(&self) -> ProjectChangeOperation {
        self.operation
    }
    pub const fn workspace(&self) -> WorkspaceReference {
        self.workspace
    }
    pub const fn project(&self) -> ProjectReference {
        self.project
    }
    pub fn origin(&self) -> &ProjectChangeOrigin {
        &self.origin
    }
    pub const fn target(&self) -> ProjectChangeTarget {
        self.target
    }
    pub fn content(&self) -> &ProjectRecordContent {
        &self.content
    }
    pub fn record_kind(&self) -> ProjectRecordKind {
        self.content.kind()
    }
    pub fn changed_fields(&self) -> &[ProjectFieldPath] {
        &self.changed_fields
    }
    pub fn statement(&self) -> &str {
        &self.statement
    }
    pub fn scope(&self) -> &ProjectReviewScope {
        &self.scope
    }
    pub fn qualifications(&self) -> &[String] {
        &self.qualifications
    }
    pub fn evidence(&self) -> &ProjectEvidence {
        &self.evidence
    }
    pub fn field_lineage(&self) -> &[ProjectFieldLineage] {
        &self.field_lineage
    }
    pub fn unresolved(&self) -> &[ProjectUnresolved] {
        &self.unresolved
    }
    pub fn dependencies(&self) -> &[ProjectDependency] {
        &self.dependencies
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

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ProjectChangeItem {
    content: ProjectChangeContent,
    candidate: ProjectCandidateReference,
}

impl ProjectChangeItem {
    pub fn from_value(value: &Value) -> Result<Self, InvalidProjectValue> {
        provenance::encoded_size(value, MAX_PROJECT_CHANGE_ITEM_BYTES, 16)?;
        let raw = value.as_object().ok_or(InvalidProjectValue)?;
        let mut keys = CONTENT_KEYS[1..].to_vec();
        keys.push("candidate");
        if raw.contains_key("content_preimage") {
            keys.push("content_preimage");
        }
        let r = parsed::object(value, &keys)?;
        let candidate = ProjectCandidateReference::from_value(&r["candidate"])?;
        let mut preimage = r.clone();
        preimage.remove("candidate");
        preimage.remove("content_preimage");
        preimage.insert(
            "kind".to_owned(),
            Value::String("arboresce.project-change-content.v1".to_owned()),
        );
        let content = ProjectChangeContent::from_value(&Value::Object(preimage))?;
        if candidate.workspace() != content.workspace()
            || candidate.content_sha256() != content.sha256()
        {
            return Err(InvalidProjectValue);
        }
        if let Some(supplied) = r.get("content_preimage") {
            let supplied = ProjectChangeContent::from_value(supplied)?;
            if supplied != content {
                return Err(InvalidProjectValue);
            }
        }
        Ok(Self { content, candidate })
    }
    pub fn content(&self) -> &ProjectChangeContent {
        &self.content
    }
    pub fn candidate(&self) -> &ProjectCandidateReference {
        &self.candidate
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.content.as_bytes()
    }
    pub fn byte_length(&self) -> usize {
        self.content.byte_length()
    }
    pub const fn sha256(&self) -> Sha256Digest {
        self.content.sha256()
    }
}

impl fmt::Debug for ProjectChangeContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProjectChangeContent(<redacted>)")
    }
}
impl fmt::Debug for ProjectChangeItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProjectChangeItem(<redacted>)")
    }
}
impl fmt::Debug for ProjectCandidateReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProjectCandidateReference(<redacted>)")
    }
}
