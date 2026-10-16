use std::{collections::BTreeSet, ffi::OsString, process::Command};

use arboresce_xtask::{
    Action,
    architecture::{GuardError, Layer, check_dependency, check_source},
    emit,
    workspace::{check_workspace, registered_layer},
};

mod support;

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
fn qualified_source_postgres_dependency_profiles() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/qualified-source-postgres-dependencies-v1/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for row in fixture["cases"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let mut input = fixture["base_input"].clone();
        for patch in row["patches"].as_array().unwrap() {
            *input.pointer_mut(patch["path"].as_str().unwrap()).unwrap() = patch["value"].clone();
        }
        let root = support::PreparedRoot::new().unwrap();
        let metadata = support::qualified_source::metadata(&root, &input, None).unwrap();
        let actual = check_workspace(&root.path, &metadata.to_string());
        assert_eq!(actual.is_ok(), row["accepted"], "{id}: {actual:?}");
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 71);
    for row in fixture["source_cases"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let root = support::PreparedRoot::new().unwrap();
        let metadata =
            support::qualified_source::metadata(&root, &fixture["base_input"], Some(row)).unwrap();
        let actual = check_workspace(&root.path, &metadata.to_string());
        assert_eq!(actual.is_ok(), row["accepted"], "{id}: {actual:?}");
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 143);
}

#[test]
fn qualified_source_dependency_profiles() {
    let fixture = support::qualified_source::fixture();
    let mut seen = BTreeSet::new();
    for row in fixture["cases"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let mut input = fixture["base_input"].clone();
        for patch in row["patches"].as_array().unwrap() {
            *input.pointer_mut(patch["path"].as_str().unwrap()).unwrap() = patch["value"].clone();
        }
        let root = support::PreparedRoot::new().unwrap();
        let metadata = support::qualified_source::metadata(&root, &input, None).unwrap();
        let actual = check_workspace(&root.path, &metadata.to_string());
        assert_eq!(actual.is_ok(), row["accepted"], "{id}: {actual:?}");
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 121);
    for row in fixture["source_cases"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let mut input = fixture["base_input"].clone();
        input["profile"] = row["profile"].clone();
        if row["profile"] == "initial-v1" {
            input["graph"]["roots"] = serde_json::json!([]);
        }
        if row["profile"] == "jcs-number-v1" {
            input["graph"] = support::qualified_number::fixture()["base_input"]["graph"].clone();
        }
        let root = support::PreparedRoot::new().unwrap();
        let metadata = support::qualified_source::metadata(&root, &input, Some(row)).unwrap();
        let actual = check_workspace(&root.path, &metadata.to_string());
        assert_eq!(actual.is_ok(), row["accepted"], "{id}: {actual:?}");
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 197);
}

#[test]
fn qualified_number_dependency_profiles() {
    let fixture = support::qualified_number::fixture();
    let mut seen = BTreeSet::new();
    for row in fixture["cases"].as_array().unwrap() {
        let id = format!("number-graph-{}", row["id"].as_str().unwrap());
        assert!(seen.insert(id.clone()));
        let mut input = fixture["base_input"].clone();
        for patch in row["patches"].as_array().unwrap() {
            *input.pointer_mut(patch["path"].as_str().unwrap()).unwrap() = patch["value"].clone();
        }
        let root = support::PreparedRoot::new().unwrap();
        let metadata = support::qualified_number::metadata(&root, &input).unwrap();
        let actual = check_workspace(&root.path, &metadata.to_string());
        assert_eq!(actual.is_ok(), row["accepted"], "{id}: {actual:?}");
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 74);
}

#[test]
fn independently_approved_collector_version_text() {
    use arboresce_xtask::collection;
    use serde_json::{Value, json};
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v7/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["action"] == "versions")
    {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let streams = ["cargo", "rustc", "plugin", "cov", "profdata"]
            .map(|key| support::collection_inputs::literal_bytes(&case["recipe"][key]));
        let actual = match collection::versions_from(streams.each_ref().map(Vec::as_slice)) {
            Ok(value) => json!({"value":value}),
            Err(error) => json!({"error":error.to_string()}),
        };
        assert_eq!(actual, case["expected"], "{id}");
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 19);
}

