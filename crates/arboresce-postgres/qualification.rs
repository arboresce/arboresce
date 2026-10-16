#[path = "src/lib.rs"]
mod implementation;
use arboresce_application::{MigrationStore, WriteOutcome};
use implementation::{Harness, MigrationFailure, SchemaVersion, migration};
use sqlx::Connection;

#[path = "tests/support/relay.rs"]
mod relay;
#[path = "tests/support/scenarios.rs"]
mod scenarios;
#[path = "tests/support/service.rs"]
mod support;

#[test]
fn checked_helper_preparation() {
    let fixtures: serde_json::Value = serde_json::from_str(include_str!(
        "../../tests/integration/postgres/fixtures/migration-service-v1/helper-cases.json"
    ))
    .unwrap();
    for case in fixtures["preparation_cases"].as_array().unwrap() {
        let prepared = serde_json::json!({"records":{"selected":case["record"]}});
        let result = match case["operation"].as_str().unwrap() {
            "record" => support::Service::record(&prepared, "selected").map(|record| {
                assert_eq!(
                    record.organization().to_string(),
                    fixtures["record_base"]["organization_id"]
                );
                assert_eq!(
                    record.record().to_string(),
                    fixtures["record_base"]["record_id"]
                );
                assert_eq!(
                    record.revision().value().to_string(),
                    fixtures["record_base"]["content_revision"]
                );
                assert_eq!(record.date().to_string(), fixtures["record_base"]["date"]);
                None
            }),
            "text" => support::Service::text(&case["recipe"]).map(|text| Some(text.len())),
            "records" => support::Records::new(&case["input"]).map(|_| None),
            "record-lookup" => support::Records::new(
                &serde_json::json!({"records":{"base":fixtures["record_base"]}}),
            )
            .unwrap()
            .get(case["name"].as_str().unwrap())
            .map(|_| None),
            _ => panic!("unknown preparation fixture"),
        };
        assert_eq!(
            result.is_ok(),
            case["accepted"].as_bool().unwrap(),
            "{}",
            case["id"]
        );
        if case.get("utf8_bytes").is_some() {
            assert_eq!(serde_json::json!(result.unwrap()), case["utf8_bytes"]);
        }
        println!(
            "collector case={} pid={}",
            case["id"].as_str().unwrap(),
            std::process::id()
        );
    }
}

#[test]
fn relay_owned_lifecycle_failures() {
    use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
    use std::sync::atomic::{AtomicU64, Ordering};
    let fixtures: serde_json::Value = serde_json::from_str(include_str!(
        "../../tests/integration/postgres/fixtures/migration-service-v1/helper-cases.json"
    ))
    .unwrap();
    for case in fixtures["relay_lifecycle_cases"].as_array().unwrap() {
        let observed = relay::lifecycle(case["recipe"].as_str().unwrap()).unwrap();
        println!("postgres relay_lifecycle={observed}");
        for (expected, actual) in [
            ("server_error_kinds", "server_error_kind"),
            ("client_error_kinds", "client_error_kind"),
        ] {
            if let Some(kinds) = case[expected].as_array() {
                assert!(
                    kinds.contains(&observed[actual]),
                    "{}: {observed}",
                    case["id"]
                );
            }
        }
        let elapsed = observed["elapsed_ms"].as_u64().unwrap();
        assert!(elapsed < case["maximum_ms"].as_u64().unwrap());
        if let Some(minimum) = case["minimum_ms"].as_u64() {
            assert!(elapsed >= minimum);
        }
        if case["recipe"]
            .as_str()
            .unwrap()
            .ends_with("target-read-closed")
        {
            assert_eq!(
                observed["producer_bytes"],
                case["producer_bytes"].as_u64().unwrap_or(1)
            );
            assert_eq!(observed["target_read_shutdown"], true);
            assert_eq!(observed["target_peer_closed"], true);
        } else {
            for key in [
                "relay_directory_absent",
                "relay_socket_absent",
                "backend_directory_absent",
            ] {
                assert_eq!(observed[key], true);
            }
            assert_eq!(
                observed["worker_results_observed"],
                case["recipe"] != "no-frontend-drop"
            );
        }
        println!(
            "collector case={} pid={}",
            case["id"].as_str().unwrap(),
            std::process::id()
        );
    }
    for (index, case) in fixtures["relay_directory_cases"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let parent_name = format!("arboresce-relay-allocation-{}-{index}", std::process::id());
        let parent = std::path::PathBuf::from("/private/tmp").join(&parent_name);
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&parent)
            .unwrap();
        let occupied = case["occupied"].as_u64().unwrap();
        let creator_uid = std::fs::symlink_metadata(&parent).unwrap().uid();
        let mut children = Vec::new();
        for number in 0..occupied {
            let path = parent.join(format!("entry-{}-{number}", std::process::id()));
            std::fs::DirBuilder::new()
                .mode(0o700)
                .create(&path)
                .unwrap();
            std::fs::write(
                path.join("marker"),
                case["original_marker"].as_str().unwrap(),
            )
            .unwrap();
            children.push(path);
        }
        if let Some(mode) = case["parent_mode_during_attempt"].as_u64() {
            std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(mode as u32))
                .unwrap();
        }
        let counter = AtomicU64::new(0);
        let mode_during = std::fs::symlink_metadata(&parent)
            .map(|metadata| metadata.permissions().mode() & 0o777);
        let started = std::time::Instant::now();
        let result = relay::owned_directory(&format!("{parent_name}/entry"), &counter);
        let elapsed = started.elapsed();
        // Restore before inspecting even an unexpected allocation outcome.
        std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o700)).unwrap();
        let restored_metadata = std::fs::symlink_metadata(&parent).unwrap();
        let mode_after = restored_metadata.permissions().mode() & 0o777;
        let markers: Vec<_> = children
            .iter()
            .map(|p| std::fs::read(p.join("marker")).unwrap())
            .collect();
        let selected = result.as_ref().ok().cloned();
        let mode = selected
            .as_ref()
            .map(|p| std::fs::symlink_metadata(p).unwrap().permissions().mode() & 0o777);
        if let Some(path) = &selected {
            assert_eq!(path.parent(), Some(parent.as_path()));
            assert!(!children.contains(path));
            std::fs::remove_dir(path).unwrap();
        }
        for path in children {
            std::fs::remove_file(path.join("marker")).unwrap();
            std::fs::remove_dir(path).unwrap();
        }
        std::fs::remove_dir(&parent).unwrap();
        println!(
            "postgres relay_directory={} result={result:?} counter={} elapsed_ms={} markers={markers:?} parent_absent={} creator_uid={creator_uid} mode_during={mode_during:?} mode_after={mode_after}",
            case["id"],
            counter.load(Ordering::Relaxed),
            elapsed.as_millis(),
            !parent.try_exists().unwrap()
        );
        assert!(!parent.try_exists().unwrap());
        assert_ne!(creator_uid, 0);
        assert_eq!(restored_metadata.uid(), creator_uid);
        assert_eq!(
            mode_during.unwrap() as u64,
            case["parent_mode_during_attempt"].as_u64().unwrap_or(0o700)
        );
        assert_eq!(mode_after, 0o700);
        assert_eq!(result.is_ok(), case["accepted"].as_bool().unwrap());
        assert_eq!(
            counter.load(Ordering::Relaxed),
            case["counter_after"].as_u64().unwrap()
        );
        assert!(elapsed < std::time::Duration::from_millis(case["maximum_ms"].as_u64().unwrap()));
        for marker in markers {
            assert_eq!(marker, case["original_marker"].as_str().unwrap().as_bytes());
        }
        if let Some(number) = case["selected_index"].as_u64() {
            assert_eq!(
                selected.unwrap(),
                parent.join(format!("entry-{}-{number}", std::process::id()))
            );
            assert_eq!(mode, Some(0o700));
        } else {
            assert_eq!(
                format!("{:?}", result.unwrap_err().kind()),
                case["error_kind"].as_str().unwrap()
            );
        }
        println!(
            "collector case={} pid={}",
            case["id"].as_str().unwrap(),
            std::process::id()
        );
    }
}

