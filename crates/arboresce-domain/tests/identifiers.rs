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