#[test]
fn independently_approved_collector_physical_boundaries() {
    use serde_json::Value;
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v7/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"].as_array().unwrap().iter().filter(|c| {
        matches!(
            c["action"].as_str(),
            Some(
                "fresh"
                    | "objects"
                    | "receipts"
                    | "profile-name"
                    | "dependency"
                    | "configs"
                    | "json"
                    | "dependency-malformed"
                    | "compiled-sources"
                    | "binary-counts"
                    | "preflight"
            )
        )
    }) {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let owner = support::PreparedRoot::new().unwrap();
        let actual = support::collection_inputs::boundary(&owner, case).unwrap();
        assert_eq!(
            actual,
            case["expected"],
            "{id}; retained {}",
            owner.path.display()
        );
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 72);
}

#[test]
fn independently_approved_collector_preparation_failures() {
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
        .filter(|c| c["action"] == "preparation")
    {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let owner = support::PreparedRoot::new().unwrap();
        assert_eq!(
            support::collection_inputs::preparation_failure(&owner, case).unwrap(),
            case["expected"],
            "{id}"
        );
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 16);
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
fn independently_approved_collector_real_mini_workspaces() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v3/cases.json"
    ))
    .unwrap();
    let protocol: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v8/cases.json"
    ))
    .unwrap();
    let mut cases = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["action"] == "mini")
        .cloned()
        .collect::<Vec<_>>();
    cases.push(
        protocol["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["action"] == "mini-service-change")
            .unwrap()
            .clone(),
    );
    let mut seen = BTreeSet::new();
    for case in &cases {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let extension = protocol["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["reuse_case"] == id || p["id"] == id);
        let mut invocation = case.clone();
        if let Some(extension) = extension {
            invocation["service_model"] = protocol["service_model"].clone();
            invocation["source_override"] = extension["source_override"].clone();
        }
        let mini = support::collection_mini::Mini::run(&fixture, &invocation).unwrap();
        assert_eq!(
            mini.actual.status.code().unwrap() as i64,
            case["expected"]["exit"].as_i64().unwrap(),
            "{id}; retained {}",
            mini.owner.display()
        );
        assert!(mini.outer_preserved().unwrap(), "{id}");
        if case["action"] == "mini-service-change" {
            assert_eq!(
                mini.actual.stderr,
                case["expected"]["stderr"].as_str().unwrap().as_bytes(),
                "{id}"
            );
            assert!(mini.actual.stdout.is_empty());
            assert!(mini.output.join("prepared-after.json").is_file());
            assert!(!mini.output.join("source-after.json").exists());
            assert!(!mini.output.join("result.json").exists());
            let witness = case["expected"]["captured_witness"]
                .as_str()
                .unwrap()
                .as_bytes();
            let objects = mini.record("objects.json").unwrap();
            assert_eq!(
                objects
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|o| o["required"] == true)
                    .count(),
                3
            );
            for object in objects
                .as_array()
                .unwrap()
                .iter()
                .filter(|o| o["required"] == true)
            {
                let program = serde_json::json!(object["path"].as_str().unwrap().as_bytes());
                let commands = mini.commands().unwrap();
                let (path, _, completion) = commands
                    .iter()
                    .find(|(_, launch, _)| {
                        launch["executable"] == program
                            && launch["arguments"]
                                .as_array()
                                .unwrap()
                                .contains(&serde_json::json!(b"--show-output".as_slice()))
                    })
                    .unwrap();
                assert_eq!(completion["exit"], 0);
                assert!(completion["error"].is_null() && completion["signal"].is_null());
                if object["package"] == "arboresce-xtask" {
                    assert!(
                        std::fs::read(path.join("stdout"))
                            .unwrap()
                            .windows(witness.len())
                            .any(|v| v == witness)
                    );
                    assert_eq!(
                        mini.record("harness-arboresce-xtask-test-value.json")
                            .unwrap()["pid"],
                        completion["pid"]
                    );
                }
            }
        } else if case["mutation"] == true {
            assert_eq!(
                mini.actual.stderr,
                case["expected"]["stderr"].as_str().unwrap().as_bytes(),
                "{id}"
            );
            assert!(mini.actual.stdout.is_empty());
            assert_eq!(
                std::fs::read(mini.source.join("README.md")).unwrap(),
                b"Collector mutation witness.\n"
            );
            let captured = std::fs::read_dir(mini.output.join("commands"))
                .unwrap()
                .map(|entry| std::fs::read(entry.unwrap().path().join("stdout")).unwrap())
                .collect::<Vec<_>>();
            let witness = fixture["mini_mutation_witness"]["captured_stdout_line"]
                .as_str()
                .unwrap();
            assert!(
                captured.iter().any(|bytes| bytes
                    .windows(witness.len())
                    .any(|window| window == witness.as_bytes())),
                "{id}"
            );
            assert_eq!(
                mini.record("source-before.json").unwrap()["source"]["README.md"]["sha256"],
                arboresce_xtask::inventory::sha256(
                    fixture["mini_workspace_files"]["README.md"]
                        .as_str()
                        .unwrap()
                        .as_bytes()
                )
            );
        } else if case["closed"] == "stdout" {
            assert_eq!(
                mini.actual.stderr,
                case["expected"]["stderr"].as_str().unwrap().as_bytes()
            );
            assert!(mini.actual.stdout.is_empty());
            assert_eq!(mini.record("result.json").unwrap()["status"], "PASS");
        } else {
            assert!(mini.actual.stderr.is_empty());
            assert_eq!(mini.successful().unwrap(), case["expected"], "{id}");
            assert_eq!(
                mini.actual.stdout,
                format!("{}\n", case["expected"]["success_record"]).as_bytes()
            );
            support::collection_mini::assert_real_mini_identity(&mini);
        }
        if let Some(service) = &mini.service {
            let extension = extension.unwrap();
            assert!(mini.unchanged_source().unwrap());
            assert_eq!(
                service.directory_modes,
                serde_json::json!({"data":0o700,"socket":0o700})
            );
            assert_eq!(
                std::fs::symlink_metadata(&service.socket)
                    .unwrap_err()
                    .kind(),
                std::io::ErrorKind::NotFound
            );
            assert_eq!(
                arboresce_xtask::inventory::sha256(&std::fs::read(&service.input).unwrap()),
                service.input_sha256
            );
            let parent = service.input.parent().unwrap();
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::symlink_metadata(parent.join("data"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o700
            );
            assert_eq!(
                std::fs::read(parent.join("selection-seen")).unwrap(),
                service.input.as_os_str().as_encoded_bytes()
            );
            for (name, contents) in protocol["service_model"]["files"].as_object().unwrap() {
                let bytes = if extension["action"] == "mini-service-change"
                    && extension["expected"]["changed_file"] == name.as_str()
                {
                    extension["expected"]["changed_bytes"]
                        .as_str()
                        .unwrap()
                        .as_bytes()
                } else {
                    contents.as_str().unwrap().as_bytes()
                };
                assert_eq!(
                    std::fs::read(parent.join(name)).unwrap(),
                    bytes,
                    "{id} {name}"
                );
            }
            let before = mini.record("source-before.json").unwrap();
            assert_eq!(
                before["native"]["postgres"],
                serde_json::json!({"path":service.input,"sha256":service.input_sha256,"record":service.record})
            );
            for (name, tool) in service.record["tools"].as_object().unwrap() {
                assert_eq!(before["tools"][format!("postgres-{name}")], tool["sha256"]);
            }
            if extension["action"] == "mini-service-stable" {
                assert_eq!(mini.record("source-after.json").unwrap(), before);
                assert_eq!(
                    mini.record("result.json").unwrap(),
                    extension["expected"]["success_record"]
                );
                let facts = mini.record("collection-facts.json").unwrap();
                assert!(facts["launches"].as_array().unwrap().iter().all(|launch| {
                    service.record["tools"]
                        .as_object()
                        .unwrap()
                        .values()
                        .all(|tool| launch["record"]["executable_sha256"] != tool["sha256"])
                }));
                println!(
                    "xtask case={} pid={}",
                    extension["id"].as_str().unwrap(),
                    std::process::id()
                );
            }
            println!(
                "xtask collector service {}",
                serde_json::json!({"case":extension["id"],"input":service.input,"input_sha256":service.input_sha256,"record":service.record,"directory_modes":service.directory_modes,"socket_removed":true})
            );
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 4);
}

#[test]
fn independently_approved_collector_native_profile_substitution() {
    use arboresce_xtask::{
        collection_modules, collection_profiles, collection_runner::Runner, inventory::sha256,
    };
    use std::{collections::BTreeMap, fs};
    let base: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v3/cases.json"
    ))
    .unwrap();
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v5/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["action"] == "mini-profile-swap")
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
        assert!(mini.actual.stdout.is_empty());
        assert_eq!(
            mini.actual.stderr,
            case["expected"]["stderr"].as_str().unwrap().as_bytes()
        );
        assert!(mini.unchanged_source().unwrap());
        assert!(mini.outer_preserved().unwrap());
        let proof = mini.output.join("profile-substitution");
        let target =
            std::path::PathBuf::from(fs::read_to_string(proof.join("target-path")).unwrap());
        let replacement_path =
            std::path::PathBuf::from(fs::read_to_string(proof.join("replacement-path")).unwrap());
        let before = fs::read(proof.join("before.profraw")).unwrap();
        let replacement = fs::read(proof.join("replacement.profraw")).unwrap();
        assert_eq!(fs::read(&target).unwrap(), replacement);
        assert_eq!(fs::read(&replacement_path).unwrap(), replacement);
        assert_ne!(before, replacement);
        let pid = |path: &std::path::Path| {
            path.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .strip_prefix("arboresce-")
                .unwrap()
                .split('-')
                .next()
                .unwrap()
                .parse::<u32>()
                .unwrap()
        };
        let target_pid = pid(&target);
        let other_pid = pid(&replacement_path);
        assert_ne!(target_pid, other_pid);
        let context = mini.output.join("context");
        let launches = [target_pid, other_pid].map(|pid| {
            serde_json::from_slice::<serde_json::Value>(
                &fs::read(context.join("launches").join(format!("{pid}.json"))).unwrap(),
            )
            .unwrap()
        });
        assert_ne!(
            launches[0]["executable_sha256"],
            launches[1]["executable_sha256"]
        );
        let objects = mini.record("objects.json").unwrap();
        for (index, bytes) in [&before, &replacement].iter().enumerate() {
            let object = objects
                .as_array()
                .unwrap()
                .iter()
                .find(|o| o["sha256"] == launches[index]["executable_sha256"])
                .unwrap();
            let object_bytes = fs::read(object["path"].as_str().unwrap()).unwrap();
            assert_eq!(sha256(&object_bytes), object["sha256"]);
            collection_modules::matching(&object_bytes, bytes).unwrap();
        }
        let wrong = objects
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["sha256"] == launches[0]["executable_sha256"])
            .unwrap();
        assert!(
            collection_modules::matching(
                &fs::read(wrong["path"].as_str().unwrap()).unwrap(),
                &replacement
            )
            .is_err()
        );
        let launch_names = fs::read_dir(context.join("launches"))
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect::<BTreeSet<_>>();
        let exit_names = fs::read_dir(context.join("exits"))
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect::<BTreeSet<_>>();
        assert_eq!(launch_names, exit_names);
        for (pid, launch) in [target_pid, other_pid].iter().zip(launches.iter()) {
            let launch_bytes =
                fs::read(context.join("launches").join(format!("{pid}.json"))).unwrap();
            let exit: serde_json::Value = serde_json::from_slice(
                &fs::read(context.join("exits").join(format!("{pid}.json"))).unwrap(),
            )
            .unwrap();
            assert_eq!(exit["launch_sha256"], sha256(&launch_bytes));
            assert_eq!(exit["exit"], 0);
            assert_eq!(launch["pid"], *pid);
        }
        let output = mini.owner.join("profile-proof");
        fs::create_dir(&output).unwrap();
        let mut runner = Runner::new(&mini.source, &output, &mini.manifest).unwrap();
        for name in ["before.profraw", "replacement.profraw"] {
            let path = proof.join(name);
            let actual = runner
                .success(
                    "llvm-profdata",
                    &[
                        "show",
                        "--all-functions",
                        "--counts",
                        path.to_str().unwrap(),
                    ],
                    30,
                    &BTreeMap::new(),
                    &output.join("target"),
                )
                .unwrap();
            let counts =
                collection_profiles::inspect(std::str::from_utf8(&actual.output.stdout).unwrap())
                    .unwrap();
            assert_eq!(
                counts["functions"].as_u64().unwrap() == 0,
                case["mode"] == "zero"
            );
        }
        let witness = case["witness"].as_str().unwrap().as_bytes();
        assert!(mini.commands().unwrap().iter().any(|(path, _, _)| {
            fs::read(path.join("stdout"))
                .unwrap()
                .windows(witness.len())
                .any(|w| w == witness)
        }));
        assert!(!mini.output.join("result.json").exists());
        assert_eq!(case["expected"]["actual_profile_replaced"], true);
        assert_eq!(case["expected"]["source_unchanged"], true);
        assert_eq!(case["expected"]["receipt_set_preserved"], true);
        assert_eq!(case["expected"]["proof_retained"], true);
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 2);
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
fn independently_approved_collector_module_identity() {
    use arboresce_xtask::collection_modules;
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v5/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["action"] == "module")
    {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let object = support::collection_inputs::hex(case, "object_hex").unwrap();
        let profile = support::collection_inputs::hex(case, "profile_hex").unwrap();
        match collection_modules::matching(&object, &profile) {
            Ok(()) => assert_eq!(case["expected"]["value"], true, "{id}"),
            Err(error) => assert_eq!(error.to_string(), case["expected"]["error"], "{id}"),
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    let case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["action"] == "module-ambiguity")
        .unwrap();
    let base = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "collector-module-exact")
        .unwrap();
    let module =
        collection_modules::object(&support::collection_inputs::hex(base, "object_hex").unwrap())
            .unwrap();
    assert_eq!(
        collection_modules::unique(&[module.clone(), module])
            .unwrap_err()
            .to_string(),
        case["expected"]["error"]
    );
    println!(
        "xtask case={} pid={}",
        case["id"].as_str().unwrap(),
        std::process::id()
    );
    assert_eq!(seen.len(), 63);
}

