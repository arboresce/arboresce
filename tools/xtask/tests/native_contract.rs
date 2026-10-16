use std::{collections::BTreeSet, ffi::OsString, process::Command};

use arboresce_xtask::Action;

pub mod support;

#[test]
fn independently_approved_native_launch_diagnostics() {
    use arboresce_xtask::{
        measurement::{self, DiagnosticScope, Request},
        process,
    };
    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};
    use std::{
        fs,
        os::unix::{ffi::OsStrExt, process::ExitStatusExt},
        time::{Duration, Instant},
    };
    use support::measurement_inputs::{DiagnosticInput, controlled_write_failure};
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-launch-diagnostics-v1/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"].as_array().unwrap().iter().filter(|case| {
        matches!(
            case["operation"].as_str(),
            Some("open" | "spawn" | "prepared_spawn" | "record" | "record_writer_unit" | "success")
        )
    }) {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let recipe = &case["input"];
        let mut setup = DiagnosticInput::new(&fixture["context"], recipe).unwrap();
        let original_root = setup.input.root.canonicalize().unwrap();
        let context_bytes = fs::read(original_root.join("context.json")).unwrap();
        let intent_bytes = fs::read(&setup.intent).unwrap();
        let mut held = None;
        let failure = if case["operation"] == "open" {
            setup.before_prepare(recipe).unwrap();
            Request::open_diagnosed(&setup.input.root).err().expect(id)
        } else {
            let request = Request::open(&setup.input.root).unwrap();
            setup.before_prepare(recipe).unwrap();
            if case["operation"] == "spawn" {
                measurement::spawn_diagnosed(&mut setup.command, Some(&request)).expect_err(id)
            } else {
                let prepared = request.prepare_diagnosed(&mut setup.command).unwrap();
                if case["operation"] == "prepared_spawn" {
                    setup.after_prepare(recipe).unwrap();
                    prepared.spawn_diagnosed(&mut setup.command).expect_err(id)
                } else {
                    // Ambient measurement owns the actual child; the selected request owns only its own record.
                    let mut child = measurement::spawn(&mut setup.command).unwrap();
                    let pid = child.id();
                    if case["operation"] == "success" {
                        prepared.record_diagnosed(&mut child).unwrap();
                        let status = process::wait(&mut child, Duration::from_secs(30)).unwrap();
                        request.complete(&mut child).unwrap();
                        assert_eq!(json!(status.code()), case["expected"]["exit"], "{id}");
                        assert_eq!(json!(status.signal()), case["expected"]["signal"], "{id}");
                        for (directory, count) in
                            [("launches", "launch_receipts"), ("exits", "exit_receipts")]
                        {
                            assert_eq!(
                                fs::read_dir(original_root.join(directory)).unwrap().count() as u64,
                                case["expected"][count].as_u64().unwrap(),
                                "{id}"
                            );
                        }
                        let launch: Value = serde_json::from_slice(
                            &fs::read(original_root.join("launches").join(format!("{pid}.json")))
                                .unwrap(),
                        )
                        .unwrap();
                        let exit: Value = serde_json::from_slice(
                            &fs::read(original_root.join("exits").join(format!("{pid}.json")))
                                .unwrap(),
                        )
                        .unwrap();
                        assert_eq!(launch["pid"], pid, "{id}");
                        assert_eq!(exit["pid"], pid, "{id}");
                        assert_eq!(launch["version"], "native-launch-v1", "{id}");
                        assert_eq!(exit["version"], "native-exit-v1", "{id}");
                        assert!(
                            !setup.input.owner.path.join("launch-failure.json").exists(),
                            "{id}"
                        );
                        let retained = setup.input.owner.path.clone();
                        let launch_path =
                            original_root.join("launches").join(format!("{pid}.json"));
                        let exit_path = original_root.join("exits").join(format!("{pid}.json"));
                        let launch_bytes = fs::read(&launch_path).unwrap();
                        let exit_bytes = fs::read(&exit_path).unwrap();
                        drop(setup);
                        assert_eq!(
                            fs::read(retained.join("launch-command.json")).unwrap(),
                            intent_bytes,
                            "{id}"
                        );
                        assert_eq!(
                            fs::read(retained.join("context-original.json")).unwrap(),
                            context_bytes,
                            "{id}"
                        );
                        assert_eq!(fs::read(launch_path).unwrap(), launch_bytes, "{id}");
                        assert_eq!(fs::read(exit_path).unwrap(), exit_bytes, "{id}");
                        println!("xtask case={id} pid={}", std::process::id());
                        continue;
                    }
                    assert!(child.try_wait().unwrap().is_none(), "{id}");
                    setup.before_record(recipe, pid).unwrap();
                    let started = Instant::now();
                    let failure = if case["operation"] == "record_writer_unit" {
                        let error = controlled_write_failure(recipe);
                        prepared.record_failure(&mut child, error)
                    } else {
                        prepared.record_diagnosed(&mut child).unwrap_err()
                    };
                    assert!(
                        started.elapsed().as_millis()
                            <= fixture["profile"]["cleanup_budget_ms"].as_u64().unwrap() as u128,
                        "{id}"
                    );
                    let status = child.try_wait().unwrap().expect("actual child reaped");
                    assert_eq!(
                        json!(status.code()),
                        case["expected"]["cleanup"]["exit"],
                        "{id}"
                    );
                    assert_eq!(
                        json!(status.signal()),
                        case["expected"]["cleanup"]["signal"],
                        "{id}"
                    );
                    measurement::complete(&mut child).unwrap();
                    setup.restore_launch_permissions(recipe).unwrap();
                    let path = original_root.join("launches").join(format!("{pid}.json"));
                    match case["expected"]["receipt"].as_str().unwrap() {
                        "absent" => assert!(!path.exists(), "{id}"),
                        "existing-bytes-unchanged" => assert_eq!(
                            fs::read(&path).unwrap(),
                            recipe["collision_bytes"].as_str().unwrap().as_bytes(),
                            "{id}"
                        ),
                        "symlink-and-target-unchanged" => {
                            assert!(fs::symlink_metadata(&path).unwrap().is_symlink(), "{id}");
                            assert_eq!(fs::read_link(&path).unwrap(), setup.selected, "{id}");
                            assert_eq!(
                                fs::read(&setup.selected).unwrap(),
                                recipe["collision_bytes"].as_str().unwrap().as_bytes(),
                                "{id}"
                            );
                        }
                        _ => panic!("unknown reviewed receipt observation"),
                    }
                    assert!(
                        !original_root
                            .join("exits")
                            .join(format!("{pid}.json"))
                            .exists(),
                        "{id}"
                    );
                    held = Some(child);
                    failure
                }
            }
        };
        let expected = &case["expected"];
        let scope = if case["operation"] == "record_writer_unit" {
            DiagnosticScope::ControlledWriterWithActualCleanup
        } else {
            DiagnosticScope::ActualRuntime
        };
        let actual = failure.envelope(&setup.intent, &intent_bytes, scope);
        let mut wanted = fixture["protocol"]["full_envelope_template"].clone();
        wanted["scope"] = expected["scope"].clone();
        wanted["command"]["path"] = json!(setup.intent.as_os_str().as_bytes());
        wanted["command"]["sha256"] = json!(format!("{:x}", Sha256::digest(&intent_bytes)));
        wanted["failure"]["stage"] = expected["stage"].clone();
        wanted["failure"]["error_kind"] = expected["kind"].clone();
        wanted["failure"]["raw_os_error"] = expected["raw_os_error"].clone();
        wanted["failure"]["parent_pid"] = json!(std::process::id());
        wanted["failure"]["child_pid"] = json!(held.as_ref().map(std::process::Child::id));
        let mut binding = fixture["protocol"]["request_template"].clone();
        binding["context_path"] = json!(original_root.as_os_str().as_bytes());
        binding["context_sha256"] = json!(format!("{:x}", Sha256::digest(&context_bytes)));
        wanted["failure"]["request"] = if expected["request_binding"] == "null" {
            Value::Null
        } else {
            binding
        };
        wanted["failure"]["receipt_path"] = held.as_ref().map_or(Value::Null, |child| {
            json!(
                original_root
                    .join("launches")
                    .join(format!("{}.json", child.id()))
                    .as_os_str()
                    .as_bytes()
            )
        });
        wanted["failure"]["cleanup"] = expected["cleanup"].clone();
        assert_eq!(actual, wanted, "{id}");
        if held.is_none() {
            assert_eq!(
                fs::read_dir(original_root.join("launches"))
                    .map(|d| d.count())
                    .unwrap_or(0),
                0,
                "{id}"
            );
            assert_eq!(
                fs::read_dir(original_root.join("exits"))
                    .map(|d| d.count())
                    .unwrap_or(0),
                0,
                "{id}"
            );
        }
        let legacy = failure.into_legacy_error();
        assert_eq!(
            json!(format!("{:?}", legacy.kind())),
            expected["kind"],
            "{id}"
        );
        assert_eq!(
            json!(legacy.raw_os_error()),
            expected["raw_os_error"],
            "{id}"
        );
        setup.save_envelope(&actual).unwrap();
        let retained = setup.input.owner.path.clone();
        let receipt = held.as_ref().map(|child| {
            original_root
                .join("launches")
                .join(format!("{}.json", child.id()))
        });
        let receipt_bytes = receipt.as_ref().and_then(|path| fs::read(path).ok());
        let receipt_link = receipt.as_ref().and_then(|path| fs::read_link(path).ok());
        let selected = setup.selected.clone();
        let selected_bytes = fs::read(&selected).ok();
        drop(setup);
        assert_eq!(
            fs::read(retained.join("launch-command.json")).unwrap(),
            intent_bytes,
            "{id}"
        );
        assert_eq!(
            fs::read(retained.join("context-original.json")).unwrap(),
            context_bytes,
            "{id}"
        );
        assert_eq!(
            fs::read(retained.join("observed-envelope.json")).unwrap(),
            format!("{actual}\n").as_bytes(),
            "{id}"
        );
        assert_eq!(
            receipt.as_ref().and_then(|path| fs::read(path).ok()),
            receipt_bytes,
            "{id}"
        );
        assert_eq!(
            receipt.as_ref().and_then(|path| fs::read_link(path).ok()),
            receipt_link,
            "{id}"
        );
        assert_eq!(fs::read(selected).ok(), selected_bytes, "{id}");
        println!(
            "xtask diagnostic observation {}",
            json!({"case":id,"frame":actual,"retained":retained})
        );
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 18);
}