#[test]
fn relay_literal_protocol_and_budgets() {
    let fixtures: serde_json::Value = serde_json::from_str(include_str!(
        "../../tests/integration/postgres/fixtures/migration-service-v1/cases.json"
    ))
    .unwrap();
    for row in fixtures["relay_cases"].as_array().unwrap() {
        let raw = row["body_hex"].as_str().unwrap();
        let body: Vec<u8> = (0..raw.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&raw[i..i + 2], 16).unwrap())
            .collect();
        let mode = if row["mode"] == "commit" {
            relay::Mode::Commit
        } else {
            relay::Mode::Rollback
        };
        let result = relay::classify(
            mode,
            row["type_byte"].as_u64().unwrap() as u8,
            row["declared_length"].as_u64().unwrap() as u32,
            &body,
        );
        let actual = match result {
            Ok(true) => "drop",
            Ok(false) => "forward",
            Err(_) => "reject",
        };
        assert_eq!(actual, row["expected"]);
        println!(
            "collector case={} pid={}",
            row["id"].as_str().unwrap(),
            std::process::id()
        );
    }
    for row in fixtures["relay_length_cases"].as_array().unwrap() {
        let actual = relay::body_length(row["declared_length"].as_u64().unwrap() as u32).ok();
        assert_eq!(serde_json::json!(actual), row["expected_body_bytes"]);
        println!(
            "collector case={} pid={}",
            row["id"].as_str().unwrap(),
            std::process::id()
        );
    }
    for row in fixtures["relay_budget_cases"].as_array().unwrap() {
        let actual = relay::account(
            row["bytes_before"].as_u64().unwrap(),
            row["frames_before"].as_u64().unwrap(),
            row["incoming_bytes"].as_u64().unwrap(),
            row["server_frame"].as_bool().unwrap(),
        )
        .ok();
        assert_eq!(serde_json::json!(actual), row["expected"]);
        println!(
            "collector case={} pid={}",
            row["id"].as_str().unwrap(),
            std::process::id()
        );
    }
}

#[test]
fn relay_real_peer_boundaries() {
    let fixtures: serde_json::Value = serde_json::from_str(include_str!(
        "../../tests/integration/postgres/fixtures/migration-service-v1/cases.json"
    ))
    .unwrap();
    let helpers: serde_json::Value = serde_json::from_str(include_str!(
        "../../tests/integration/postgres/fixtures/migration-service-v1/helper-cases.json"
    ))
    .unwrap();
    for row in fixtures["physical_relay_cases"]
        .as_array()
        .unwrap()
        .iter()
        .chain(helpers["physical_relay_cases"].as_array().unwrap())
    {
        let id = row["id"].as_str().unwrap();
        let observed = relay::physical(id).unwrap();
        println!(
            "postgres physical_relay={id} result={:?} peer={:?} elapsed_ms={} bytes={}",
            observed.result,
            observed.peer,
            observed.elapsed.as_millis(),
            observed.forwarded.len()
        );
        let actual = if observed.result.is_ok() {
            "forward"
        } else {
            "reject"
        };
        if row["error_kinds"].is_array() {
            assert_eq!(actual, "reject", "{id}");
        } else {
            assert_eq!(actual, row["expected"], "{id}");
        }
        if let Err(error) = &observed.result {
            let kind = error.kind();
            if let Some(kinds) = row["error_kinds"].as_array() {
                assert!(
                    kinds.contains(&serde_json::json!(format!("{kind:?}"))),
                    "{id}: {error}"
                );
            } else if id.contains("one-over") {
                assert_eq!(kind, std::io::ErrorKind::InvalidData, "{id}");
            } else if id.contains("truncated") {
                assert_eq!(kind, std::io::ErrorKind::UnexpectedEof, "{id}");
            } else {
                assert!(
                    matches!(
                        kind,
                        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                    ),
                    "{id}: {error}"
                );
            }
        }
        if row["producer_success"] == true {
            assert!(observed.peer.is_ok(), "{id}: {:?}", observed.peer);
            assert_eq!(
                observed.forwarded.len(),
                row["forwarded_bytes"].as_u64().unwrap() as usize
            );
            assert!(
                observed.elapsed
                    < std::time::Duration::from_millis(row["maximum_ms"].as_u64().unwrap())
            );
            if let Some(minimum) = row["minimum_ms"].as_u64() {
                assert!(observed.elapsed >= std::time::Duration::from_millis(minimum));
            }
        }
        if id.contains("-client-") {
            let peer = observed.peer.as_ref().unwrap();
            assert!(peer.boundary_acknowledged);
            assert_eq!(peer.extra_byte_sent, id.ends_with("one-over"));
            assert_eq!(observed.forwarded.len(), 8_388_608);
            assert!(observed.forwarded.iter().all(|byte| *byte == 0));
            if let Ok(stats) = &observed.result {
                assert_eq!(stats.frames, 0);
            }
        } else if id.contains("bytes-") {
            assert_eq!(observed.forwarded.len(), 8_388_608);
            for frame in observed.forwarded.as_chunks::<1_048_576>().0.iter() {
                assert_eq!(&frame[..5], &[b'Z', 0, 15, 255, 255]);
                assert!(frame[5..].iter().all(|byte| *byte == 0));
            }
        } else if id.contains("frames-") {
            assert_eq!(observed.forwarded.len(), 4096 * 5);
            assert!(
                observed
                    .forwarded
                    .as_chunks::<5>()
                    .0
                    .iter()
                    .all(|frame| *frame == [b'Z', 0, 0, 0, 4])
            );
        } else if id.ends_with("io-deadline") {
            assert!(observed.elapsed >= std::time::Duration::from_millis(1900));
            assert!(observed.elapsed < std::time::Duration::from_secs(3));
            assert!(observed.forwarded.is_empty());
        } else if id.ends_with("total-deadline") {
            assert!(observed.elapsed >= std::time::Duration::from_millis(9900));
            assert!(observed.elapsed < std::time::Duration::from_secs(11));
            assert!(!observed.forwarded.is_empty());
            assert!(
                observed
                    .forwarded
                    .as_chunks::<5>()
                    .0
                    .iter()
                    .all(|frame| *frame == [b'Z', 0, 0, 0, 4])
            );
        } else {
            assert!(observed.forwarded.is_empty());
        }
        if row["expected"] == "forward" {
            observed.peer.unwrap();
            let stats = observed.result.unwrap();
            assert_eq!(stats.bytes, observed.forwarded.len() as u64);
            assert_eq!(stats.dropped, 0);
        }
        println!(
            "postgres physical_relay={id} elapsed_ms={} bytes={}",
            observed.elapsed.as_millis(),
            observed.forwarded.len()
        );
        println!("collector case={id} pid={}", std::process::id());
    }
}