#[test]
fn independently_approved_collector_runtime_identity() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v5/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["action"] == "runtime")
    {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let input =
            support::report_inputs::replace(&fixture["base_runtime"], &case["patch"]).unwrap();
        match arboresce_xtask::collection_prepared::runtime(
            &input,
            case["actual_sysroot"].as_str().unwrap(),
        ) {
            Ok(()) => assert_eq!(case["expected"]["value"], true, "{id}"),
            Err(error) => assert_eq!(error.to_string(), case["expected"]["error"], "{id}"),
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 7);
}

#[test]
fn independently_approved_collector_bounded_reads_and_final_objects() {
    use arboresce_xtask::{collection_inputs, collection_prepared as prepared};
    use std::time::{Duration, Instant};
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v5/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"].as_array().unwrap().iter().filter(|case| {
        matches!(
            case["action"].as_str(),
            Some("read" | "dependency-read" | "object-freshness")
        )
    }) {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let owner = support::PreparedRoot::new().unwrap();
        let deadline = if case["recipe"]["expired"] == true {
            Instant::now() - Duration::from_millis(1)
        } else {
            Instant::now() + Duration::from_secs(30)
        };
        let actual = if case["action"] == "object-freshness" {
            let (path, hash) = support::collection_inputs::object_change(&owner, case).unwrap();
            prepared::unchanged_object(&path, &hash, deadline).map(|()| serde_json::json!(true))
        } else {
            let path = support::collection_inputs::read(&owner, case).unwrap();
            let limit = case["limit"]
                .as_u64()
                .unwrap_or(collection_inputs::MAX_PROTOCOL_BYTES as u64);
            prepared::read_text(&path, limit, deadline).and_then(|(raw, _)| {
                if case["action"] == "dependency-read" {
                    collection_inputs::dependencies(&raw, "/output/image")
                        .map(|set| serde_json::to_value(set).unwrap())
                } else {
                    Ok(raw.into())
                }
            })
        };
        match actual {
            Ok(value) => assert_eq!(value, case["expected"]["value"], "{id}"),
            Err(error) => assert_eq!(error.to_string(), case["expected"]["error"], "{id}"),
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 15);
}