#[test]
fn independently_approved_native_diagnostic_retention() {
    use arboresce_xtask::measurement::{self, DiagnosticScope, Request};
    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};
    use std::{fs, os::unix::ffi::OsStrExt};
    use support::measurement_inputs::{DiagnosticInput, RetentionInput, representation_packet};
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-launch-diagnostics-v1/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"].as_array().unwrap().iter().filter(|case| {
        matches!(
            case["operation"].as_str(),
            Some("retain" | "retain_unit" | "retention_size_unit" | "artifact_writer_unit")
        )
    }) {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let recipe = &case["input"];
        let mut setup = DiagnosticInput::new(&fixture["context"], recipe).unwrap();
        let original_root = setup.input.root.canonicalize().unwrap();
        let context_bytes = fs::read(original_root.join("context.json")).unwrap();
        let intent_bytes = fs::read(&setup.intent).unwrap();
        let expected = &case["expected"];
        let frame;
        let mut wanted;
        if case["operation"] == "retention_size_unit" {
            // Literal packet input has no Child/OS-status claim.
            let packet = representation_packet(&fixture["protocol"]["unit_packet_base"], recipe);
            frame = packet.clone();
            wanted = packet;
        } else {
            let request = Request::open(&setup.input.root).unwrap();
            setup
                .before_prepare(&json!({"recipe":recipe["failure_recipe"]}))
                .unwrap();
            let failure =
                measurement::spawn_diagnosed(&mut setup.command, Some(&request)).expect_err(id);
            frame = failure.envelope(&setup.intent, &intent_bytes, DiagnosticScope::ActualRuntime);
            let primary = &expected["primary"];
            wanted = fixture["protocol"]["full_envelope_template"].clone();
            wanted["scope"] = expected["envelope_scope"].clone();
            wanted["command"]["path"] = json!(setup.intent.as_os_str().as_bytes());
            wanted["command"]["sha256"] = json!(format!("{:x}", Sha256::digest(&intent_bytes)));
            wanted["failure"]["stage"] = primary["stage"].clone();
            wanted["failure"]["error_kind"] = primary["kind"].clone();
            wanted["failure"]["raw_os_error"] = primary["raw_os_error"].clone();
            wanted["failure"]["parent_pid"] = json!(std::process::id());
            wanted["failure"]["child_pid"] = Value::Null;
            wanted["failure"]["cleanup"] = primary["cleanup"].clone();
            wanted["failure"]["receipt_path"] = Value::Null;
            let mut binding = fixture["protocol"]["request_template"].clone();
            binding["context_path"] = json!(original_root.as_os_str().as_bytes());
            binding["context_sha256"] = json!(format!("{:x}", Sha256::digest(&context_bytes)));
            wanted["failure"]["request"] = binding;
            assert_eq!(frame, wanted, "{id}");
        }
        let full = format!("{wanted}\n").into_bytes();
        if case["operation"] == "retention_size_unit" {
            assert_eq!(
                full.len() as u64,
                expected["serialized_bytes_including_lf"].as_u64().unwrap(),
                "{id}"
            );
            assert_eq!(
                json!(format!("{:x}", Sha256::digest(&full))),
                expected["serialized_sha256"],
                "{id}"
            );
        }
        setup.save_envelope(&frame).unwrap();
        let mut retention = RetentionInput::new(&setup, recipe).unwrap();
        let result = retention.retain(&frame);
        retention.restore_permissions().unwrap();
        let artifact = &retention.artifact;
        let target = &retention.target;
        let trace_path = &retention.trace_path;
        let returned=result.err().map_or(Value::Null,|failure|json!({
            "stage":failure.stage(),"kind":format!("{:?}",failure.error().kind()),"raw_os_error":failure.error().raw_os_error()
        }));
        assert_eq!(returned, expected["return_error"], "{id}");
        match expected["artifact"]["kind"].as_str().unwrap() {
            "full-envelope" | "complete-bytes-with-failed-flush" => {
                assert_eq!(fs::read(artifact).unwrap(), full, "{id}")
            }
            "absent" => assert!(!artifact.exists(), "{id}"),
            "existing-bytes-unchanged" => assert_eq!(
                fs::read(artifact).unwrap(),
                recipe["existing_bytes"].as_str().unwrap().as_bytes(),
                "{id}"
            ),
            "symlink-and-target-unchanged" => {
                assert!(fs::symlink_metadata(artifact).unwrap().is_symlink(), "{id}");
                assert_eq!(&fs::read_link(artifact).unwrap(), target, "{id}");
                assert_eq!(
                    fs::read(target).unwrap(),
                    recipe["existing_bytes"].as_str().unwrap().as_bytes(),
                    "{id}"
                );
            }
            "preserved-partial" => {
                let bytes = fs::read(artifact).unwrap();
                assert_eq!(
                    json!(bytes),
                    expected["artifact"]["exact_prefix_bytes"],
                    "{id}"
                );
                assert_eq!(
                    bytes.len() as u64,
                    expected["artifact"]["exact_size"].as_u64().unwrap(),
                    "{id}"
                );
            }
            _ => panic!("unknown reviewed artifact observation"),
        }
        let trace_bytes = fs::read(trace_path).unwrap();
        if let Some(kind) = expected["trace"]["kind"].as_str() {
            if kind == "full-envelope" {
                assert_eq!(trace_bytes, full, "{id}");
            } else if let Some(literal) = expected["trace"].get("literal") {
                assert_eq!(trace_bytes, format!("{literal}\n").as_bytes(), "{id}");
                assert_eq!(
                    trace_bytes.len() as u64,
                    expected["trace"]["bytes_including_lf"].as_u64().unwrap(),
                    "{id}"
                );
                assert_eq!(
                    json!(format!("{:x}", Sha256::digest(&trace_bytes))),
                    expected["trace"]["sha256"],
                    "{id}"
                );
            } else {
                let mut compact = fixture["protocol"]["fallback_template"].clone();
                compact["scope"] = expected["envelope_scope"].clone();
                compact["command"]["path_sha256"] = json!(format!(
                    "{:x}",
                    Sha256::digest(setup.intent.as_os_str().as_bytes())
                ));
                compact["command"]["sha256"] =
                    json!(format!("{:x}", Sha256::digest(&intent_bytes)));
                compact["diagnostic_sha256"] = json!(format!("{:x}", Sha256::digest(&full)));
                compact["diagnostic_bytes"] = json!(full.len());
                let primary = &expected["primary"];
                compact["failure"]["stage"] = primary["stage"].clone();
                compact["failure"]["error_kind"] = primary["kind"].clone();
                compact["failure"]["raw_os_error"] = primary["raw_os_error"].clone();
                compact["failure"]["parent_pid"] = json!(std::process::id());
                compact["failure"]["child_pid"] = Value::Null;
                compact["failure"]["cleanup"] = primary["cleanup"].clone();
                compact["failure"]["receipt_path_sha256"] = Value::Null;
                let mut binding = fixture["protocol"]["compact_request_template"].clone();
                binding["context_path_sha256"] = json!(format!(
                    "{:x}",
                    Sha256::digest(original_root.as_os_str().as_bytes())
                ));
                binding["context_sha256"] = json!(format!("{:x}", Sha256::digest(&context_bytes)));
                compact["failure"]["request"] = binding;
                compact["retention_error"] = expected["retention_error"].clone();
                assert_eq!(trace_bytes, format!("{compact}\n").as_bytes(), "{id}");
            }
            assert!(
                trace_bytes.len()
                    <= fixture["profile"]["diagnostic_limit_bytes"]
                        .as_u64()
                        .unwrap() as usize,
                "{id}"
            );
        } else if recipe["recipe"] == "trace-zero-write" {
            assert!(trace_bytes.is_empty(), "{id}");
        } else {
            assert_eq!(trace_bytes, full, "{id}");
        }

        if case["id"] == "diagnostic-retain-exclusive-success" {
            let native: Value = serde_json::from_str(include_str!(
                "../../../tests/architecture/fixtures/native-collector-v3/cases.json"
            ))
            .unwrap();
            let outer: Value = serde_json::from_str(include_str!(
                "../../../tests/architecture/fixtures/native-collector-v7/cases.json"
            ))
            .unwrap();
            let runner_setup = DiagnosticInput::new(&fixture["context"], recipe).unwrap();
            let original = runner_setup.input.root.canonicalize().unwrap();
            let original_bytes = fs::read(original.join("context.json")).unwrap();
            let observed = runner_setup
                .runner_failure(&native["mini_workspace_files"], recipe)
                .unwrap();
            let error = match observed.result {
                Err(error) => error,
                Ok(_) => panic!("missing selected context must fail actual Runner"),
            };
            let outer_oracle = outer["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["id"] == "collector-boundary-runner-false")
                .unwrap();
            assert_eq!(
                json!(error.to_string()),
                outer_oracle["expected"]["error"],
                "{id}"
            );
            let mini_oracle = native["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["id"] == "collector-native-real-mini")
                .unwrap();
            assert_eq!(
                observed.mini.record("result.json").unwrap(),
                mini_oracle["expected"]["success_record"],
                "{id}"
            );
            assert!(observed.mini.unchanged_source().unwrap(), "{id}");
            assert!(observed.mini.outer_preserved().unwrap(), "{id}");
            let command = runner_setup.input.owner.path.join("runner/commands/0000");
            let saved_intent = command.join("launch-command.json");
            let saved_bytes = fs::read(&saved_intent).unwrap();
            let mut expected_frame = fixture["protocol"]["full_envelope_template"].clone();
            let primary = &expected["primary"];
            expected_frame["scope"] = expected["envelope_scope"].clone();
            expected_frame["command"]["path"] = json!(saved_intent.as_os_str().as_bytes());
            expected_frame["command"]["sha256"] =
                json!(format!("{:x}", Sha256::digest(&saved_bytes)));
            expected_frame["failure"]["stage"] = primary["stage"].clone();
            expected_frame["failure"]["error_kind"] = primary["kind"].clone();
            expected_frame["failure"]["raw_os_error"] = primary["raw_os_error"].clone();
            expected_frame["failure"]["parent_pid"] = json!(std::process::id());
            expected_frame["failure"]["child_pid"] = Value::Null;
            expected_frame["failure"]["cleanup"] = primary["cleanup"].clone();
            expected_frame["failure"]["receipt_path"] = Value::Null;
            let mut binding = fixture["protocol"]["request_template"].clone();
            binding["context_path"] = json!(original.as_os_str().as_bytes());
            binding["context_sha256"] = json!(format!("{:x}", Sha256::digest(&original_bytes)));
            expected_frame["failure"]["request"] = binding;
            let expected_bytes = format!("{expected_frame}\n").into_bytes();
            let actual_bytes = fs::read(command.join("launch-failure.json")).unwrap();
            let actual_frame: Value = serde_json::from_slice(&actual_bytes).unwrap();
            assert_eq!(actual_bytes, expected_bytes, "{id}");
            assert_eq!(
                fs::read(command.join("trace")).unwrap(),
                expected_bytes,
                "{id}"
            );
            assert!(fs::read(command.join("stdout")).unwrap().is_empty(), "{id}");
            assert!(fs::read(command.join("stderr")).unwrap().is_empty(), "{id}");
            assert!(!command.join("completion.json").exists(), "{id}");
            assert_eq!(
                fs::read_dir(original.join("launches")).unwrap().count(),
                0,
                "{id}"
            );
            assert_eq!(
                fs::read_dir(original.join("exits")).unwrap().count(),
                0,
                "{id}"
            );
            assert_eq!(fs::read(&saved_intent).unwrap(), saved_bytes, "{id}");
            let runner_retained = runner_setup.input.owner.path.clone();
            drop(runner_setup);
            assert_eq!(
                fs::read(runner_retained.join("context-original.json")).unwrap(),
                original_bytes,
                "{id}"
            );
            assert_eq!(fs::read(&saved_intent).unwrap(), saved_bytes, "{id}");
            assert_eq!(
                fs::read(command.join("launch-failure.json")).unwrap(),
                actual_bytes,
                "{id}"
            );
            assert_eq!(
                fs::read(command.join("trace")).unwrap(),
                actual_bytes,
                "{id}"
            );
            assert!(!command.join("completion.json").exists(), "{id}");
            println!(
                "xtask diagnostic Runner observation {}",
                json!({
                    "case":id,"outer_error":error.to_string(),"frame":actual_frame,"retained":runner_retained,
                    "actual_prepared_mini":observed.mini.owner,"empty_streams":true,"completion_absent":true
                })
            );
        }

        let retained = setup.input.owner.path.clone();
        let artifact = artifact.clone();
        let target = target.clone();
        let trace_path = trace_path.clone();
        let artifact_bytes = fs::read(&artifact).ok();
        let artifact_link = fs::read_link(&artifact).ok();
        let target_bytes = fs::read(&target).ok();
        drop(retention);
        drop(setup);
        assert_eq!(
            fs::read(retained.join("launch-command.json")).unwrap(),
            intent_bytes,
            "{id}"
        );
        assert_eq!(
            fs::read(retained.join("context-original.json")).unwrap(),
            context_bytes,
            "{id}"
        );
        assert_eq!(
            fs::read(retained.join("observed-envelope.json")).unwrap(),
            format!("{frame}\n").as_bytes(),
            "{id}"
        );
        assert_eq!(fs::read(&artifact).ok(), artifact_bytes, "{id}");
        assert_eq!(fs::read_link(artifact).ok(), artifact_link, "{id}");
        assert_eq!(fs::read(target).ok(), target_bytes, "{id}");
        assert_eq!(fs::read(trace_path).unwrap(), trace_bytes, "{id}");
        println!(
            "xtask diagnostic retention {}",
            json!({"case":id,"returned":returned,"trace_bytes":trace_bytes.len(),"retained":retained})
        );
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 10);
}

#[test]
fn independently_approved_native_diagnostic_error_units() {
    use arboresce_xtask::measurement::{self, FailureCauses};
    use serde_json::{Value, json};
    use support::measurement_inputs::{constructed_error, controlled_write_failure};
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-launch-diagnostics-v1/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"].as_array().unwrap().iter().filter(|case| {
        matches!(
            case["operation"].as_str(),
            Some("writer_unit" | "cleanup_composition_unit")
        )
    }) {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let recipe = &case["input"];
        if case["operation"] == "writer_unit" {
            let failure = controlled_write_failure(recipe);
            let actual = json!({"phase":failure.phase().name(),"kind":format!("{:?}",failure.error().kind()),
                "raw_os_error":failure.error().raw_os_error(),"scope":"controlled-writer-unit-only"});
            assert_eq!(actual, case["expected"], "{id}");
            let legacy = failure.into_error();
            assert_eq!(
                json!(format!("{:?}", legacy.kind())),
                case["expected"]["kind"],
                "{id}"
            );
        } else {
            let causes = FailureCauses::new(
                constructed_error(&recipe["primary"]),
                Some(Err(constructed_error(&recipe["cleanup"]))),
            );
            let primary = causes.primary();
            let cleanup = causes.cleanup();
            let legacy = causes.into_legacy_error();
            let actual = json!({"scope":"controlled-error-composition-unit-only","primary":primary,
                "cleanup":cleanup,"legacy_error":measurement::io_cause(&legacy),"child_status":null});
            assert_eq!(actual, case["expected"], "{id}");
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 4);
}

