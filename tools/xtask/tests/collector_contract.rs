use std::{collections::BTreeSet, process::Command};

pub mod support;

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