#[test]
fn independently_approved_collector_final_output_deadline() {
    use std::time::{Duration, Instant};
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v5/cases.json"
    ))
    .unwrap();
    let case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["action"] == "api-deadline-output")
        .unwrap();
    let prior: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v3/cases.json"
    ))
    .unwrap();
    let value = &prior["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["action"] == "api-final-flush")
        .unwrap()["success_record"];
    let deadline = Instant::now() + Duration::from_millis(100);
    let mut writer = support::collection_inputs::DeadlineOutput {
        bytes: Vec::new(),
        deadline,
    };
    let mut stderr = Vec::new();
    let exit = arboresce_xtask::collection::finish_until(value, deadline, &mut writer, &mut stderr);
    assert_eq!(exit as u64, case["expected"]["exit"].as_u64().unwrap());
    assert_eq!(writer.bytes, format!("{value}\n").as_bytes());
    assert_eq!(
        stderr,
        case["expected"]["stderr"].as_str().unwrap().as_bytes()
    );
    println!(
        "xtask case={} pid={}",
        case["id"].as_str().unwrap(),
        std::process::id()
    );
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
fn independently_approved_collector_final_flush() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v3/cases.json"
    ))
    .unwrap();
    let case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["action"] == "api-final-flush")
        .unwrap();
    let mut writer = support::FailedOutput {
        write_fails: false,
        bytes: Vec::new(),
    };
    let mut stderr = Vec::new();
    let exit =
        arboresce_xtask::collection::finish(&case["success_record"], &mut writer, &mut stderr);
    assert_eq!(exit as u64, case["expected"]["exit"].as_u64().unwrap());
    assert_eq!(
        writer.bytes,
        case["expected"]["written_stdout"]
            .as_str()
            .unwrap()
            .as_bytes()
    );
    assert_eq!(
        stderr,
        case["expected"]["stderr"].as_str().unwrap().as_bytes()
    );
    println!(
        "xtask case={} pid={}",
        case["id"].as_str().unwrap(),
        std::process::id()
    );
}

