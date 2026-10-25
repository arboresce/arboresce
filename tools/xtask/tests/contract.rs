use std::{collections::BTreeSet, ffi::OsString, process::Command};

use arboresce_xtask::{
    Action,
    architecture::{GuardError, Layer, check_dependency, check_source},
    emit,
    workspace::{check_workspace, registered_layer},
};

pub mod support;

#[test]
fn migration_service_physical_inputs() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/integration/postgres/fixtures/migration-service-v1/cases.json"
    ))
    .unwrap();
    for row in fixture["physical_service_cases"]
        .as_array()
        .unwrap()
        .iter()
        .chain(fixture["serialized_input_cases"].as_array().unwrap())
    {
        let owner = support::PreparedRoot::new().unwrap();
        let observed = support::collection_inputs::service(&owner, &fixture, row).unwrap();
        assert_eq!(
            observed["accepted"], row["accepted"],
            "{}: {observed}",
            row["id"]
        );
        let (changed, error, kind) = if let Some(size) = row["target_bytes"].as_u64() {
            assert_eq!(observed["physical_after"]["input"]["size"], size);
            (
                "input",
                if size > 2_097_152 {
                    Some("Intake")
                } else {
                    None
                },
                "file",
            )
        } else {
            match row["recipe"].as_str().unwrap() {
                "none" | "explicit-environment" => ("", None, ""),
                "remove-input" => ("input", Some("Intake"), "absent"),
                "change-tool" => ("postgres", Some("Changed"), "file"),
                "remove-tool" => ("postgres", Some("Intake"), "absent"),
                "change-configuration" => ("configuration", Some("Changed"), "file"),
                "change-readiness" => ("readiness", Some("Changed"), "file"),
                "change-profile" => ("input", None, "file"),
                "symlink-tool" => ("postgres", Some("Intake"), "symlink"),
                "symlink-ancestor" => ("native", Some("Intake"), "symlink"),
                "data-mode-0755" => ("data", Some("Prepared"), "directory"),
                "socket-mode-0755" => ("socket", Some("Prepared"), "directory"),
                "expire-lease" => ("input", Some("Prepared"), "file"),
                _ => panic!("unknown service recipe"),
            }
        };
        assert_eq!(observed["error_class"], serde_json::json!(error));
        assert_eq!(
            observed["physical_before"]["input"]["sha256"],
            observed["before"]["sha256"]
        );
        let mut before = observed["physical_before"].as_object().unwrap().clone();
        let mut after = observed["physical_after"].as_object().unwrap().clone();
        if !changed.is_empty() {
            let previous = before.remove(changed).unwrap();
            let actual = after.remove(changed).unwrap();
            assert_eq!(actual["path"], previous["path"]);
            assert_eq!(actual["kind"], kind);
            if kind == "file" {
                assert_ne!(actual["sha256"], previous["sha256"]);
            } else if kind == "directory" {
                assert_eq!(previous["mode"], 0o700);
                assert_eq!(actual["mode"], 0o755);
            } else if kind == "symlink" {
                let target = std::path::Path::new(previous["path"].as_str().unwrap())
                    .with_file_name(if changed == "native" {
                        "original-native"
                    } else {
                        "original-postgres"
                    });
                assert_eq!(actual["target"], serde_json::json!(target));
            }
        }
        assert_eq!(before, after, "unselected paths changed: {}", row["id"]);
        if row["recipe"] == "expire-lease" {
            let actual = observed["physical_after"]["input"]["lease_deadline_unix_ms"]
                .as_u64()
                .unwrap();
            assert!(
                u128::from(actual)
                    < std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
            );
        }
        if row["recipe"] == "change-profile" {
            assert_eq!(observed["before"]["record"], observed["after"]["record"]);
            assert_ne!(observed["before"]["sha256"], observed["after"]["sha256"]);
        }
        if row["recipe"] == "explicit-environment" {
            assert_eq!(observed["environment"]["selected"], observed["input_path"]);
            assert_eq!(observed["environment"]["pg_keys"], serde_json::json!([]));
        }
        println!(
            "xtask physical service case={} observed={observed}",
            row["id"]
        );
        println!(
            "collector case={} pid={}",
            row["id"].as_str().unwrap(),
            std::process::id()
        );
    }
}

