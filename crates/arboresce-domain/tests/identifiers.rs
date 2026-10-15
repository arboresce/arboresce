use std::{collections::HashSet, error::Error, fmt::Write};

use arboresce_domain::{Identifier, InvalidIdentifier};

#[path = "fixtures/primitives-v1/expected.rs"]
mod expected;
mod support;

#[test]
fn complete_literal_text_inventory() {
    assert_eq!(expected::ID_CASES.len(), 34);
    let mut seen = HashSet::new();
    for case in expected::ID_CASES {
        assert!(seen.insert(case.id), "duplicate {}", case.id);
        let parsed = case.input.parse::<Identifier>();
        assert_eq!(Identifier::try_from(case.input), parsed, "{}", case.id);
        match case.bytes {
            Some(bytes) => {
                let value = parsed.expect(case.id);
                assert_eq!(value.as_bytes(), &bytes, "{}", case.id);
                assert_eq!(value.to_string(), case.input, "{}", case.id);
                assert_eq!(value.to_string().parse::<Identifier>(), Ok(value));
            }
            None => assert_eq!(parsed, Err(InvalidIdentifier), "{}", case.id),
        }
        println!("identifier case={} pid={}", case.id, std::process::id());
    }
}

#[test]
fn complete_literal_binary_inventory() {
    assert_eq!(expected::BINARY_ID_CASES.len(), 8);
    let mut seen = HashSet::new();
    for case in expected::BINARY_ID_CASES {
        assert!(seen.insert(case.id), "duplicate {}", case.id);
        let parsed = Identifier::from_bytes(case.bytes);
        match case.canonical {
            Some(text) => {
                let value = parsed.expect(case.id);
                assert_eq!(value.as_bytes(), &case.bytes, "{}", case.id);
                assert_eq!(value.to_string(), text, "{}", case.id);
                assert_eq!(text.parse::<Identifier>(), Ok(value));
            }
            None => assert_eq!(parsed, Err(InvalidIdentifier), "{}", case.id),
        }
        println!("identifier case={} pid={}", case.id, std::process::id());
    }
}

#[test]
fn immutable_value_traits_and_input_independent_error() {
    let first = Identifier::from_bytes(expected::BINARY_ID_CASES[0].bytes).unwrap();
    let second = Identifier::from_bytes(expected::BINARY_ID_CASES[1].bytes).unwrap();
    let copied = first;
    assert_eq!(copied, first);
    assert_ne!(first, second);
    let values: HashSet<_> = [first, copied, second].into_iter().collect();
    assert_eq!(values.len(), 2);
    assert!(format!("{first:?}").starts_with("Identifier("));
    let error = "untrusted invalid input".parse::<Identifier>().unwrap_err();
    assert_eq!(error, InvalidIdentifier);
    assert_eq!(
        error.to_string(),
        include_str!("fixtures/identifier-api-v1.txt")
            .strip_suffix('\n')
            .unwrap()
    );
    assert_eq!(format!("{error:?}"), "InvalidIdentifier");
    assert!(Error::source(&error).is_none());
}

#[test]
fn formatter_errors_propagate() {
    let value = Identifier::from_bytes(expected::BINARY_ID_CASES[0].bytes).unwrap();
    let mut output = support::RejectedOutput;
    assert!(write!(&mut output, "{value}").is_err());
    assert!(write!(&mut output, "{InvalidIdentifier}").is_err());
}

#[test]
fn checked_revision_decimal_payloads() {
    use arboresce_domain::RevisionError;
    let fixture = support::revision_fixture();
    let mut seen = HashSet::new();
    for case in fixture["payloads"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let result = support::revision_payload(
            case["kind"].as_str().unwrap(),
            case["input"].as_str().unwrap(),
        );
        if case["expected"]["accepted"] == true {
            let (integer, text) = result.expect(id);
            assert_eq!(text, case["expected"]["canonical"], "{id}");
            assert_eq!(
                integer,
                case["expected"]["canonical"]
                    .as_str()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap(),
                "{id}"
            );
        } else {
            assert_eq!(result, Err(RevisionError::InvalidValue), "{id}");
        }
        println!("collector case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 66);
}

#[test]
fn checked_revision_numeric_construction() {
    use arboresce_domain::RevisionError;
    let fixture = support::revision_fixture();
    let mut seen = HashSet::new();
    for case in fixture["numeric"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let input = case["input"].as_u64().unwrap();
        let result = support::revision_numeric(case["kind"].as_str().unwrap(), input);
        if case["expected"]["accepted"] == true {
            let (integer, text) = result.expect(id);
            assert_eq!(integer, input, "{id}");
            assert_eq!(text, case["expected"]["canonical"], "{id}");
        } else {
            assert_eq!(result, Err(RevisionError::InvalidValue), "{id}");
        }
        println!("collector case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 15);
}

#[test]
fn checked_immutable_revision_advancement() {
    use arboresce_domain::RevisionError;
    use serde_json::json;
    let fixture = support::revision_fixture();
    let mut seen = HashSet::new();
    for case in fixture["advance"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let (actual, error) = support::revision_advance(
            case["kind"].as_str().unwrap(),
            case["before"].as_str().unwrap(),
        );
        assert_eq!(
            actual,
            json!({"accepted":case["accepted"],"after":case["after"],"original_after_attempt":case["original_after_attempt"]}),
            "{id}"
        );
        if case["accepted"] == false {
            assert_eq!(error, Some(RevisionError::Exhausted), "{id}");
            assert_eq!(
                error.unwrap().to_string(),
                fixture["api"]["exhausted_error"],
                "{id}"
            );
        } else {
            assert!(error.is_none(), "{id}");
        }
        println!("collector case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 9);
}

#[test]
fn revision_traits_and_formatter_failures() {
    use arboresce_domain::{
        ContentRevision, ControlRevision, Generation, MAX_REVISION_VALUE, RevisionError,
    };
    use std::any::TypeId;
    let fixture = support::revision_fixture();
    assert_eq!(MAX_REVISION_VALUE, fixture["maximum"].as_u64().unwrap());
    assert_eq!(
        TypeId::of::<ContentRevision>() != TypeId::of::<ControlRevision>()
            && TypeId::of::<ContentRevision>() != TypeId::of::<Generation>()
            && TypeId::of::<ControlRevision>() != TypeId::of::<Generation>(),
        fixture["api"]["types_distinct"].as_bool().unwrap()
    );
    assert_eq!(
        RevisionError::InvalidValue.to_string(),
        fixture["api"]["invalid_error"]
    );
    assert_eq!(
        RevisionError::Exhausted.to_string(),
        fixture["api"]["exhausted_error"]
    );
    for error in [RevisionError::InvalidValue, RevisionError::Exhausted] {
        assert!(Error::source(&error).is_none());
        let mut output = support::RejectedOutput;
        assert!(write!(&mut output, "{error}").is_err());
    }
    let mut seen = HashSet::new();
    for case in fixture["traits"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        assert_eq!(
            support::revision_traits(
                case["kind"].as_str().unwrap(),
                case["values"].as_array().unwrap()
            ),
            case["expected"],
            "{id}"
        );
        println!("collector case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 3);
}