#[test]
fn independently_approved_collector_prepared_tool_facts() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v3/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"].as_array().unwrap().iter().filter(|c| {
        matches!(
            c["action"].as_str(),
            Some("prepared" | "prepared-resource-recipe")
        )
    }) {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let input = support::collection_inputs::prepared(&fixture["base_prepared"], case).unwrap();
        match arboresce_xtask::collection_prepared::validate(&input) {
            Ok(()) => assert_eq!(case["expected"]["value"], true, "{id}"),
            Err(error) => assert_eq!(error.to_string(), case["expected"]["error"], "{id}"),
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 39);
}

#[test]
fn independently_approved_collector_physical_file_intake() {
    use std::time::{Duration, Instant};
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v3/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["action"] == "intake")
    {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let owner = support::PreparedRoot::new().unwrap();
        let root = support::collection_inputs::physical(&owner, case).unwrap();
        let limits = arboresce_xtask::collection_prepared::Limits {
            files: case["limits"]["files"].as_u64().unwrap() as usize,
            per_file: case["limits"]["per_file"].as_u64().unwrap(),
            total: case["limits"]["total"].as_u64().unwrap(),
            depth: case["limits"]["depth"].as_u64().unwrap() as usize,
        };
        match arboresce_xtask::collection_prepared::tree(
            &root,
            limits,
            Instant::now() + Duration::from_secs(30),
        ) {
            Ok(value) => assert_eq!(value, case["expected"]["value"], "{id}"),
            Err(error) => assert_eq!(error.to_string(), case["expected"]["error"], "{id}"),
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 18);
}