#[test]
fn independently_approved_collector_native_boundaries() {
    use serde_json::Value;
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v7/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"].as_array().unwrap().iter().filter(|c| {
        matches!(
            c["action"].as_str(),
            Some("probe-error" | "capture-error" | "postspawn-io")
        )
    }) {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        if case["action"] == "postspawn-io" {
            assert_eq!(
                support::collection_native::postspawn_io(case).unwrap(),
                case["expected"],
                "{id}"
            );
        } else if case["action"] == "capture-error" {
            assert_eq!(
                support::collection_native::pre_spawn(case).unwrap(),
                case["expected"],
                "{id}"
            );
        } else {
            let error = support::collection_native::probe(&case["recipe"]).unwrap_err();
            let message = error.to_string();
            let payload = message
                .strip_prefix(case["expected"]["error_contains"].as_str().unwrap())
                .unwrap();
            let actual: Value = serde_json::from_str(payload).unwrap();
            assert_eq!(actual["exit"], case["expected"]["peer_exit"], "{id}");
            assert!(actual["signal"].is_null());
            assert!(actual["stdout"].as_array().unwrap().is_empty());
            let stderr = String::from_utf8(support::collection_inputs::literal_bytes(
                &serde_json::json!({"bytes":actual["stderr"]}),
            ))
            .unwrap();
            assert!(
                stderr.starts_with(case["expected"]["peer_stderr_prefix"].as_str().unwrap()),
                "{id}"
            );
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 5);
}