#[test]
fn migration_service_structural_profiles() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/integration/postgres/fixtures/migration-service-v1/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for row in fixture["service_cases"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let mut value = fixture["base_input"].clone();
        for patch in row["patches"].as_array().unwrap() {
            *value.pointer_mut(patch["path"].as_str().unwrap()).unwrap() = patch["value"].clone();
        }
        assert_eq!(
            arboresce_xtask::collection_services::validate(&value).is_ok(),
            row["accepted"],
            "{id}"
        );
        println!("collector case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 31);
}

#[test]
fn migration_service_command_contract() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/integration/postgres/fixtures/migration-service-v1/cases.json"
    ))
    .unwrap();
    for row in fixture["collector_command_cases"].as_array().unwrap() {
        let args: Vec<_> = row["arguments"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| OsString::from(s.as_str().unwrap()))
            .collect();
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        let exit = arboresce_xtask::collection::run(&args, &mut stdout, &mut stderr);
        assert_eq!(exit, row["exit"].as_u64().unwrap() as u8);
        assert_eq!(stdout, row["stdout"].as_str().unwrap().as_bytes());
        assert_eq!(stderr, row["stderr"].as_str().unwrap().as_bytes());
        let output =
            support::observe(Command::new(env!("CARGO_BIN_EXE_arboresce-verify")).args(&args))
                .unwrap();
        assert_eq!(output.status.code(), Some(i32::from(exit)));
        assert_eq!(output.stdout, stdout);
        assert_eq!(output.stderr, stderr);
        println!(
            "collector case={} pid={}",
            row["id"].as_str().unwrap(),
            std::process::id()
        );
    }
    let protocol: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v8/cases.json"
    ))
    .unwrap();
    for case in protocol["cases"].as_array().unwrap().iter().filter(|case| {
        matches!(
            case["action"].as_str(),
            Some("selected-executable" | "closed-command-output")
        )
    }) {
        let id = case["id"].as_str().unwrap();
        if case["action"] == "selected-executable" {
            let base = &protocol["base_identity"];
            let service = case.get("service").unwrap_or(&base["service"]);
            let service = if service == false {
                None
            } else {
                Some(service)
            };
            let executable = case.get("executable_bytes").cloned().unwrap_or_else(|| {
                serde_json::json!(
                    case["record"]["executable_utf8"]
                        .as_str()
                        .unwrap()
                        .as_bytes()
                )
            });
            let record = serde_json::json!({
                "executable_sha256":case["record"]["sha256"],"executable":executable,
            });
            let actual = arboresce_xtask::collection::selected_executable(
                case.get("objects")
                    .unwrap_or(&base["objects"])
                    .as_array()
                    .unwrap(),
                case.get("tools").unwrap_or(&base["tools"]),
                service,
                &record,
            );
            assert_eq!(
                serde_json::json!({"accepted":actual.is_ok(),"error":actual.err().map(|e|e.to_string())}),
                case["expected"],
                "{id}"
            );
        } else {
            let arguments = case["arguments"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| OsString::from(v.as_str().unwrap()))
                .collect::<Vec<_>>();
            let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
            let mut closed = support::FailedOutput {
                write_fails: true,
                bytes: Vec::new(),
            };
            let exit = if case["closed"] == "stdout" {
                arboresce_xtask::collection::run(&arguments, &mut closed, &mut stderr)
            } else {
                arboresce_xtask::collection::run(&arguments, &mut stdout, &mut closed)
            };
            assert!(closed.bytes.is_empty(), "{id}");
            assert_eq!(
                exit as u64,
                case["expected"]["exit"].as_u64().unwrap(),
                "{id}"
            );
            assert_eq!(
                stdout,
                case["expected"]["stdout"].as_str().unwrap().as_bytes(),
                "{id}"
            );
            assert_eq!(
                stderr,
                case["expected"]["stderr"].as_str().unwrap().as_bytes(),
                "{id}"
            );
        }
        println!("collector case={id} pid={}", std::process::id());
    }
}

#[test]
fn migration_service_selection_closes_command_defaults() {
    use arboresce_xtask::collection_services::{SELECTOR, environment};
    use std::{collections::BTreeMap, ffi::OsStr, path::Path};
    let mut command = Command::new("/usr/bin/true");
    command
        .env("PGHOST", "unselected")
        .env("PGDATABASE", "unselected")
        .env(SELECTOR, "/unselected/input.json");
    environment(&mut command, Some(Path::new("/selected/input.json")));
    let values: BTreeMap<_, _> = command.get_envs().collect();
    assert_eq!(values.get(OsStr::new("PGHOST")), Some(&None));
    assert_eq!(values.get(OsStr::new("PGDATABASE")), Some(&None));
    assert_eq!(
        values.get(OsStr::new(SELECTOR)),
        Some(&Some(OsStr::new("/selected/input.json")))
    );
    environment(&mut command, None);
    assert_eq!(
        command.get_envs().find(|(k, _)| *k == SELECTOR).unwrap().1,
        None
    );
}

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

