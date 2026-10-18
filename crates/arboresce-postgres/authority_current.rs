#[path = "tests/support/authority_probes.rs"]
pub mod authority_probes;
#[path = "tests/support/authority_races.rs"]
pub mod authority_races;
#[path = "tests/support/authority_service.rs"]
pub mod authority_service;
#[path = "tests/support/current_authority.rs"]
mod current_authority;
#[path = "src/lib.rs"]
pub mod implementation;
#[path = "tests/support/relay.rs"]
pub mod relay;
#[path = "tests/support/shared_admission.rs"]
pub mod shared_admission;
#[path = "tests/support/shared_probes.rs"]
pub mod shared_probes;
#[path = "tests/support/shared_races.rs"]
pub mod shared_races;
#[path = "tests/support/shared_seed.rs"]
pub mod shared_seed;

use current_authority as support;
use implementation::Harness;
use serde_json::json;

#[test]
fn complete_shared_input_preparation() {
    let fixture = shared_admission::preparation_fixtures().unwrap();
    let cases = fixture["groups"]["shared_preparation"].as_array().unwrap();
    assert_eq!(cases.len(), 10);
    for case in cases {
        let observed = match shared_admission::preparation(&case["input"]) {
            Ok(value) => value,
            Err(error) => json!({"error":error.to_string()}),
        };
        assert_eq!(observed, case["expected"], "{}", case["id"]);
        println!(
            "collector case={} pid={}",
            case["id"].as_str().unwrap(),
            std::process::id()
        );
    }
}

#[test]
fn complete_ordered_claim_preparation() {
    let fixture = shared_admission::preparation_fixtures().unwrap();
    let cases = fixture["groups"]["ordered_claim_preparation"]
        .as_array()
        .unwrap();
    assert_eq!(cases.len(), 31);
    for case in cases {
        let input = &case["input"];
        let common = input.get("common").unwrap_or(&fixture["common_inputs"]);
        let observed =
            json!({"prepared":shared_races::prepare_ordered(common, &input["value"]).is_ok()});
        assert_eq!(observed, case["expected"], "{}", case["id"]);
        println!(
            "collector case={} pid={}",
            case["id"].as_str().unwrap(),
            std::process::id()
        );
    }
}

#[test]
fn complete_rls_query_preparation() {
    let fixture = shared_admission::preparation_fixtures().unwrap();
    let cases = fixture["groups"]["rls_preparation"].as_array().unwrap();
    assert_eq!(cases.len(), 33);
    for case in cases {
        let input = &case["input"];
        let common = input.get("common").unwrap_or(&fixture["common_inputs"]);
        let observed =
            json!({"prepared":shared_probes::prepare_rls(common, &input["value"]).is_ok()});
        assert_eq!(observed, case["expected"], "{}", case["id"]);
        println!(
            "collector case={} pid={}",
            case["id"].as_str().unwrap(),
            std::process::id()
        );
    }
}

#[test]
fn complete_shared_guard_preparation() {
    let fixture = shared_admission::guard_fixtures().unwrap();
    for (group, count) in [("native_preparation", 41), ("race_preparation", 13)] {
        let cases = fixture["groups"][group].as_array().unwrap();
        assert_eq!(cases.len(), count);
        for case in cases {
            let input = &case["input"];
            let observed = match group {
                "native_preparation" => match shared_admission::preparation(input) {
                    Ok(value) => value,
                    Err(error) => json!({"error":error.to_string()}),
                },
                "race_preparation" => json!({"prepared":shared_races::prepare_permission(
                    &input["common"], &input["value"]).is_ok()}),
                _ => unreachable!(),
            };
            assert_eq!(observed, case["expected"], "{}", case["id"]);
            println!(
                "collector case={} pid={}",
                case["id"].as_str().unwrap(),
                std::process::id()
            );
        }
    }
}

