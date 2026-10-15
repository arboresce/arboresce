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

#[test]
fn canonical_calendar_dates() {
    use arboresce_domain::{CalendarDate, InvalidCalendarDate};
    use serde_json::json;
    let original = support::primitive_fixture();
    let extra = support::time_fixture();
    let mut seen = HashSet::new();
    for case in original["groups"]["calendar_date"]
        .as_array()
        .unwrap()
        .iter()
        .chain(extra["calendar_lexical"].as_array().unwrap())
    {
        let id = case["id"].as_str().unwrap();
        let result = case["input"].as_str().unwrap().parse::<CalendarDate>();
        if case["accepted"] == true {
            let value = result.expect(id);
            let (year, month, day) = value.components();
            assert_eq!(json!([year, month, day]), case["year_month_day"], "{id}");
            assert_eq!(value.to_string(), case["roundtrip"], "{id}");
            assert_eq!(CalendarDate::new(year, month, day), Ok(value), "{id}");
        } else {
            assert_eq!(result, Err(InvalidCalendarDate), "{id}");
        }
        let event = if id.starts_with("time-") {
            id.to_owned()
        } else {
            format!("time-{id}")
        };
        assert!(seen.insert(event.clone()));
        println!("collector case={event} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 28);
}

#[test]
fn canonical_utc_millisecond_timestamps() {
    use arboresce_domain::{InvalidUtcTimestamp, UtcTimestamp};
    let original = support::primitive_fixture();
    let extra = support::time_fixture();
    let mut seen = HashSet::new();
    for case in original["groups"]["timestamp"]
        .as_array()
        .unwrap()
        .iter()
        .chain(extra["timestamp_lexical"].as_array().unwrap())
    {
        let id = case["id"].as_str().unwrap();
        let result = case["input"].as_str().unwrap().parse::<UtcTimestamp>();
        if case["accepted"] == true {
            let value = result.expect(id);
            assert_eq!(value.to_string(), case["roundtrip"], "{id}");
            assert_eq!(value.unix_milliseconds(), case["unix_milliseconds"], "{id}");
            let (hour, minute, second, millisecond) = value.time_components();
            assert_eq!(
                UtcTimestamp::new(value.date(), hour, minute, second, millisecond),
                Ok(value),
                "{id}"
            );
        } else {
            assert_eq!(result, Err(InvalidUtcTimestamp), "{id}");
        }
        let event = if id.starts_with("time-") {
            id.to_owned()
        } else {
            format!("time-{id}")
        };
        assert!(seen.insert(event.clone()));
        println!("collector case={event} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 32);
}

#[test]
fn checked_time_components() {
    use arboresce_domain::{CalendarDate, InvalidCalendarDate, InvalidUtcTimestamp, UtcTimestamp};
    let fixture = support::time_fixture();
    let mut seen = HashSet::new();
    for case in fixture["calendar_components"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let input = case["input"].as_array().unwrap();
        let value = CalendarDate::new(
            u16::try_from(input[0].as_u64().unwrap()).unwrap(),
            u8::try_from(input[1].as_u64().unwrap()).unwrap(),
            u8::try_from(input[2].as_u64().unwrap()).unwrap(),
        );
        if case["accepted"] == true {
            assert_eq!(value.unwrap().to_string(), case["canonical"], "{id}");
        } else {
            assert_eq!(value, Err(InvalidCalendarDate), "{id}");
        }
        println!("collector case={id} pid={}", std::process::id());
    }
    for case in fixture["timestamp_components"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let date = case["date"].as_str().unwrap().parse().unwrap();
        let input = case["input"].as_array().unwrap();
        let value = UtcTimestamp::new(
            date,
            u8::try_from(input[0].as_u64().unwrap()).unwrap(),
            u8::try_from(input[1].as_u64().unwrap()).unwrap(),
            u8::try_from(input[2].as_u64().unwrap()).unwrap(),
            u16::try_from(input[3].as_u64().unwrap()).unwrap(),
        );
        if case["accepted"] == true {
            let value = value.unwrap();
            assert_eq!(value.to_string(), case["canonical"], "{id}");
            assert_eq!(value.unix_milliseconds(), case["unix_milliseconds"], "{id}");
        } else {
            assert_eq!(value, Err(InvalidUtcTimestamp), "{id}");
        }
        println!("collector case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 48);
}

#[test]
fn injected_clock_preserves_sequence_and_errors() {
    use serde_json::{Value, json};
    let original = support::primitive_fixture();
    let fixture = support::time_fixture();
    let case = &original["groups"]["clock"][0];
    let supplied: Vec<_> = case["supplied"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| json!({"ok":value}))
        .collect();
    let mut clock = support::SuppliedClock::new(&supplied);
    let actual = support::read_clock(&mut clock, supplied.len());
    let expected: Vec<_> = case["expected_same_sequence"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| json!({"ok":value}))
        .collect();
    assert_eq!(actual, Value::Array(expected));
    assert_eq!(clock.calls, supplied.len());
    println!(
        "collector case=time-{} pid={}",
        case["id"].as_str().unwrap(),
        std::process::id()
    );

    let case = &fixture["clock_api"];
    let mut clock = support::SuppliedClock::new(case["supplied"].as_array().unwrap());
    let count = usize::try_from(case["expected_total_calls"].as_u64().unwrap()).unwrap();
    let actual = support::read_clock(&mut clock, count);
    assert_eq!(actual, case["expected_returned"]);
    assert_eq!(clock.calls, count);
    let outcomes = actual.as_array().unwrap();
    assert_eq!(
        outcomes.iter().filter(|v| v.get("error").is_some()).count(),
        usize::try_from(case["expected_errors"].as_u64().unwrap()).unwrap()
    );
    assert_eq!(
        outcomes.iter().filter(|v| v.get("ok").is_some()).count(),
        usize::try_from(case["expected_successful_reads"].as_u64().unwrap()).unwrap()
    );
    println!(
        "collector case={} pid={}",
        case["id"].as_str().unwrap(),
        std::process::id()
    );
}

#[test]
fn time_traits_and_formatter_failures() {
    use arboresce_domain::{CalendarDate, InvalidCalendarDate, InvalidUtcTimestamp, UtcTimestamp};
    use std::any::TypeId;
    let fixture = support::time_fixture();
    assert_eq!(
        TypeId::of::<CalendarDate>() != TypeId::of::<UtcTimestamp>(),
        fixture["api"]["types_distinct"].as_bool().unwrap()
    );
    assert_eq!(
        InvalidCalendarDate.to_string(),
        fixture["api"]["date_error"]
    );
    assert_eq!(
        InvalidUtcTimestamp.to_string(),
        fixture["api"]["timestamp_error"]
    );
    assert!(Error::source(&InvalidCalendarDate).is_none());
    assert!(Error::source(&InvalidUtcTimestamp).is_none());
    let mut output = support::RejectedOutput;
    assert!(write!(&mut output, "{InvalidCalendarDate}").is_err());
    assert!(write!(&mut output, "{InvalidUtcTimestamp}").is_err());
    let mut seen = HashSet::new();
    for case in fixture["traits"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        assert_eq!(
            support::time_traits(
                case["kind"].as_str().unwrap(),
                case["values"].as_array().unwrap()
            ),
            case["expected"],
            "{id}"
        );
        println!("collector case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 2);
}