#[test]
fn migration_application_and_adapter_roles_are_explicit() {
    for (id, package, version, expected) in [
        (
            "application-layer",
            "arboresce-application",
            "0.0.0",
            Ok(Layer::Application),
        ),
        (
            "postgres-layer",
            "arboresce-postgres",
            "0.0.0",
            Ok(Layer::Adapter),
        ),
        (
            "application-wrong-version",
            "arboresce-application",
            "0.0.1",
            Err(GuardError::Dependency),
        ),
        (
            "postgres-wrong-version",
            "arboresce-postgres",
            "0.0.1",
            Err(GuardError::Dependency),
        ),
        (
            "application-lookalike",
            "arboresce-application-extra",
            "0.0.0",
            Err(GuardError::Dependency),
        ),
        (
            "postgres-lookalike",
            "arboresce-postgresql",
            "0.0.0",
            Err(GuardError::Dependency),
        ),
    ] {
        assert_eq!(registered_layer(package, version), expected, "{id}");
        println!(
            "collector case=migration-role-{id} pid={}",
            std::process::id()
        );
    }
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

#[test]
fn independently_approved_supplied_report_counts() {
    use arboresce_xtask::coverage::{Metric, evaluate};

    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/report-gate-v1/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let input =
            support::report_inputs::build(&fixture, case).expect("invalid reviewed input recipe");
        let actual = evaluate(&input.root, &input.inventory, &input.report);
        if let Some(error) = case["error"].as_str() {
            assert_eq!(actual.unwrap_err().to_string(), error, "{id}");
        } else {
            let actual = actual.unwrap();
            for (scope, metrics) in case["expected_scopes"].as_object().unwrap() {
                let counts = actual.scopes.get(scope).unwrap();
                assert_eq!(
                    counts.lines.count,
                    metrics["lines"]["count"].as_u64().unwrap(),
                    "{id}/{scope}"
                );
                assert_eq!(
                    counts.lines.covered,
                    metrics["lines"]["covered"].as_u64().unwrap(),
                    "{id}/{scope}"
                );
                assert_eq!(
                    counts.regions.count,
                    metrics["regions"]["count"].as_u64().unwrap(),
                    "{id}/{scope}"
                );
                assert_eq!(
                    counts.regions.covered,
                    metrics["regions"]["covered"].as_u64().unwrap(),
                    "{id}/{scope}"
                );
            }
            if let Some(count) = case["expected_scope_count"].as_u64() {
                assert_eq!(actual.scopes.len() as u64, count, "{id}");
                let mut files = 0;
                for (scope, counts) in &actual.scopes {
                    if scope.starts_with("file:") {
                        files += 1;
                        assert_eq!(
                            counts.lines.count,
                            case["expected_per_file"]["lines"]["count"]
                                .as_u64()
                                .unwrap()
                        );
                        assert_eq!(
                            counts.lines.covered,
                            case["expected_per_file"]["lines"]["covered"]
                                .as_u64()
                                .unwrap()
                        );
                        assert_eq!(
                            counts.regions.count,
                            case["expected_per_file"]["regions"]["count"]
                                .as_u64()
                                .unwrap()
                        );
                        assert_eq!(
                            counts.regions.covered,
                            case["expected_per_file"]["regions"]["covered"]
                                .as_u64()
                                .unwrap()
                        );
                    }
                }
                assert_eq!(files, case["expected_file_scope_count"].as_u64().unwrap());
            } else {
                assert_eq!(
                    actual.scopes.len(),
                    case["expected_scopes"].as_object().unwrap().len()
                );
            }
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    for case in fixture["metric_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let metric = Metric {
            count: case["count"].as_u64().unwrap(),
            covered: case["covered"].as_u64().unwrap(),
        };
        assert_eq!(metric.strict(), case["strict"].as_bool().unwrap(), "{id}");
        println!("xtask case={id} pid={}", std::process::id());
    }
    for case in fixture["fixture_builder_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let accepted = if let Some(patch) = case.get("patch") {
            support::report_inputs::replace(&fixture["base_report"], patch).is_ok()
        } else {
            support::report_inputs::build(&fixture, &case["input_case"]).is_ok()
        };
        assert_eq!(accepted, case["accepted"].as_bool().unwrap(), "{id}");
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 88);
}