#[test]
fn complete_additional_shared_preparation() {
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
        let fixture = shared_admission::supplemental_fixtures().unwrap();
        for (group, count) in [("native_preparation",31), ("seed_preparation",46),
            ("race_guards",2), ("race_preparation",11)] {
            let cases = fixture["groups"][group].as_array().unwrap();
            assert_eq!(cases.len(), count);
            for case in cases {
                let input = &case["input"];
                let observed = match group {
                    "native_preparation" => shared_admission::preparation(input).unwrap(),
                    "seed_preparation" => match shared_seed::PreparedSeed::new(&input["common"], &input["ledger"]) {
                        Ok(prepared) => prepared.observation(),
                        Err(_) => json!({"prepared":false}),
                    },
                    "race_preparation" => json!({"prepared":shared_races::prepare_permission(&input["common"], &input["value"]).is_ok()}),
                    "race_guards" => shared_races::guard(input).await.unwrap(),
                    _ => unreachable!(),
                };
                assert_eq!(observed, case["expected"], "{}", case["id"]);
                println!("collector case={} pid={}", case["id"].as_str().unwrap(), std::process::id());
            }
        }
    });
}

#[test]
fn zz_actual_shared_reservations() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let (original, additional) = shared_admission::fixtures().unwrap();
            let common = &additional["common_inputs"];
            let mut service = authority_service::Service::open().await.unwrap();
            let mut mismatches = Vec::new();
            let old: Vec<_> = original["cases"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|case| case["group"] == "shared_admission")
                .collect();
            assert_eq!(old.len(), 23);
            for case in old {
                let observed = shared_admission::original(&service, common, &case["input"])
                    .await
                    .unwrap();
                let retained = support::retain_observation(&observed).unwrap();
                if observed["semantic"] != case["expected"] {
                    mismatches.push(format!("{} {retained}", case["id"]));
                }
                assert_eq!(
                    observed["evidence"]["census"]["writers"], 0,
                    "{}",
                    case["id"]
                );
                println!(
                    "postgres shared_original={} observation={retained}",
                    case["id"]
                );
                println!(
                    "collector case={} pid={}",
                    case["id"].as_str().unwrap(),
                    std::process::id()
                );
            }
            let supplemental = shared_admission::supplemental_fixtures().unwrap();
            assert_eq!(supplemental["common_inputs"], *common);
            for (group, count) in [
                ("sql_decoders", 59),
                ("native_controls", 20),
                ("native_races", 2),
            ] {
                let cases = supplemental["groups"][group].as_array().unwrap();
                assert_eq!(cases.len(), count);
                for case in cases {
                    let observed = if group == "sql_decoders" {
                        shared_probes::decoder(&service, &case["input"])
                            .await
                            .unwrap()
                    } else {
                        shared_admission::native(&service, common, &case["input"])
                            .await
                            .unwrap()
                    };
                    let retained = support::retain_observation(&observed).unwrap();
                    let mut actual = observed["semantic"].clone();
                    let mut expected = case["expected"].clone();
                    if let Some(authority) = actual.get_mut("canonical_authority_after") {
                        *authority = support::relation_order(authority);
                        expected["canonical_authority_after"] =
                            support::relation_order(&expected["canonical_authority_after"]);
                    }
                    if actual != expected {
                        mismatches.push(format!("{} {retained}", case["id"]));
                    }
                    if let Some(fault) = observed["evidence"].get("stored_fault") {
                        assert_eq!(fault["affected_rows"], 1, "{}", case["id"]);
                        assert_eq!(
                            fault["canonical_storage_before"], actual["canonical_storage_after"],
                            "{}",
                            case["id"]
                        );
                        assert_eq!(
                            support::relation_order(&fault["canonical_authority_before"]),
                            actual["canonical_authority_after"],
                            "{}",
                            case["id"]
                        );
                    }
                    assert_eq!(
                        observed["evidence"]["census"]["writers"], 0,
                        "{}",
                        case["id"]
                    );
                    println!(
                        "postgres shared_additional={} observation={retained}",
                        case["id"]
                    );
                    println!(
                        "collector case={} pid={}",
                        case["id"].as_str().unwrap(),
                        std::process::id()
                    );
                }
            }
            let guards = shared_admission::guard_fixtures().unwrap();
            assert_eq!(guards["common_inputs"], *common);
            for (group, count) in [("native_controls", 2), ("native_races", 1)] {
                let cases = guards["groups"][group].as_array().unwrap();
                assert_eq!(cases.len(), count);
                for case in cases {
                    let observed = shared_admission::native(&service, common, &case["input"])
                        .await
                        .unwrap();
                    let retained = support::retain_observation(&observed).unwrap();
                    let mut actual = observed["semantic"].clone();
                    let mut expected = case["expected"].clone();
                    actual["canonical_authority_after"] =
                        support::relation_order(&actual["canonical_authority_after"]);
                    expected["canonical_authority_after"] =
                        support::relation_order(&expected["canonical_authority_after"]);
                    if actual != expected {
                        mismatches.push(format!("{} {retained}", case["id"]));
                    }
                    if let Some(guard) = observed["evidence"].get("reset_guard") {
                        assert_eq!(
                            observed["evidence"]["before"],
                            observed["evidence"]["after"]
                        );
                        assert!(guard["backend_id"].as_i64().unwrap() > 0);
                        assert!(guard["elapsed_seconds"].as_f64().unwrap().is_finite());
                        assert!(guard["actual_timeout"].is_string());
                        assert!(guard["diagnostic"].as_str().unwrap().starts_with(
                            "owned database still has live sessions: observed=1, timeout="
                        ));
                    }
                    assert_eq!(
                        observed["evidence"]["census"]["writers"], 0,
                        "{}",
                        case["id"]
                    );
                    println!(
                        "postgres shared_guard={} observation={retained}",
                        case["id"]
                    );
                    println!(
                        "collector case={} pid={}",
                        case["id"].as_str().unwrap(),
                        std::process::id()
                    );
                }
            }
            let native = additional["groups"]["native"].as_array().unwrap();
            assert_eq!(native.len(), 80);
            let ordered: Vec<_> = native
                .iter()
                .filter(|case| case["input"]["recipe"] != "unavailable-actual-restart")
                .chain(
                    native
                        .iter()
                        .filter(|case| case["input"]["recipe"] == "unavailable-actual-restart"),
                )
                .collect();
            assert_eq!(ordered.len(), 80);
            assert_eq!(
                ordered.last().unwrap()["input"]["recipe"],
                "unavailable-actual-restart"
            );
            for case in ordered {
                let observed = if case["input"]["recipe"] == "unavailable-actual-restart" {
                    shared_admission::restart(&mut service, common, &case["input"])
                        .await
                        .unwrap()
                } else {
                    shared_admission::native(&service, common, &case["input"])
                        .await
                        .unwrap()
                };
                let retained = support::retain_observation(&observed).unwrap();
                let mut actual = observed["semantic"].clone();
                let mut expected = case["expected"].clone();
                actual["canonical_authority_after"] =
                    support::relation_order(&actual["canonical_authority_after"]);
                expected["canonical_authority_after"] =
                    support::relation_order(&expected["canonical_authority_after"]);
                if actual != expected {
                    mismatches.push(format!("{} {retained}", case["id"]));
                }
                assert_eq!(
                    observed["evidence"]["census"]["writers"], 0,
                    "{}",
                    case["id"]
                );
                println!(
                    "postgres shared_native={} observation={retained}",
                    case["id"]
                );
                println!(
                    "collector case={} pid={}",
                    case["id"].as_str().unwrap(),
                    std::process::id()
                );
            }
            service.finish().await.unwrap();
            assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
        });
}