#[test]
fn independently_approved_collector_boundary_modules() {
    use arboresce_xtask::collection_modules;
    use serde_json::Value;
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v7/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["action"] == "module")
    {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let object = support::collection_inputs::hex(case, "object_hex").unwrap();
        let profile = support::collection_inputs::hex(case, "profile_hex").unwrap();
        assert_eq!(
            collection_modules::matching(&object, &profile)
                .unwrap_err()
                .to_string(),
            case["expected"]["error"],
            "{id}"
        );
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 6);
}

#[test]
fn independently_approved_collector_native_exercise_vectors() {
    use arboresce_xtask::{collection_artifacts, inventory::sha256};
    use serde_json::json;
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v6/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"].as_array().unwrap().iter().filter(|case| {
        matches!(
            case["action"].as_str(),
            Some("exercise" | "exercise-recipe")
        )
    }) {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let binary = support::collection_inputs::exercise_binary(case);
        match collection_artifacts::exercises(&binary) {
            Ok(vectors) => {
                let actual = vectors
                    .iter()
                    .map(|v| {
                        json!({"arguments":v.arguments,"exit":v.exit,
                    "stdout_size":v.stdout.len(),"stdout_sha256":sha256(&v.stdout),
                    "stderr_size":v.stderr.len(),"stderr_sha256":sha256(&v.stderr)})
                    })
                    .collect::<Vec<_>>();
                assert_eq!(json!(actual), case["expected"]["value"], "{id}");
            }
            Err(error) => assert_eq!(error.to_string(), case["expected"]["error"], "{id}"),
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 29);
}

#[test]
fn independently_approved_collector_real_native_binary_modules() {
    use arboresce_xtask::collection_inputs;
    use serde_json::json;
    use std::fs;
    let base: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v3/cases.json"
    ))
    .unwrap();
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v6/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["action"] == "mini-binary")
    {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let mini = support::collection_mini::Mini::run(&base, case).unwrap();
        assert_eq!(
            mini.actual.status.code().unwrap() as i64,
            case["expected"]["exit"].as_i64().unwrap(),
            "{id}; retained {}",
            mini.owner.display()
        );
        assert_eq!(
            mini.actual.stderr,
            case["expected"]["stderr"].as_str().unwrap().as_bytes()
        );
        assert!(mini.unchanged_source().unwrap());
        assert!(mini.outer_preserved().unwrap());
        let objects = mini.record("objects.json").unwrap();
        let binary = objects
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["role"] == "binary")
            .unwrap();
        let exercised = mini.record("native-exercises.json").unwrap();
        let actual = exercised.as_array().unwrap();
        assert_eq!(actual.len(), 2);
        assert_eq!(actual[0]["arguments"], json!([]));
        assert_eq!(actual[0]["exit"], 0);
        assert_eq!(actual[1]["arguments"], json!(["--failure"]));
        assert_eq!(actual[1]["exit"], 255);
        assert_eq!(
            actual[0]["stdout_sha256"],
            "10159baf262b43a92d95db59dae1f72c645127301661e0a3ce4e38b295a97c58"
        );
        assert_eq!(
            actual[1]["stderr_sha256"],
            "92e33578c3788e4379042de27928686e83a61b7f2b6f8903e08b5d4655abd353"
        );
        for row in actual {
            assert_eq!(row["object_sha256"], binary["sha256"]);
            let command = mini
                .commands()
                .unwrap()
                .into_iter()
                .find(|(_, _, c)| c["pid"] == row["pid"])
                .unwrap();
            assert_eq!(
                fs::read(command.0.join("stdout")).unwrap(),
                if row["exit"] == 0 {
                    b"7\n".as_slice()
                } else {
                    b""
                }
            );
            assert_eq!(
                fs::read(command.0.join("stderr")).unwrap(),
                if row["exit"] == 0 {
                    b"".as_slice()
                } else {
                    "échec\n".as_bytes()
                }
            );
            assert_eq!(command.2["exit"], row["exit"]);
        }
        if mini.actual.status.success() {
            assert_eq!(
                mini.record("result.json").unwrap(),
                case["expected"]["stdout_record"]
            );
            assert_eq!(
                mini.actual.stdout,
                format!("{}\n", case["expected"]["stdout_record"]).as_bytes()
            );
            support::collection_mini::assert_mini_module_pairs(
                &mini,
                case["expected"]["module_gate_pairs"].as_u64().unwrap() as usize,
            );
            let facts = mini.record("collection-facts.json").unwrap();
            let [scope] = facts["binary_scopes"].as_array().unwrap().as_slice() else {
                panic!("{id}")
            };
            assert_eq!(scope["actual_sources"], case["expected"]["binary_sources"]);
            assert_eq!(
                scope["registered_sources"],
                case["expected"]["binary_sources"]
            );
            assert_eq!(
                scope["profile_pids"].as_array().unwrap().len(),
                case["expected"]["binary_profile_pids"].as_u64().unwrap() as usize
            );
            assert_eq!(
                scope["profile_pids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_u64().unwrap())
                    .collect::<BTreeSet<_>>(),
                actual.iter().map(|r| r["pid"].as_u64().unwrap()).collect()
            );
        } else {
            assert!(mini.actual.stdout.is_empty());
            assert!(!mini.output.join("result.json").exists());
            assert_eq!(case["expected"]["failed_output_retained"], true);
            if case["expected"]["entrypoint_only_rejected"] == true {
                let facts = mini.record("collection-facts.json").unwrap();
                let path = std::path::Path::new(
                    facts["binary_scopes"][0]["dependency_target"]
                        .as_str()
                        .unwrap(),
                );
                let raw = fs::read_to_string(path.with_extension("d")).unwrap();
                let sources = collection_inputs::dependencies(&raw, path.to_str().unwrap())
                    .unwrap()
                    .into_iter()
                    .map(|p| {
                        mini.source
                            .join(p)
                            .canonicalize()
                            .unwrap()
                            .strip_prefix(&mini.source)
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_owned()
                    })
                    .collect::<BTreeSet<_>>();
                assert_eq!(
                    json!(sources),
                    case["expected"]["actual_dependency_sources"]
                );
                assert_eq!(
                    binary["sources"],
                    json!(["tools/xtask/src/bin/fixture-native/main.rs"])
                );
            } else {
                let registered: serde_json::Value =
                    serde_json::from_str(case["source_override"]["tests.json"].as_str().unwrap())
                        .unwrap();
                assert_eq!(
                    registered["native_binaries"][0]["exercises"][1]["stderr_utf8"],
                    "wrong\n"
                );
                assert_eq!(case["expected"]["mismatch_in_second_vector"], true);
                assert_eq!(case["expected"]["actual_stdout"], "7\n");
                assert_eq!(case["expected"]["actual_second_exit"], 255);
                assert_eq!(case["expected"]["actual_second_stderr"], "échec\n");
            }
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 3);
}

#[test]
fn independently_approved_collector_native_cli() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v3/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["action"] == "cli")
    {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let actual = support::collection_native::cli(case).unwrap();
        assert_eq!(
            actual.status.code().unwrap(),
            case["expected"]["exit"].as_i64().unwrap() as i32,
            "{id}"
        );
        assert_eq!(
            actual.stdout,
            case["expected"]["stdout"].as_str().unwrap().as_bytes(),
            "{id}"
        );
        assert_eq!(
            actual.stderr,
            case["expected"]["stderr"].as_str().unwrap().as_bytes(),
            "{id}"
        );
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 6);
}

#[test]
fn independently_approved_collector_explicit_nesting() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v3/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"].as_array().unwrap().iter().filter(|c| {
        matches!(
            c["action"].as_str(),
            Some("nest" | "outer-negative" | "postspawn-io")
        )
    }) {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let actual = if case["action"] == "postspawn-io" {
            support::collection_native::postspawn_io(case)
        } else {
            support::collection_native::probe(case)
        }
        .unwrap();
        assert_eq!(actual, case["expected"], "{id}");
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 9);
}