#[cfg(unix)]
#[test]
fn independently_approved_actual_inventory_models() {
    use arboresce_xtask::inventory::{portable_path, sha256, snapshot};
    use serde_json::{Value, json};

    let fixture: Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/source-inventory-v1/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let prepared = support::inventory_inputs::prepare(&fixture, case)
            .unwrap_or_else(|error| panic!("{id}: reviewed recipe preparation failed: {error}"));
        let actual = snapshot(&prepared.root, &prepared.profile, &prepared.metadata);
        if let Some(wanted) = case["expected"]["error"].as_str() {
            assert_eq!(format!("{:?}", actual.unwrap_err()), wanted, "{id}");
        } else {
            let actual = actual.unwrap_or_else(|error| panic!("{id}: {error}"));
            assert_eq!(actual["version"], "rust-source-inventory-v1", "{id}");
            let mut files = case["expected"]["files"].clone();
            if let Some(recipe) = case["expected"].get("generated_empty_files") {
                for index in 0..recipe["count"].as_u64().unwrap() {
                    files[format!("{}{index}", recipe["prefix"].as_str().unwrap())] =
                        json!({"sha256":recipe["sha256"],"size":0});
                }
                assert_eq!(
                    actual["files"].as_object().unwrap().len(),
                    case["expected"]["file_count"].as_u64().unwrap() as usize,
                    "{id}"
                );
            }
            assert_eq!(actual["files"], files, "{id}");
            for (actual_rows, expected_rows) in [
                (&actual["sources"], &case["expected"]["source_paths"]),
                (
                    &actual["coverage_roles"]["sources"],
                    &case["expected"]["coverage_source_paths"],
                ),
            ] {
                let paths: BTreeSet<_> = actual_rows
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|row| row["path"].as_str().unwrap())
                    .collect();
                let wanted: BTreeSet<_> = expected_rows
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|row| row.as_str().unwrap())
                    .collect();
                assert_eq!(paths, wanted, "{id}");
            }
            assert_eq!(
                actual["targets"].as_array().unwrap().len(),
                case["expected"]["target_count"].as_u64().unwrap() as usize,
                "{id}"
            );
            assert_eq!(actual["coverage_roles"]["version"], "rust-report-roles-v1");
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    for case in fixture["sha256_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        assert_eq!(
            sha256(case["input"].as_str().unwrap().as_bytes()),
            case["sha256"].as_str().unwrap(),
            "{id}"
        );
        println!("xtask case={id} pid={}", std::process::id());
    }
    for case in fixture["path_cases"].as_array().unwrap() {
        use std::{os::unix::ffi::OsStringExt, path::PathBuf};
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let bytes = case["bytes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| u8::try_from(b.as_u64().unwrap()).unwrap())
            .collect();
        let path = PathBuf::from(OsString::from_vec(bytes));
        assert_eq!(
            format!("{:?}", portable_path(&path).unwrap_err()),
            case["expected_error"].as_str().unwrap(),
            "{id}"
        );
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 121);
}

