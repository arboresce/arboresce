use std::{fmt, str::FromStr};

use serde_json::Value;

use crate::{CanonicalJsonNumber, ContentRevision, Identifier, Sha256Digest};

use super::{
    InvalidProjectValue, ProjectDate, ProjectRecordReference, ProjectReference,
    ProjectUnresolvedRequirement, WorkspaceReference, parsed,
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ProjectFieldPath(String);

impl ProjectFieldPath {
    pub fn from_value(value: &Value) -> Result<Self, InvalidProjectValue> {
        value.as_str().ok_or(InvalidProjectValue)?.parse()
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for ProjectFieldPath {
    type Err = InvalidProjectValue;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty()
            || value.len() > 128
            || value.split('.').any(|part| {
                part.is_empty()
                    || !part.as_bytes()[0].is_ascii_lowercase()
                    || !part
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
            })
        {
            return Err(InvalidProjectValue);
        }
        Ok(Self(value.to_owned()))
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ProjectCaptureReference {
    workspace: WorkspaceReference,
    capture_id: Identifier,
    version: ContentRevision,
}

impl ProjectCaptureReference {
    pub(super) fn from_value(value: &Value) -> Result<Self, InvalidProjectValue> {
        let r = parsed::object(
            value,
            &["organization_id", "domain_id", "capture_id", "version"],
        )?;
        Ok(Self {
            workspace: WorkspaceReference::new(
                parsed::identifier(&r["organization_id"])?,
                parsed::identifier(&r["domain_id"])?,
            ),
            capture_id: parsed::identifier(&r["capture_id"])?,
            version: parsed::revision(&r["version"])?,
        })
    }
    pub const fn workspace(self) -> WorkspaceReference {
        self.workspace
    }
    pub const fn capture_id(self) -> Identifier {
        self.capture_id
    }
    pub const fn version(self) -> ContentRevision {
        self.version
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ProjectObservationReference {
    organization_id: Identifier,
    observation_id: Identifier,
}

impl ProjectObservationReference {
    pub(super) fn from_value(value: &Value) -> Result<Self, InvalidProjectValue> {
        let r = parsed::object(value, &["organization_id", "observation_id"])?;
        Ok(Self {
            organization_id: parsed::identifier(&r["organization_id"])?,
            observation_id: parsed::identifier(&r["observation_id"])?,
        })
    }
    pub const fn organization_id(self) -> Identifier {
        self.organization_id
    }
    pub const fn observation_id(self) -> Identifier {
        self.observation_id
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ProjectTemporalBasis {
    event: ProjectDate,
    timezone: String,
    relative_date_rule_version: String,
}

impl ProjectTemporalBasis {
    fn from_value(value: &Value) -> Result<Self, InvalidProjectValue> {
        let r = parsed::object(value, &["event", "timezone", "relative_date_rule_version"])?;
        let timezone = parsed::text(&r["timezone"], 64)?;
        if timezone.split('/').any(|part| {
            part.is_empty()
                || !part.as_bytes()[0].is_ascii_alphabetic()
                || !part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'+' | b'-'))
        }) {
            return Err(InvalidProjectValue);
        }
        Ok(Self {
            event: parsed::date(&r["event"])?,
            timezone: timezone.to_owned(),
            relative_date_rule_version: parsed::slug(&r["relative_date_rule_version"])?.to_owned(),
        })
    }
    pub const fn event(&self) -> ProjectDate {
        self.event
    }
    pub fn timezone(&self) -> &str {
        &self.timezone
    }
    pub fn relative_date_rule_version(&self) -> &str {
        &self.relative_date_rule_version
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ProjectChangeOrigin {
    capture: ProjectCaptureReference,
    analysis_id: Identifier,
    analysis_revision: ContentRevision,
    processing_profile_digest: Sha256Digest,
    temporal_basis: ProjectTemporalBasis,
}

impl ProjectChangeOrigin {
    pub(super) fn from_value(value: &Value) -> Result<Self, InvalidProjectValue> {
        let r = parsed::object(
            value,
            &[
                "capture",
                "analysis_id",
                "analysis_revision",
                "processing_profile_digest",
                "temporal_basis",
            ],
        )?;
        Ok(Self {
            capture: ProjectCaptureReference::from_value(&r["capture"])?,
            analysis_id: parsed::identifier(&r["analysis_id"])?,
            analysis_revision: parsed::revision(&r["analysis_revision"])?,
            processing_profile_digest: digest(&r["processing_profile_digest"])?,
            temporal_basis: ProjectTemporalBasis::from_value(&r["temporal_basis"])?,
        })
    }
    pub const fn capture(&self) -> ProjectCaptureReference {
        self.capture
    }
    pub const fn analysis_id(&self) -> Identifier {
        self.analysis_id
    }
    pub const fn analysis_revision(&self) -> ContentRevision {
        self.analysis_revision
    }
    pub const fn processing_profile_digest(&self) -> Sha256Digest {
        self.processing_profile_digest
    }
    pub fn temporal_basis(&self) -> &ProjectTemporalBasis {
        &self.temporal_basis
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ProjectReviewScope {
    summary: String,
    applies_to: Vec<String>,
    excludes: Vec<String>,
}

impl ProjectReviewScope {
    pub(super) fn from_value(value: &Value) -> Result<Self, InvalidProjectValue> {
        let r = parsed::object(value, &["summary", "applies_to", "excludes"])?;
        Ok(Self {
            summary: free_text(&r["summary"], 8192)?.to_owned(),
            applies_to: strings(&r["applies_to"], 1, 32, 2048, false)?,
            excludes: strings(&r["excludes"], 0, 32, 2048, false)?,
        })
    }
    pub fn summary(&self) -> &str {
        &self.summary
    }
    pub fn applies_to(&self) -> &[String] {
        &self.applies_to
    }
    pub fn excludes(&self) -> &[String] {
        &self.excludes
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ProjectEvidence {
    supporting: Vec<ProjectObservationReference>,
    refuting: Vec<ProjectObservationReference>,
    qualifying: Vec<ProjectObservationReference>,
}

impl ProjectEvidence {
    pub(super) fn from_value(
        value: &Value,
        organization: Identifier,
    ) -> Result<Self, InvalidProjectValue> {
        let r = parsed::object(value, &["supporting", "refuting", "qualifying"])?;
        fn role(
            v: &Value,
            organization: Identifier,
        ) -> Result<Vec<ProjectObservationReference>, InvalidProjectValue> {
            let mut refs = Vec::new();
            for value in parsed::array(v, 0, 64)? {
                let reference = ProjectObservationReference::from_value(value)?;
                if reference.organization_id() != organization || refs.contains(&reference) {
                    return Err(InvalidProjectValue);
                }
                refs.push(reference);
            }
            refs.sort_unstable_by(|a, b| {
                a.organization_id
                    .as_bytes()
                    .cmp(b.organization_id.as_bytes())
                    .then_with(|| a.observation_id.as_bytes().cmp(b.observation_id.as_bytes()))
            });
            Ok(refs)
        }
        Ok(Self {
            supporting: role(&r["supporting"], organization)?,
            refuting: role(&r["refuting"], organization)?,
            qualifying: role(&r["qualifying"], organization)?,
        })
    }
    pub fn supporting(&self) -> &[ProjectObservationReference] {
        &self.supporting
    }
    pub fn refuting(&self) -> &[ProjectObservationReference] {
        &self.refuting
    }
    pub fn qualifying(&self) -> &[ProjectObservationReference] {
        &self.qualifying
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
enum LineageData {
    SourceObservation(ProjectObservationReference),
    RetainedContext(ProjectRecordReference),
    DeterministicDefault {
        rule_version: String,
        applies_to: String,
    },
    ModelInference {
        profile: String,
        profile_digest: Sha256Digest,
        basis: String,
        alternatives: Vec<String>,
        unresolved: Vec<String>,
    },
}

/// Immutable projection of checked lineage, not a value-admission constructor.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum ProjectLineageView<'a> {
    SourceObservation(ProjectObservationReference),
    RetainedContext(ProjectRecordReference),
    DeterministicDefault {
        rule_version: &'a str,
        applies_to: &'a str,
    },
    ModelInference {
        profile: &'a str,
        profile_digest: Sha256Digest,
        basis: &'a str,
        alternatives: &'a [String],
        unresolved: &'a [String],
    },
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ProjectLineage {
    data: LineageData,
}

impl ProjectLineage {
    fn from_value(value: &Value, organization: Identifier) -> Result<Self, InvalidProjectValue> {
        let r = value.as_object().ok_or(InvalidProjectValue)?;
        let category = r
            .get("category")
            .and_then(Value::as_str)
            .ok_or(InvalidProjectValue)?;
        let data = match category {
            "source_observation" => {
                let r = parsed::object(value, &["category", "observation"])?;
                let reference = ProjectObservationReference::from_value(&r["observation"])?;
                if reference.organization_id() != organization {
                    return Err(InvalidProjectValue);
                }
                LineageData::SourceObservation(reference)
            }
            "retained_context" => {
                let r = parsed::object(value, &["category", "record"])?;
                LineageData::RetainedContext(parsed::record_reference(&r["record"])?)
            }
            "deterministic_default" => {
                let r = parsed::object(value, &["category", "rule_version", "applies_to"])?;
                LineageData::DeterministicDefault {
                    rule_version: parsed::slug(&r["rule_version"])?.to_owned(),
                    applies_to: parsed::text(&r["applies_to"], 1024)?.to_owned(),
                }
            }
            "model_inference" => {
                let r = parsed::object(
                    value,
                    &[
                        "category",
                        "profile",
                        "profile_digest",
                        "basis",
                        "alternatives",
                        "unresolved",
                    ],
                )?;
                LineageData::ModelInference {
                    profile: parsed::slug(&r["profile"])?.to_owned(),
                    profile_digest: digest(&r["profile_digest"])?,
                    basis: parsed::text(&r["basis"], 2048)?.to_owned(),
                    alternatives: strings(&r["alternatives"], 0, 16, 1024, true)?,
                    unresolved: strings(&r["unresolved"], 0, 16, 1024, true)?,
                }
            }
            _ => return Err(InvalidProjectValue),
        };
        Ok(Self { data })
    }
    pub fn view(&self) -> ProjectLineageView<'_> {
        match &self.data {
            LineageData::SourceObservation(v) => ProjectLineageView::SourceObservation(*v),
            LineageData::RetainedContext(v) => ProjectLineageView::RetainedContext(*v),
            LineageData::DeterministicDefault {
                rule_version,
                applies_to,
            } => ProjectLineageView::DeterministicDefault {
                rule_version,
                applies_to,
            },
            LineageData::ModelInference {
                profile,
                profile_digest,
                basis,
                alternatives,
                unresolved,
            } => ProjectLineageView::ModelInference {
                profile,
                profile_digest: *profile_digest,
                basis,
                alternatives,
                unresolved,
            },
        }
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ProjectFieldLineage {
    field: ProjectFieldPath,
    entries: Vec<ProjectLineage>,
}

impl ProjectFieldLineage {
    pub(super) fn from_value(
        value: &Value,
        organization: Identifier,
    ) -> Result<Self, InvalidProjectValue> {
        let r = parsed::object(value, &["field", "entries"])?;
        let entries = parsed::array(&r["entries"], 1, 16)?
            .iter()
            .map(|v| ProjectLineage::from_value(v, organization))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            field: ProjectFieldPath::from_value(&r["field"])?,
            entries,
        })
    }
    pub fn field(&self) -> &ProjectFieldPath {
        &self.field
    }
    pub fn entries(&self) -> &[ProjectLineage] {
        &self.entries
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ProjectUnresolved {
    field: ProjectFieldPath,
    requirement: ProjectUnresolvedRequirement,
}
impl ProjectUnresolved {
    pub(super) fn from_value(value: &Value) -> Result<Self, InvalidProjectValue> {
        let r = parsed::object(value, &["field", "requirement"])?;
        Ok(Self {
            field: ProjectFieldPath::from_value(&r["field"])?,
            requirement: parsed::enumeration(&r["requirement"])?,
        })
    }
    pub fn field(&self) -> &ProjectFieldPath {
        &self.field
    }
    pub const fn requirement(&self) -> ProjectUnresolvedRequirement {
        self.requirement
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ProjectDependency {
    Item {
        item_id: Identifier,
        content_revision: ContentRevision,
    },
    Record(ProjectRecordReference),
}
impl ProjectDependency {
    pub(super) fn from_value(value: &Value) -> Result<Self, InvalidProjectValue> {
        let r = value.as_object().ok_or(InvalidProjectValue)?;
        if r.contains_key("record") {
            let r = parsed::object(value, &["record"])?;
            Ok(Self::Record(parsed::record_reference(&r["record"])?))
        } else {
            let r = parsed::object(value, &["item_id", "content_revision"])?;
            Ok(Self::Item {
                item_id: parsed::identifier(&r["item_id"])?,
                content_revision: parsed::revision(&r["content_revision"])?,
            })
        }
    }
}

pub(super) fn digest(value: &Value) -> Result<Sha256Digest, InvalidProjectValue> {
    parsed::text(value, 64)?
        .parse()
        .map_err(|_| InvalidProjectValue)
}

pub(super) fn project_reference(value: &Value) -> Result<ProjectReference, InvalidProjectValue> {
    let r = parsed::object(value, &["project_id", "content_revision"])?;
    Ok(ProjectReference::new(
        parsed::identifier(&r["project_id"])?,
        parsed::revision(&r["content_revision"])?,
    ))
}

pub(super) fn free_text(value: &Value, maximum: usize) -> Result<&str, InvalidProjectValue> {
    let text = value.as_str().ok_or(InvalidProjectValue)?;
    if text.is_empty() || text.len() > 4 * maximum || text.chars().count() > maximum {
        return Err(InvalidProjectValue);
    }
    Ok(text)
}

pub(super) fn strings(
    value: &Value,
    minimum: usize,
    maximum: usize,
    text_maximum: usize,
    controlled: bool,
) -> Result<Vec<String>, InvalidProjectValue> {
    parsed::array(value, minimum, maximum)?
        .iter()
        .map(|value| {
            let text = if controlled {
                parsed::text(value, text_maximum)?
            } else {
                free_text(value, text_maximum)?
            };
            Ok(text.to_owned())
        })
        .collect()
}

pub(super) fn observation_json(value: ProjectObservationReference) -> Value {
    serde_json::json!({"organization_id":value.organization_id.to_string(),
        "observation_id":value.observation_id.to_string()})
}
impl ProjectEvidence {
    pub(super) fn json(&self) -> Value {
        let role = |v: &[ProjectObservationReference]| {
            v.iter().copied().map(observation_json).collect::<Vec<_>>()
        };
        serde_json::json!({"supporting":role(&self.supporting),"refuting":role(&self.refuting),
            "qualifying":role(&self.qualifying)})
    }
}

macro_rules! redacted_debug {
    ($($name:ident),+ $(,)?) => { $(
        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($name), "(<redacted>)"))
            }
        }
    )+ };
}
redacted_debug!(
    ProjectFieldPath,
    ProjectCaptureReference,
    ProjectObservationReference,
    ProjectTemporalBasis,
    ProjectChangeOrigin,
    ProjectReviewScope,
    ProjectEvidence,
    ProjectLineage,
    ProjectFieldLineage,
    ProjectUnresolved
);
impl fmt::Debug for ProjectLineageView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProjectLineageView(<redacted>)")
    }
}

/// Count compact common-JCS bytes without serializing a whole unchecked value.
/// Every recursive entry checks its container bound before visiting children.
pub(super) fn encoded_size(
    value: &Value,
    maximum: usize,
    depth_limit: usize,
) -> Result<usize, InvalidProjectValue> {
    fn add(total: &mut usize, n: usize, maximum: usize) -> Result<(), InvalidProjectValue> {
        *total = total
            .checked_add(n)
            .filter(|v| *v <= maximum)
            .ok_or(InvalidProjectValue)?;
        Ok(())
    }
    fn quoted(text: &str, total: &mut usize, maximum: usize) -> Result<(), InvalidProjectValue> {
        if text.len() > maximum - *total || text.chars().take(65_537).count() > 65_536 {
            return Err(InvalidProjectValue);
        }
        // One bounded string uses the already selected ordinary JSON escaping.
        // At most six encoded bytes per admitted input byte are allocated here.
        let bytes = serde_json::to_vec(text).map_err(|_| InvalidProjectValue)?;
        add(total, bytes.len(), maximum)
    }
    fn visit(
        value: &Value,
        total: &mut usize,
        maximum: usize,
        parents: usize,
        limit: usize,
    ) -> Result<(), InvalidProjectValue> {
        match value {
            Value::Null => add(total, 4, maximum),
            Value::Bool(v) => add(total, if *v { 4 } else { 5 }, maximum),
            Value::String(v) => quoted(v, total, maximum),
            Value::Number(v) => {
                let n = CanonicalJsonNumber::from_token(&v.to_string())
                    .map_err(|_| InvalidProjectValue)?;
                add(total, n.byte_length(), maximum)
            }
            Value::Array(values) => {
                if parents >= limit || values.len() > 256 {
                    return Err(InvalidProjectValue);
                }
                add(total, 2 + values.len().saturating_sub(1), maximum)?;
                for value in values {
                    visit(value, total, maximum, parents + 1, limit)?;
                }
                Ok(())
            }
            Value::Object(values) => {
                if parents >= limit || values.len() > 128 {
                    return Err(InvalidProjectValue);
                }
                add(total, 2 + values.len().saturating_sub(1), maximum)?;
                for (key, value) in values {
                    if !(1..=128).contains(&key.chars().take(129).count())
                        || key.chars().any(|c| c <= '\u{1f}' || c == '\u{7f}')
                    {
                        return Err(InvalidProjectValue);
                    }
                    quoted(key, total, maximum)?;
                    add(total, 1, maximum)?;
                    visit(value, total, maximum, parents + 1, limit)?;
                }
                Ok(())
            }
        }
    }
    let mut total = 0;
    visit(value, &mut total, maximum, 0, depth_limit)?;
    Ok(total)
}

pub(super) fn index(value: &Value, maximum: usize) -> Result<usize, InvalidProjectValue> {
    let number = value.as_number().ok_or(InvalidProjectValue)?;
    let canonical =
        CanonicalJsonNumber::from_token(&number.to_string()).map_err(|_| InvalidProjectValue)?;
    let value = std::str::from_utf8(canonical.as_bytes())
        .map_err(|_| InvalidProjectValue)?
        .parse::<usize>()
        .map_err(|_| InvalidProjectValue)?;
    if value > maximum {
        return Err(InvalidProjectValue);
    }
    Ok(value)
}

/// JSON value meaning preserves types and ordered arrays. Numeric equality uses
/// the admitted common number profile, never Boolean or string coercion.
pub(super) fn equal_value(a: &Value, b: &Value) -> bool {
    if a == b {
        return true;
    }
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => {
            match (
                CanonicalJsonNumber::from_token(&a.to_string()),
                CanonicalJsonNumber::from_token(&b.to_string()),
            ) {
                (Ok(a), Ok(b)) => a == b,
                _ => false,
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| equal_value(a, b))
        }
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(key, a)| b.get(key).is_some_and(|b| equal_value(a, b)))
        }
        _ => a == b,
    }
}