#[test]
fn independently_approved_collector_descendant_probes() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v9/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["receipt_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let facts =
            support::report_inputs::replace(&fixture["base_facts"], &case["patch"]).unwrap();
        match arboresce_xtask::collection_receipts::evaluate(&facts) {
            Ok(value) => assert_eq!(value, case["expected"]["value"], "{id}"),
            Err(error) => assert_eq!(error.to_string(), case["expected"]["error"], "{id}"),
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 33);
    for case in fixture["artifact_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let input =
            support::report_inputs::replace(&fixture["artifact_base"], &case["patch"]).unwrap();
        let raw = input["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|record| format!("{record}\n"))
            .collect::<String>();
        match arboresce_xtask::collection_artifacts::select(
            input["root"].as_str().unwrap(),
            input["target"].as_str().unwrap(),
            &input["metadata"],
            &input["registry"],
            &raw,
        ) {
            Ok(value) => assert_eq!(value, case["expected"]["value"], "{id}"),
            Err(error) => assert_eq!(error.to_string(), case["expected"]["error"], "{id}"),
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 51);
}

#[cfg(unix)]
#[test]
fn native_help_usage_intake_and_delegated_child_streams() {
    use std::os::unix::ffi::OsStringExt;
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/guard-v1/cases.json"
    ))
    .unwrap();
    for (args, profile) in [
        (vec![OsString::from("help")], "help"),
        (Vec::new(), "usage"),
        (vec![OsString::from("coverage")], "usage"),
        (vec![OsString::from_vec(vec![0xff])], "usage"),
    ] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_arboresce-xtask"));
        command.args(&args);
        let actual = support::observe(&mut command).unwrap();
        let expected = &fixture["output_profile"][profile];
        assert_eq!(
            actual.status.code(),
            expected["exit"].as_i64().map(|v| v as i32)
        );
        assert_eq!(
            actual.stdout,
            expected["stdout"].as_str().unwrap().as_bytes()
        );
        assert_eq!(
            actual.stderr,
            expected["stderr"].as_str().unwrap().as_bytes()
        );
    }
    println!(
        "xtask case=non-utf8-command-intake pid={}",
        std::process::id()
    );
    let prepared = support::PreparedRoot::new().unwrap();
    let peer = prepared.cargo_peer().unwrap();
    for action in ["fmt", "check", "test", "clippy"] {
        for (mode, exit) in [("pass", 0), ("fail", 101)] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_arboresce-xtask"));
            command
                .arg(action)
                .env("PATH", &peer)
                .env("ARBORESCE_XTASK_TEST_PEER", mode);
            let actual = support::observe(&mut command).unwrap();
            assert_eq!(actual.status.code(), Some(exit));
            assert_eq!(actual.stdout, b"child stdout\n");
            assert_eq!(actual.stderr, b"child stderr\n");
        }
    }
    for (mode, profile) in [
        ("metadata-invalid", "invalid_guard_input"),
        ("metadata-warning", "invalid_guard_input"),
    ] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_arboresce-xtask"));
        command
            .arg("architecture")
            .env("PATH", &peer)
            .env("ARBORESCE_XTASK_TEST_PEER", mode);
        let actual = support::observe(&mut command).unwrap();
        assert_eq!(actual.status.code(), Some(1));
        assert!(actual.stdout.is_empty());
        let expected = fixture["output_profile"][profile]["stderr"]
            .as_str()
            .unwrap();
        let prefix = if mode == "metadata-warning" {
            "child stderr\n"
        } else {
            ""
        };
        assert_eq!(actual.stderr, format!("{prefix}{expected}").as_bytes());
    }
    for action in ["check", "architecture"] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_arboresce-xtask"));
        command
            .arg(action)
            .env("PATH", prepared.path.join("no-programs"));
        let actual = support::observe(&mut command).unwrap();
        assert_eq!(actual.status.code(), Some(1));
        assert!(actual.stdout.is_empty());
        assert_eq!(actual.stderr, b"verification command failed\n");
    }
}