#[cfg(unix)]
#[test]
fn independently_approved_source_inventory_native_boundary() {
    use serde_json::Value;
    use std::{fs, os::unix::ffi::OsStringExt, path::Path};

    let fixture: Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/source-inventory-v1/cases.json"
    ))
    .unwrap();
    let raw_profile = include_str!("../../../tests/architecture/rust-source-roles-v1.json");
    let profile: Value = serde_json::from_str(raw_profile).unwrap();
    let reviewed_counts = &fixture["native_source_counts"];
    assert_eq!(
        reviewed_counts["version"],
        "source-inventory-native-source-counts-v1"
    );
    assert_eq!(
        reviewed_counts["case_id"],
        "inventory-native-current-public-checkout"
    );
    let profile_sha256 = arboresce_xtask::inventory::sha256(raw_profile.as_bytes());
    let mut matching_counts = reviewed_counts["profiles"]
        .as_array()
        .expect("native inventory fixture needs input profiles")
        .iter()
        .filter(|row| row["profile_sha256"].as_str() == Some(profile_sha256.as_str()));
    let expected_source_count = usize::try_from(
        matching_counts
            .next()
            .expect("native inventory fixture needs this reviewed input profile")["source_count"]
            .as_u64()
            .expect("reviewed native source count must be unsigned"),
    )
    .expect("reviewed native source count exceeds usize");
    assert!(
        matching_counts.next().is_none(),
        "native inventory fixture duplicates this input profile"
    );
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["native_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        if case["mode"] == "actual" {
            assert_eq!(case["id"], reviewed_counts["case_id"], "{id}");
        }
        let temporary = support::PreparedRoot::new().unwrap();
        let temp_profile = temporary.path.join("profile.json");
        let mut command = Command::new(env!("CARGO_BIN_EXE_arboresce-source-inventory"));
        let mut closed = match case["close"].as_str() {
            Some("stdout") => Some(support::ClosedStream::Stdout),
            Some("stderr") => Some(support::ClosedStream::Stderr),
            None => None,
            _ => panic!("invalid reviewed closed stream"),
        };
        match case["mode"].as_str().unwrap() {
            "help" | "-h" | "--help" => {
                command.arg(case["mode"].as_str().unwrap());
            }
            "empty" | "closed-usage" => {}
            "arity" => {
                command.args(["one", "two", "three"]);
            }
            "missing-root" => {
                command
                    .arg(temporary.path.join("absent"))
                    .arg(&temp_profile);
            }
            "missing-profile" => {
                command.arg(root).arg(&temp_profile);
            }
            "invalid-profile" => {
                fs::write(&temp_profile, "{").unwrap();
                command.arg(root).arg(&temp_profile);
            }
            "directory-profile" => {
                command.arg(root).arg(&temporary.path);
            }
            "non-utf8-root" => {
                command
                    .arg(OsString::from_vec(vec![0xff]))
                    .arg(&temp_profile);
            }
            "closed-help" => {
                command.arg("--help");
                closed = Some(support::ClosedStream::Stdout);
            }
            "metadata-unavailable" => {
                command
                    .arg(root)
                    .arg(root.join("tests/architecture/rust-source-roles-v1.json"));
                command.env("PATH", &temporary.path);
            }
            "actual" | "closed-actual" => {
                command
                    .arg(root)
                    .arg(root.join("tests/architecture/rust-source-roles-v1.json"));
            }
            "bad-manifest" => {
                fs::write(temporary.path.join("Cargo.toml"), "invalid toml [").unwrap();
                fs::write(&temp_profile, raw_profile).unwrap();
                command.arg(&temporary.path).arg(&temp_profile);
                command.env("RUSTUP_TOOLCHAIN", "1.99.0");
            }
            "peer-invalid" | "peer-bytes" | "peer-warning-closed-stderr" => {
                let bin = temporary.cargo_peer().unwrap();
                let mut paths = vec![bin];
                paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap()));
                command.env("PATH", std::env::join_paths(paths).unwrap());
                let mode = match case["mode"].as_str().unwrap() {
                    "peer-invalid" => "metadata-invalid",
                    "peer-warning-closed-stderr" => "metadata-warning",
                    "peer-bytes" => {
                        let bytes = temporary.path.join("metadata.bytes");
                        fs::write(&bytes, [0xff]).unwrap();
                        command.env("ARBORESCE_XTASK_TEST_METADATA", bytes);
                        "metadata"
                    }
                    _ => unreachable!(),
                };
                command.env("ARBORESCE_XTASK_TEST_PEER", mode);
                command
                    .arg(root)
                    .arg(root.join("tests/architecture/rust-source-roles-v1.json"));
            }
            "bad-role" | "missing-generated" | "bad-declaration" => {
                let mut input = profile.clone();
                match case["mode"].as_str().unwrap() {
                    "bad-role" => input["sources"][0]["role"] = "unsupported".into(),
                    "missing-generated" => {
                        input["generated_sources"] = serde_json::json!(["missing-generated.rs"])
                    }
                    "bad-declaration" => {
                        let row = input["sources"]
                            .as_array_mut()
                            .unwrap()
                            .iter_mut()
                            .find(|s| s["role"] == "declaration")
                            .unwrap();
                        row["declaration_sha256"] = "0".repeat(64).into();
                    }
                    _ => unreachable!(),
                }
                fs::write(&temp_profile, input.to_string()).unwrap();
                command.arg(root).arg(&temp_profile);
            }
            "profile-too-large" => {
                assert_eq!(
                    case["profile_payload"],
                    "complete-current-role-profile-json-followed-only-by-ascii-space-padding"
                );
                let bytes = support::report_inputs::pad(
                    raw_profile.to_owned(),
                    case["profile_padding_bytes"].as_u64().unwrap(),
                )
                .unwrap();
                fs::write(&temp_profile, bytes).unwrap();
                command.arg(root).arg(&temp_profile);
            }
            _ => panic!("unknown reviewed native inventory case"),
        }
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
            Some(case["exit"].as_i64().unwrap() as i32),
            "{id}"
        );
        if let Some(suffix) = case["stderr_suffix"].as_str() {
            let error = std::str::from_utf8(&result.stderr).unwrap();
            assert!(
                error.ends_with(suffix),
                "{id}: missing complete error suffix"
            );
            assert!(
                error.contains(case["stderr_contains"].as_str().unwrap()),
                "{id}: missing decisive diagnostic witness"
            );
        } else {
            assert_eq!(
                result.stderr,
                case["stderr"].as_str().unwrap().as_bytes(),
                "{id}"
            );
        }
        if case["mode"] == "actual" {
            let actual: Value = serde_json::from_slice(&result.stdout).unwrap();
            assert_eq!(actual["version"], case["stdout_json_version"], "{id}");
            let literal = &case["literal_file"];
            let file = &actual["files"][literal["path"].as_str().unwrap()];
            assert_eq!(file["sha256"], literal["sha256"], "{id}");
            assert_eq!(file["size"], literal["size"], "{id}");
            let sources = actual["sources"].as_array().unwrap();
            assert_eq!(sources.len(), expected_source_count, "{id}");
            assert_eq!(actual["sources"], profile["sources"], "{id}");
            assert_eq!(actual["targets"], profile["targets"], "{id}");
            assert_eq!(
                actual["targets"].as_array().unwrap().len(),
                case["target_count"].as_u64().unwrap() as usize,
                "{id}"
            );
            let declaration = sources
                .iter()
                .find(|s| s["path"] == case["literal_na"]["path"])
                .unwrap();
            assert_eq!(declaration["role"], "declaration", "{id}");
            assert_eq!(
                declaration["declaration_sha256"], case["literal_na"]["sha256"],
                "{id}"
            );
            assert!(
                actual["metadata_trace"]
                    .as_str()
                    .unwrap()
                    .contains("within_budget"),
                "{id}"
            );
        } else {
            assert_eq!(
                result.stdout,
                case["stdout"].as_str().unwrap().as_bytes(),
                "{id}"
            );
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 23);
}

