use std::{collections::BTreeSet, ffi::OsString, process::Command};

use arboresce_xtask::{
    Action,
    architecture::{GuardError, Layer, check_dependency, check_source},
    emit,
    workspace::{check_workspace, registered_layer},
};

mod support;

#[test]
fn complete_approved_dependency_and_source_cases() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/guard-v1/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["dependency_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let actual = Layer::from_name(case["layer"].as_str().unwrap()).is_some_and(|layer| {
            check_dependency(
                layer,
                case["package"].as_str().unwrap(),
                case["version"].as_str().unwrap(),
                case["kind"].as_str().unwrap(),
            )
            .is_ok()
        });
        assert_eq!(actual, case["accepted"].as_bool().unwrap(), "{id}");
        println!("xtask case={id} pid={}", std::process::id());
    }
    for case in fixture["source_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let actual = Layer::from_name(case["layer"].as_str().unwrap())
            .is_some_and(|layer| check_source(layer, case["rust"].as_str().unwrap()).is_ok());
        assert_eq!(actual, case["accepted"].as_bool().unwrap(), "{id}");
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 82);
}

#[test]
fn complete_command_vectors_and_owned_output_failures() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/guard-v1/cases.json"
    ))
    .unwrap();
    for case in fixture["command_cases"].as_array().unwrap() {
        let args: Vec<_> = case["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| OsString::from(v.as_str().unwrap()))
            .collect();
        let parsed = Action::parse(&args);
        assert_eq!(parsed.map(Action::name), case["action"].as_str());
        if let Some(action) = parsed {
            let expected: Vec<_> = case["cargo_args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect();
            assert_eq!(action.cargo_args(), expected);
        }
        println!(
            "xtask case={} pid={}",
            case["id"].as_str().unwrap(),
            std::process::id()
        );
    }
    assert_eq!(
        arboresce_xtask::USAGE,
        fixture["outputs"]["usage_utf8"].as_str().unwrap()
    );
    assert_eq!(
        arboresce_xtask::COMMAND_FAILURE,
        fixture["outputs"]["command_failure_utf8"].as_str().unwrap()
    );
    for (error, key) in [
        (GuardError::InvalidInput, "invalid_input_utf8"),
        (GuardError::Dependency, "dependency_failure_utf8"),
        (GuardError::Source, "source_failure_utf8"),
    ] {
        assert_eq!(error.to_string(), fixture["outputs"][key].as_str().unwrap());
    }
    let mut output = Vec::new();
    assert!(emit(&mut output, b"line\n"));
    assert_eq!(output, b"line\n");
    let mut failed = support::FailedOutput {
        write_fails: true,
        bytes: Vec::new(),
    };
    assert!(!emit(&mut failed, b"line\n"));
    let mut failed = support::FailedOutput {
        write_fails: false,
        bytes: Vec::new(),
    };
    assert!(!emit(&mut failed, b"line\n"));
    assert_eq!(failed.bytes, b"line\n");
}

#[test]
fn workspace_metadata_shapes_missing_and_duplicate_members_fail() {
    let prepared = support::PreparedRoot::new().unwrap();
    let base = prepared.metadata().unwrap();
    assert!(check_workspace(&prepared.path, &base.to_string()).is_ok());
    for bad in [
        serde_json::Value::Null,
        serde_json::json!({}),
        serde_json::json!({"workspace_members":[]}),
    ] {
        assert_eq!(
            check_workspace(&prepared.path, &bad.to_string()),
            Err(GuardError::InvalidInput)
        );
    }
    assert_eq!(
        check_workspace(&prepared.path, "not-json"),
        Err(GuardError::InvalidInput)
    );
    let mut bad = base.clone();
    bad["workspace_members"] = serde_json::json!(["arboresce", "arboresce"]);
    assert_eq!(
        check_workspace(&prepared.path, &bad.to_string()),
        Err(GuardError::InvalidInput)
    );
    let mut bad = base.clone();
    bad["workspace_members"] = serde_json::json!([42]);
    assert_eq!(
        check_workspace(&prepared.path, &bad.to_string()),
        Err(GuardError::InvalidInput)
    );
    let mut bad = base.clone();
    bad["workspace_members"] = serde_json::json!(["missing"]);
    assert_eq!(
        check_workspace(&prepared.path, &bad.to_string()),
        Err(GuardError::InvalidInput)
    );
    let mut bad = base.clone();
    bad["packages"][0]["id"] = serde_json::json!("arboresce-domain");
    assert_eq!(
        check_workspace(&prepared.path, &bad.to_string()),
        Err(GuardError::InvalidInput)
    );
    let mut bad = base.clone();
    bad["resolve"]["nodes"][0]["id"] = serde_json::json!("arboresce-domain");
    assert_eq!(
        check_workspace(&prepared.path, &bad.to_string()),
        Err(GuardError::InvalidInput)
    );
    let mut bad = base.clone();
    bad["resolve"]["nodes"] = serde_json::json!([]);
    assert_eq!(
        check_workspace(&prepared.path, &bad.to_string()),
        Err(GuardError::InvalidInput)
    );
    let mut bad = base.clone();
    bad["packages"][0]["name"] = serde_json::json!("unregistered");
    assert_eq!(
        check_workspace(&prepared.path, &bad.to_string()),
        Err(GuardError::Dependency)
    );
    assert_eq!(
        registered_layer("arboresce-xtask", "0.0.1"),
        Err(GuardError::Dependency)
    );
    assert_eq!(
        registered_layer("arboresce-xtask", "0.0.0"),
        Ok(Layer::Tool)
    );
}

#[test]
fn resolved_dependencies_and_source_paths_cannot_bypass_current_roles() {
    let prepared = support::PreparedRoot::new().unwrap();
    let mut base = prepared.metadata().unwrap();
    base["resolve"]["nodes"][1]["deps"] =
        serde_json::json!([{"pkg":"arboresce","dep_kinds":[{"kind":null}]}]);
    assert_eq!(
        check_workspace(&prepared.path, &base.to_string()),
        Err(GuardError::Dependency)
    );
    base["resolve"]["nodes"][1]["deps"][0]["dep_kinds"][0]["kind"] = serde_json::json!("dev");
    assert!(check_workspace(&prepared.path, &base.to_string()).is_ok());
    for value in [serde_json::json!("optional"), serde_json::json!(true)] {
        let mut bad = base.clone();
        bad["resolve"]["nodes"][1]["deps"][0]["dep_kinds"][0]["kind"] = value;
        assert_eq!(
            check_workspace(&prepared.path, &bad.to_string()),
            Err(GuardError::InvalidInput)
        );
    }
    let mut bad = base.clone();
    bad["resolve"]["nodes"][1]["deps"][0]["dep_kinds"] = serde_json::json!([]);
    assert_eq!(
        check_workspace(&prepared.path, &bad.to_string()),
        Err(GuardError::InvalidInput)
    );
    let mut bad = base.clone();
    bad["resolve"]["nodes"][1]["deps"][0]["pkg"] = serde_json::json!("missing");
    assert_eq!(
        check_workspace(&prepared.path, &bad.to_string()),
        Err(GuardError::InvalidInput)
    );
    let mut bad = base.clone();
    bad["packages"][1]["manifest_path"] = serde_json::json!(prepared.path.join("missing"));
    assert_eq!(
        check_workspace(&prepared.path, &bad.to_string()),
        Err(GuardError::InvalidInput)
    );
    std::fs::create_dir(prepared.path.join("crates/arboresce-domain/src/nested")).unwrap();
    std::fs::write(
        prepared
            .path
            .join("crates/arboresce-domain/src/nested/unimported.rs"),
        "use std::fs::File;",
    )
    .unwrap();
    assert_eq!(
        check_workspace(&prepared.path, &base.to_string()),
        Err(GuardError::Source)
    );
    std::fs::remove_file(
        prepared
            .path
            .join("crates/arboresce-domain/src/nested/unimported.rs"),
    )
    .unwrap();
    std::fs::write(
        prepared.path.join("crates/arboresce-domain/src/notes.txt"),
        "not Rust",
    )
    .unwrap();
    assert!(check_workspace(&prepared.path, &base.to_string()).is_ok());
    std::fs::write(
        prepared.path.join("crates/arboresce-domain/src/lib.rs"),
        [0xff],
    )
    .unwrap();
    assert_eq!(
        check_workspace(&prepared.path, &base.to_string()),
        Err(GuardError::InvalidInput)
    );
    assert_eq!(
        check_source(Layer::Domain, &" ".repeat(4 * 1024 * 1024 + 1)),
        Err(GuardError::InvalidInput)
    );
    assert_eq!(
        check_workspace(&prepared.path, &" ".repeat(8 * 1024 * 1024 + 1)),
        Err(GuardError::InvalidInput)
    );
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
fn syntax_source_symlinks_outside_manifests_and_empty_collections_fail() {
    let inside = support::PreparedRoot::new().unwrap();
    let outside = support::PreparedRoot::new().unwrap();
    let base = inside.metadata().unwrap();
    let other = outside.metadata().unwrap();
    let mut invalid = base.clone();
    invalid["packages"][0]["manifest_path"] = other["packages"][0]["manifest_path"].clone();
    assert_eq!(
        check_workspace(&inside.path, &invalid.to_string()),
        Err(GuardError::InvalidInput)
    );
    std::os::unix::fs::symlink(
        outside.path.join("crates/arboresce/src/lib.rs"),
        inside.path.join("crates/arboresce-domain/src/linked.rs"),
    )
    .unwrap();
    assert_eq!(
        check_workspace(&inside.path, &base.to_string()),
        Err(GuardError::InvalidInput)
    );
    std::fs::remove_file(inside.path.join("crates/arboresce-domain/src/linked.rs")).unwrap();
    std::fs::remove_file(inside.path.join("crates/arboresce-domain/src/lib.rs")).unwrap();
    assert_eq!(
        check_workspace(&inside.path, &base.to_string()),
        Err(GuardError::InvalidInput)
    );
    std::fs::remove_dir(inside.path.join("crates/arboresce-domain/src")).unwrap();
    assert_eq!(
        check_workspace(&inside.path, &base.to_string()),
        Err(GuardError::InvalidInput)
    );
}

#[test]
fn independently_approved_explicit_source_regressions() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/guard-v2/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["source_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let actual = Layer::from_name(case["layer"].as_str().unwrap())
            .is_some_and(|layer| check_source(layer, case["rust"].as_str().unwrap()).is_ok());
        assert_eq!(actual, case["accepted"].as_bool().unwrap(), "{id}");
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 72);
}

#[cfg(unix)]
#[test]
fn independently_approved_cargo_target_inventory_regressions() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/guard-v2/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["inventory_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let prepared = support::PreparedRoot::new().unwrap();
        let mut value = prepared.metadata().unwrap();
        let package = prepared.path.join("crates/arboresce-domain");
        match id {
            "inventory-custom-library-outside-src" | "inventory-custom-binary-outside-src" => {
                let outside = package.join("outside.rs");
                std::fs::write(&outside, "pub fn n() { let _ = std::fs::read(\"a\"); }").unwrap();
                value["packages"][1]["targets"][0]["src_path"] = serde_json::json!(outside);
                if id == "inventory-custom-binary-outside-src" {
                    value["packages"][1]["targets"][0]["kind"] = serde_json::json!(["bin"]);
                }
            }
            "inventory-missing-targets" => {
                value["packages"][1]
                    .as_object_mut()
                    .unwrap()
                    .remove("targets");
            }
            "inventory-empty-targets" => value["packages"][1]["targets"] = serde_json::json!([]),
            "inventory-unknown-target-kind" => {
                value["packages"][1]["targets"][0]["kind"] = serde_json::json!(["unselected"])
            }
            "inventory-empty-target-kinds" => {
                value["packages"][1]["targets"][0]["kind"] = serde_json::json!([])
            }
            "inventory-missing-target-file" => {
                value["packages"][1]["targets"][0]["src_path"] =
                    serde_json::json!(package.join("src/missing.rs"))
            }
            "inventory-outside-workspace-root" => {
                value["workspace_root"] = serde_json::json!(package)
            }
            "inventory-source-root-symlink" => {
                std::fs::remove_dir_all(package.join("src")).unwrap();
                std::os::unix::fs::symlink(
                    prepared.path.join("crates/arboresce/src"),
                    package.join("src"),
                )
                .unwrap();
            }
            "inventory-unregistered-peer-target" => {
                let peer = package.join("tests/support/peer.rs");
                std::fs::create_dir_all(peer.parent().unwrap()).unwrap();
                std::fs::write(&peer, "fn main() {}").unwrap();
                value["packages"][1]["targets"][0] = serde_json::json!({"name":"arboresce-xtask-test-peer","kind":["bin"],"src_path":peer});
            }
            "inventory-duplicate-target-names" => {
                let target = value["packages"][1]["targets"][0].clone();
                value["packages"][1]["targets"]
                    .as_array_mut()
                    .unwrap()
                    .push(target);
            }
            "inventory-registered-test-peer" => {
                let tool = prepared.path.join("tools/xtask");
                std::fs::create_dir_all(tool.join("src")).unwrap();
                std::fs::create_dir_all(tool.join("tests/support")).unwrap();
                std::fs::write(
                    tool.join("Cargo.toml"),
                    "[package]\nname=\"arboresce-xtask\"\nversion=\"0.0.0\"\n",
                )
                .unwrap();
                std::fs::write(tool.join("src/lib.rs"), "pub fn n() {}").unwrap();
                std::fs::write(tool.join("tests/support/peer.rs"), "fn main() {}").unwrap();
                value["packages"].as_array_mut().unwrap().push(serde_json::json!({
                    "id":"arboresce-xtask","name":"arboresce-xtask","version":"0.0.0","manifest_path":tool.join("Cargo.toml"),
                    "targets":[{"name":"arboresce-xtask","kind":["lib"],"src_path":tool.join("src/lib.rs")},
                        {"name":"arboresce-xtask-test-peer","kind":["bin"],"src_path":tool.join("tests/support/peer.rs")}]
                }));
                value["workspace_members"]
                    .as_array_mut()
                    .unwrap()
                    .push(serde_json::json!("arboresce-xtask"));
                value["resolve"]["nodes"]
                    .as_array_mut()
                    .unwrap()
                    .push(serde_json::json!({"id":"arboresce-xtask","deps":[]}));
            }
            "inventory-default-targets" => {}
            _ => panic!("unknown independently approved inventory ID {id}"),
        }
        let actual = check_workspace(&prepared.path, &value.to_string())
            .err()
            .map(|e| e.to_string());
        assert_eq!(actual.as_deref(), case["error"].as_str(), "{id}");
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 13);
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

#[test]
fn additional_approved_actual_targets_and_panic_paths() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/guard-v3/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["source_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let actual = check_source(
            Layer::from_name(case["layer"].as_str().unwrap()).unwrap(),
            case["rust"].as_str().unwrap(),
        )
        .is_ok();
        assert_eq!(actual, case["accepted"].as_bool().unwrap(), "{id}");
        println!("xtask case={id} pid={}", std::process::id());
    }
    for case in fixture["inventory_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let prepared = support::PreparedRoot::new().unwrap();
        let mut value = prepared.metadata().unwrap();
        let package = prepared.path.join("crates/arboresce-domain");
        let source = package.join("src/runtime.txt");
        let pure = id == "inventory-non-rs-pure-only-library";
        std::fs::write(
            &source,
            if pure {
                "pub fn n() -> u8 { 1 }"
            } else {
                "pub fn n() { let _ = std::fs::read(\"a\"); }"
            },
        )
        .unwrap();
        value["packages"][1]["targets"][0]["src_path"] = serde_json::json!(source);
        if id == "inventory-non-rs-forbidden-binary" {
            value["packages"][1]["targets"][0]["kind"] = serde_json::json!(["bin"]);
        }
        if pure {
            std::fs::remove_file(package.join("src/lib.rs")).unwrap();
        }
        let actual = check_workspace(&prepared.path, &value.to_string())
            .err()
            .map(|e| e.to_string());
        assert_eq!(actual.as_deref(), case["error"].as_str(), "{id}");
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 5);
}

