use std::{collections::HashSet, error::Error, fmt::Write};

use arboresce_domain::{Identifier, InvalidIdentifier};

#[path = "fixtures/primitives-v1/expected.rs"]
mod expected;
mod support;

#[test]
fn observed_and_recorded_pairs() {
    use support::event_support as e;
    let fixture = e::fixture();
    let mut seen = HashSet::new();
    for row in fixture["groups"]["pairs"].as_array().unwrap() {
        let id = format!("event-pairs-{}", row["id"].as_str().unwrap());
        assert!(seen.insert(id.clone()));
        if let Some(pair) = row.get("input_pair") {
            let old = e::event(&pair[0]);
            let new = e::event(&pair[1]);
            assert_eq!(e::event_output(old), row["expected_pair"][0], "{id}");
            assert_eq!(e::event_output(new), row["expected_pair"][1], "{id}");
            assert_eq!(old == new, row["expected_equal"].as_bool().unwrap());
            assert_eq!(
                [old, new].into_iter().collect::<HashSet<_>>().len() as u64,
                row["expected_set_size"].as_u64().unwrap()
            );
            assert_eq!(e::event_output(old), row["expected_pair"][0], "{id}");
        } else {
            assert_eq!(
                e::event_output(e::event(&row["input"])),
                row["expected"],
                "{id}"
            );
        }
        println!("collector case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 31);
}

#[test]
fn attribution_and_calendar_policy() {
    use support::event_support as e;
    let fixture = e::fixture();
    let mut seen = HashSet::new();
    for row in fixture["groups"]["policies"].as_array().unwrap() {
        let id = format!("event-policies-{}", row["id"].as_str().unwrap());
        assert!(seen.insert(id.clone()));
        let result = e::reporting(&row["observed"], &row["policy"]);
        if row["accepted"] == true {
            assert_eq!(
                e::reporting_output(result.expect(&id)),
                row["expected"],
                "{id}"
            );
        } else {
            let error = result.unwrap_err();
            assert_eq!(format!("{error:?}"), row["error"], "{id}");
            assert_eq!(error.to_string(), row["message"], "{id}");
            assert!(Error::source(&error).is_none());
        }
        println!("collector case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 42);
}

#[test]
fn injected_event_time_receipts() {
    use arboresce_domain::{Clock, EventTimes};
    use serde_json::json;
    use support::event_support as e;
    let fixture = e::fixture();
    let mut seen = HashSet::new();
    for row in fixture["groups"]["clock"].as_array().unwrap() {
        let id = format!("event-clock-{}", row["id"].as_str().unwrap());
        assert!(seen.insert(id.clone()));
        let steps = row["steps"].as_array().unwrap();
        let supplied: Vec<_> = steps
            .iter()
            .map(|step| {
                if step["supplied"].is_string() {
                    json!({"ok":step["supplied"]})
                } else {
                    step["supplied"].clone()
                }
            })
            .collect();
        let mut clock = support::SuppliedClock::new(&supplied);
        let observed = (!row["observed"].is_null()).then(|| e::observed(&row["observed"]));
        let mut prior = Vec::new();
        for step in steps {
            let source: &mut dyn Clock<Error = String> = &mut clock;
            let result = EventTimes::record(source, observed);
            assert_eq!(clock.calls as u64, step["expected_calls"].as_u64().unwrap());
            if step["success"] == true {
                let value = result.expect(&id);
                assert_eq!(e::event_output(value), step["expected"], "{id}");
                prior.push((value, step["expected"].clone()));
            } else {
                assert_eq!(
                    result,
                    Err(step["expected_error"].as_str().unwrap().to_owned()),
                    "{id}"
                );
            }
            for (value, expected) in &prior {
                assert_eq!(e::event_output(*value), *expected, "{id}");
            }
        }
        assert_eq!(
            clock.calls as u64,
            row["expected_total_calls"].as_u64().unwrap()
        );
        assert!(row["previous_successes_unchanged"].as_bool().unwrap());
        println!("collector case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 12);
}

#[test]
fn unsupported_observed_time_input() {
    use arboresce_domain::{InvalidObservedTime, ObservedTime, ObservedValue};
    use support::event_support as e;
    let fixture = e::fixture();
    let attribution = e::attribution(&fixture["bases"]["attributions"][0]);
    let mut seen = HashSet::new();
    for row in fixture["groups"]["lexical"].as_array().unwrap() {
        let id = format!("event-lexical-{}", row["id"].as_str().unwrap());
        assert!(seen.insert(id.clone()));
        let result = ObservedValue::from_text(
            row["precision"].as_str().unwrap(),
            row["input"].as_str().unwrap(),
        );
        if row["accepted"] == true {
            assert_eq!(e::value_output(result.expect(&id)), row["expected"], "{id}");
        } else {
            assert_eq!(result, Err(InvalidObservedTime), "{id}");
            assert_eq!(InvalidObservedTime.to_string(), row["message"]);
            assert_eq!(format!("{:?}", InvalidObservedTime), row["error"]);
            let optional = result.map(|value| Some(ObservedTime::new(value, attribution)));
            assert_eq!(optional, Err(InvalidObservedTime));
            assert!(!row["expected_optional_observed_success"].as_bool().unwrap());
        }
        println!("collector case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 37);
}

#[test]
fn event_time_identity_and_traits() {
    use arboresce_domain::{InvalidObservedTime, TimePolicyError};
    use support::event_support as e;
    let fixture = e::fixture();
    let expected = &fixture["trait_expectations"];
    let mut seen = HashSet::new();
    for row in fixture["groups"]["traits"].as_array().unwrap() {
        let id = format!("event-traits-{}", row["id"].as_str().unwrap());
        assert!(seen.insert(id.clone()));
        let target = row["target"].as_str().unwrap();
        let first = e::identity(target, &row["input_pair"][0]);
        let second = e::identity(target, &row["input_pair"][1]);
        let copied = first;
        let cloned = e::cloned(&first);
        assert_eq!(
            copied == first && cloned == first,
            expected["copied_or_cloned_equal"].as_bool().unwrap()
        );
        assert_eq!(
            [first, copied, cloned]
                .into_iter()
                .collect::<HashSet<_>>()
                .len() as u64,
            expected["same_value_set_size"].as_u64().unwrap()
        );
        assert_eq!(first == second, row["expected_equal"].as_bool().unwrap());
        assert_eq!(
            [first, second].into_iter().collect::<HashSet<_>>().len() as u64,
            row["expected_set_size"].as_u64().unwrap()
        );
        assert_eq!(
            e::identity_output(first),
            row["expected_roundtrip_pair"][0],
            "{id}"
        );
        assert_eq!(
            e::identity_output(second),
            row["expected_roundtrip_pair"][1],
            "{id}"
        );
        if let e::Identity::Attribution(value) = first {
            assert_eq!(
                value.organization_id().to_string(),
                row["expected_organization_pair"][0]
            );
            let e::Identity::Attribution(other) = second else {
                unreachable!()
            };
            assert_eq!(
                other.organization_id().to_string(),
                row["expected_organization_pair"][1]
            );
        }
        println!("collector case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 32);
    let recorded = e::event(&fixture["groups"]["pairs"][0]["input"]).recorded();
    assert_eq!(
        write!(&mut support::RejectedOutput, "{recorded}").is_err(),
        expected["formatter_error_propagates"].as_bool().unwrap()
    );
    let errors: [&dyn Error; 4] = [
        &InvalidObservedTime,
        &TimePolicyError::MissingPolicy,
        &TimePolicyError::UnsupportedProfile,
        &TimePolicyError::ForeignOrganization,
    ];
    for error in errors {
        assert_eq!(
            error.source().is_none(),
            expected["error_source_none"].as_bool().unwrap()
        );
        assert_eq!(
            write!(&mut support::RejectedOutput, "{error}").is_err(),
            expected["formatter_error_propagates"].as_bool().unwrap()
        );
    }
}

#[test]
fn complete_original_locator_wire_cases() {
    use arboresce_domain::{InvalidLocatorWire, MAX_LOCATOR_WIRE_OUTPUT_BYTES};
    use support::record_support as r;
    let fixture = r::fixture();
    let originals = r::original_cases();
    let mut seen = HashSet::new();
    for id in fixture["original_wire_case_ids"].as_array().unwrap() {
        let id = id.as_str().unwrap();
        let event = format!("record-original-{id}");
        assert!(seen.insert(event.clone()));
        let row = originals["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        let input = serde_json::to_vec(&row["value"]).unwrap();
        let actual = r::read(row["target"].as_str().unwrap(), &input);
        if row["valid"] == true {
            let actual = actual.unwrap();
            assert_eq!(r::output(&actual), row["value"], "{id}");
            let serialized = actual.to_string();
            assert!(serialized.len() <= MAX_LOCATOR_WIRE_OUTPUT_BYTES);
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&serialized).unwrap(),
                row["value"],
                "{id}"
            );
        } else {
            assert_eq!(actual, Err(InvalidLocatorWire), "{id}");
        }
        println!("collector case={event} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 89);
}

#[test]
fn raw_unsigned_locator_fields() {
    use arboresce_domain::InvalidLocatorWire;
    use support::record_support as r;
    let fixture = r::fixture();
    let mut seen = HashSet::new();
    for row in fixture["groups"]["fields"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        let target = row["base"].as_str().unwrap();
        let event = format!("record-fields-{id}");
        assert!(seen.insert(event.clone()));
        let actual = r::read(target, &r::field_input(&fixture, row));
        if row["accepted"] == true {
            let mut expected = fixture["bases"][target].clone();
            r::replace(
                &mut expected,
                row["path"].as_array().unwrap(),
                row["expected_integer"].clone(),
            );
            let actual = actual.unwrap();
            assert_eq!(r::output(&actual), expected, "{id}");
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&actual.to_string()).unwrap(),
                expected,
                "{id}"
            );
        } else {
            assert_eq!(actual, Err(InvalidLocatorWire), "{id}");
        }
        let token = row["token"].as_str().unwrap();
        if (token == "0"
            || token
                .as_bytes()
                .first()
                .is_some_and(|b| matches!(b, b'1'..=b'9')))
            && token.bytes().all(|b| b.is_ascii_digit())
        {
            let number = token.parse::<u64>().unwrap();
            let mut input = fixture["bases"][target].clone();
            r::replace(&mut input, row["path"].as_array().unwrap(), number.into());
            let constructed = r::construct(target, &input);
            if row["accepted"] == true {
                assert_eq!(r::output(&constructed.unwrap()), input, "{id}");
            } else {
                assert_eq!(constructed, Err(InvalidLocatorWire), "{id}");
            }
        }
        println!("collector case={event} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 600);
}

#[test]
fn declared_original_profiles_and_constructors() {
    use arboresce_domain::{
        ByteIdentity, DeclaredOriginal, InvalidLocatorWire, Sha256Digest, SourceFormat,
        UnsignedJsonInteger,
    };
    use support::record_support as r;
    let fixture = r::fixture();
    let mut seen = HashSet::new();
    for row in fixture["groups"]["values"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        let event = format!("record-values-{id}");
        assert!(seen.insert(event.clone()));
        match row["call"].as_str().unwrap() {
            "declared" => {
                let format = row["format"]
                    .as_str()
                    .unwrap()
                    .parse::<SourceFormat>()
                    .unwrap();
                let length = row["byte_length"].as_u64().unwrap();
                assert_eq!(format.as_str(), row["format"]);
                assert_eq!(format.media_type(), row["media_type"]);
                let actual =
                    DeclaredOriginal::new(format, Sha256Digest::from_bytes([0xaa; 32]), length);
                let input = serde_json::json!({"format":row["format"],"media_type":row["media_type"],"sha256":"a".repeat(64),"byte_length":length});
                let wire = DeclaredOriginal::from_json(&serde_json::to_vec(&input).unwrap());
                assert_eq!(actual, wire, "{id}");
                if row["accepted"] == true {
                    let value = actual.unwrap();
                    assert_eq!(r::source_output(value), input, "{id}");
                    assert_eq!(
                        serde_json::from_str::<serde_json::Value>(&value.to_string()).unwrap(),
                        input
                    );
                } else {
                    assert_eq!(actual, Err(InvalidLocatorWire), "{id}");
                }
            }
            "unsigned" => {
                let token = row["token"].as_str().unwrap();
                let actual = UnsignedJsonInteger::from_token(token.as_bytes());
                assert_eq!(actual, token.parse::<UnsignedJsonInteger>());
                if row["accepted"] == true {
                    let value = actual.unwrap();
                    assert_eq!(value.value(), row["expected_integer"].as_u64().unwrap());
                    assert_eq!(value.to_string(), token);
                    assert_eq!([value, value].into_iter().collect::<HashSet<_>>().len(), 1);
                    assert!(format!("{value:?}").contains("UnsignedJsonInteger"));
                    assert!(write!(&mut support::RejectedOutput, "{value}").is_err());
                } else {
                    assert_eq!(actual, Err(InvalidLocatorWire), "{id}");
                }
                if let Some(input) = row.get("new_input") {
                    let input = input.as_u64().unwrap();
                    let actual = UnsignedJsonInteger::new(input);
                    let bytes = ByteIdentity::new(Sha256Digest::from_bytes([0; 32]), input);
                    assert_eq!(actual.is_ok(), row["new_accepted"].as_bool().unwrap());
                    assert_eq!(bytes.is_ok(), row["new_accepted"].as_bool().unwrap());
                    if let Ok(v) = bytes {
                        assert_eq!(v.byte_length(), input);
                    }
                }
            }
            "digest" => {
                let actual = row["input"].as_str().unwrap().parse::<Sha256Digest>();
                if row["accepted"] == true {
                    let value = actual.unwrap();
                    let expected = row["expected_hex"].as_str().unwrap();
                    assert_eq!(value.to_string(), expected);
                    assert_eq!(value.as_bytes().as_slice(), support::decode_hex(expected));
                    assert_eq!(Sha256Digest::from_bytes(*value.as_bytes()), value);
                    assert_eq!([value, value].into_iter().collect::<HashSet<_>>().len(), 1);
                    assert!(format!("{value:?}").contains("Sha256Digest"));
                    assert!(write!(&mut support::RejectedOutput, "{value}").is_err());
                } else {
                    assert_eq!(actual, Err(InvalidLocatorWire), "{id}");
                }
            }
            _ => panic!("unknown frozen value call"),
        }
        println!("collector case={event} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 56);
}

#[test]
fn pointer_syntax_and_decoded_tokens() {
    use arboresce_domain::{InvalidLocatorWire, JsonPointer};
    use support::record_support as r;
    let fixture = r::fixture();
    let mut seen = HashSet::new();
    for row in fixture["groups"]["pointers"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        let event = format!("record-pointers-{id}");
        assert!(seen.insert(event.clone()));
        let input = r::pointer_input(row);
        let actual = JsonPointer::parse(&input);
        assert_eq!(actual, input.parse::<JsonPointer>());
        if row["accepted"] == true {
            let value = actual.unwrap();
            assert_eq!(value.as_str(), input);
            let tokens: Vec<_> = value.tokens().collect();
            let expected = if row["tokens"].is_array() {
                row["tokens"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_owned())
                    .collect::<Vec<_>>()
            } else {
                let recipe = &row["expected_single_token_repeat"];
                vec![
                    recipe["unit"]
                        .as_str()
                        .unwrap()
                        .repeat(recipe["count"].as_u64().unwrap().try_into().unwrap()),
                ]
            };
            assert_eq!(tokens, expected, "{id}");
            assert_eq!(value.tokens().len(), expected.len());
            let cloned = value.clone();
            assert_eq!(value, cloned);
            assert_eq!(
                [value.clone(), cloned]
                    .into_iter()
                    .collect::<HashSet<_>>()
                    .len(),
                1
            );
            if !input.is_empty() {
                assert!(!format!("{value:?}").contains(&input), "{id}");
            }
            assert!(write!(&mut support::RejectedOutput, "{value:?}").is_err());
        } else {
            assert_eq!(actual, Err(InvalidLocatorWire), "{id}");
        }
        println!("collector case={event} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 27);
}

#[test]
fn closed_nested_record_json() {
    use arboresce_domain::InvalidLocatorWire;
    use support::record_support as r;
    let fixture = r::fixture();
    let mut seen = HashSet::new();
    for row in fixture["groups"]["objects"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        let event = format!("record-objects-{id}");
        assert!(seen.insert(event.clone()));
        let actual = r::read(row["target"].as_str().unwrap(), &r::raw_input(row));
        if row["accepted"] == true {
            let actual = actual.unwrap();
            assert_eq!(r::output(&actual), row["roundtrip"]);
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&actual.to_string()).unwrap(),
                row["roundtrip"]
            );
        } else {
            assert_eq!(actual, Err(InvalidLocatorWire), "{id}");
        }
        println!("collector case={event} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 458);
}

#[test]
fn raw_record_text_and_input_boundaries() {
    use arboresce_domain::{InvalidLocatorWire, MAX_LOCATOR_WIRE_OUTPUT_BYTES};
    use support::record_support as r;
    let fixture = r::fixture();
    let mut seen = HashSet::new();
    for row in fixture["groups"]["text"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        let event = format!("record-text-{id}");
        assert!(seen.insert(event.clone()));
        let input = r::raw_input(row);
        if let Some(size) = row.get("expected_input_bytes") {
            assert_eq!(input.len() as u64, size.as_u64().unwrap());
        }
        let actual = r::read(row["target"].as_str().unwrap(), &input);
        if row["accepted"] == true {
            let actual = actual.unwrap();
            assert_eq!(r::output(&actual), row["roundtrip"], "{id}");
            let out = actual.to_string();
            assert!(out.len() <= MAX_LOCATOR_WIRE_OUTPUT_BYTES);
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&out).unwrap(),
                row["roundtrip"],
                "{id}"
            );
            if let Some(expected) = row.get("output_utf8") {
                assert_eq!(out, expected.as_str().unwrap(), "{id}");
            }
        } else {
            assert_eq!(actual, Err(InvalidLocatorWire), "{id}");
        }
        println!("collector case={event} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 44);
}

#[test]
fn reference_locator_traits_and_getters() {
    use arboresce_domain::{
        InvalidLocatorWire, MAX_LOCATOR_WIRE_BYTES, MAX_LOCATOR_WIRE_OUTPUT_BYTES,
    };
    use support::record_support as r;
    let fixture = r::fixture();
    let expected = &fixture["trait_expectations"];
    let mut seen = HashSet::new();
    for row in fixture["groups"]["traits"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        let event = format!("record-traits-{id}");
        assert!(seen.insert(event.clone()));
        match row["call"].as_str().unwrap() {
            "record" => {
                let target = row["target"].as_str().unwrap();
                let input = &row["roundtrip"];
                let actual = r::read(target, &serde_json::to_vec(input).unwrap()).unwrap();
                assert_eq!(r::output(&actual), *input);
                assert_eq!(r::construct(target, input).unwrap(), actual);
                let traits = r::same_traits(&actual);
                assert_eq!(
                    traits,
                    (
                        expected["copied_or_cloned_equal"].as_bool().unwrap(),
                        expected["same_value_set_size"].as_u64().unwrap() as usize,
                        expected["formatter_error_propagates"].as_bool().unwrap()
                    )
                );
                if row["debug_must_omit_pointer"] == true {
                    assert!(!format!("{actual:?}").contains(input["pointer"].as_str().unwrap()));
                }
            }
            "identity" | "optional" => {
                let target = row["target"].as_str().unwrap();
                let inputs = if let Some(pair) = row.get("input_pair") {
                    pair.as_array().unwrap().clone()
                } else {
                    let first = fixture["bases"][row["base"].as_str().unwrap()].clone();
                    let mut second = first.clone();
                    r::replace(
                        &mut second,
                        row["path"].as_array().unwrap(),
                        row["replacement"].clone(),
                    );
                    vec![first, second]
                };
                let a = r::read(target, &serde_json::to_vec(&inputs[0]).unwrap()).unwrap();
                let b = r::read(target, &serde_json::to_vec(&inputs[1]).unwrap()).unwrap();
                assert_eq!(
                    a == b,
                    row.get("expected_equal")
                        .unwrap_or(&expected["different_values_equal"])
                        .as_bool()
                        .unwrap(),
                    "{id}"
                );
                assert_eq!(
                    [a.clone(), b.clone()]
                        .into_iter()
                        .collect::<HashSet<_>>()
                        .len() as u64,
                    row.get("expected_set_size")
                        .unwrap_or(&expected["different_value_set_size"])
                        .as_u64()
                        .unwrap(),
                    "{id}"
                );
                let outputs = vec![r::output(&a), r::output(&b)];
                assert_eq!(
                    outputs,
                    row.get("expected_roundtrip_pair")
                        .map(|v| v.as_array().unwrap())
                        .unwrap_or(&inputs)
                        .to_vec(),
                    "{id}"
                );
                assert_eq!(r::construct(target, &inputs[0]).unwrap(), a);
                assert_eq!(r::construct(target, &inputs[1]).unwrap(), b);
            }
            "union" => {
                let target = row["base"].as_str().unwrap();
                let input = &fixture["bases"][target];
                let bytes = serde_json::to_vec(input).unwrap();
                let actual = r::read("locator", &bytes).unwrap();
                assert_eq!(r::read(target, &bytes).unwrap(), actual);
                assert_eq!(r::construct(target, input).unwrap(), actual);
                assert_eq!(r::output(&actual)["kind"], row["kind"]);
                assert_eq!(r::output(&actual), *input);
                for other in ["utf8_range", "json_pointer", "page_region", "audio_span"] {
                    assert_eq!(r::read(other, &bytes).is_ok(), other == target);
                }
            }
            "distinct_kinds" => {
                let inputs = row["input_values"].as_array().unwrap();
                let values: Vec<_> = inputs
                    .iter()
                    .map(|i| r::read("locator", &serde_json::to_vec(i).unwrap()).unwrap())
                    .collect();
                assert_eq!(
                    values.iter().cloned().collect::<HashSet<_>>().len() as u64,
                    row["expected_set_size"].as_u64().unwrap()
                );
                let mut pairs = Vec::new();
                for a in 0..values.len() {
                    for b in a + 1..values.len() {
                        pairs.push(values[a] == values[b]);
                    }
                }
                assert_eq!(serde_json::json!(pairs), row["expected_pairwise_equal"]);
                assert_eq!(
                    serde_json::json!(values.iter().map(r::output).collect::<Vec<_>>()),
                    row["expected_roundtrips"]
                );
            }
            "error" => {
                let error = r::read("byte_identity", b"protected invalid input").unwrap_err();
                assert_eq!(error, InvalidLocatorWire);
                assert_eq!(error.to_string(), row["display"]);
                assert_eq!(format!("{error:?}"), row["debug"]);
                assert_eq!(
                    error.source().is_none(),
                    expected["error_source_none"].as_bool().unwrap()
                );
                assert!(write!(&mut support::RejectedOutput, "{error}").is_err());
                assert_eq!(
                    MAX_LOCATOR_WIRE_BYTES as u64,
                    row["input_maximum"].as_u64().unwrap()
                );
                assert_eq!(
                    MAX_LOCATOR_WIRE_OUTPUT_BYTES as u64,
                    row["output_maximum"].as_u64().unwrap()
                );
            }
            _ => panic!("unknown frozen record trait call"),
        }
        println!("collector case={event} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 113);
}

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