#[test]
fn actual_owned_transport_close() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let fixture = authority_probes::close_fixtures().unwrap();
            let service = authority_service::Service::open().await.unwrap();
            service.reset().await.unwrap();
            service
                .harness
                .migrate(implementation::SchemaVersion::CurrentAuthority)
                .await
                .unwrap();
            let cases = fixture["groups"]["native"].as_array().unwrap();
            assert_eq!(cases.len(), 6);
            let mut mismatches = Vec::new();
            for case in cases {
                let observed = authority_probes::native_close(
                    &service,
                    &fixture["common_inputs"],
                    &case["input"],
                )
                .await
                .unwrap();
                let retained = support::retain_observation(&observed).unwrap();
                if observed["semantic"] != case["expected"] {
                    mismatches.push(format!("{} {retained}", case["id"]));
                }
                println!(
                    "postgres native_close={} observation={retained}",
                    case["id"]
                );
                println!(
                    "collector case={} pid={}",
                    case["id"].as_str().unwrap(),
                    std::process::id()
                );
            }
            service.finish().await.unwrap();
            assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
        });
}

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
    let input_errors = authority_probes::helper_input_error_fixtures().unwrap();
    let cases = input_errors["groups"]["close_faults"].as_array().unwrap();
    assert_eq!(cases.len(), 3);
    for case in cases {
        let actual = authority_probes::close_package_guard(&case["input"]);
        let observed = serde_json::to_value(actual.map_err(|error| error.to_string())).unwrap();
        assert_eq!(observed, case["expected"], "{}", case["id"]);
        let retained = support::retain_observation(&observed).unwrap();
        println!(
            "postgres close_input_error={} observation={retained}",
            case["id"]
        );
        println!(
            "collector case={} pid={}",
            case["id"].as_str().unwrap(),
            std::process::id()
        );
    }
    let cases = input_errors["groups"]["owned_pid_files"]
        .as_array()
        .unwrap();
    assert_eq!(cases.len(), 1);
    for case in cases {
        let observed = authority_probes::missing_owned_pid_guard(&case["input"]).unwrap();
        assert_eq!(observed["semantic"], case["expected"], "{}", case["id"]);
        assert_eq!(observed["evidence"]["cleanup_completed"], true);
        assert!(
            observed["evidence"]["before"]
                .get("postmaster.pid")
                .is_none()
        );
        assert_eq!(
            observed["evidence"]["before"],
            observed["evidence"]["after"]
        );
        let retained = support::retain_observation(&observed).unwrap();
        println!(
            "postgres missing_owned_pid={} observation={retained}",
            case["id"]
        );
        println!(
            "collector case={} pid={}",
            case["id"].as_str().unwrap(),
            std::process::id()
        );
    }
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let fixture = authority_probes::census_fixtures().unwrap();
            let cases = fixture["census"].as_array().unwrap();
            assert_eq!(cases.len(), 3);
            for case in cases {
                let observed = authority_service::controlled_census(&case["input"])
                    .await
                    .unwrap();
                assert_eq!(observed["semantic"], case["expected"], "{}", case["id"]);
                assert!(
                    observed["evidence"]["elapsed_seconds"]
                        .as_f64()
                        .unwrap()
                        .is_finite()
                );
                let retained = support::retain_observation(&observed).unwrap();
                println!(
                    "postgres controlled_census={} observation={retained}",
                    case["id"]
                );
                println!(
                    "collector case={} pid={}",
                    case["id"].as_str().unwrap(),
                    std::process::id()
                );
            }
            let guards = authority_probes::close_guard_fixtures().unwrap();
            let cases = guards["groups"]["package_guards"].as_array().unwrap();
            assert_eq!(cases.len(), 16);
            for case in cases {
                let observed = authority_probes::close_package_guard(&case["input"]).unwrap();
                assert_eq!(observed["semantic"], case["expected"], "{}", case["id"]);
                assert_eq!(observed["evidence"]["cleanup_completed"], true);
                let retained = support::retain_observation(&observed).unwrap();
                println!(
                    "postgres close_package_guard={} observation={retained}",
                    case["id"]
                );
                println!(
                    "collector case={} pid={}",
                    case["id"].as_str().unwrap(),
                    std::process::id()
                );
            }
            let fixture = authority_probes::close_fixtures().unwrap();
            let cases = fixture["groups"]["completion"].as_array().unwrap();
            assert_eq!(cases.len(), 5);
            for case in cases {
                let observed = authority_probes::controlled_close(&case["input"])
                    .await
                    .unwrap();
                assert_eq!(observed["semantic"], case["expected"], "{}", case["id"]);
                let retained = support::retain_observation(&observed).unwrap();
                println!(
                    "postgres controlled_close={} observation={retained}",
                    case["id"]
                );
                println!(
                    "collector case={} pid={}",
                    case["id"].as_str().unwrap(),
                    std::process::id()
                );
            }
        });
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
