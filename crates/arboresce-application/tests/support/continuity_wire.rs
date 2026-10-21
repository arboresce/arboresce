//! Input-only preparation and observations; no expected object or ID is accepted.

use arboresce_application::lifecycle::{
    match_scope,
    wire::{ControlDocument, Definition, WireRecord},
};
use serde_json::{Value, json};

pub fn observe_definition(input: &Value) -> Value {
    json!({"accepted": Definition::from_name(input["value"].as_str().unwrap()).is_some()})
}

pub fn observe_parsed(target: &str, value: &Value) -> bool {
    let name = target.strip_prefix("internal.").unwrap();
    Definition::from_name(name).unwrap().matches_parsed(value)
}

fn raw_bytes(input: &Value) -> Vec<u8> {
    let fragments = input["fragments"].as_array().unwrap();
    assert!(fragments.len() <= 16);
    let mut bytes = Vec::new();
    for fragment in fragments {
        let part = if let Some(text) = fragment.get("utf8") {
            text.as_str().unwrap().as_bytes().to_vec()
        } else {
            let hex = fragment["hex"].as_str().unwrap();
            assert!(hex.is_ascii() && hex.len().is_multiple_of(2));
            (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect()
        };
        let repeat = fragment
            .get("repeat")
            .map(|n| usize::try_from(n.as_u64().unwrap()).unwrap())
            .unwrap_or(1);
        assert!((1..=1_048_577).contains(&repeat));
        let total = bytes
            .len()
            .checked_add(part.len().checked_mul(repeat).unwrap())
            .unwrap();
        assert!(total <= 1_048_577);
        bytes.extend_from_slice(&part.repeat(repeat));
    }
    bytes
}

fn inspect(bytes: &[u8], definition: Definition) -> (Value, Option<WireRecord<'_>>) {
    match ControlDocument::parse(bytes) {
        Err(_) => (
            json!({"raw_valid":false, "shape_valid":null, "wire_valid":false, "retained_exact":null}),
            None,
        ),
        Ok(document) => {
            let shape = document.matches_shape(definition);
            let record = document.check(definition);
            let retained = record
                .as_ref()
                .map(|record| record.as_bytes() == bytes && record.definition() == definition);
            (
                json!({"raw_valid":true, "shape_valid":shape, "wire_valid":record.is_some(), "retained_exact":retained}),
                record,
            )
        }
    }
}

pub fn observe_raw(input: &Value) -> Value {
    let bytes = raw_bytes(input);
    let definition = Definition::from_name(input["definition"].as_str().unwrap()).unwrap();
    inspect(&bytes, definition).0
}

pub fn observe_composition(input: &Value) -> Value {
    let bytes = raw_bytes(input);
    let definition = Definition::from_name(input["definition"].as_str().unwrap()).unwrap();
    let (mut observed, record) = inspect(&bytes, definition);
    let batch = record.as_ref().and_then(WireRecord::project_batch);
    observed["projection"] = Value::Null;
    observed["scope"] = Value::Null;
    if let Some(batch) = batch {
        let result = match_scope(
            super::continuity::namespace(&input["expected_namespace"]),
            super::continuity::predecessor(&input["predecessor"]),
            &batch.checkpoint,
            &batch.entries,
        );
        let mut projection = super::continuity::checkpoint_view(&batch.checkpoint);
        projection["entries"] = json!(
            batch
                .entries
                .iter()
                .copied()
                .map(super::continuity::entry_view)
                .collect::<Vec<_>>()
        );
        let range = result.as_ref().ok().map(|range| json!({
            "first":range.first.value().to_string(), "last":range.last.value().to_string(), "count":range.count,
        }));
        observed["projection"] = projection;
        observed["scope"] = json!({"matched":result.is_ok(), "failure":result.err().map(|failure| format!("{failure:?}")), "range":range});
        observed["retained_exact"] = json!(record.as_ref().unwrap().as_bytes() == bytes);
    }
    observed
}

pub fn observe_edge(input: &Value) -> Value {
    let operation = input["operation"].as_str().unwrap();
    if operation == "raw" {
        return observe_raw(input);
    }
    let bytes = raw_bytes(input);
    let definition = Definition::from_name(input["definition"].as_str().unwrap()).unwrap();
    if operation == "non_batch_projection" {
        let (mut observed, record) = inspect(&bytes, definition);
        observed["projection_available"] = json!(
            record
                .as_ref()
                .and_then(WireRecord::project_batch)
                .is_some()
        );
        return observed;
    }
    let renderings = match ControlDocument::parse(&bytes) {
        Err(error) if operation == "diagnostic_error" => {
            Some(vec![format!("{error:?}"), format!("{error}")])
        }
        Ok(document) => match operation {
            "diagnostic_document" => Some(vec![format!("{document:?}")]),
            "diagnostic_record" => document
                .check(definition)
                .map(|record| vec![format!("{record:?}")]),
            "diagnostic_batch" => document
                .check(definition)
                .and_then(|record| record.project_batch())
                .map(|batch| vec![format!("{batch:?}")]),
            _ => None,
        },
        _ => None,
    };
    let markers = input["protected_markers"].as_array().unwrap();
    assert!(markers.len() <= 16);
    let raw = std::str::from_utf8(&bytes).unwrap();
    let markers: Vec<_> = markers
        .iter()
        .map(|marker| {
            let marker = marker.as_str().unwrap();
            assert!(!marker.is_empty() && marker.is_ascii() && marker.len() <= 128);
            assert!(raw.contains(marker));
            marker
        })
        .collect();
    let nonempty = renderings
        .as_ref()
        .map(|values| !values.is_empty() && values.iter().all(|value| !value.is_empty()));
    let leaked_markers = renderings.as_ref().map(|values| {
        markers
            .iter()
            .copied()
            .filter(|marker| values.iter().any(|value| value.contains(marker)))
            .collect::<Vec<_>>()
    });
    json!({
        "available":renderings.is_some(),
        "nonempty":nonempty,
        "leaked_markers":leaked_markers,
    })
}