#[test]
fn independently_approved_inventory_api_stream_failures() {
    use arboresce_xtask::inventory_command;
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/source-inventory-v1/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["stream_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let args: Vec<OsString> = case["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| OsString::from(v.as_str().unwrap()))
            .collect();
        let mut failed = support::FailedOutput {
            write_fails: case["write_fails"].as_bool().unwrap(),
            bytes: Vec::new(),
        };
        let mut other = Vec::new();
        let code = match case["selected"].as_str().unwrap() {
            "stdout" => inventory_command::run(&args, &mut failed, &mut other),
            "stderr" => inventory_command::run(&args, &mut other, &mut failed),
            _ => panic!("invalid reviewed stream selector"),
        };
        assert_eq!(code, case["exit"].as_u64().unwrap() as u8, "{id}");
        assert_eq!(
            other,
            case["other_stream"].as_str().unwrap().as_bytes(),
            "{id}"
        );
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 4);
}

#[cfg(unix)]
#[test]
fn independently_approved_inventory_preparation_failures() {
    use serde_json::{Value, json};
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/source-inventory-v1/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["recipe_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let mut model = fixture.clone();
        let mut recipe = json!({"file_operations":[],"profile_patch":[],"metadata_patch":[]});
        for (target, changes) in [
            (&mut model, &case["fixture_changes"]),
            (&mut recipe, &case["case_changes"]),
        ] {
            for (key, value) in changes.as_object().unwrap() {
                target[key] = value.clone();
            }
        }
        let error = match support::inventory_inputs::prepare(&model, &recipe) {
            Err(error) => error,
            Ok(_) => panic!("{id}: malformed preparation unexpectedly succeeded"),
        };
        assert_eq!(
            format!("{:?}", error.kind()),
            case["expected_error"].as_str().unwrap(),
            "{id}"
        );
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 41);
}

