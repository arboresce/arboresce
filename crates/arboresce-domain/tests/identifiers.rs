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

#[test]
fn checked_utf8_byte_selections() {
    use arboresce_domain::InvalidLocator;
    let original = support::primitive_fixture();
    let fixture = support::locator_fixture();
    let mut seen = HashSet::new();
    for case in original["groups"]["utf8_range"]
        .as_array()
        .unwrap()
        .iter()
        .chain(fixture["utf8_ranges"].as_array().unwrap())
    {
        let id = case["id"].as_str().unwrap();
        let actual = support::utf8_output(
            case["source_hex"].as_str().unwrap(),
            case["start_byte"].as_i64().unwrap(),
            case["end_byte"].as_i64().unwrap(),
        );
        if case["accepted"] == true {
            let expected = case
                .get("expected")
                .unwrap_or(&fixture["original_admitted_values"][id]);
            assert_eq!(actual.unwrap(), *expected, "{id}");
        } else {
            assert_eq!(actual, Err(InvalidLocator), "{id}");
        }
        let event = if id.starts_with("locator-") {
            id.to_owned()
        } else {
            format!("locator-{id}")
        };
        assert!(seen.insert(event.clone()));
        println!("collector case={event} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 30);
}

#[test]
fn checked_upright_page_bounds() {
    use arboresce_domain::InvalidLocator;
    let original = support::primitive_fixture();
    let fixture = support::locator_fixture();
    let mut seen = HashSet::new();
    for case in original["groups"]["page_region"]
        .as_array()
        .unwrap()
        .iter()
        .chain(fixture["page_regions"].as_array().unwrap())
    {
        let id = case["id"].as_str().unwrap();
        let actual = support::page_case(case).map(support::page_output);
        if case["accepted"] == true {
            let expected = case
                .get("expected")
                .unwrap_or(&fixture["original_admitted_values"][id]);
            assert_eq!(actual.unwrap(), *expected, "{id}");
        } else {
            assert_eq!(actual, Err(InvalidLocator), "{id}");
        }
        let event = if id.starts_with("locator-") {
            id.to_owned()
        } else {
            format!("locator-{id}")
        };
        assert!(seen.insert(event.clone()));
        println!("collector case={event} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 38);
}

#[test]
fn all_orientation_pixel_edges() {
    use arboresce_domain::{InvalidLocator, Orientation, PageBounds, PageRegion};
    use serde_json::json;
    let fixture = support::locator_fixture();
    let mut seen = HashSet::new();
    for case in fixture["orientations"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let orientation =
            Orientation::from_exif(case["source_orientation"].as_i64().unwrap()).unwrap();
        let raw_dimensions = support::pair(&case["raw_dimensions"]);
        assert_eq!(
            json!(orientation.upright_dimensions(raw_dimensions).unwrap()),
            case["upright_dimensions"],
            "{id}"
        );
        let bounds = PageBounds {
            page_count: support::fixture_u32(&case["actual_page_count"]),
            width: raw_dimensions[0],
            height: raw_dimensions[1],
        };
        // Both independently frozen variants execute under every orientation ID.
        for sample in [case, &case["full_page"]] {
            let input = sample["raw_corners"].as_array().unwrap();
            let expected = sample["expected_corners"].as_array().unwrap();
            assert_eq!(input.len(), 4, "{id}");
            assert_eq!(expected.len(), 4, "{id}");
            for (point, expected) in input.iter().zip(expected) {
                assert_eq!(
                    json!(
                        orientation
                            .map_edge(raw_dimensions, support::pair(point))
                            .unwrap()
                    ),
                    *expected,
                    "{id}"
                );
            }
            let page = PageRegion::from_raw(
                bounds,
                case["page_index"].as_i64().unwrap(),
                orientation,
                support::coordinates(&sample["raw_rectangle"]),
            )
            .unwrap();
            let actual = support::page_output(page);
            assert_eq!(actual["coordinates"], sample["expected_rectangle"], "{id}");
            assert_eq!(actual["page_index"], case["page_index"], "{id}");
            assert_eq!(
                actual["source_orientation"], case["source_orientation"],
                "{id}"
            );
            assert_eq!(
                actual["bounds"]["page_count"], case["actual_page_count"],
                "{id}"
            );
            assert_eq!(
                actual["bounds"]["width"], case["upright_dimensions"][0],
                "{id}"
            );
            assert_eq!(
                actual["bounds"]["height"], case["upright_dimensions"][1],
                "{id}"
            );
            assert_eq!(
                actual["coordinate_space"], fixture["api"]["coordinate_space"],
                "{id}"
            );
        }
        println!("collector case={id} pid={}", std::process::id());
    }
    for case in fixture["orientation_errors"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        assert_eq!(
            Orientation::from_exif(case["input"].as_i64().unwrap()),
            Err(InvalidLocator),
            "{id}"
        );
        println!("collector case={id} pid={}", std::process::id());
    }
    for case in fixture["edge_errors"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let orientation =
            Orientation::from_exif(case["source_orientation"].as_i64().unwrap()).unwrap();
        assert_eq!(
            orientation.map_edge(
                support::pair(&case["raw_dimensions"]),
                support::pair(&case["point"])
            ),
            Err(InvalidLocator),
            "{id}"
        );
        println!("collector case={id} pid={}", std::process::id());
    }
    for case in fixture["raw_page_values"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let dimensions = support::pair(&case["raw_dimensions"]);
        let orientation =
            Orientation::from_exif(case["source_orientation"].as_i64().unwrap()).unwrap();
        let input = case["raw_corners"].as_array().unwrap();
        let expected = case["expected_corners"].as_array().unwrap();
        assert_eq!(input.len(), 4, "{id}");
        assert_eq!(expected.len(), 4, "{id}");
        for (point, expected) in input.iter().zip(expected) {
            assert_eq!(
                json!(
                    orientation
                        .map_edge(dimensions, support::pair(point))
                        .unwrap()
                ),
                *expected,
                "{id}"
            );
        }
        let bounds = PageBounds {
            page_count: support::fixture_u32(&case["actual_page_count"]),
            width: dimensions[0],
            height: dimensions[1],
        };
        let page = PageRegion::from_raw(
            bounds,
            case["page_index"].as_i64().unwrap(),
            orientation,
            support::coordinates(&case["raw_rectangle"]),
        )
        .unwrap();
        assert_eq!(support::page_output(page), case["expected"], "{id}");
        println!("collector case={id} pid={}", std::process::id());
    }
    for case in fixture["raw_page_errors"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let dims = support::pair(&case["raw_dimensions"]);
        let bounds = PageBounds {
            page_count: support::fixture_u32(&case["actual_page_count"]),
            width: dims[0],
            height: dims[1],
        };
        let orientation =
            Orientation::from_exif(case["source_orientation"].as_i64().unwrap()).unwrap();
        assert_eq!(
            PageRegion::from_raw(
                bounds,
                case["page_index"].as_i64().unwrap(),
                orientation,
                support::coordinates(&case["raw_rectangle"])
            ),
            Err(InvalidLocator),
            "{id}"
        );
        println!("collector case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 43);
}

#[test]
fn checked_audio_span_bounds() {
    use arboresce_domain::InvalidLocator;
    let original = support::primitive_fixture();
    let fixture = support::locator_fixture();
    let mut seen = HashSet::new();
    for case in original["groups"]["audio_span"]
        .as_array()
        .unwrap()
        .iter()
        .chain(fixture["audio_spans"].as_array().unwrap())
    {
        let id = case["id"].as_str().unwrap();
        let actual = support::audio_case(case).map(support::audio_output);
        if case["accepted"] == true {
            let expected = case
                .get("expected")
                .unwrap_or(&fixture["original_admitted_values"][id]);
            assert_eq!(actual.unwrap(), *expected, "{id}");
        } else {
            assert_eq!(actual, Err(InvalidLocator), "{id}");
        }
        let event = if id.starts_with("locator-") {
            id.to_owned()
        } else {
            format!("locator-{id}")
        };
        assert!(seen.insert(event.clone()));
        println!("collector case={event} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 24);
}

#[test]
fn locator_traits_and_input_independent_errors() {
    use arboresce_domain::{
        InvalidLocator, MAX_AUDIO_END_MS, MAX_BYTE_OFFSET, MAX_PIXEL_COMPONENT, Utf8Selection,
    };
    let fixture = support::locator_fixture();
    assert_eq!(
        MAX_BYTE_OFFSET,
        fixture["api"]["max_byte_offset"].as_u64().unwrap()
    );
    assert_eq!(
        u64::from(MAX_PIXEL_COMPONENT),
        fixture["api"]["max_pixel_component"].as_u64().unwrap()
    );
    assert_eq!(
        u64::from(MAX_AUDIO_END_MS),
        fixture["api"]["max_audio_end_ms"].as_u64().unwrap()
    );
    let error = Utf8Selection::new(b"\xff", 0, 1).unwrap_err();
    assert_eq!(error, InvalidLocator);
    assert_eq!(error.to_string(), fixture["api"]["error"]);
    assert_eq!(
        Error::source(&error).is_none(),
        fixture["api"]["error_source_none"].as_bool().unwrap()
    );
    let mut rejected = support::RejectedOutput;
    assert_eq!(
        write!(&mut rejected, "{error}").is_err(),
        fixture["api"]["error_formatter_failure_propagates"]
            .as_bool()
            .unwrap()
    );
    let value = Utf8Selection::new(b"synthetic text", 0, 9).unwrap();
    assert!(write!(&mut rejected, "{value:?}").is_err());
    let mut seen = HashSet::new();
    for case in fixture["traits"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        assert_eq!(support::locator_traits(case), case["expected"], "{id}");
        println!("collector case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 4);
}

#[test]
fn checked_tagged_revision_wire_objects() {
    use arboresce_domain::InvalidRevisionWire;
    let original = support::primitive_fixture();
    let fixture = support::wire_fixture();
    let mut seen = HashSet::new();
    for case in original["groups"]["revision_parse"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let event = format!("wire-original-{id}");
        assert!(seen.insert(event.clone()));
        let input = serde_json::to_vec(&case["input"]).unwrap();
        let actual =
            support::wire_typed(case["target"].as_str().unwrap(), &input).map(support::wire_output);
        if case["accepted"] == true {
            let actual = actual.unwrap();
            assert_eq!(actual, fixture["original_admitted_values"][id], "{id}");
            let roundtrip: serde_json::Value =
                serde_json::from_str(actual["canonical_utf8"].as_str().unwrap()).unwrap();
            assert_eq!(roundtrip, case["roundtrip"], "{id}");
        } else {
            assert_eq!(actual, Err(InvalidRevisionWire), "{id}");
        }
        println!("collector case={event} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 48);
}

#[test]
fn raw_tagged_revision_json() {
    use arboresce_domain::InvalidRevisionWire;
    let fixture = support::wire_fixture();
    let mut seen = HashSet::new();
    for case in fixture["raw_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let bytes = support::wire_bytes(case);
        let actual =
            support::wire_typed(case["target"].as_str().unwrap(), &bytes).map(support::wire_output);
        if case["accepted"] == true {
            assert_eq!(actual.unwrap(), case["expected"], "{id}");
        } else {
            assert_eq!(actual, Err(InvalidRevisionWire), "{id}");
        }
        println!("collector case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 92);
}

#[test]
fn revision_wire_traits_and_input_independent_errors() {
    use arboresce_domain::{InvalidRevisionWire, MAX_REVISION_WIRE_BYTES, WireRevision};
    let fixture = support::wire_fixture();
    let api = &fixture["api"];
    assert_eq!(
        MAX_REVISION_WIRE_BYTES as u64,
        api["max_wire_bytes"].as_u64().unwrap()
    );
    let error = WireRevision::from_json(b"invalid").unwrap_err();
    assert_eq!(error, InvalidRevisionWire);
    assert_eq!(error.to_string(), api["error"]);
    assert_eq!(format!("{error:?}"), api["error_debug"]);
    assert_eq!(
        Error::source(&error).is_none(),
        api["error_source_none"].as_bool().unwrap()
    );
    let mut rejected = support::RejectedOutput;
    assert_eq!(
        write!(&mut rejected, "{error}").is_err(),
        api["error_formatter_failure_propagates"].as_bool().unwrap()
    );
    let value = WireRevision::from_json(b"{\"kind\":\"content\",\"value\":\"1\"}").unwrap();
    assert_eq!(
        write!(&mut rejected, "{value}").is_err(),
        api["wire_formatter_failure_propagates"].as_bool().unwrap()
    );
    let mut seen = HashSet::new();
    for case in fixture["traits"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        assert_eq!(support::wire_traits(case), case["expected"], "{id}");
        println!("collector case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 4);
}