#[test]
fn additional_approved_source_include_aliases() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/guard-v4/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["source_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let actual = check_source(
            Layer::from_name(case["layer"].as_str().unwrap()).unwrap(),
            case["rust"].as_str().unwrap(),
        )
        .is_ok();
        assert_eq!(actual, case["accepted"].as_bool().unwrap(), "{id}");
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 7);
}

#[cfg(unix)]
#[test]
fn additional_approved_collector_and_fixture_preparation_failures() {
    use std::io::ErrorKind;
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/guard-v5/cases.json"
    ))
    .unwrap();
    let case = &fixture["output_case"];
    let prepared = support::PreparedRoot::new().unwrap();
    let payload = prepared.path.join("exact-byte-payload");
    let count = usize::try_from(case["input_recipe"]["count"].as_u64().unwrap()).unwrap();
    assert_eq!(
        case["collector_byte_limit"].as_u64().unwrap() + 1,
        count as u64
    );
    std::fs::write(
        &payload,
        vec![u8::try_from(case["input_recipe"]["byte"].as_u64().unwrap()).unwrap(); count],
    )
    .unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_arboresce-xtask-test-peer"));
    command
        .env("ARBORESCE_XTASK_TEST_PEER", case["mode"].as_str().unwrap())
        .env("ARBORESCE_XTASK_TEST_METADATA", &payload);
    assert_eq!(
        support::observe(&mut command).unwrap_err().kind(),
        ErrorKind::InvalidData
    );
    println!(
        "xtask case={} pid={}",
        case["id"].as_str().unwrap(),
        std::process::id()
    );
    let mut seen = BTreeSet::new();
    for case in fixture["fixture_failure_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let prepared = support::PreparedRoot::new().unwrap();
        match id {
            "fixture-source-root-file-conflict" => {
                std::fs::write(prepared.path.join("crates"), b"owned conflict").unwrap();
                assert_eq!(
                    prepared.metadata().unwrap_err().kind(),
                    ErrorKind::NotADirectory
                );
            }
            "fixture-manifest-directory-conflict" => {
                std::fs::create_dir_all(prepared.path.join("crates/arboresce/Cargo.toml")).unwrap();
                assert_eq!(
                    prepared.metadata().unwrap_err().kind(),
                    ErrorKind::IsADirectory
                );
            }
            "fixture-source-directory-conflict" => {
                std::fs::create_dir_all(prepared.path.join("crates/arboresce/src/lib.rs")).unwrap();
                assert_eq!(
                    prepared.metadata().unwrap_err().kind(),
                    ErrorKind::IsADirectory
                );
            }
            "fixture-duplicate-peer-directory" => {
                let directory = prepared.cargo_peer().unwrap();
                let before = std::fs::read_link(directory.join("cargo")).unwrap();
                assert_eq!(
                    prepared.cargo_peer().unwrap_err().kind(),
                    ErrorKind::AlreadyExists
                );
                assert_eq!(std::fs::read_link(directory.join("cargo")).unwrap(), before);
            }
            _ => panic!("unknown independently approved support failure ID {id}"),
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 4);
}

#[test]
fn additional_approved_grouped_self_identities() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/guard-v6/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["source_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let actual = check_source(
            Layer::from_name(case["layer"].as_str().unwrap()).unwrap(),
            case["rust"].as_str().unwrap(),
        )
        .is_ok();
        assert_eq!(actual, case["accepted"].as_bool().unwrap(), "{id}");
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 6);
}

#[test]
fn additional_approved_transitive_source_include_aliases() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/guard-v7/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["source_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let actual = check_source(
            Layer::from_name(case["layer"].as_str().unwrap()).unwrap(),
            case["rust"].as_str().unwrap(),
        )
        .is_ok();
        assert_eq!(actual, case["accepted"].as_bool().unwrap(), "{id}");
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 8);
}
