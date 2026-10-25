//! Closed borrowed shape checks. Raw transport decoding remains with its owner.

use std::str::FromStr;

use serde_json::{Map, Value};

use crate::{CanonicalJsonSelection, ContentRevision, Identifier, JsonPointer, SourceJson};

use super::{
    InvalidProjectValue, PartyReference, ProjectDate, ProjectRecordReference, RelationshipSubject,
    WorkspaceReference,
};

pub(super) fn object<'a>(
    value: &'a Value,
    keys: &[&str],
) -> Result<&'a Map<String, Value>, InvalidProjectValue> {
    let object = value.as_object().ok_or(InvalidProjectValue)?;
    if object.len() != keys.len() || keys.iter().any(|key| !object.contains_key(*key)) {
        return Err(InvalidProjectValue);
    }
    Ok(object)
}

pub(super) fn text(value: &Value, maximum: usize) -> Result<&str, InvalidProjectValue> {
    let text = value.as_str().ok_or(InvalidProjectValue)?;
    // Every caller supplies a fixed schema capacity no larger than 4096.
    if text.is_empty()
        || text.len() > 4 * maximum
        || text.chars().count() > maximum
        || text.bytes().any(|b| b < 32 || b == 127)
    {
        return Err(InvalidProjectValue);
    }
    Ok(text)
}

pub(super) fn slug(value: &Value) -> Result<&str, InvalidProjectValue> {
    let value = text(value, 64)?;
    if !value.as_bytes()[0].is_ascii_lowercase()
        || !value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
    {
        return Err(InvalidProjectValue);
    }
    Ok(value)
}

pub(super) fn enumeration<T: FromStr<Err = InvalidProjectValue>>(
    value: &Value,
) -> Result<T, InvalidProjectValue> {
    text(value, 64)?.parse()
}

pub(super) fn identifier(value: &Value) -> Result<Identifier, InvalidProjectValue> {
    text(value, 36)?.parse().map_err(|_| InvalidProjectValue)
}

pub(super) fn revision(value: &Value) -> Result<ContentRevision, InvalidProjectValue> {
    let object = object(value, &["kind", "value"])?;
    if text(&object["kind"], 7)? != "content" {
        return Err(InvalidProjectValue);
    }
    text(&object["value"], 18)?
        .parse()
        .map_err(|_| InvalidProjectValue)
}

pub(super) fn array(
    value: &Value,
    minimum: usize,
    maximum: usize,
) -> Result<&[Value], InvalidProjectValue> {
    let values = value.as_array().ok_or(InvalidProjectValue)?;
    if values.len() < minimum || values.len() > maximum {
        return Err(InvalidProjectValue);
    }
    Ok(values)
}

pub(super) fn boolean(value: &Value) -> Result<bool, InvalidProjectValue> {
    value.as_bool().ok_or(InvalidProjectValue)
}

pub(super) fn workspace(value: &Value) -> Result<WorkspaceReference, InvalidProjectValue> {
    let object = object(value, &["organization_id", "domain_id"])?;
    Ok(WorkspaceReference::new(
        identifier(&object["organization_id"])?,
        identifier(&object["domain_id"])?,
    ))
}

pub(super) fn party_reference(value: &Value) -> Result<PartyReference, InvalidProjectValue> {
    let object = object(value, &["party_id", "content_revision"])?;
    Ok(PartyReference::new(
        identifier(&object["party_id"])?,
        revision(&object["content_revision"])?,
    ))
}

pub(super) fn record_reference(
    value: &Value,
) -> Result<ProjectRecordReference, InvalidProjectValue> {
    let object = object(value, &["kind", "record_id", "content_revision"])?;
    Ok(ProjectRecordReference::new(
        enumeration(&object["kind"])?,
        identifier(&object["record_id"])?,
        revision(&object["content_revision"])?,
    ))
}

pub(super) fn subject(value: &Value) -> Result<RelationshipSubject, InvalidProjectValue> {
    let value = value.as_object().ok_or(InvalidProjectValue)?;
    if value.len() != 1 {
        return Err(InvalidProjectValue);
    }
    if let Some(scope) = value.get("scope") {
        return if scope.as_str() == Some("project") {
            Ok(RelationshipSubject::Project)
        } else {
            Err(InvalidProjectValue)
        };
    }
    Ok(RelationshipSubject::Record(record_reference(
        value.get("record").ok_or(InvalidProjectValue)?,
    )?))
}

pub(super) fn date(value: &Value) -> Result<ProjectDate, InvalidProjectValue> {
    let value = object(value, &["precision", "value"])?;
    let text = text(&value["value"], 24)?;
    match value["precision"].as_str() {
        Some("date") => text
            .parse()
            .map(ProjectDate::Date)
            .map_err(|_| InvalidProjectValue),
        Some("instant") => text
            .parse()
            .map(ProjectDate::Instant)
            .map_err(|_| InvalidProjectValue),
        _ => Err(InvalidProjectValue),
    }
}

/// Call only after every field and collection of the complete value is checked.
pub(super) fn canonical(
    value: &Value,
    maximum: usize,
) -> Result<CanonicalJsonSelection, InvalidProjectValue> {
    let bytes = serde_json::to_vec(value).map_err(|_| InvalidProjectValue)?;
    if bytes.len() > maximum {
        return Err(InvalidProjectValue);
    }
    let source = SourceJson::parse(&bytes).map_err(|_| InvalidProjectValue)?;
    let root = JsonPointer::parse("").map_err(|_| InvalidProjectValue)?;
    source.select(&root).map_err(|_| InvalidProjectValue)
}
