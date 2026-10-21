use std::collections::BTreeSet;

use arboresce_xtask::workspace::check_workspace;

pub mod support;

#[test]
fn qualified_continuity_wire_dependency_profiles() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/qualified-continuity-wire-dependencies-v1/cases.json"
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
        let metadata = support::qualified_continuity_wire::metadata(&root, &input, None).unwrap();
        let actual = check_workspace(&root.path, &metadata.to_string());
        assert_eq!(actual.is_ok(), row["accepted"], "{id}: {actual:?}");
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 96);
}

#[test]
fn qualified_continuity_wire_source_policy() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/architecture/fixtures/qualified-continuity-wire-dependencies-v1/cases.json"
    ))
    .unwrap();
    let mut seen = BTreeSet::new();
    for row in fixture["source_cases"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let root = support::PreparedRoot::new().unwrap();
        let metadata = support::qualified_continuity_wire::metadata(
            &root,
            &fixture["base_input"],
            Some(row["source"].as_str().unwrap()),
        )
        .unwrap();
        let actual = check_workspace(&root.path, &metadata.to_string());
        assert_eq!(actual.is_ok(), row["accepted"], "{id}: {actual:?}");
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 8);
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
