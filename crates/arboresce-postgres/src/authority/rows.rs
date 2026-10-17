//! Fixed SQL guards bound variable-size output before driver allocation.

use arboresce_application::policy::authority::{PrincipalKind, PrincipalReference, RecordState};
use arboresce_application::policy::{ReferenceKind, ScopedReference, Workspace};
use arboresce_domain::{ControlRevision, Identifier, UtcTimestamp};
use sqlx::{Row, postgres::PgRow};

use super::AuthorityFailure as Failure;

pub(crate) fn token(column: &'static str) -> String {
    format!(
        "CASE WHEN octet_length({column}) BETWEEN 1 AND 96 \
        THEN CASE WHEN {column} ~ '^[A-Za-z0-9_.-]+$' THEN {column} END END AS {column}"
    )
}

pub(crate) fn provenance(column: &'static str) -> String {
    format!(
        "CASE WHEN octet_length({column}) BETWEEN 1 AND 96 \
        THEN CASE WHEN {column} ~ '^[ -~]+$' THEN {column} END END AS {column}"
    )
}

pub(crate) fn tokens(column: &'static str) -> String {
    format!(
        "CASE WHEN cardinality({column}) BETWEEN 0 AND 16 THEN \
        CASE WHEN cardinality({column})=0 OR (array_ndims({column})=1 AND array_lower({column},1)=1) THEN \
        CASE WHEN (SELECT coalesce(bool_and(v IS NOT NULL AND octet_length(v) BETWEEN 1 AND 96 \
            AND v ~ '^[A-Za-z0-9_.-]+$'),true) FROM unnest({column}) v) \
        THEN {column} END END END AS {column}"
    )
}

pub(crate) fn audience() -> String {
    "CASE WHEN cardinality(permitted_principals) BETWEEN 0 AND 32 THEN \
        CASE WHEN cardinality(permitted_principals)=0 OR (array_ndims(permitted_principals)=1 \
            AND array_lower(permitted_principals,1)=1) THEN \
        CASE WHEN (SELECT coalesce(bool_and(v IS NOT NULL),true) FROM unnest(permitted_principals) v) \
        THEN ARRAY(SELECT v::text FROM unnest(permitted_principals) v) END END END \
        AS permitted_principals".to_owned()
}

pub(crate) fn instant(column: &'static str) -> String {
    format!(
        "CASE WHEN isfinite({column}) THEN CASE WHEN \
        {column} BETWEEN timestamptz '0001-01-01 00:00:00+00' AND timestamptz '9999-12-31 23:59:59.999+00' \
        AND mod(extract(microseconds FROM {column}),1000)=0 \
        THEN to_char({column} AT TIME ZONE 'UTC','YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"') END END AS {column}"
    )
}

pub(crate) fn text(row: &PgRow, field: &str) -> Result<String, Failure> {
    row.try_get(field).map_err(|_| Failure::InvalidStoredData)
}

pub(crate) fn id(row: &PgRow, field: &str) -> Result<Identifier, Failure> {
    text(row, field)?
        .parse()
        .map_err(|_| Failure::InvalidStoredData)
}

pub(crate) fn optional_id(row: &PgRow, field: &str) -> Result<Option<Identifier>, Failure> {
    let raw: Option<String> = row.try_get(field).map_err(|_| Failure::InvalidStoredData)?;
    raw.map(|v| v.parse().map_err(|_| Failure::InvalidStoredData))
        .transpose()
}

pub(crate) fn revision(row: &PgRow, field: &str) -> Result<ControlRevision, Failure> {
    let n: i64 = row.try_get(field).map_err(|_| Failure::InvalidStoredData)?;
    n.to_string()
        .parse()
        .map_err(|_| Failure::InvalidStoredData)
}

pub(crate) fn time(row: &PgRow, field: &str) -> Result<UtcTimestamp, Failure> {
    text(row, field)?
        .parse()
        .map_err(|_| Failure::InvalidStoredData)
}

pub(crate) fn optional_time(row: &PgRow, field: &str) -> Result<Option<UtcTimestamp>, Failure> {
    // A NULL stored value differs from a non-NULL invalid value guarded to NULL.
    let present: bool = row
        .try_get(format!("{field}_present").as_str())
        .map_err(|_| Failure::InvalidStoredData)?;
    if present {
        Ok(Some(time(row, field)?))
    } else {
        Ok(None)
    }
}

pub(crate) fn set(row: &PgRow, field: &str) -> Result<Vec<String>, Failure> {
    row.try_get(field).map_err(|_| Failure::InvalidStoredData)
}

pub(crate) fn state(row: &PgRow) -> Result<RecordState, Failure> {
    Ok(if text(row, "state")? == "active" {
        RecordState::Active
    } else {
        RecordState::Inactive
    })
}

pub(crate) fn principal(
    row: &PgRow,
    id_field: &str,
    kind_field: &str,
) -> Result<PrincipalReference, Failure> {
    let kind = match text(row, kind_field)?.as_str() {
        "human" => PrincipalKind::Human,
        "agent" => PrincipalKind::Agent,
        "service" => PrincipalKind::Service,
        _ => PrincipalKind::Unsupported,
    };
    Ok(PrincipalReference {
        id: id(row, id_field)?,
        kind,
    })
}

pub(crate) fn workspace(row: &PgRow) -> Result<Workspace, Failure> {
    Ok(Workspace::new(
        id(row, "organization_id")?,
        id(row, "domain_id")?,
    ))
}

pub(crate) fn kind(value: &str) -> Result<ReferenceKind, Failure> {
    match value {
        "project" => Ok(ReferenceKind::Project),
        "expense" => Ok(ReferenceKind::Expense),
        _ => Err(Failure::InvalidStoredData),
    }
}

pub(crate) fn resource(row: &PgRow) -> Result<ScopedReference, Failure> {
    Ok(ScopedReference::new(
        workspace(row)?,
        kind(&text(row, "resource_kind")?)?,
        id(row, "resource_id")?,
    ))
}

pub(crate) fn reference_id(reference: ScopedReference) -> Result<Identifier, Failure> {
    reference
        .workspace()
        .require_reference(reference)
        .map_err(|_| Failure::InvalidRequest)
}

pub(crate) const fn resource_name(kind: ReferenceKind) -> &'static str {
    match kind {
        ReferenceKind::Project => "project",
        ReferenceKind::Expense => "expense",
        ReferenceKind::Party => "party",
        ReferenceKind::Capture => "capture",
        ReferenceKind::Grant => "grant",
        ReferenceKind::Membership => "membership",
    }
}

/// Budget includes namespace and independent sealed rows, before typed records.
pub(crate) struct RowBudget(usize);

impl RowBudget {
    pub(crate) fn new() -> Self {
        Self(32)
    }
    pub(crate) fn lookahead(&self) -> usize {
        self.0 + 1
    }
    pub(crate) fn take(&mut self, count: usize) -> Result<(), Failure> {
        self.0 = self
            .0
            .checked_sub(count)
            .ok_or(Failure::InvalidStoredData)?;
        Ok(())
    }
}
