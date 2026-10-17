#[path = "tests/support/authority_probes.rs"]
pub mod authority_probes;
#[path = "tests/support/authority_races.rs"]
pub mod authority_races;
#[path = "tests/support/authority_service.rs"]
pub mod authority_service;
#[path = "tests/support/current_authority.rs"]
pub mod current_authority;
#[path = "src/lib.rs"]
pub mod implementation;
#[path = "tests/support/relay.rs"]
pub mod relay;

use current_authority as support;
use implementation::Harness;

#[test]
fn actual_authority_races() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let (authority, _, realization) = support::fixtures().unwrap();
            let clock = support::input_configuration().unwrap();
            let service = authority_service::Service::open().await.unwrap();
            let cases: Vec<_> = authority["cases"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|c| c["group"] == "authority_races")
                .collect();
            assert_eq!(cases.len(), 7);
            for case in cases {
                let observed = authority_races::run(
                    &service,
                    &case["input"],
                    &support::recipe(&realization, case["id"].as_str().unwrap()),
                    &clock,
                )
                .await
                .unwrap();
                for (key, expected) in case["expected"].as_object().unwrap() {
                    if key == "canonical_snapshot_after" {
                        assert_eq!(
                            support::relation_order(&observed[key]),
                            support::relation_order(expected),
                            "{} full SQL after-state\n{observed}",
                            case["id"]
                        );
                    } else {
                        assert_eq!(&observed[key], expected, "{} {key}\n{observed}", case["id"]);
                    }
                }
                assert_eq!(observed["census"]["writers"], 0);
                assert!(observed["census"]["controls"].as_u64().unwrap() <= 3);
                assert!(observed["accounting"]["canonical_total"].as_i64().unwrap() <= 32);
                assert_eq!(observed["accounting"]["migration_receipts"], 4);
                assert!(
                    observed["accounting"]["physical_user_rows"]
                        .as_i64()
                        .unwrap()
                        <= 36
                );
                println!(
                    "postgres authority_race={} observation={observed}",
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