#[test]
fn real_migration_scenarios() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    tokio::task::LocalSet::new().block_on(&runtime, async {
        if std::env::var_os(support::SELECTOR_PROBE).is_some() {
            let error = support::Service::open()
                .await
                .err()
                .map(|error| error.to_string());
            println!(
                "postgres selector_denial={}",
                serde_json::json!({"error":error})
            );
            return;
        }
        let mut service = support::Service::open().await.unwrap();
        let helpers: serde_json::Value = serde_json::from_str(include_str!(
            "../../tests/integration/postgres/fixtures/migration-service-v1/helper-cases.json"
        ))
        .unwrap();
        let input_cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../tests/integration/postgres/fixtures/migration-service-v1/cases.json"
        ))
        .unwrap();
        let namespace_before = service.snapshot().await.unwrap();
        let base = service.base().unwrap().to_owned();
        for case in helpers["service_directory_guard_cases"].as_array().unwrap() {
            use std::os::unix::fs::PermissionsExt;
            let files = || {
                std::fs::read_dir(&base).unwrap().map(|entry| {
                    let entry = entry.unwrap();
                    let metadata = std::fs::symlink_metadata(entry.path()).unwrap();
                    (entry.file_name(), if metadata.is_file() { Some(std::fs::read(entry.path()).unwrap()) } else { None })
                }).collect::<std::collections::BTreeMap<_,_>>()
            };
            let before = files();
            std::fs::set_permissions(&base, std::fs::Permissions::from_mode(case["mode_during_attempt"].as_u64().unwrap() as u32)).unwrap();
            let mode_during = std::fs::symlink_metadata(&base)
                .map(|metadata| metadata.permissions().mode() & 0o777);
            let started = std::time::Instant::now();
            let outcome = match case["operation"].as_str().unwrap() {
                "base" => service.base().map(|_| ()),
                "publish" => service.control(support::ControlKind::Ack, &serde_json::json!({})).map(|_| ()),
                "wait" => service.control_wait(support::ControlKind::Ack, "invalid", started + std::time::Duration::from_secs(1)).await.map(|_| ()),
                "native" => service.native(false, started + std::time::Duration::from_secs(1)),
                "selector" => service.selector_probe().map(|_| ()),
                "copy" => service.copy_input(1000, &serde_json::Value::Null).map(|_| ()),
                "control-probe" => service.control_probe(1000, &serde_json::json!({})).await.map(|_| ()),
                _ => panic!("unknown directory guard fixture"),
            };
            std::fs::set_permissions(&base, std::fs::Permissions::from_mode(0o700)).unwrap();
            let elapsed = started.elapsed();
            let after = files();
            let mode_after = std::fs::symlink_metadata(&base).unwrap().permissions().mode() & 0o777;
            println!("postgres service_directory={} outcome={outcome:?} elapsed_ms={} files_unchanged={} mode_during={mode_during:?} mode_after={mode_after}", case["id"], elapsed.as_millis(), before == after);
            assert_eq!(before, after);
            assert_eq!(mode_during.unwrap() as u64, case["mode_during_attempt"].as_u64().unwrap());
            assert_eq!(mode_after as u64, case["mode_after"].as_u64().unwrap());
            assert!(elapsed < std::time::Duration::from_millis(case["maximum_ms"].as_u64().unwrap()));
            assert_eq!(outcome.unwrap_err().to_string(), case["error"].as_str().unwrap());
            println!("collector case={} pid={}", case["id"].as_str().unwrap(), std::process::id());
        }
        assert_eq!(service.snapshot().await.unwrap(), namespace_before);
        let mut control = service.admin(true).await.unwrap();
        let system_identifier: String = sqlx::query_scalar("SELECT system_identifier::text FROM pg_control_system()")
            .fetch_one(&mut control).await.unwrap();
        drop(control);
        for (index, case) in helpers["live_fixture_cases"].as_array().unwrap().iter().enumerate() {
            let input = if case["recipe"].is_string() {
                serde_json::json!({"field":"system_identifier", "replacement":system_identifier})
            } else { case["input"].clone() };
            let outcome = service.live_denial(1000 + index, &input).await;
            assert_eq!(outcome.unwrap_err().to_string(), case["error"].as_str().unwrap());
            assert!(!base.join(format!("live-input-denial-{}.json", 1000 + index)).try_exists().unwrap());
            assert_eq!(service.snapshot().await.unwrap(), namespace_before);
            println!("collector case={} pid={}", case["id"].as_str().unwrap(), std::process::id());
        }
        for (index, case) in helpers["control_fixture_cases"].as_array().unwrap().iter().enumerate() {
            let outcome = service.control_probe(2000 + index, &case["input"]).await;
            assert_eq!(outcome.unwrap_err().to_string(), case["error"].as_str().unwrap());
            let directory = base.join(format!("qualifier-control-{}", 2000 + index));
            assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 0);
            assert_eq!(service.snapshot().await.unwrap(), namespace_before);
            println!("collector case={} pid={}", case["id"].as_str().unwrap(), std::process::id());
        }
        for (index, case) in input_cases["live_service_cases"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            let observed = service.live_denial(index, case).await.unwrap();
            assert_eq!(observed["accepted"], case["accepted"]);
            assert_eq!(observed["static_checks_passed"], true);
            assert_ne!(observed["original"], observed["replacement"]);
            assert!(observed["metadata_denial"].is_string());
            assert_eq!(service.snapshot().await.unwrap(), namespace_before);
            println!(
                "collector case={} pid={}",
                case["id"].as_str().unwrap(),
                std::process::id()
            );
        }
        service.reset().await.unwrap();
        service
            .harness
            .migrate(SchemaVersion::Initial)
            .await
            .unwrap();
        service.writer_identity(true).await.unwrap();
        let fixtures: serde_json::Value = serde_json::from_str(include_str!(
            "../../tests/integration/postgres/fixtures/migration-harness-v1/cases.json"
        ))
        .unwrap();
        let records = support::Records::new(&fixtures).unwrap();
        let row = &fixtures["records"]["exact-large"];
        let record = arboresce_application::MigrationRecord::new(
            row["organization_id"].as_str().unwrap().parse().unwrap(),
            row["record_id"].as_str().unwrap().parse().unwrap(),
            row["content_revision"].as_str().unwrap().parse().unwrap(),
            row["date"].as_str().unwrap().parse().unwrap(),
        );
        assert_eq!(
            service.harness.insert(record).await,
            WriteOutcome::Committed(record)
        );
        for case in helpers["scenario_cases"].as_array().unwrap() {
            let before = service.snapshot().await.unwrap();
            let occupied = case["id"] == "migration-helper-scenario-occupied-socket";
            let absent = service.socket().join("absent");
            if occupied {
                std::fs::create_dir(&absent).unwrap();
            }
            let error = scenarios::run(&service, &records, case["steps"].as_array().unwrap())
                .await
                .err()
                .map(|error| error.to_string());
            if occupied {
                std::fs::remove_dir(&absent).unwrap();
                assert!(!absent.try_exists().unwrap());
            }
            assert_eq!(serde_json::json!(error), case["error"]);
            assert_eq!(service.snapshot().await.unwrap(), before);
            println!(
                "collector case={} pid={}",
                case["id"].as_str().unwrap(),
                std::process::id()
            );
        }
        for (index, case) in helpers["service_guard_cases"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            let observed = service
                .guard_probe(index, case["recipe"].as_str().unwrap(), record)
                .await
                .unwrap();
            println!("postgres service_guard={observed}");
            assert_eq!(observed["error"], case["error"]);
            assert_eq!(observed["state_unchanged"], true);
            assert_eq!(observed["original_identity_unchanged"], true);
            if case["recipe"] == "short-restart-lease" {
                let remaining = observed["detail"]["remaining_ms_before_denial"]
                    .as_u64()
                    .unwrap();
                assert!((1..25000).contains(&remaining));
                assert_eq!(observed["detail"]["lease_still_valid"], true);
                assert_eq!(observed["detail"]["restart_controls_absent"], true);
            }
            println!(
                "collector case={} pid={}",
                case["id"].as_str().unwrap(),
                std::process::id()
            );
        }
        for (index, case) in helpers["control_file_cases"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            let observed = service.control_probe(index, case).await.unwrap();
            println!("postgres control_file={observed}");
            assert_eq!(observed["accepted"], case["accepted"]);
            if case.get("io_error_kind").is_some() {
                assert_eq!(observed["io_error_kind"], case["io_error_kind"]);
                assert!(observed["error"].is_string());
            } else {
                assert_eq!(observed["error"], case["error"]);
            }
            if case["phase"] == "publish" {
                assert_eq!(observed["final"], case["final"]);
                assert_eq!(observed["pending"], case["pending"]);
                assert_eq!(observed["base_mode_restored"], true);
                assert!(observed["elapsed_ms"].as_u64().unwrap() < 2000);
                if case["recipe"] == "base-mode" {
                    assert_ne!(observed["mode_during_attempt"], 0o700);
                }
                assert_eq!(
                    observed["alias_during_attempt"],
                    case["recipe"] == "base-symlink"
                );
                for (expected, actual) in [("final", "path"), ("pending", "pending_path")] {
                    let path = std::path::Path::new(observed[actual].as_str().unwrap());
                    if case[expected].is_null() {
                        assert!(!path.try_exists().unwrap());
                    } else {
                        let bytes = std::fs::read(path).unwrap();
                        assert_eq!(
                            bytes.len(),
                            case[expected]["file_bytes"].as_u64().unwrap() as usize
                        );
                        assert_eq!(
                            arboresce_xtask::inventory::sha256(&bytes),
                            case[expected]["sha256"]
                        );
                    }
                }
            }
            if let Some(bytes) = case.get("file_bytes") {
                assert_eq!(observed["bytes"], *bytes);
                assert_eq!(observed["regular_file"], true);
                let path = std::path::Path::new(observed["path"].as_str().unwrap());
                let bytes = std::fs::read(path).unwrap();
                let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                assert_eq!(parsed["request_sha256"], observed["request_sha256"]);
                assert_eq!(parsed["version"], "postgres-harness-restart-ack-v1");
                let body = serde_json::to_vec(&parsed).unwrap();
                assert_eq!(&bytes[..body.len()], body);
                assert!(bytes[body.len()..].iter().all(|byte| *byte == b' '));
                assert_eq!(
                    observed["sha256"],
                    arboresce_xtask::inventory::sha256(&bytes)
                );
            }
            println!(
                "collector case={} pid={}",
                case["id"].as_str().unwrap(),
                std::process::id()
            );
        }
        let fresh = crate::Harness::new(service.socket()).unwrap();
        assert_eq!(
            fresh
                .read(record.organization(), record.record())
                .await
                .unwrap(),
            Some(record)
        );
        let mut reader = service.admin(false).await.unwrap();
        let values: (String, String) = sqlx::query_as(
            "SELECT content_revision::text,record_date::text FROM migration_harness.records",
        )
        .fetch_one(&mut reader)
        .await
        .unwrap();
        assert_eq!(values, ("9007199254740993".into(), "2000-02-29".into()));
        let versions: Vec<i64> =
            sqlx::query_scalar("SELECT version FROM public._sqlx_migrations ORDER BY version")
                .fetch_all(&mut reader)
                .await
                .unwrap();
        assert_eq!(versions, vec![1]);
        drop(reader);
        for (mode, id) in [
            (relay::Mode::Commit, "commit-completion-lost"),
            (relay::Mode::Rollback, "rollback-completion-lost"),
        ] {
            service.reset().await.unwrap();
            service
                .harness
                .migrate(SchemaVersion::Initial)
                .await
                .unwrap();
            service.writer_identity(true).await.unwrap();
            let target = if matches!(mode, relay::Mode::Rollback) {
                assert_eq!(
                    service.harness.insert(record).await,
                    WriteOutcome::Committed(record)
                );
                let row = &fixtures["records"]["conflicting"];
                arboresce_application::MigrationRecord::new(
                    row["organization_id"].as_str().unwrap().parse().unwrap(),
                    row["record_id"].as_str().unwrap().parse().unwrap(),
                    row["content_revision"].as_str().unwrap().parse().unwrap(),
                    row["date"].as_str().unwrap().parse().unwrap(),
                )
            } else {
                record
            };
            let traffic = relay::Relay::start(&service.socket(), mode).unwrap();
            let writer = crate::Harness::new(traffic.directory.clone()).unwrap();
            assert_eq!(writer.insert(target).await, WriteOutcome::Unknown);
            let (server, client) = traffic.finish().unwrap();
            let stats = server.unwrap();
            assert_eq!(stats.dropped, 1);
            assert!(stats.bytes <= 8_388_608 && stats.frames <= 4096);
            println!("postgres relay={stats:?} client={client:?}");
            assert_eq!(
                service
                    .harness
                    .read(record.organization(), record.record())
                    .await
                    .unwrap(),
                Some(record)
            );
            println!(
                "collector case=migration-scenario-{id} pid={}",
                std::process::id()
            );
        }
        let core = [
            "empty-initial",
            "typed-exact-large",
            "typed-maximum",
            "separate-organization-key",
            "duplicate-is-not-replacement",
            "initial-migration-replay",
            "forward-compatible-expansion",
            "expanded-replay",
            "old-reader-preserves-expanded-data",
            "schema-downgrade-denied",
            "applied-checksum-mismatch",
            "failed-ddl-is-atomic",
            "failed-ddl-retry",
            "connection-unavailable-before-write",
            "fresh-connection-after-write",
            "interior-applied-history-gap",
        ];
        for id in core {
            let row = fixtures["scenarios"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| v["id"] == id)
                .unwrap();
            service.reset().await.unwrap();
            let observed = scenarios::run(&service, &records, row["steps"].as_array().unwrap())
                .await
                .unwrap();
            if let Some(expected) = row.get("migration_outcome") {
                assert_eq!(
                    format!("Rejected/{:?}", observed.migration.unwrap().unwrap_err()),
                    expected.as_str().unwrap(),
                    "{id}"
                );
            } else if let Some(result) = observed.migration {
                result.unwrap();
            }
            if let Some(expected) = row.get("outcome") {
                let actual = match observed.write.unwrap() {
                    WriteOutcome::Committed(_) => "Committed".to_owned(),
                    WriteOutcome::NotCommitted(error) => format!("NotCommitted/{error:?}"),
                    WriteOutcome::Unknown => "Unknown".into(),
                };
                assert_eq!(actual, expected.as_str().unwrap(), "{id}");
            }
            let (versions, records, annotations) = service.snapshot().await.unwrap();
            if let Some(expected) = row.get("versions") {
                assert_eq!(serde_json::json!(versions), *expected, "{id}");
            }
            if let Some(names) = row.get("rows") {
                let expected: Vec<_> = names
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|n| fixtures["records"][n.as_str().unwrap()].clone())
                    .collect();
                assert_eq!(records, expected, "{id}");
            }
            if let Some(expected) = row.get("annotations") {
                assert_eq!(serde_json::json!(annotations), *expected, "{id}");
            }
            if row["steps"]
                .as_array()
                .unwrap()
                .iter()
                .any(|s| s == "read-original-columns" || s == "new-direct-reader")
            {
                assert_eq!(observed.read, Some(record), "{id}");
            }
            if row.get("failed_relation_present").is_some() {
                let mut connection = service.admin(false).await.unwrap();
                let exists: bool = sqlx::query_scalar(
                    "SELECT to_regclass('migration_harness.failed_probe') IS NOT NULL",
                )
                .fetch_one(&mut connection)
                .await
                .unwrap();
                assert_eq!(
                    serde_json::json!(exists),
                    row["failed_relation_present"],
                    "{id}"
                );
            }
            println!(
                "collector case=migration-scenario-{id} pid={}",
                std::process::id()
            );
        }
        for case in helpers["native_baseline_cases"].as_array().unwrap() {
            let before = service.snapshot().await.unwrap();
            let mut worker = service.harness.connect(true).await.unwrap();
            let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
                .fetch_one(&mut worker)
                .await
                .unwrap();
            let error = scenarios::locked(&service, true)
                .await
                .err()
                .map(|error| error.to_string());
            let still_present: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
                .fetch_one(&mut worker)
                .await
                .unwrap();
            drop(worker);
            assert_eq!(pid, still_present);
            assert_eq!(serde_json::json!(error), case["error"]);
            assert_eq!(service.snapshot().await.unwrap(), before);
            println!("postgres occupied_migrator_pid={pid} unchanged_pid={still_present}");
            println!(
                "collector case={} pid={}",
                case["id"].as_str().unwrap(),
                std::process::id()
            );
        }
        for (id, simultaneous) in [
            ("concurrent-ordered-migrations", true),
            ("migration-lock-budget", false),
        ] {
            service.reset().await.unwrap();
            let row = fixtures["scenarios"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["id"] == id)
                .unwrap();
            let observed = scenarios::locked(&service, simultaneous).await.unwrap();
            assert!(observed.holder > 0);
            assert_eq!(observed.waiters.len(), if simultaneous { 2 } else { 1 });
            assert!(
                observed
                    .waiters
                    .iter()
                    .all(|pid| *pid > 0 && *pid != observed.holder)
            );
            assert_eq!(observed.relations_before_release, (false, false));
            if simultaneous {
                let outcomes: Vec<_> = observed
                    .outcomes
                    .into_iter()
                    .map(|result| {
                        result.unwrap();
                        "Applied"
                    })
                    .collect();
                assert_eq!(serde_json::json!(outcomes), row["migration_outcomes"]);
            } else {
                assert_eq!(
                    observed.outcomes,
                    vec![Err(crate::MigrationFailure::LockTimeout)]
                );
                assert_eq!(row["migration_outcome"], "Rejected/LockTimeout");
                assert!(observed.elapsed >= std::time::Duration::from_millis(4500));
                assert!(observed.elapsed < std::time::Duration::from_secs(7));
            }
            let (versions, records, _) = service.snapshot().await.unwrap();
            assert_eq!(serde_json::json!(versions), row["versions"]);
            assert_eq!(serde_json::json!(records), row["rows"]);
            println!(
                "postgres lock holder={} waiters={:?} elapsed_ms={}",
                observed.holder,
                observed.waiters,
                observed.elapsed.as_millis()
            );
            println!(
                "collector case=migration-scenario-{id} pid={}",
                std::process::id()
            );
        }
        for case in helpers["constraint_controls"].as_array().unwrap() {
            service.reset().await.unwrap();
            service
                .harness
                .migrate(SchemaVersion::Annotation)
                .await
                .unwrap();
            let result = scenarios::constraint(&service, &records, case).await;
            if let Some(expected) = case["expected"].as_str() {
                assert_eq!(result.unwrap(), expected);
                assert_eq!(
                    service
                        .harness
                        .read(record.organization(), record.record())
                        .await
                        .unwrap(),
                    Some(record)
                );
                assert_eq!(
                    service.snapshot().await.unwrap().1,
                    vec![fixtures["records"]["exact-large"].clone()]
                );
            } else {
                assert_eq!(
                    result.unwrap_err().to_string(),
                    case["error"].as_str().unwrap()
                );
                assert!(service.snapshot().await.unwrap().1.is_empty());
            }
            println!(
                "collector case={} pid={}",
                case["id"].as_str().unwrap(),
                std::process::id()
            );
        }
        service.reset().await.unwrap();
        service
            .harness
            .migrate(SchemaVersion::Annotation)
            .await
            .unwrap();
        let empty_constraints = service.snapshot().await.unwrap();
        assert_eq!(empty_constraints, (vec![1, 2], vec![], vec![]));
        let constraint_schema = service.schema().await.unwrap();
        assert!(!constraint_schema.is_empty());
        for row in fixtures["direct_constraints"].as_array().unwrap() {
            assert_eq!(service.snapshot().await.unwrap(), empty_constraints);
            assert_eq!(service.schema().await.unwrap(), constraint_schema);
            service.writer_identity(true).await.unwrap();
            assert_eq!(
                scenarios::constraint(&service, &records, row)
                    .await
                    .unwrap(),
                row["sqlstate"].as_str().unwrap()
            );
            assert_eq!(service.snapshot().await.unwrap(), empty_constraints);
            assert_eq!(service.schema().await.unwrap(), constraint_schema);
            println!(
                "collector case=migration-constraint-{} pid={}",
                row["id"].as_str().unwrap(),
                std::process::id()
            );
        }
        service.reset().await.unwrap();
        service
            .harness
            .migrate(SchemaVersion::Annotation)
            .await
            .unwrap();
        assert_eq!(
            service.harness.insert(record).await,
            WriteOutcome::Committed(record)
        );
        let empty_annotation = service.snapshot().await.unwrap();
        let annotation_schema = service.schema().await.unwrap();
        assert_eq!(annotation_schema, constraint_schema);
        assert_eq!(
            empty_annotation,
            (
                vec![1, 2],
                vec![fixtures["records"]["exact-large"].clone()],
                vec![String::new()]
            )
        );
        for case in helpers["snapshot_boundary_cases"].as_array().unwrap() {
            let recipe = case["recipe"].as_str().unwrap();
            let mut connection = service.admin(false).await.unwrap();
            let original_rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
                "SELECT organization_id::text,record_id::text,content_revision::text,record_date::text,annotation FROM migration_harness.records ORDER BY organization_id,record_id"
            ).fetch_all(&mut connection).await.unwrap();
            let original_history: Vec<String> = sqlx::query_scalar(
                "SELECT to_jsonb(m)::text FROM public._sqlx_migrations m ORDER BY version"
            ).fetch_all(&mut connection).await.unwrap();
            let original_counts: (i64, i64) = sqlx::query_as(
                "SELECT (SELECT COUNT(*) FROM pg_attribute WHERE attrelid='migration_harness.records'::regclass AND attnum>0 AND NOT attisdropped), (SELECT COUNT(*) FROM pg_constraint WHERE conrelid='migration_harness.records'::regclass)"
            ).fetch_one(&mut connection).await.unwrap();
            assert_eq!(original_counts, (5, 6));
            assert_eq!(service.snapshot().await.unwrap(), empty_annotation);
            assert_eq!(service.schema().await.unwrap(), annotation_schema);
            let mut changed = false;
            let mut actual_rows = Vec::<(String, String, String, String, String)>::new();
            let mut actual_columns = Vec::<(i16, String, String, bool, Option<String>)>::new();
            let mut actual_count = 0_i64;
            let mut added_names = Vec::new();
            let rename = match recipe {
                "ledger-column" => Some(("ALTER TABLE public._sqlx_migrations RENAME COLUMN version TO qualification_withheld_version", "ALTER TABLE public._sqlx_migrations RENAME COLUMN qualification_withheld_version TO version")),
                "row-column" => Some(("ALTER TABLE migration_harness.records RENAME COLUMN organization_id TO qualification_withheld_organization", "ALTER TABLE migration_harness.records RENAME COLUMN qualification_withheld_organization TO organization_id")),
                "annotation-column" => Some(("ALTER TABLE migration_harness.records RENAME COLUMN annotation TO qualification_withheld_annotation", "ALTER TABLE migration_harness.records RENAME COLUMN qualification_withheld_annotation TO annotation")),
                _ => None,
            };
            // No assertion runs while a temporary committed state still needs restoration.
            let observed: Result<serde_json::Value, Box<dyn std::error::Error>> = async {
                if recipe.starts_with("rows-") {
                    let rows = &helpers["snapshot_rows"].as_array().unwrap()[..case["rows"].as_u64().unwrap() as usize];
                    let mut transaction = connection.begin().await?;
                    sqlx::query("DELETE FROM migration_harness.records").execute(&mut *transaction).await?;
                    sqlx::query("INSERT INTO migration_harness.records (organization_id,record_id,content_revision,record_date) SELECT x.organization_id::uuid,x.record_id::uuid,x.content_revision::bigint,x.date::date FROM jsonb_to_recordset($1::jsonb) AS x(organization_id text,record_id text,content_revision text,date text)")
                        .bind(serde_json::to_string(rows)?).execute(&mut *transaction).await?;
                    transaction.commit().await?;
                    changed = true;
                    actual_count = sqlx::query_scalar("SELECT COUNT(*) FROM migration_harness.records").fetch_one(&mut connection).await?;
                    actual_rows = sqlx::query_as("SELECT organization_id::text,record_id::text,content_revision::text,record_date::text,annotation FROM migration_harness.records ORDER BY organization_id,record_id").fetch_all(&mut connection).await?;
                    service.snapshot().await.map(|state| serde_json::json!(state))
                } else if recipe.starts_with("schema-") {
                    added_names = (0..case["columns_to_add"].as_u64().unwrap()).map(|n| format!("qualification_bound_{n:02}")).collect();
                    let additions = added_names.iter().map(|name| format!("ADD COLUMN {name} text")).collect::<Vec<_>>().join(",");
                    let mut ddl = sqlx::QueryBuilder::<sqlx::Postgres>::new("ALTER TABLE migration_harness.records ");
                    // Identifiers come only from the fixed prefix and bounded decimal ordinals.
                    ddl.push(additions).build().execute(&mut connection).await?;
                    changed = true;
                    actual_count = sqlx::query_scalar("SELECT (SELECT COUNT(*) FROM pg_attribute WHERE attrelid='migration_harness.records'::regclass AND attnum>0 AND NOT attisdropped) + (SELECT COUNT(*) FROM pg_constraint WHERE conrelid='migration_harness.records'::regclass)").fetch_one(&mut connection).await?;
                    actual_columns = sqlx::query_as("SELECT a.attnum,a.attname::text,format_type(a.atttypid,a.atttypmod),a.attnotnull,pg_get_expr(d.adbin,d.adrelid) FROM pg_attribute a LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum WHERE a.attrelid='migration_harness.records'::regclass AND a.attnum>0 AND NOT a.attisdropped ORDER BY a.attnum").fetch_all(&mut connection).await?;
                    service.schema().await.map(|state| serde_json::json!(state))
                } else {
                    let (apply, _) = rename.ok_or("unknown snapshot fixture")?;
                    sqlx::query(apply).execute(&mut connection).await?;
                    changed = true;
                    if recipe == "annotation-column" {
                        service.annotate(record, "").await.map(|_| serde_json::Value::Null)
                    } else {
                        service.snapshot().await.map(|state| serde_json::json!(state))
                    }
                }
            }.await;
            if changed {
                if recipe.starts_with("rows-") {
                    let mut transaction = connection.begin().await.unwrap();
                    sqlx::query("DELETE FROM migration_harness.records").execute(&mut *transaction).await.unwrap();
                    for row in &original_rows {
                        sqlx::query("INSERT INTO migration_harness.records (organization_id,record_id,content_revision,record_date,annotation) VALUES ($1::uuid,$2::uuid,$3::bigint,$4::date,$5)")
                            .bind(&row.0).bind(&row.1).bind(&row.2).bind(&row.3).bind(&row.4).execute(&mut *transaction).await.unwrap();
                    }
                    transaction.commit().await.unwrap();
                } else if recipe.starts_with("schema-") {
                    let removals = added_names.iter().map(|name| format!("DROP COLUMN {name}")).collect::<Vec<_>>().join(",");
                    let mut ddl = sqlx::QueryBuilder::<sqlx::Postgres>::new("ALTER TABLE migration_harness.records ");
                    ddl.push(removals).build().execute(&mut connection).await.unwrap();
                } else {
                    let (_, restore) = rename.unwrap();
                    sqlx::query(restore).execute(&mut connection).await.unwrap();
                }
            }
            let restored_rows: Vec<(String, String, String, String, String)> = sqlx::query_as("SELECT organization_id::text,record_id::text,content_revision::text,record_date::text,annotation FROM migration_harness.records ORDER BY organization_id,record_id").fetch_all(&mut connection).await.unwrap();
            let restored_history: Vec<String> = sqlx::query_scalar("SELECT to_jsonb(m)::text FROM public._sqlx_migrations m ORDER BY version").fetch_all(&mut connection).await.unwrap();
            let restored_counts: (i64, i64) = sqlx::query_as("SELECT (SELECT COUNT(*) FROM pg_attribute WHERE attrelid='migration_harness.records'::regclass AND attnum>0 AND NOT attisdropped), (SELECT COUNT(*) FROM pg_constraint WHERE conrelid='migration_harness.records'::regclass)").fetch_one(&mut connection).await.unwrap();
            drop(connection);
            assert_eq!(restored_rows, original_rows);
            assert_eq!(restored_history, original_history);
            assert_eq!(restored_counts, original_counts);
            assert_eq!(service.snapshot().await.unwrap(), empty_annotation);
            assert_eq!(service.schema().await.unwrap(), annotation_schema);
            println!("postgres snapshot_boundary={} result={observed:?} actual_count={actual_count} rows={actual_rows:?} columns={actual_columns:?} restored=true", case["id"]);
            if recipe.starts_with("rows-") {
                let count = case["rows"].as_u64().unwrap() as usize;
                assert_eq!(actual_count as usize, count);
                let expected: Vec<_> = helpers["snapshot_rows"].as_array().unwrap()[..count].iter().map(|row| (
                    row["organization_id"].as_str().unwrap().to_owned(), row["record_id"].as_str().unwrap().to_owned(),
                    row["content_revision"].as_str().unwrap().to_owned(), row["date"].as_str().unwrap().to_owned(), String::new()
                )).collect();
                assert_eq!(actual_rows, expected);
                if case["accepted"] == true {
                    assert_eq!(observed.unwrap(), serde_json::json!((vec![1,2], &helpers["snapshot_rows"].as_array().unwrap()[..count], vec![String::new();count])));
                } else {
                    assert_eq!(observed.unwrap_err().to_string(), case["error"].as_str().unwrap());
                }
            } else if recipe.starts_with("schema-") {
                assert_eq!(actual_count as u64, case["expected_total"].as_u64().unwrap());
                let added: Vec<_> = actual_columns.iter().filter(|column| column.1.starts_with("qualification_bound_")).collect();
                assert_eq!(added.iter().map(|column| &column.1).collect::<Vec<_>>(), added_names.iter().collect::<Vec<_>>());
                for column in &added { assert_eq!((&column.2,column.3,&column.4), (&"text".to_owned(),false,&None)); }
                if case["accepted"] == true {
                    let output = observed.unwrap();
                    let definitions = output.as_array().unwrap();
                    assert_eq!(definitions.len() as u64, case["expected_total"].as_u64().unwrap());
                    for definition in &annotation_schema { assert!(definitions.contains(&serde_json::json!(definition))); }
                    let parsed: Vec<serde_json::Value> = definitions.iter().map(|value| serde_json::from_str(value.as_str().unwrap()).unwrap()).collect();
                    for column in added {
                        assert!(parsed.contains(&serde_json::json!({"kind":"column","ordinal":column.0,"name":column.1,"type":"text","not_null":false,"default":null})));
                    }
                } else {
                    assert_eq!(observed.unwrap_err().to_string(), case["error"].as_str().unwrap());
                }
            } else {
                let error = observed.unwrap_err();
                let database = error.downcast_ref::<sqlx::Error>().unwrap().as_database_error().unwrap();
                assert_eq!(database.code().as_deref(), case["sqlstate"].as_str());
            }
            println!("collector case={} pid={}", case["id"].as_str().unwrap(), std::process::id());
        }
        for row in fixtures["annotation_boundaries"].as_array().unwrap() {
            service.annotate(record, "").await.unwrap();
            assert_eq!(service.snapshot().await.unwrap(), empty_annotation);
            assert_eq!(service.schema().await.unwrap(), annotation_schema);
            service.writer_identity(true).await.unwrap();
            let text = support::Service::text(&row["text"]).unwrap();
            assert_eq!(text.len() as u64, row["utf8_bytes"].as_u64().unwrap());
            let mut connection = service.admin(false).await.unwrap();
            let result = sqlx::query("UPDATE migration_harness.records SET annotation=$1")
                .bind(&text)
                .execute(&mut connection)
                .await;
            if row["accepted"] == true {
                result.unwrap();
                let actual: String =
                    sqlx::query_scalar("SELECT annotation FROM migration_harness.records")
                        .fetch_one(&mut connection)
                        .await
                        .unwrap();
                assert_eq!(actual, text);
                assert_eq!(
                    service.snapshot().await.unwrap(),
                    (
                        empty_annotation.0.clone(),
                        empty_annotation.1.clone(),
                        vec![text]
                    )
                );
            } else {
                let error = result.unwrap_err();
                assert_eq!(
                    error.as_database_error().unwrap().code().as_deref(),
                    row["sqlstate"].as_str()
                );
                assert_eq!(service.snapshot().await.unwrap(), empty_annotation);
            }
            println!(
                "collector case={} pid={}",
                row["id"].as_str().unwrap(),
                std::process::id()
            );
            assert_eq!(service.schema().await.unwrap(), annotation_schema);
        }
        service.reset().await.unwrap();
        service
            .harness
            .migrate(SchemaVersion::Annotation)
            .await
            .unwrap();
        service.writer_identity(true).await.unwrap();
        assert_eq!(
            service.harness.insert(record).await,
            WriteOutcome::Committed(record)
        );
        assert_eq!(service.snapshot().await.unwrap(), empty_annotation);
        service.restart(record, "").await.unwrap();
        assert!(service.stopped_probe_error.is_none(), "{:?}", service.stopped_probe_error);
        assert!(service.stopped_elapsed < std::time::Duration::from_secs(2));
        assert_eq!(service.stopped_observations.len(), helpers["stopped_service_cases"].as_array().unwrap().len());
        for (case, (operation, result)) in helpers["stopped_service_cases"].as_array().unwrap().iter().zip(&service.stopped_observations) {
            assert_eq!(*operation, case["operation"].as_str().unwrap());
            let error = result.as_ref().unwrap_err();
            if case["error_type"] == "sqlx-io" {
                let sqlx::Error::Io(io) = error.downcast_ref::<sqlx::Error>().unwrap() else { panic!("unexpected outage error: {error:?}"); };
                println!("postgres stopped_service={} error={io:?} aggregate_ms={}", operation, service.stopped_elapsed.as_millis());
                assert_eq!(format!("{:?}", io.kind()), case["io_error_kind"].as_str().unwrap());
                assert_eq!(serde_json::json!(io.raw_os_error()), case["raw_os_error"]);
            } else {
                assert_eq!(error.to_string(), case["error"].as_str().unwrap());
            }
            println!("collector case={} pid={}", case["id"].as_str().unwrap(), std::process::id());
        }
        let restart = fixtures["scenarios"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == "owned-server-restart")
            .unwrap();
        let (versions, records, _) = service.snapshot().await.unwrap();
        assert_eq!(serde_json::json!(versions), restart["versions"]);
        assert_eq!(records, vec![fixtures["records"]["exact-large"].clone()]);
        let fresh = crate::Harness::new(service.socket()).unwrap();
        assert_eq!(
            fresh
                .read(record.organization(), record.record())
                .await
                .unwrap(),
            Some(record)
        );
        assert_eq!(restart["packaged_recovery_claim"], false);
        println!(
            "collector case=migration-scenario-owned-server-restart pid={}",
            std::process::id()
        );
        service.unchanged().await.unwrap();
        println!(
            "collector case=migration-first-real-record pid={}",
            std::process::id()
        );
    });
}
