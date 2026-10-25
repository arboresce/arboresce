use std::{collections::HashSet, error::Error};

use arboresce_domain::project::*;
use serde_json::Value;

mod support;

#[test]
fn complete_project_change_literals() {
    let inputs = support::inputs();
    let oracle = support::oracle();
    let rows = inputs["cases"].as_array().unwrap();
    let answers = oracle["cases"].as_array().unwrap();
    assert_eq!(rows.len(), 516);
    assert_eq!(answers.len(), rows.len());
    let mut seen = HashSet::new();
    let mut valid = 0;
    for (row, expected) in rows.iter().zip(answers) {
        let id = row["id"].as_str().unwrap();
        assert!(seen.insert(id));
        assert_eq!(row["id"], expected["id"]);
        let original = row["value"].clone();
        match support::admit(row) {
            Ok(value) => {
                valid += 1;
                assert_eq!(expected["valid"], true, "{id}");
                let actual = support::output(&value);
                for (field, value) in expected.as_object().unwrap() {
                    if matches!(
                        field.as_str(),
                        "id" | "valid" | "reason" | "qualification_scope"
                    ) {
                        continue;
                    }
                    assert_eq!(&actual[field], value, "{id}: {field}");
                }
                assert_eq!(value.clone(), value);
                assert_eq!(support::hash(&value.clone()), support::hash(&value));
            }
            Err(error) => {
                assert_eq!(expected["valid"], false, "{id}");
                assert_eq!(error.to_string(), expected["error"], "{id}");
                assert_eq!(format!("{error:?}"), expected["debug"], "{id}");
                assert!(error.source().is_none());
            }
        }
        assert_eq!(row["value"], original, "{id}");
        support::event(id);
    }
    assert_eq!(valid, 161);
}

#[test]
fn complete_project_change_traits() {
    let inputs = support::inputs();
    let oracle = support::oracle();
    let rows = inputs["cases"].as_array().unwrap();
    let pairs = oracle["traits"].as_array().unwrap();
    assert_eq!(pairs.len(), 15);
    for pair in pairs {
        let find = |key: &str| rows.iter().find(|r| r["id"] == pair[key]).unwrap();
        let a = support::admit(find("left")).unwrap();
        let b = support::admit(find("right")).unwrap();
        let equal = pair["equal"].as_bool().unwrap();
        assert_eq!(a == b, equal, "{}", pair["id"]);
        if equal {
            assert_eq!(support::hash(&a), support::hash(&b));
        }
        let mut set = HashSet::new();
        assert!(set.insert(a.clone()));
        assert_eq!(set.insert(b), !equal);
        assert_eq!(a.clone(), a);
        support::event(pair["id"].as_str().unwrap());
    }
}

#[test]
fn complete_project_change_semantics() {
    let inputs = support::semantic_inputs();
    let oracle = support::semantic_oracle();
    let rows = inputs["cases"].as_array().unwrap();
    let answers = oracle["cases"].as_array().unwrap();
    assert_eq!(rows.len(), 302);
    assert_eq!(answers.len(), rows.len());
    let mut seen = HashSet::new();
    for (row, expected) in rows.iter().zip(answers) {
        let id = row["id"].as_str().unwrap();
        assert!(seen.insert(id));
        assert_eq!(row["id"], expected["id"]);
        let original = row["value"].clone();
        assert_eq!(support::compare(row), expected["expected"], "{id}");
        assert_eq!(row["value"], original, "{id}");
        support::event(id);
    }
}

#[test]
fn values_are_owned_immutable_snapshots() {
    let inputs = support::inputs();
    let oracle = support::oracle();
    for (row, expected) in inputs["cases"]
        .as_array()
        .unwrap()
        .iter()
        .zip(oracle["cases"].as_array().unwrap())
    {
        if expected["valid"] != true {
            continue;
        }
        let mut row = row.clone();
        let value = support::admit(&row).unwrap();
        row["value"] = Value::Null;
        drop(row);
        assert_eq!(support::projection(&value), expected["projection"]);
        assert_eq!(value.clone(), value);
        assert_eq!(support::hash(&value), support::hash(&value.clone()));
    }
    assert_eq!(MAX_PROJECT_CHANGE_BYTES, 1_048_576);
    assert_eq!(MAX_PROJECT_CHANGE_ITEM_BYTES, 1_048_576);
    assert_eq!(MAX_PROJECT_COMPARISON_BYTES, 1_048_576);
}

#[test]
fn errors_are_fixed_and_formatter_failures_propagate() {
    let oracle = support::oracle();
    let error = ProjectChangeContent::from_value(&Value::Null).unwrap_err();
    assert_eq!(error.to_string(), oracle["fixed_error"]);
    assert_eq!(format!("{error:?}"), oracle["fixed_error_debug"]);
    assert!(error.source().is_none());
    let copied = error;
    assert_eq!(error, copied);
    assert!(support::formatting_fails(error));
    for v in [
        ProjectChangeOperation::Create,
        ProjectChangeOperation::Revise,
        ProjectChangeOperation::Associate,
    ] {
        assert!(support::formatting_fails(v));
    }
    for v in [
        ProjectObservationKind::DocumentObservation,
        ProjectObservationKind::UserAssertion,
        ProjectObservationKind::ModelInference,
    ] {
        assert!(support::formatting_fails(v));
    }
    for v in [
        ProjectUnresolvedRequirement::Clarification,
        ProjectUnresolvedRequirement::Unknown,
        ProjectUnresolvedRequirement::Conflict,
    ] {
        assert!(support::formatting_fails(v));
    }
    let inputs = support::inputs();
    for row in inputs["cases"].as_array().unwrap() {
        if let Ok(value) = support::admit(row) {
            support::check_debug_failure(&value);
        }
    }
}