#[test]
fn independently_approved_collector_complete_profile_text() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v3/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["action"] == "profile")
    {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        match arboresce_xtask::collection_profiles::inspect(case["raw"].as_str().unwrap()) {
            Ok(value) => assert_eq!(value, case["expected"]["value"], "{id}"),
            Err(error) => assert_eq!(error.to_string(), case["expected"]["error"], "{id}"),
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 21);
}

#[test]
fn independently_approved_collector_cargo_artifact_selection() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v4/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"].as_array().unwrap() {
        let input = support::report_inputs::replace(&fixture["base"], &case["patch"]).unwrap();
        let raw = input["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|record| format!("{record}\n"))
            .collect::<String>();
        let actual = arboresce_xtask::collection_artifacts::select(
            input["root"].as_str().unwrap(),
            input["target"].as_str().unwrap(),
            &input["metadata"],
            &input["registry"],
            &raw,
        );
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        match actual {
            Ok(value) => assert_eq!(value, case["expected"]["value"], "{id}"),
            Err(error) => assert_eq!(error.to_string(), case["expected"]["error"], "{id}"),
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    for case in fixture["raw_cases"].as_array().unwrap() {
        let input = &fixture["base"];
        let actual = arboresce_xtask::collection_artifacts::select(
            input["root"].as_str().unwrap(),
            input["target"].as_str().unwrap(),
            &input["metadata"],
            &input["registry"],
            case["raw"].as_str().unwrap(),
        );
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        assert_eq!(
            actual.unwrap_err().to_string(),
            case["expected"]["error"],
            "{id}"
        );
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 53);
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

#[test]
fn independently_approved_collector_process_profile_conservation() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v2/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"].as_array().unwrap() {
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
    assert_eq!(seen.len(), 87);
}

#[test]
fn independently_approved_collector_instrumentation_and_compilation_inputs() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v1/cases.json"
    ))
    .unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 103);
    let mut seen = BTreeSet::new();
    for case in fixture["cases"].as_array().unwrap() {
        let action = case["action"].as_str().unwrap();
        let raw = case["raw"].as_str().unwrap();
        let actual = match action {
            "environment" => {
                let names = case["crate_names"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_owned())
                    .collect();
                arboresce_xtask::collection_inputs::environment(
                    raw,
                    case["target"].as_str().unwrap(),
                    case["wrapper"].as_str().unwrap(),
                    &names,
                    case["workspace_name"].as_str().unwrap(),
                )
                .map(|value| serde_json::to_value(value).unwrap())
            }
            "dependencies" => arboresce_xtask::collection_inputs::dependencies(
                raw,
                case["target"].as_str().unwrap(),
            )
            .map(|value| serde_json::to_value(value).unwrap()),
            _ => continue,
        };
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        match actual {
            Ok(value) => assert_eq!(value, case["expected"]["value"], "{id}"),
            Err(error) => assert_eq!(error.to_string(), case["expected"]["error"], "{id}"),
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 47);
}

#[test]
fn independently_approved_collector_selection_and_execution_inputs() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/native-collector-v1/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"].as_array().unwrap() {
        let raw = case["raw"].as_str().unwrap();
        let actual = match case["action"].as_str().unwrap() {
            "list" => arboresce_xtask::collection_tests::list(raw)
                .map(|value| serde_json::to_value(value).unwrap()),
            "execution" => arboresce_xtask::collection_tests::execution(
                raw,
                &case["tests"],
                u32::try_from(case["pid"].as_u64().unwrap()).unwrap(),
            )
            .map(|value| serde_json::to_value(value).unwrap()),
            _ => continue,
        };
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        match actual {
            Ok(value) => assert_eq!(value, case["expected"]["value"], "{id}"),
            Err(error) => assert_eq!(error.to_string(), case["expected"]["error"], "{id}"),
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 56);
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
    assert_eq!(seen.len(), 101);
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
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["native_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
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
            assert_eq!(
                sources.len(),
                case["source_count"].as_u64().unwrap() as usize,
                "{id}"
            );
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
    assert_eq!(seen.len(), 40);
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