#[cfg(unix)]
#[test]
fn independently_approved_inventory_process_failures() {
    use arboresce_xtask::process::{Observation, observe};
    use serde_json::json;
    use std::{
        fs,
        io::Write,
        os::unix::process::ExitStatusExt,
        process::Stdio,
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    };
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/source-inventory-v1/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["process_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let root = support::PreparedRoot::new().unwrap();
        let stdout = root.path.join("stdout");
        let stderr = root.path.join("stderr");
        let running = case["mode"] == "running";
        let mut command = if running {
            let mut command = Command::new("/bin/sleep");
            command.arg("1");
            command
        } else {
            let mut command = Command::new(env!("CARGO_BIN_EXE_arboresce-xtask-test-peer"));
            command.env("ARBORESCE_XTASK_TEST_PEER", "pass");
            command
        };
        command
            .stdout(Stdio::from(fs::File::create_new(&stdout).unwrap()))
            .stderr(Stdio::from(fs::File::create_new(&stderr).unwrap()));
        let started = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let mut child = arboresce_xtask::measurement::spawn(&mut command).unwrap();
        let mut preparation_trace = Vec::new();
        if !running {
            let prepared = observe(
                &mut child,
                &stdout,
                &stderr,
                Observation::default(),
                Instant::now(),
                &mut preparation_trace,
            )
            .unwrap_or_else(|error| panic!("{id}: bounded peer preparation failed: {error}"));
            assert_eq!(prepared.status.code(), Some(0), "{id}");
            assert_eq!(child.try_wait().unwrap().unwrap().code(), Some(0), "{id}");
        }
        let raw_stdout = fs::read(&stdout).unwrap();
        let raw_stderr = fs::read(&stderr).unwrap();
        assert_eq!(
            raw_stdout,
            if running {
                b"".as_slice()
            } else {
                b"child stdout\n"
            },
            "{id}"
        );
        assert_eq!(
            raw_stderr,
            if running {
                b"".as_slice()
            } else {
                b"child stderr\n"
            },
            "{id}"
        );
        match case["mode"].as_str().unwrap() {
            "missing-stdout" => fs::remove_file(&stdout).unwrap(),
            "missing-stderr" => fs::remove_file(&stderr).unwrap(),
            "directory-stdout" => {
                fs::remove_file(&stdout).unwrap();
                fs::create_dir(&stdout).unwrap();
            }
            "trace" | "running" | "late" | "success" => {}
            _ => panic!("unknown reviewed observation input"),
        }
        let mut trace = support::ObservationTrace::new(
            case["fail_after_lines"].as_u64().map(|n| n as usize),
            case["delay_after_line"].as_u64().unwrap_or(1) as usize,
            Duration::from_millis(case["delay_after_first_line_ms"].as_u64().unwrap_or(0)),
        );
        let actual = observe(
            &mut child,
            &stdout,
            &stderr,
            Observation {
                budget: Duration::from_millis(case["budget_ms"].as_u64().unwrap_or(30_000)),
                ..Observation::default()
            },
            Instant::now(),
            &mut trace,
        );
        trace.flush().unwrap();
        let error = if let Some(wanted) = case["expected_error"].as_str() {
            let error = actual.unwrap_err();
            assert_eq!(format!("{:?}", error.kind()), wanted, "{id}");
            json!(format!("{:?}", error.kind()))
        } else {
            let output = actual.unwrap_or_else(|error| panic!("{id}: {error}"));
            assert_eq!(output.status.code(), Some(0), "{id}");
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
            serde_json::Value::Null
        };
        let status = child
            .try_wait()
            .unwrap()
            .expect("owned child must be reaped");
        assert_eq!(json!(status.code()), case["expected_exit"], "{id}");
        assert_eq!(json!(status.signal()), case["expected_signal"], "{id}");
        println!(
            "xtask process observation {}",
            json!({
                "case":id,"pid":child.id(),"exit":status.code(),"signal":status.signal(),
                "stdout":raw_stdout,"stderr":raw_stderr,"trace":trace.bytes,
                "error":error,"reaped":true,"preparation_trace":preparation_trace,
                "started_ms":started.as_millis(),
                "finished_ms":SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
            })
        );
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 11);
}
