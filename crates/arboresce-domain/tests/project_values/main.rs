use std::{collections::HashSet, error::Error};

use arboresce_domain::project::*;

mod support;

#[test]
fn complete_project_value_literals() {
    let inputs = support::inputs();
    let oracle = support::oracle();
    let rows = inputs["cases"].as_array().unwrap();
    let answers = oracle["answers"].as_array().unwrap();
    assert_eq!(rows.len(), 352);
    assert_eq!(answers.len(), rows.len());
    let mut seen = HashSet::new();
    let mut admitted = 0;
    for (row, expected) in rows.iter().zip(answers) {
        let id = row["id"].as_str().unwrap();
        assert!(seen.insert(id));
        assert_eq!(row["id"], expected["id"]);
        let original = row["value"].clone();
        match support::admit(row) {
            Ok(value) => {
                admitted += 1;
                assert_eq!(expected["valid"], true, "{id}");
                let actual = support::output(&value);
                for field in [
                    "typed_projection",
                    "canonical_utf8",
                    "canonical_bytes",
                    "sha256",
                    "debug",
                ] {
                    assert_eq!(actual[field], expected[field], "{id}: {field}");
                }
                assert_eq!(value.clone(), value);
            }
            Err(error) => {
                assert_eq!(expected["valid"], false, "{id}");
                assert_eq!(error.to_string(), expected["error"], "{id}");
                assert_eq!(format!("{error:?}"), expected["debug"], "{id}");
                assert_eq!(
                    error.source().is_none(),
                    expected["source_is_none"].as_bool().unwrap()
                );
            }
        }
        assert_eq!(row["value"], original, "{id}");
        support::case_event(id);
    }
    assert_eq!(admitted, 92);
}

#[test]
fn complete_project_value_traits() {
    let inputs = support::inputs();
    let oracle = support::oracle();
    let rows = inputs["cases"].as_array().unwrap();
    for pair in oracle["trait_pairs"].as_array().unwrap() {
        let find = |key: &str| rows.iter().find(|row| row["id"] == pair[key]).unwrap();
        let left = support::admit(find("left")).unwrap();
        let right = support::admit(find("right")).unwrap();
        let expected = pair["equal"].as_bool().unwrap();
        assert_eq!(left == right, expected);
        if expected {
            assert_eq!(support::hash(&left), support::hash(&right));
        }
        let mut set = HashSet::new();
        assert!(set.insert(left.clone()));
        assert_eq!(set.insert(right), !expected);
        assert_eq!(left.clone(), left);
        support::case_event(pair["id"].as_str().unwrap());
    }
}

#[test]
fn values_are_owned_immutable_snapshots() {
    let inputs = support::inputs();
    let oracle = support::oracle();
    assert_eq!(oracle["input_remains_unchanged"], true);
    assert_eq!(oracle["returned_values_survive_input_mutation"], true);
    for id in ["party-external-business", "content-work_item"] {
        let mut row = inputs["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["id"] == id)
            .unwrap()
            .clone();
        let value = support::admit(&row).unwrap();
        let expected = oracle["answers"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["id"] == id)
            .unwrap();
        row["value"] = serde_json::Value::Null;
        drop(row);
        assert_eq!(support::projection(&value), expected["typed_projection"]);
        assert_eq!(
            support::output(&value)["canonical_utf8"],
            expected["canonical_utf8"]
        );
        if let support::CheckedValue::Party(party) = &value {
            let reference = ProjectReference::new(party.party_id(), party.content_revision());
            assert_eq!(
                reference.project_id().to_string(),
                expected["typed_projection"]["party_id"]
            );
            assert_eq!(reference.content_revision(), party.content_revision());
        }
    }
    assert_eq!(MAX_PARTY_VALUE_BYTES, 262_144);
    assert_eq!(MAX_PROJECT_CONTENT_BYTES, 32_768);
}

#[test]
fn errors_are_fixed_and_formatter_failures_propagate() {
    let oracle = support::oracle();
    let error = PartyRecord::from_value(&serde_json::Value::Null).unwrap_err();
    assert_eq!(error.to_string(), oracle["fixed_error"]);
    assert_eq!(format!("{error:?}"), oracle["error_debug"]);
    assert!(error.source().is_none());
    let copied = error;
    assert_eq!(error, copied);
    assert_eq!(
        support::formatting_fails(error),
        oracle["formatter_failure_propagates"].as_bool().unwrap()
    );
    assert!(support::formatting_fails(PartyKind::Person));
}