#[cfg(unix)]
#[test]
fn native_architecture_rejection_success_and_output_transport_failure() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/guard-v1/cases.json"
    ))
    .unwrap();
    let prepared = support::PreparedRoot::new().unwrap();
    let peer = prepared.cargo_peer().unwrap();
    let metadata = prepared.path.join("metadata.json");
    let base = prepared.metadata().unwrap();
    for (mut value, profile) in [
        (base.clone(), "architecture_success"),
        (base.clone(), "unqualified_dependency"),
        (base.clone(), "forbidden_source"),
    ] {
        if profile == "unqualified_dependency" {
            value["resolve"]["nodes"][1]["deps"] =
                serde_json::json!([{"pkg":"arboresce","dep_kinds":[{"kind":null}]}]);
        }
        if profile == "forbidden_source" {
            std::fs::write(
                prepared.path.join("crates/arboresce-domain/src/lib.rs"),
                "use std::fs::File;",
            )
            .unwrap();
        }
        std::fs::write(&metadata, value.to_string()).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_arboresce-xtask"));
        command
            .arg("architecture")
            .current_dir(&prepared.path)
            .env("PATH", &peer)
            .env("ARBORESCE_XTASK_TEST_PEER", "metadata")
            .env("ARBORESCE_XTASK_TEST_METADATA", &metadata);
        let actual = support::observe(&mut command).unwrap();
        let expected = &fixture["output_profile"][profile];
        assert_eq!(
            actual.status.code(),
            expected["exit"].as_i64().map(|n| n as i32)
        );
        assert_eq!(
            actual.stdout,
            expected["stdout"].as_str().unwrap().as_bytes()
        );
        assert_eq!(
            actual.stderr,
            expected["stderr"].as_str().unwrap().as_bytes()
        );
    }
    let mut command = Command::new(env!("CARGO_BIN_EXE_arboresce-xtask"));
    command
        .arg("architecture")
        .env("PATH", &peer)
        .env("ARBORESCE_XTASK_TEST_PEER", "fail");
    let actual = support::observe(&mut command).unwrap();
    assert_eq!(actual.status.code(), Some(101));
    assert!(actual.stdout.is_empty());
    assert_eq!(actual.stderr, b"child stderr\n");
    let mut command = Command::new(env!("CARGO_BIN_EXE_arboresce-xtask-test-peer"));
    command.env_remove("ARBORESCE_XTASK_TEST_PEER");
    let actual = support::observe(&mut command).unwrap();
    assert_eq!(actual.status.code(), Some(1));
    assert!(actual.stdout.is_empty());
    assert_eq!(actual.stderr, b"test peer mode missing\n");
    let mut command = Command::new(env!("CARGO_BIN_EXE_arboresce-xtask"));
    command.arg("help");
    let actual = support::observe_with(
        &mut command,
        support::Observation {
            closed: Some(support::ClosedStream::Stdout),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(actual.status.code(), Some(1));
    assert!(actual.stdout.is_empty());
    assert_eq!(actual.stderr, b"verification command failed\n");
    let mut command = Command::new(env!("CARGO_BIN_EXE_arboresce-xtask"));
    command.arg("unknown");
    let actual = support::observe_with(
        &mut command,
        support::Observation {
            closed: Some(support::ClosedStream::Stderr),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(actual.status.code(), Some(1));
    assert!(actual.stdout.is_empty());
    assert!(actual.stderr.is_empty());
}

#[cfg(unix)]
#[test]
fn independently_approved_native_observation_budgets_and_streams() {
    use std::{io::ErrorKind, time::Duration};
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/guard-v2/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    let prepared = support::PreparedRoot::new().unwrap();
    for case in fixture["observer_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let mut command = match id {
            "observer-native-success" | "observer-native-late-observation" => {
                Command::new("/usr/bin/true")
            }
            "observer-native-timeout" => {
                let mut c = Command::new("/bin/sleep");
                c.arg("5");
                c
            }
            "observer-spawn-error" => Command::new(prepared.path.join("absent-native-program")),
            "observer-broken-stdout" => {
                let mut c = Command::new(env!("CARGO_BIN_EXE_arboresce-xtask"));
                c.arg("help");
                c
            }
            "observer-broken-stderr" => {
                let mut c = Command::new(env!("CARGO_BIN_EXE_arboresce-xtask"));
                c.arg("unknown");
                c
            }
            _ => panic!("unknown independently approved observer ID {id}"),
        };
        let closed = match id {
            "observer-broken-stdout" => Some(support::ClosedStream::Stdout),
            "observer-broken-stderr" => Some(support::ClosedStream::Stderr),
            _ => None,
        };
        let actual = support::observe_with(
            &mut command,
            support::Observation {
                budget: Duration::from_millis(case["budget_ms"].as_u64().unwrap()),
                poll: Duration::from_millis(case["poll_ms"].as_u64().unwrap()),
                initial_delay: Duration::from_millis(
                    case["initial_observation_delay_ms"].as_u64().unwrap(),
                ),
                closed,
            },
        );
        match id {
            "observer-native-timeout" | "observer-native-late-observation" => {
                assert_eq!(actual.unwrap_err().kind(), ErrorKind::TimedOut)
            }
            "observer-spawn-error" => assert_eq!(actual.unwrap_err().kind(), ErrorKind::NotFound),
            _ => {
                let actual = actual.unwrap();
                assert!(actual.stdout.is_empty());
                if id == "observer-native-success" {
                    assert_eq!(actual.status.code(), Some(0));
                    assert!(actual.stderr.is_empty());
                } else {
                    assert_eq!(actual.status.code(), Some(1));
                    assert_eq!(
                        actual.stderr,
                        if id == "observer-broken-stdout" {
                            b"verification command failed\n".as_slice()
                        } else {
                            b"".as_slice()
                        }
                    );
                }
            }
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 6);
}

#[cfg(unix)]
#[test]
fn independently_approved_native_report_boundaries() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/report-gate-v1/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["native_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let prepared = support::PreparedRoot::new().unwrap();
        let selected = if let Some(wanted) = case["report_case"].as_str() {
            fixture["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|case| case["id"].as_str() == Some(wanted))
                .unwrap()
        } else {
            case
        };
        let mut input = support::report_inputs::build(&fixture, selected)
            .expect("invalid reviewed native input recipe");
        if let Some(bytes) = case["pad_inventory_to_bytes"].as_u64() {
            input.inventory = support::report_inputs::pad(input.inventory, bytes).unwrap();
        }
        if let Some(bytes) = case["pad_report_to_bytes"].as_u64() {
            input.report = support::report_inputs::pad(input.report, bytes).unwrap();
        }
        let inventory = prepared.path.join("inventory.json");
        let report = prepared.path.join("report.json");
        std::fs::write(&inventory, input.inventory).unwrap();
        std::fs::write(&report, input.report).unwrap();
        if let Some(which) = case["missing"].as_str() {
            std::fs::remove_file(if which == "inventory" {
                &inventory
            } else {
                &report
            })
            .unwrap();
        }
        if let Some(which) = case["directory"].as_str() {
            let path = if which == "inventory" {
                &inventory
            } else {
                &report
            };
            std::fs::remove_file(path).unwrap();
            std::fs::create_dir(path).unwrap();
        }
        let args = case
            .get("args_override")
            .unwrap_or(&case["args"])
            .as_array()
            .unwrap();
        let mut args: Vec<OsString> = args
            .iter()
            .map(|arg| match arg.as_str().unwrap() {
                "INVENTORY" => inventory.as_os_str().to_owned(),
                "REPORT" => report.as_os_str().to_owned(),
                "/workspace" => OsString::from(&input.root),
                other => OsString::from(other),
            })
            .collect();
        if let Some(bytes) = case["non_utf8_root_bytes"].as_array() {
            args[0] = OsString::from_vec(
                bytes
                    .iter()
                    .map(|b| u8::try_from(b.as_u64().unwrap()).unwrap())
                    .collect(),
            );
        }
        let mut command = Command::new(env!("CARGO_BIN_EXE_arboresce-report-gate"));
        command.args(args);
        let closed = match case["close"].as_str() {
            Some("stdout") => Some(support::ClosedStream::Stdout),
            Some("stderr") => Some(support::ClosedStream::Stderr),
            None => None,
            _ => panic!("invalid reviewed closed-output selector"),
        };
        let result = support::observe_with(
            &mut command,
            support::Observation {
                closed,
                ..support::Observation::default()
            },
        )
        .unwrap();
        assert_eq!(
            result.status.code(),
            Some(i32::try_from(case["exit"].as_u64().unwrap()).unwrap()),
            "{id}"
        );
        assert_eq!(
            result.stdout,
            case["stdout"].as_str().unwrap().as_bytes(),
            "{id}"
        );
        assert_eq!(
            result.stderr,
            case["stderr"].as_str().unwrap().as_bytes(),
            "{id}"
        );
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 23);
}

#[test]
fn independently_approved_native_measurement_contexts() {
    use arboresce_xtask::measurement::Request;
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-measurement-v1/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"].as_array().unwrap().iter().filter(|c| {
        matches!(
            c["operation"].as_str(),
            Some("context" | "budget" | "preparation")
        )
    }) {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        if case["operation"] == "budget" {
            let action =
                Action::parse(&[OsString::from(case["action"].as_str().unwrap())]).unwrap();
            assert_eq!(
                action.operation_budget().as_millis(),
                case["expected_budget_ms"].as_u64().unwrap() as u128,
                "{id}"
            );
            println!("xtask case={id} pid={}", std::process::id());
            continue;
        }
        if case["operation"] == "preparation" {
            let error = if case.get("program_mode").is_some() {
                let input =
                    support::measurement_inputs::Input::new(&fixture["context"], case).unwrap();
                input
                    .command(case)
                    .expect_err("builder must reject unknown program recipe")
            } else {
                support::measurement_inputs::Input::new(&fixture["context"], case)
                    .err()
                    .expect("builder must reject unknown preparation recipe")
            };
            assert_eq!(
                serde_json::json!(format!("{:?}", error.kind())),
                case["expected_error"],
                "{id}"
            );
            println!("xtask case={id} pid={}", std::process::id());
            continue;
        }
        let input = support::measurement_inputs::Input::new(&fixture["context"], case).unwrap();
        let result = Request::open(&input.root);
        if let Some(expected) = case["expected_error"].as_str() {
            assert_eq!(
                format!("{:?}", result.err().expect(id).kind()),
                expected,
                "{id}"
            );
        } else {
            assert!(result.is_ok(), "{id}");
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 34);
}

#[test]
fn independently_approved_native_measurement_launches_and_completion() {
    use arboresce_xtask::{
        measurement::{self, Request},
        process,
    };
    use serde_json::json;
    use sha2::{Digest, Sha256};
    use std::{
        fs,
        os::unix::{ffi::OsStrExt, fs::symlink, process::ExitStatusExt},
        time::{Duration, Instant},
    };
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-measurement-v1/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| matches!(c["operation"].as_str(), Some("launch" | "prepare" | "wait")))
    {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let input = support::measurement_inputs::Input::new(&fixture["context"], case).unwrap();
        let request = Request::open(&input.root).unwrap();
        let mut command = input.command(case).unwrap();
        if !command.get_program().as_bytes().contains(&b'/') && case["skip_selection"] != true {
            match measurement::selected_program(&command) {
                Ok(selected) => {
                    command = support::measurement_inputs::bind_selected(&command, &selected)
                }
                Err(error) => {
                    assert_eq!(
                        json!(format!("{:?}", error.kind())),
                        case["expected_error"],
                        "{id}"
                    );
                    assert_eq!(
                        std::fs::read_dir(input.root.join("launches"))
                            .unwrap()
                            .count(),
                        0
                    );
                    assert_eq!(
                        std::fs::read_dir(input.root.join("exits")).unwrap().count(),
                        0
                    );
                    println!("xtask case={id} pid={}", std::process::id());
                    continue;
                }
            }
        }
        if let Some(key) = case["remove_environment"].as_str() {
            assert_eq!(
                command
                    .get_envs()
                    .find(|(name, _)| *name == key)
                    .map(|(_, value)| value),
                Some(None),
                "{id}"
            );
        }
        let prepared = match request.prepare(&mut command) {
            Ok(prepared) => prepared,
            Err(error) => {
                assert_eq!(
                    json!(format!("{:?}", error.kind())),
                    case["expected_error"],
                    "{id}"
                );
                assert_eq!(
                    fs::read_dir(input.root.join("launches")).unwrap().count(),
                    0,
                    "{id}"
                );
                assert_eq!(
                    fs::read_dir(input.root.join("exits")).unwrap().count(),
                    0,
                    "{id}"
                );
                println!("xtask case={id} pid={}", std::process::id());
                continue;
            }
        };
        if case["operation"] == "prepare" {
            assert!(case["expected_error"].is_null(), "{id}");
            assert_eq!(
                fs::read_dir(input.root.join("launches")).unwrap().count(),
                0
            );
            println!("xtask case={id} pid={}", std::process::id());
            continue;
        }
        let mut child = match measurement::spawn(&mut command) {
            Ok(child) => child,
            Err(error) => {
                assert_eq!(
                    json!(format!("{:?}", error.kind())),
                    case["expected_error"],
                    "{id}"
                );
                assert_eq!(case["expected_spawn_count"], 0, "{id}");
                assert_eq!(
                    fs::read_dir(input.root.join("launches")).unwrap().count(),
                    0
                );
                assert_eq!(fs::read_dir(input.root.join("exits")).unwrap().count(), 0);
                println!("xtask case={id} pid={}", std::process::id());
                continue;
            }
        };
        let pid = child.id();
        let launch_path = input.root.join("launches").join(format!("{pid}.json"));
        let exit_path = input.root.join("exits").join(format!("{pid}.json"));
        let mut running = child.try_wait().unwrap().is_none();
        let mut started = Instant::now();
        if case["collision"] == "launch" || case.get("before_record_set_context").is_some() {
            let expected_running = case
                .get("expected_running_before_record")
                .unwrap_or(&case["expected_running_before_collision"]);
            assert_eq!(json!(running), *expected_running, "{id}");
            let original_context_bytes = fs::read(input.root.join("context.json")).unwrap();
            let mut changed_context = None;
            if case["collision"] == "launch" {
                fs::write(&launch_path, case["collision_bytes"].as_str().unwrap()).unwrap();
            } else {
                let mut context: serde_json::Value =
                    serde_json::from_slice(&original_context_bytes).unwrap();
                assert_eq!(context, fixture["context"], "{id}");
                context.as_object_mut().unwrap().extend(
                    case["before_record_set_context"]
                        .as_object()
                        .unwrap()
                        .clone(),
                );
                let bytes = context.to_string().into_bytes();
                fs::write(input.root.join("context.json"), &bytes).unwrap();
                changed_context = Some(bytes);
            }
            let error = prepared.record(&mut child).unwrap_err();
            assert_eq!(
                json!(format!("{:?}", error.kind())),
                case["expected_error"],
                "{id}"
            );
            assert!(
                started.elapsed().as_millis()
                    <= case["cleanup_budget_ms"].as_u64().unwrap() as u128,
                "{id}"
            );
            let status = child
                .try_wait()
                .unwrap()
                .expect("actual owned child reaped");
            assert_eq!(json!(status.code()), case["expected_exit"], "{id}");
            assert_eq!(json!(status.signal()), case["expected_signal"], "{id}");
            if let Some(ref bytes) = changed_context {
                assert_eq!(
                    fs::read(input.root.join("context.json")).unwrap(),
                    *bytes,
                    "{id}"
                );
                assert_eq!(
                    fs::read_dir(input.root.join("launches")).unwrap().count(),
                    case["expected_receipts"].as_u64().unwrap() as usize,
                    "{id}"
                );
            } else {
                assert_eq!(
                    fs::read(&launch_path).unwrap(),
                    case["collision_bytes"].as_str().unwrap().as_bytes(),
                    "{id}"
                );
            }
            assert!(!exit_path.exists(), "{id}");
            measurement::complete(&mut child).unwrap();
            println!(
                "xtask measurement observation {}",
                json!({"case":id,"pid":pid,"initial_running":running,"exit":status.code(),"signal":status.signal(),"elapsed_ms":started.elapsed().as_millis(),"reaped":true,"error":format!("{:?}",error.kind()),"launch_collision_bytes":case["collision_bytes"],"context_before":original_context_bytes,"context_after":changed_context,"spawn_count":1})
            );
            println!("xtask case={id} pid={}", std::process::id());
            continue;
        }
        prepared
            .record(&mut child)
            .unwrap_or_else(|e| panic!("{id}: {e}"));
        let original_launch_bytes = fs::read(&launch_path).unwrap();
        let launch: serde_json::Value = serde_json::from_slice(&original_launch_bytes).unwrap();
        assert_eq!(launch["version"], "native-launch-v1", "{id}");
        assert_eq!(launch["pid"], pid, "{id}");
        assert_eq!(launch["parent_pid"], std::process::id(), "{id}");
        assert_eq!(launch["run_id"], fixture["context"]["run_id"], "{id}");
        assert_eq!(
            launch["source_sha256"], fixture["context"]["source_sha256"],
            "{id}"
        );
        assert_eq!(
            launch["context_path"],
            json!(input.root.canonicalize().unwrap().as_os_str().as_bytes()),
            "{id}"
        );
        assert_eq!(
            launch["context_sha256"],
            format!(
                "{:x}",
                Sha256::digest(fs::read(input.root.join("context.json")).unwrap())
            ),
            "{id}"
        );
        let executable_bytes: Vec<u8> =
            serde_json::from_value(launch["executable"].clone()).unwrap();
        let executable = std::path::Path::new(std::ffi::OsStr::from_bytes(&executable_bytes));
        assert!(executable.is_absolute());
        assert_eq!(executable.canonicalize().unwrap(), executable);
        assert_eq!(
            launch["executable_sha256"],
            format!("{:x}", Sha256::digest(fs::read(executable).unwrap())),
            "{id}"
        );
        assert_eq!(
            launch["program"],
            json!(command.get_program().as_bytes()),
            "{id}"
        );
        assert_eq!(
            launch["arguments"],
            json!(command.get_args().map(|a| a.as_bytes()).collect::<Vec<_>>()),
            "{id}"
        );
        assert_eq!(
            launch["cwd"],
            json!(
                command
                    .get_current_dir()
                    .unwrap()
                    .canonicalize()
                    .unwrap()
                    .as_os_str()
                    .as_bytes()
            ),
            "{id}"
        );
        if case["prepare_exited"] == true {
            let prepared_status = process::wait(&mut child, Duration::from_secs(30)).unwrap();
            assert_eq!(prepared_status.code(), Some(0), "{id}");
            assert_eq!(child.try_wait().unwrap(), Some(prepared_status), "{id}");
            running = false;
            started = Instant::now();
        }
        let actual = if case["complete_while_running"] == true {
            assert_eq!(
                json!(running),
                case["expected_running_before_completion"],
                "{id}"
            );
            let error = request.complete(&mut child).unwrap_err();
            assert_eq!(
                json!(format!("{:?}", error.kind())),
                case["expected_error"],
                "{id}"
            );
            assert!(!exit_path.exists(), "{id}");
            process::terminate(&mut child).unwrap();
            measurement::complete(&mut child).unwrap();
            request.complete(&mut child).unwrap();
            Some(format!("{:?}", error.kind()))
        } else if case["operation"] == "wait" {
            assert_eq!(
                json!(running),
                case["expected_running_before_completion"],
                "{id}"
            );
            let error = process::wait(
                &mut child,
                Duration::from_millis(case["budget_ms"].as_u64().unwrap()),
            )
            .unwrap_err();
            assert_eq!(
                json!(format!("{:?}", error.kind())),
                case["expected_error"],
                "{id}"
            );
            request.complete(&mut child).unwrap();
            Some(format!("{:?}", error.kind()))
        } else {
            let actual_status = process::wait(&mut child, Duration::from_secs(30)).unwrap();
            assert_eq!(
                process::terminate(&mut child).unwrap(),
                actual_status,
                "{id}"
            );
            match case["after"].as_str().unwrap_or("") {
                "remove-launch" => fs::remove_file(&launch_path).unwrap(),
                "change-launch" => {
                    let mut changed = launch.clone();
                    changed
                        .as_object_mut()
                        .unwrap()
                        .extend(case["after_launch_set"].as_object().unwrap().clone());
                    fs::write(&launch_path, changed.to_string()).unwrap();
                }
                "" => {}
                _ => panic!("unknown reviewed launch mutation"),
            }
            if case["collision"] == "exit" {
                fs::write(&exit_path, b"existing completion\n").unwrap();
            }
            if case["collision"] == "exit-link" {
                fs::write(
                    input.owner.path.join("exit-target"),
                    b"existing completion\n",
                )
                .unwrap();
                symlink(input.owner.path.join("exit-target"), &exit_path).unwrap();
            }
            let completed = request.complete(&mut child);
            if let Some(expected) = case["expected_error"].as_str() {
                assert_eq!(
                    format!("{:?}", completed.unwrap_err().kind()),
                    expected,
                    "{id}"
                );
                if matches!(case["collision"].as_str(), Some("exit" | "exit-link")) {
                    assert_eq!(
                        fs::read(&exit_path).unwrap(),
                        b"existing completion\n",
                        "{id}"
                    );
                }
                Some(expected.to_owned())
            } else {
                completed.unwrap_or_else(|e| panic!("{id}: {e}"));
                None
            }
        };
        let status = child
            .try_wait()
            .unwrap()
            .expect("actual owned child reaped");
        if let Some(expected) = case.get("expected_exit") {
            assert_eq!(json!(status.code()), *expected, "{id}");
        }
        assert_eq!(
            json!(status.signal()),
            case.get("expected_signal")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
            "{id}"
        );
        if let Some(budget) = case["cleanup_budget_ms"].as_u64() {
            assert!(started.elapsed().as_millis() <= budget as u128, "{id}");
        }
        let exit = if actual.is_none()
            || case["complete_while_running"] == true
            || case["operation"] == "wait"
        {
            let bytes = fs::read(&exit_path).unwrap();
            let exit: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(exit["version"], "native-exit-v1", "{id}");
            for key in [
                "pid",
                "parent_pid",
                "run_id",
                "source_sha256",
                "context_path",
                "context_sha256",
            ] {
                assert_eq!(exit[key], launch[key], "{id}: {key}");
            }
            assert_eq!(
                exit["launch_sha256"],
                format!("{:x}", Sha256::digest(&original_launch_bytes)),
                "{id}"
            );
            assert_eq!(exit["exit"], json!(status.code()), "{id}");
            assert_eq!(exit["signal"], json!(status.signal()), "{id}");
            if case["repeat"] == "identical" {
                let modified = fs::metadata(&exit_path).unwrap().modified().unwrap();
                request.complete(&mut child).unwrap();
                assert_eq!(fs::read(&exit_path).unwrap(), bytes, "{id}");
                assert_eq!(
                    fs::metadata(&exit_path).unwrap().modified().unwrap(),
                    modified,
                    "{id}"
                );
            }
            exit
        } else {
            serde_json::Value::Null
        };
        println!(
            "xtask measurement observation {}",
            json!({"case":id,"pid":pid,"exit":status.code(),"signal":status.signal(),"initial_running":running,"reaped":true,"launch":launch,"completion":exit,"error":actual,"elapsed_ms":started.elapsed().as_millis(),"spawn_count":1})
        );
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 29);
}

#[test]
fn independently_approved_native_measurement_activation_and_failure_receipts() {
    use arboresce_xtask::measurement;
    use serde_json::json;
    use sha2::{Digest, Sha256};
    use std::{fs, os::unix::ffi::OsStrExt};
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-measurement-v1/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["operation"] == "native")
    {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let input = support::measurement_inputs::Input::new(&fixture["context"], case).unwrap();
        if case["context"] == "invalid" {
            fs::write(input.root.join("context.json"), b"{}").unwrap();
        }
        let mut command = Command::new(env!("CARGO_BIN_EXE_arboresce-xtask-test-peer"));
        command
            .env("ARBORESCE_XTASK_TEST_PEER", case["mode"].as_str().unwrap())
            .env(
                "ARBORESCE_NATIVE_PROGRAM",
                case["program"].as_str().unwrap(),
            )
            .current_dir(&input.owner.path);
        if let Some(path) = case["path"].as_str() {
            command.env("PATH", path);
        }
        if case["context"] == false {
            command.env_remove(measurement::ENV);
        } else {
            command.env(measurement::ENV, &input.root);
        }
        let (peer_pid, output) =
            support::observe_identity(&mut command).unwrap_or_else(|e| panic!("{id}: {e}"));
        assert_eq!(json!(output.status.code()), case["expected_exit"], "{id}");
        assert_eq!(
            output.stdout,
            case["expected_stdout"].as_str().unwrap().as_bytes(),
            "{id}"
        );
        assert_eq!(
            output.stderr,
            case["expected_stderr"].as_str().unwrap().as_bytes(),
            "{id}"
        );
        let launches: Vec<_> = fs::read_dir(input.root.join("launches"))
            .unwrap()
            .map(|p| p.unwrap().path())
            .collect();
        let exits: Vec<_> = fs::read_dir(input.root.join("exits"))
            .unwrap()
            .map(|p| p.unwrap().path())
            .collect();
        assert_eq!(
            launches.len() as u64,
            case["expected_receipts"].as_u64().unwrap(),
            "{id}"
        );
        assert_eq!(exits.len(), launches.len(), "{id}");
        let mut receipts = Vec::new();
        for path in launches {
            let launch_bytes = fs::read(&path).unwrap();
            let launch: serde_json::Value = serde_json::from_slice(&launch_bytes).unwrap();
            let exit: serde_json::Value = serde_json::from_slice(
                &fs::read(input.root.join("exits").join(path.file_name().unwrap())).unwrap(),
            )
            .unwrap();
            assert_eq!(launch["version"], "native-launch-v1", "{id}");
            assert_eq!(exit["version"], "native-exit-v1", "{id}");
            assert_eq!(launch["parent_pid"], peer_pid, "{id}");
            assert_ne!(launch["pid"], peer_pid, "{id}");
            assert!(launch["pid"].as_u64().unwrap() > 0, "{id}");
            for key in [
                "pid",
                "parent_pid",
                "run_id",
                "source_sha256",
                "context_path",
                "context_sha256",
            ] {
                assert_eq!(exit[key], launch[key], "{id}: {key}");
            }
            assert_eq!(launch["run_id"], fixture["context"]["run_id"], "{id}");
            assert_eq!(
                launch["source_sha256"], fixture["context"]["source_sha256"],
                "{id}"
            );
            assert_eq!(
                launch["context_path"],
                json!(input.root.canonicalize().unwrap().as_os_str().as_bytes()),
                "{id}"
            );
            assert_eq!(
                launch["context_sha256"],
                format!(
                    "{:x}",
                    Sha256::digest(fs::read(input.root.join("context.json")).unwrap())
                ),
                "{id}"
            );
            let executable = std::path::Path::new(
                case.get("expected_executable")
                    .unwrap_or(&case["program"])
                    .as_str()
                    .unwrap(),
            )
            .canonicalize()
            .unwrap();
            assert_eq!(
                launch["executable"],
                json!(executable.as_os_str().as_bytes()),
                "{id}"
            );
            assert_eq!(
                launch["executable_sha256"],
                format!("{:x}", Sha256::digest(fs::read(&executable).unwrap())),
                "{id}"
            );
            assert_eq!(
                launch["program"],
                json!(
                    case.get("expected_program")
                        .unwrap_or(&case["program"])
                        .as_str()
                        .unwrap()
                        .as_bytes()
                ),
                "{id}"
            );
            assert_eq!(
                launch["arguments"],
                if case["mode"] == "measured-output" {
                    json!([b"measurement child".as_slice()])
                } else {
                    json!([])
                },
                "{id}"
            );
            assert_eq!(
                launch["cwd"],
                json!(
                    input
                        .owner
                        .path
                        .canonicalize()
                        .unwrap()
                        .as_os_str()
                        .as_bytes()
                ),
                "{id}"
            );
            assert_eq!(exit["exit"], case["expected_child_exit"], "{id}");
            assert_eq!(exit["signal"], serde_json::Value::Null, "{id}");
            assert_eq!(
                exit["launch_sha256"],
                format!("{:x}", Sha256::digest(&launch_bytes)),
                "{id}"
            );
            receipts.push(json!({"launch":launch,"completion":exit}));
        }
        println!(
            "xtask measurement native {}",
            json!({"case":id,"pid":peer_pid,"exit":output.status.code(),"stdout":output.stdout,"stderr":output.stderr,"receipts":receipts})
        );
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 12);
}

