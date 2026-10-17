#[path = "tests/support/authority_probes.rs"]
pub mod authority_probes;
#[path = "tests/support/authority_races.rs"]
pub mod authority_races;
#[path = "tests/support/authority_service.rs"]
mod authority_service;
#[path = "tests/support/current_authority.rs"]
mod current_authority;
#[path = "src/lib.rs"]
pub mod implementation;
#[path = "tests/support/relay.rs"]
pub mod relay;

use current_authority as support;
use implementation::Harness;
use serde_json::json;

#[test]
fn authority_failure_contract() {
    let (_, additional, _) = support::fixtures().unwrap();
    assert_eq!(
        additional["fixture_version"],
        "current-sql-authority-cases-v1"
    );
    let cases = additional["error_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 10);
    for case in cases {
        let observed = support::error_observation(&case["input"]);
        assert_eq!(observed, case["expected"], "{}", case["id"]);
        println!(
            "postgres current_failure={} observation={observed}",
            case["id"]
        );
        println!(
            "collector case=sql-native-{} pid={}",
            case["id"].as_str().unwrap(),
            std::process::id()
        );
    }
}

#[test]
fn authority_helper_guard_contract() {
    let (_, additional, _) = support::fixtures().unwrap();
    let cases = additional["guard_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 28);
    for case in cases {
        let observed = authority_service::helper_guard(&case["input"])
            .unwrap_or_else(|error| json!({"input_failure":error.to_string()}));
        assert_eq!(observed, case["expected"], "{}", case["id"]);
        println!(
            "authority helper_guard={} observation={observed}",
            case["id"]
        );
        println!(
            "collector case=sql-helper-{} pid={}",
            case["id"].as_str().unwrap(),
            std::process::id()
        );
    }
    let cleanup = additional["cleanup_cases"].as_array().unwrap();
    assert_eq!(cleanup.len(), 4);
    for case in cleanup {
        let observed = support::cleanup_result(&case["input"]);
        assert_eq!(observed, case["expected"], "{}", case["id"]);
        println!(
            "finite cleanup_result={} observation={observed}",
            case["id"]
        );
        println!(
            "collector case=sql-cleanup-{} pid={}",
            case["id"].as_str().unwrap(),
            std::process::id()
        );
    }
    let preparation = additional["race_guard_cases"].as_array().unwrap();
    assert_eq!(preparation.len(), 17);
    for case in preparation {
        let observed = support::race_preparation(&case["input"]);
        assert_eq!(observed, case["expected"], "{}", case["id"]);
        println!(
            "finite race_preparation={} observation={observed}",
            case["id"]
        );
        println!(
            "collector case=sql-preparation-{} pid={}",
            case["id"].as_str().unwrap(),
            std::process::id()
        );
    }
}

#[test]
fn migration_failure_mapping_contract() {
    let (_, additional, _) = support::fixtures().unwrap();
    let cases = additional["migration_error_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 4);
    for case in cases {
        let observed = support::migration_error_observation(&case["input"]).unwrap();
        assert_eq!(observed, case["expected"], "{}", case["id"]);
        println!(
            "literal migration_classification={} observation={observed}",
            case["id"]
        );
        println!(
            "collector case=sql-classifier-{} pid={}",
            case["id"].as_str().unwrap(),
            std::process::id()
        );
    }
}

#[test]
fn actual_current_authority() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let (authority, additional, realization) = support::fixtures().unwrap();
            let service = authority_service::Service::open().await.unwrap();
            let cases: Vec<_> = authority["cases"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|c| c["group"] == "current_authority")
                .collect();
            assert_eq!(cases.len(), 72);
            for case in cases {
                let observed = support::current(
                    &service,
                    &case["input"],
                    &support::recipe(&realization, case["id"].as_str().unwrap()),
                )
                .await
                .unwrap();
                let mut expected = case["expected"].clone();
                let state = expected
                    .as_object_mut()
                    .unwrap()
                    .remove("canonical_state_after")
                    .unwrap();
                // Reasons explain fixture rules; they are not runtime branch selectors.
                expected.as_object_mut().unwrap().remove("reason");
                let decision = observed["decision"].clone();
                assert_eq!(decision, expected, "{}\n{observed}", case["id"]);
                assert_eq!(
                    support::relation_order(&observed["canonical_state_after"]),
                    support::relation_order(&state),
                    "{} canonical SQL after-state",
                    case["id"]
                );
                assert_eq!(observed["markers"], json!([]), "{}", case["id"]);
                assert!(
                    observed["accounting"]["canonical_total"].as_i64().unwrap() <= 32,
                    "{}",
                    case["id"]
                );
                assert_eq!(observed["accounting"]["migration_receipts"], 4);
                assert_eq!(
                    observed["accounting"]["migration_versions"],
                    json!([1, 2, 3, 4])
                );
                assert_eq!(observed["accounting"]["other_user_rows"], 0);
                assert!(
                    observed["accounting"]["physical_user_rows"]
                        .as_i64()
                        .unwrap()
                        <= 36
                );
                assert_eq!(observed["census"]["writers"], 0);
                assert!(observed["census"]["controls"].as_u64().unwrap() <= 3);
                assert!(observed["same_backend"].as_bool().unwrap());
                assert!(observed["clock_calls"].as_u64().unwrap() <= 2);
                println!(
                    "postgres current_authority={} observation={observed}",
                    case["id"]
                );
                println!(
                    "collector case=sql-native-{} pid={}",
                    case["id"].as_str().unwrap(),
                    std::process::id()
                );
            }
            let cases: Vec<_> = additional["cases"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|c| c["group"] == "effect")
                .collect();
            assert_eq!(cases.len(), 4);
            for case in cases {
                let base = authority["cases"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|c| c["id"] == case["input"]["base"])
                    .unwrap();
                let observed = support::held_effect(
                    &service,
                    &case["input"],
                    &base["input"],
                    &support::recipe(&realization, base["id"].as_str().unwrap()),
                )
                .await
                .unwrap();
                for (key, expected) in case["expected"].as_object().unwrap() {
                    assert_eq!(&observed[key], expected, "{} {key}\n{observed}", case["id"]);
                }
                assert_eq!(observed["census"]["writers"], 0);
                assert!(observed["census"]["controls"].as_u64().unwrap() <= 3);
                assert!(
                    observed["accounting_after"]["canonical_total"]
                        .as_i64()
                        .unwrap()
                        <= 32
                );
                assert!(
                    observed["accounting_after"]["physical_user_rows"]
                        .as_i64()
                        .unwrap()
                        <= 36
                );
                println!("postgres held_fence={} observation={observed}", case["id"]);
                println!(
                    "collector case=sql-native-{} pid={}",
                    case["id"].as_str().unwrap(),
                    std::process::id()
                );
            }
            let trusted = support::probe_inputs().unwrap();
            let supplemental: Vec<_> = additional["cases"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|c| c["group"] != "effect")
                .collect();
            assert_eq!(supplemental.len(), 126);
            for case in supplemental {
                let base = case["input"]["base"].as_str().map(|id| {
                    authority["cases"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|c| c["id"] == id)
                        .unwrap()
                });
                let recipe = base.map(|c| support::recipe(&realization, c["id"].as_str().unwrap()));
                let observed = authority_probes::observe(
                    &service,
                    &case["input"],
                    base.map(|c| &c["input"]),
                    recipe.as_ref(),
                    &trusted,
                )
                .await
                .unwrap();
                for (key, expected) in case["expected"].as_object().unwrap() {
                    assert_eq!(&observed[key], expected, "{} {key}\n{observed}", case["id"]);
                }
                assert_eq!(observed["census"]["writers"], 0);
                assert!(observed["census"]["controls"].as_u64().unwrap() <= 3);
                assert!(observed["accounting"]["canonical_total"].as_i64().unwrap() <= 32);
                assert!(
                    observed["accounting"]["physical_user_rows"]
                        .as_i64()
                        .unwrap()
                        <= 36
                        || case["input"]["operation"] == "history_fault_after_four"
                );
                println!(
                    "postgres authority_probe={} observation={observed}",
                    case["id"]
                );
                println!(
                    "collector case=sql-native-{} pid={}",
                    case["id"].as_str().unwrap(),
                    std::process::id()
                );
            }
            service.finish().await.unwrap();
        });
}