#[test]
fn complete_native_trace_streams_and_failures() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-trace-v1/cases.json"
    ))
    .unwrap();
    let mut seen = std::collections::BTreeSet::new();
    for case in fixture["cases"].as_array().unwrap() {
        let id = format!("trace-stream-{}", case["id"].as_str().unwrap());
        assert!(seen.insert(id.clone()));
        let summary = support::native_trace::observe(&case["recipe"])
            .unwrap_or_else(|error| panic!("{id}: {error}"));
        println!("xtask native trace {summary}");
        for (key, expected) in case["expected"].as_object().unwrap() {
            assert_eq!(summary["observed"][key], *expected, "{id}: {key}");
        }
        for key in [
            "explicit_completion_absent_before_observe",
            "ambient_completion_succeeded",
            "captured_inputs_exact",
        ] {
            assert_eq!(summary["proof"][key], true, "{id}: {key}");
        }
        if !case["recipe"]["fail_once_after_bytes"].is_null() {
            assert_eq!(
                summary["proof"]["one_shot_failure_after_actual_bytes"],
                case["recipe"]["fail_once_after_bytes"],
                "{id}: actual fault threshold"
            );
        }
        if case["recipe"]["flush_delay_ms"].as_u64().unwrap() > 0 {
            assert_eq!(summary["proof"]["delayed_flushes_returned"], 1, "{id}");
            assert!(
                summary["proof"]["observation_elapsed_ms"].as_u64().unwrap()
                    >= case["recipe"]["flush_delay_ms"].as_u64().unwrap(),
                "{id}: actual delayed successful flush"
            );
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    for case in fixture["preparation_cases"].as_array().unwrap() {
        let id = format!("trace-stream-{}", case["id"].as_str().unwrap());
        assert!(seen.insert(id.clone()));
        let (retained, error) = support::native_trace::preparation(&case["recipe"])
            .unwrap_or_else(|error| panic!("{id}: {error}"));
        assert_eq!(
            format!("{:?}", error.kind()),
            case["expected"]["error_kind"].as_str().unwrap(),
            "{id}: actual Err"
        );
        let summary =
            support::native_trace::preparation_observation(&retained, &error, &case["recipe"])
                .unwrap_or_else(|error| panic!("{id}: {error}"));
        println!("xtask native preparation {summary}");
        for (key, expected) in case["expected"].as_object().unwrap() {
            assert_eq!(summary["observed"][key], *expected, "{id}: {key}");
        }
        assert_eq!(
            summary["observed"]["ambient_environment_unchanged"], true,
            "{id}"
        );
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 17);
}
