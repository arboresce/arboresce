//! Real finite collector runs. Failed and successful native evidence stays owned.

use super::{PreparedRoot, collection_native};
use arboresce_xtask::{collection_prepared as prepared, inventory, measurement};
use serde_json::{Value, json};
use std::{
    fs,
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

pub struct Mini {
    pub owner: PathBuf,
    pub source: PathBuf,
    pub output: PathBuf,
    pub manifest: Value,
    pub actual: std::process::Output,
    pub ambient: Value,
    pub outer_hash: String,
    pub outer: PathBuf,
    pub conflict: Value,
}

fn installed() -> io::Result<(PathBuf, PathBuf)> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::other("installed tool home missing"))?;
    let selected = std::env::var_os("RUSTC")
        .map(PathBuf::from)
        .and_then(|p| p.canonicalize().ok())
        .filter(|p| p.file_name().is_some_and(|name| name == "rustc"));
    let sysroot = if let Some(compiler) = selected {
        compiler.parent().and_then(Path::parent).unwrap().to_owned()
    } else {
        std::env::var_os("RUSTUP_HOME")
            .map_or_else(|| home.join(".rustup"), PathBuf::from)
            .join("toolchains/1.99.0-aarch64-apple-darwin")
    }
    .canonicalize()?;
    let plugin = if let Some(wrapper) = std::env::var_os("RUSTC_WRAPPER")
        .map(PathBuf::from)
        .filter(|p| p.file_name().is_some_and(|name| name == "cargo-llvm-cov"))
    {
        wrapper
    } else {
        measurement::selected_program(&Command::new("cargo-llvm-cov"))?
    };
    Ok((sysroot, plugin))
}

impl Mini {
    pub fn run(fixture: &Value, case: &Value) -> io::Result<Self> {
        let temporary = PreparedRoot::new()?;
        let owner = temporary.path.clone();
        std::mem::forget(temporary);
        let source = owner.join("source");
        fs::create_dir(&source)?;
        let mut files = fixture["mini_workspace_files"].as_object().unwrap().clone();
        if case["mutation"] == true {
            files.extend(fixture["mini_mutation_files"].as_object().unwrap().clone());
        }
        if let Some(overrides) = case["source_override"].as_object() {
            files.extend(overrides.clone());
        }
        for (name, value) in &files {
            let path = source.join(name);
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(path, value.as_str().unwrap())?;
        }
        fs::write(
            owner.join("source-before.json"),
            format!("{}\n", json!(files)),
        )?;
        let ambient = prepared::environment().map_err(io::Error::other)?;
        let inherited = std::env::var_os(measurement::ENV).map(PathBuf::from);
        let outer = if let Some(path) = inherited.as_ref() {
            path.clone()
        } else {
            collection_native::context(&owner, "outer")?
        };
        let outer_hash = inventory::sha256(&fs::read(outer.join("context.json"))?);
        let local = if inherited.is_none() {
            Some(measurement::Request::open(&outer)?)
        } else {
            None
        };
        let (sysroot, plugin) = installed()?;
        let cargo_home = owner.join("cargo-home");
        fs::create_dir_all(cargo_home.join("bin"))?;
        let discovered = cargo_home.join("bin/cargo-llvm-cov");
        let selected_before = fs::read(&plugin)?;
        fs::copy(&plugin, &discovered)?;
        let copy_before = fs::read(&discovered)?;
        let copy_mode = fs::metadata(&discovered)?.permissions().mode();
        fs::set_permissions(&discovered, fs::Permissions::from_mode(copy_mode | 0o200))?;
        fs::OpenOptions::new()
            .append(true)
            .open(&discovered)?
            .write_all(b"\nUnselected collector discovery witness.\n")?;
        fs::set_permissions(&discovered, fs::Permissions::from_mode(copy_mode))?;
        let conflict = json!({"selected":plugin,"selected_sha256":inventory::sha256(&selected_before),
            "selected_size":selected_before.len(),"selected_mode":fs::metadata(&plugin)?.permissions().mode(),
            "discovered":discovered,"copy_before_sha256":inventory::sha256(&copy_before),
            "copy_before_size":copy_before.len(),"copy_before_mode":copy_mode,"copy_after_sha256":inventory::sha256(&fs::read(&discovered)?),
            "copy_after_size":fs::metadata(&discovered)?.len(),"copy_after_mode":fs::metadata(&discovered)?.permissions().mode(),
            "classification":"unselected-adversarial-discovery-candidate"});
        fs::write(owner.join("plugin-conflict.json"), format!("{conflict}\n"))?;
        let manifest_path = owner.join("prepared.json");
        let mut prepare = Command::new(env!("CARGO_BIN_EXE_arboresce-verify"));
        prepare
            .args([
                std::ffi::OsStr::new("prepare"),
                source.as_os_str(),
                sysroot.as_os_str(),
                plugin.as_os_str(),
                cargo_home.as_os_str(),
                manifest_path.as_os_str(),
                outer.as_os_str(),
            ])
            .current_dir(&source)
            .env(measurement::ENV, &outer);
        if inherited.is_none() {
            prepare.env("LLVM_PROFILE_FILE", owner.join("outer-%p-%12m.profraw"));
        }
        let prepare_output = owner.join("preparation");
        fs::create_dir(&prepare_output)?;
        let actual =
            collection_native::capture(&mut prepare, &prepare_output, None, local.as_ref())?;
        if !actual.status.success() || !actual.stdout.is_empty() || !actual.stderr.is_empty() {
            return Err(io::Error::other(format!(
                "collector mini preparation failed; retained {}: {:?} {}",
                owner.display(),
                actual.status,
                String::from_utf8_lossy(&actual.stderr)
            )));
        }
        let manifest: Value = serde_json::from_slice(&fs::read(&manifest_path)?)?;
        let output = owner.join("collection");
        let mut command = Command::new(env!("CARGO_BIN_EXE_arboresce-verify"));
        command
            .args([
                source.as_os_str(),
                source.join("roles.json").as_os_str(),
                source.join("tests.json").as_os_str(),
                manifest_path.as_os_str(),
                output.as_os_str(),
            ])
            .current_dir(&source)
            .env(measurement::ENV, &outer);
        if inherited.is_none() {
            command.env("LLVM_PROFILE_FILE", owner.join("outer-%p-%12m.profraw"));
        }
        let streams = owner.join("invocation");
        fs::create_dir(&streams)?;
        let actual = collection_native::capture(
            &mut command,
            &streams,
            case["closed"].as_str(),
            local.as_ref(),
        )?;
        println!(
            "xtask collector mini {}",
            json!({"case":case["id"],"owner":owner,"exit":actual.status.code(),"stdout":actual.stdout,"stderr":actual.stderr})
        );
        Ok(Self {
            owner,
            source,
            output,
            manifest,
            actual,
            ambient,
            outer_hash,
            outer,
            conflict,
        })
    }

    pub fn outer_preserved(&self) -> io::Result<bool> {
        Ok(
            prepared::environment().map_err(io::Error::other)? == self.ambient
                && inventory::sha256(&fs::read(self.outer.join("context.json"))?)
                    == self.outer_hash,
        )
    }

    pub fn record(&self, name: &str) -> io::Result<Value> {
        serde_json::from_slice(&fs::read(self.output.join(name))?).map_err(io::Error::other)
    }

    pub fn unchanged_source(&self) -> io::Result<bool> {
        let before: Value =
            serde_json::from_slice(&fs::read(self.owner.join("source-before.json"))?)?;
        for (name, body) in before.as_object().unwrap() {
            if fs::read(self.source.join(name))? != body.as_str().unwrap().as_bytes() {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn commands(&self) -> io::Result<Vec<(PathBuf, Value, Value)>> {
        let mut paths = fs::read_dir(self.output.join("commands"))?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<io::Result<Vec<_>>>()?;
        paths.sort();
        paths
            .into_iter()
            .map(|path| {
                let launch = serde_json::from_slice(&fs::read(path.join("launch-command.json"))?)?;
                let completion = serde_json::from_slice(&fs::read(path.join("completion.json"))?)?;
                Ok((path, launch, completion))
            })
            .collect()
    }

    pub fn successful(&self) -> io::Result<Value> {
        let result = self.record("result.json")?;
        let facts = self.record("collection-facts.json")?;
        let tools = self.record("prepared-before.json")?;
        if tools != self.manifest || tools != self.record("prepared-after.json")? {
            return Err(io::Error::other("mini prepared identity mismatch"));
        }
        let mut observed = result.as_object().unwrap().clone();
        observed.remove("version");
        observed.remove("status");
        observed.insert("exit".into(), json!(self.actual.status.code()));
        observed.insert(
            "zero_failed_ignored_filtered".into(),
            json!(
                facts["commands"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|row| row["role"] == "run")
                    .count()
                    == 6
            ),
        );
        observed.insert(
            "all_actual_tools".into(),
            json!(tools["tools"].as_object().unwrap().len() == 15),
        );
        observed.insert(
            "outer_context_preserved".into(),
            json!(self.outer_preserved()?),
        );
        observed.insert("success_record".into(), result);
        Ok(Value::Object(observed))
    }
}

pub fn assert_mini_module_pairs(mini: &Mini, expected: usize) {
    use arboresce_xtask::{collection_modules, inventory::sha256};
    let facts = mini.record("collection-facts.json").unwrap();
    let profiles = facts["profiles"].as_array().unwrap();
    assert_eq!(profiles.len(), expected);
    for profile in profiles {
        let object = facts["objects"]
            .as_array()
            .unwrap()
            .iter()
            .find(|object| object["sha256"] == profile["object_sha256"])
            .unwrap();
        let object_bytes = std::fs::read(object["path"].as_str().unwrap()).unwrap();
        let profile_bytes = std::fs::read(
            mini.output
                .join("raw")
                .join(profile["name"].as_str().unwrap()),
        )
        .unwrap();
        assert_eq!(sha256(&object_bytes), object["sha256"]);
        assert_eq!(sha256(&profile_bytes), profile["sha256"]);
        collection_modules::matching(&object_bytes, &profile_bytes).unwrap();
        let launch = facts["launches"]
            .as_array()
            .unwrap()
            .iter()
            .find(|l| l["record"]["pid"] == profile["pid"])
            .unwrap();
        assert_eq!(launch["record"]["executable_sha256"], object["sha256"]);
        assert!(
            facts["exits"]
                .as_array()
                .unwrap()
                .iter()
                .any(|exit| exit["record"]["pid"] == profile["pid"]
                    && exit["record"]["launch_sha256"] == launch["sha256"])
        );
    }
}

pub fn assert_real_mini_identity(mini: &Mini) {
    use arboresce_xtask::{
        collection_modules, collection_prepared, collection_runner::Runner, inventory::sha256,
    };
    use serde_json::json;
    use std::{
        collections::{BTreeMap, BTreeSet},
        fs,
        os::unix::fs::PermissionsExt,
        path::Path,
        time::{Duration, Instant},
    };
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../tests/architecture/fixtures/native-collector-v5/cases.json"
    ))
    .unwrap();
    let boundaries: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../tests/architecture/fixtures/native-collector-v7/cases.json"
    ))
    .unwrap();
    let facts = mini.record("collection-facts.json").unwrap();
    let mut seen = BTreeSet::new();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .chain(boundaries["cases"].as_array().unwrap())
        .filter(|case| {
            matches!(
                case["action"].as_str(),
                Some(
                    "mini-module"
                        | "mini-runtime"
                        | "mini-plugin"
                        | "mini-phase"
                        | "mini-runner"
                        | "mini-preflight"
                        | "mini-merge"
                )
            )
        })
    {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        match case["action"].as_str().unwrap() {
            "mini-preflight" => assert_mini_preflight(mini, case),
            "mini-merge" => assert_mini_merge(mini, case),
            "mini-module" => {
                assert_mini_module_pairs(
                    mini,
                    case["expected"]["positive_pairs"].as_u64().unwrap() as usize,
                );
                let profiles = facts["profiles"].as_array().unwrap();
                let before = profiles
                    .iter()
                    .map(|p| {
                        (
                            p["name"].as_str().unwrap(),
                            fs::read(mini.output.join("raw").join(p["name"].as_str().unwrap()))
                                .unwrap(),
                        )
                    })
                    .collect::<BTreeMap<_, _>>();
                let positive = profiles
                    .iter()
                    .filter(|p| p["positive_functions"].as_u64().unwrap() > 0)
                    .collect::<Vec<_>>();
                let first = positive[0];
                let other = positive
                    .iter()
                    .find(|p| p["object_sha256"] != first["object_sha256"])
                    .unwrap();
                let object = facts["objects"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|o| o["sha256"] == first["object_sha256"])
                    .unwrap();
                let copied = mini.owner.join("cross-object.profraw");
                fs::write(&copied, &before[other["name"].as_str().unwrap()]).unwrap();
                assert!(
                    collection_modules::matching(
                        &fs::read(object["path"].as_str().unwrap()).unwrap(),
                        &fs::read(&copied).unwrap()
                    )
                    .is_err()
                );
                for (name, bytes) in before {
                    assert_eq!(fs::read(mini.output.join("raw").join(name)).unwrap(), bytes);
                }
                assert!(mini.unchanged_source().unwrap());
                assert_eq!(case["expected"]["cross_object_rejected"], true);
                assert_eq!(case["expected"]["original_evidence_unchanged"], true);
            }
            "mini-runtime" => {
                let actual = fs::read_to_string(
                    mini.owner
                        .join("prepared.json.evidence/commands/0000/stdout"),
                )
                .unwrap();
                collection_prepared::runtime(&mini.manifest, &actual).unwrap();
                let fake = mini.owner.join("different-sysroot");
                for name in ["bin", "lib", "libexec"] {
                    fs::create_dir_all(fake.join(name)).unwrap();
                    fs::write(
                        fake.join(name).join("resource"),
                        b"physical wrong runtime\n",
                    )
                    .unwrap();
                }
                let mut candidate = mini.manifest.clone();
                candidate["sysroot"] = json!(fake);
                fs::write(
                    mini.owner.join("different-runtime.json"),
                    format!("{candidate}\n"),
                )
                .unwrap();
                assert_eq!(
                    collection_prepared::runtime(&candidate, &actual)
                        .unwrap_err()
                        .to_string(),
                    case["expected"]["error"]
                );
                assert_eq!(candidate["tools"], mini.manifest["tools"]);
            }
            "mini-plugin" => {
                let selected = &mini.manifest["tools"]["cargo-llvm-cov"];
                let discovered = Path::new(mini.conflict["discovered"].as_str().unwrap());
                let selected_path = Path::new(selected["program"].as_str().unwrap());
                assert_eq!(
                    sha256(&fs::read(discovered).unwrap()),
                    mini.conflict["copy_after_sha256"]
                );
                assert_eq!(
                    sha256(&fs::read(selected_path).unwrap()),
                    mini.conflict["selected_sha256"]
                );
                assert_eq!(
                    mini.conflict["copy_before_sha256"],
                    mini.conflict["selected_sha256"]
                );
                assert_ne!(
                    mini.conflict["copy_after_sha256"],
                    mini.conflict["selected_sha256"]
                );
                assert_ne!(
                    fs::metadata(discovered).unwrap().permissions().mode() & 0o111,
                    0
                );
                assert_eq!(
                    fs::metadata(discovered).unwrap().permissions().mode(),
                    mini.conflict["copy_after_mode"].as_u64().unwrap() as u32
                );
                assert_eq!(
                    fs::metadata(discovered).unwrap().len(),
                    mini.conflict["copy_after_size"].as_u64().unwrap()
                );
                let commands = mini.commands().unwrap();
                assert!(
                    commands
                        .iter()
                        .any(|(_, l, _)| l["executable_sha256"] == selected["sha256"])
                );
                assert!(
                    !commands
                        .iter()
                        .any(|(_, l, _)| l["executable_sha256"]
                            == mini.conflict["copy_after_sha256"])
                );
                assert!(!facts["launches"].as_array().unwrap().iter().any(
                    |l| l["record"]["executable_sha256"] == mini.conflict["copy_after_sha256"]
                ));
                let show = commands
                    .iter()
                    .find(|(_, l, _)| {
                        l["arguments"]
                            .as_array()
                            .unwrap()
                            .contains(&json!(b"show-env"))
                    })
                    .unwrap();
                assert_eq!(
                    show.1["program"],
                    json!(selected_path.as_os_str().as_encoded_bytes())
                );
                let raw = fs::read_to_string(show.0.join("stdout")).unwrap();
                let target = mini.output.join("target");
                for (key, value) in [
                    ("RUSTC_WRAPPER", selected_path),
                    ("CARGO_LLVM_COV_TARGET_DIR", target.as_path()),
                    ("CARGO_LLVM_COV_BUILD_DIR", target.as_path()),
                ] {
                    assert!(
                        raw.lines()
                            .any(|line| line == format!("{key}={}", value.display()))
                    );
                }
                assert_eq!(
                    case["expected"],
                    json!({"selected_plugin_launched":true,"discovered_plugin_launched":false,
                    "wrapper_selected":true,"target_equal":true,"build_dir_equal":true})
                );
            }
            "mini-phase" => {
                let output = mini.owner.join("phase");
                fs::create_dir(&output).unwrap();
                let mut runner = Runner::new(&mini.source, &output, &mini.manifest).unwrap();
                let deadline = Instant::now()
                    + Duration::from_millis(case["recipe"]["phase_budget_ms"].as_u64().unwrap());
                runner.deadline = Some(deadline);
                let target = output.join("target");
                let actual = runner
                    .tool("true", &[], 30, &BTreeMap::new(), &target)
                    .unwrap();
                assert_eq!(
                    actual.output.status.code().unwrap(),
                    case["expected"]["first_exit"].as_i64().unwrap() as i32
                );
                while Instant::now() < deadline {
                    std::thread::sleep(Duration::from_millis(1));
                }
                let error = runner
                    .tool("true", &[], 30, &BTreeMap::new(), &target)
                    .err()
                    .unwrap();
                assert_eq!(error.to_string(), case["expected"]["second_error"]);
                assert_eq!(fs::read_dir(output.join("commands")).unwrap().count(), 1);
                assert_eq!(case["expected"]["second_spawned"], false);
            }
            "mini-runner" => {
                let output = mini.owner.join(id);
                fs::create_dir(&output).unwrap();
                let context = super::collection_native::context(&output, "B").unwrap();
                let request = arboresce_xtask::measurement::Request::open(&context).unwrap();
                let mut manifest = mini.manifest.clone();
                let mode = case["recipe"]["mode"].as_str().unwrap();
                if mode == "changed" {
                    manifest["tools"]["true"]["sha256"] = "0".repeat(64).into();
                }
                let mut runner = Runner::new(&mini.source, &output, &manifest).unwrap();
                runner.request = Some(request);
                runner.context = Some(context.clone());
                let target = runner.target.clone();
                let deadline = Instant::now()
                    + Duration::from_millis(case["recipe"]["phase_budget_ms"].as_u64().unwrap());
                let result = match mode {
                    "timeout" => {
                        runner.deadline = Some(deadline);
                        runner.tool("sleep", &["30"], 30, &BTreeMap::new(), &target)
                    }
                    "false" => runner.success("false", &[], 30, &BTreeMap::new(), &target),
                    "changed" => runner.tool("true", &[], 30, &BTreeMap::new(), &target),
                    _ => unreachable!(),
                };
                let finished = Instant::now();
                let error = result.err().unwrap().to_string();
                let launches = fs::read_dir(context.join("launches"))
                    .unwrap()
                    .map(|e| fs::read(e.unwrap().path()).unwrap())
                    .collect::<Vec<_>>();
                let exits = fs::read_dir(context.join("exits"))
                    .unwrap()
                    .map(|e| {
                        serde_json::from_slice::<serde_json::Value>(
                            &fs::read(e.unwrap().path()).unwrap(),
                        )
                        .unwrap()
                    })
                    .collect::<Vec<_>>();
                let actual = if mode == "changed" {
                    assert!(exits.is_empty());
                    assert!(!output.join("commands/0000/launch-command.json").exists());
                    json!({"error":error,"spawned":!launches.is_empty()})
                } else {
                    assert_eq!(launches.len(), 1);
                    assert_eq!(exits.len(), 1);
                    let completion: serde_json::Value = serde_json::from_slice(
                        &fs::read(output.join("commands/0000/completion.json")).unwrap(),
                    )
                    .unwrap();
                    assert!(
                        fs::read(output.join("commands/0000/stdout"))
                            .unwrap()
                            .is_empty()
                    );
                    assert!(
                        fs::read(output.join("commands/0000/stderr"))
                            .unwrap()
                            .is_empty()
                    );
                    if mode == "false" {
                        json!({"error":error,"exit":completion["exit"],"signal":completion["signal"],"completion_error":!completion["error"].is_null()})
                    } else {
                        let launch: serde_json::Value =
                            serde_json::from_slice(&launches[0]).unwrap();
                        let trace = fs::read_to_string(output.join("commands/0000/trace")).unwrap();
                        json!({"error":error,"exit":completion["exit"],"signal":completion["signal"],"completion_error":!completion["error"].is_null(),"reaped":trace.contains(&format!("xtask cleanup pid={} reaped=true",completion["pid"])),"local_launches":launches.len(),"local_exits":exits.len(),"matching_actual_pid":launch["pid"]==completion["pid"]&&exits[0]["pid"]==completion["pid"],"matching_launch_hash":exits[0]["launch_sha256"]==sha256(&launches[0]),"within_cleanup_budget":finished.saturating_duration_since(deadline).as_nanos()<=case["recipe"]["cleanup_budget_ns"].as_u64().unwrap()as u128})
                    }
                };
                fs::write(output.join("observed.json"), format!("{actual}\n")).unwrap();
                assert_eq!(
                    actual,
                    case["expected"],
                    "{id}; retained {}",
                    output.display()
                );
            }
            _ => unreachable!(),
        }
        println!("xtask case={id} pid={}", std::process::id());
    }
    assert_eq!(seen.len(), 13);
}

pub fn assert_mini_merge(mini: &Mini, case: &Value) {
    use arboresce_xtask::{collection, collection_runner::Runner};
    use std::{
        collections::BTreeMap,
        time::{Duration, Instant},
    };
    let deadline = Instant::now() + Duration::from_secs(180);
    let raw = mini.output.join("raw");
    let before = prepared::tree(&raw, prepared::PROFILES, deadline).unwrap();
    let source_limits = prepared::Limits {
        files: 4096,
        per_file: 64 << 20,
        total: 256 << 20,
        depth: 32,
    };
    let source_before = prepared::tree(&mini.source, source_limits, deadline).unwrap();
    let selected = before
        .as_object()
        .unwrap()
        .keys()
        .take(case["recipe"]["profiles"].as_u64().unwrap() as usize)
        .map(|name| raw.join(name))
        .collect::<Vec<_>>();
    assert_eq!(selected.len() as u64, case["recipe"]["profiles"]);
    let output = mini.owner.join(case["id"].as_str().unwrap());
    fs::create_dir(&output).unwrap();
    let mut runner = Runner::new(&mini.source, &output, &mini.manifest).unwrap();
    runner.deadline = Some(deadline);
    let bounded = output.join("bounded.profdata");
    collection::merge(&mut runner, &selected, &bounded).unwrap();
    let mut commands = fs::read_dir(output.join("commands"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    commands.sort();
    let mut graph = Vec::new();
    let mut leaves = BTreeMap::new();
    for path in &selected {
        leaves.insert(path.clone(), vec![path.clone()]);
    }
    let mut uses = BTreeMap::<PathBuf, usize>::new();
    let mut maximum = 0;
    let mut partials = Vec::new();
    for directory in &commands {
        let launch: Value =
            serde_json::from_slice(&fs::read(directory.join("launch-command.json")).unwrap())
                .unwrap();
        let encoded: Vec<Vec<u8>> = serde_json::from_value(launch["arguments"].clone()).unwrap();
        let args = encoded
            .into_iter()
            .map(|bytes| String::from_utf8(bytes).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(&args[..2], ["merge", "-sparse"]);
        assert_eq!(args[args.len() - 2], "-o");
        let inputs = args[2..args.len() - 2]
            .iter()
            .map(PathBuf::from)
            .collect::<Vec<_>>();
        let result = PathBuf::from(args.last().unwrap());
        assert!(inputs.len() <= case["recipe"]["fan_in"].as_u64().unwrap() as usize);
        maximum = maximum.max(inputs.len());
        let mut consumed = Vec::new();
        for input in &inputs {
            *uses.entry(input.clone()).or_default() += 1;
            consumed.extend(leaves.get(input).unwrap().clone());
        }
        assert!(leaves.insert(result.clone(), consumed).is_none());
        assert!(fs::metadata(&result).unwrap().len() > 0);
        if result == bounded && selected.len() == 9 {
            assert_eq!(inputs, partials);
        }
        if result != bounded {
            partials.push(result.clone());
        }
        graph.push(json!({"inputs":inputs,"output":result,"launch":launch}));
    }
    let mut consumed = leaves[&bounded].clone();
    consumed.sort();
    let mut all_selected = selected.clone();
    all_selected.sort();
    let conserved = consumed == all_selected
        && selected.iter().chain(&partials).all(|path| uses[path] == 1)
        && uses.len() == selected.len() + partials.len();
    let reference = output.join("reference.profdata");
    let mut args = vec!["merge".to_owned(), "-sparse".to_owned()];
    args.extend(
        selected
            .iter()
            .map(|path| path.to_str().unwrap().to_owned()),
    );
    args.extend(["-o".into(), reference.to_str().unwrap().to_owned()]);
    let tool = runner.program("llvm-profdata").unwrap();
    let target = runner.target.clone();
    assert!(
        runner
            .run(&tool, &args, 180, &BTreeMap::new(), &target)
            .unwrap()
            .output
            .status
            .success()
    );
    let facts = mini.record("collection-facts.json").unwrap();
    let tool = runner.program("llvm-cov").unwrap();
    let mut exports = Vec::new();
    for (profile, name) in [(&bounded, "bounded.json"), (&reference, "reference.json")] {
        let mut args = vec![
            "export".into(),
            "-format=text".into(),
            format!("-instr-profile={}", profile.display()),
        ];
        for object in facts["objects"].as_array().unwrap() {
            args.extend([
                "-object".into(),
                object["path"].as_str().unwrap().to_owned(),
            ]);
        }
        let actual = runner
            .run(&tool, &args, 180, &BTreeMap::new(), &target)
            .unwrap();
        assert!(actual.output.status.success());
        fs::write(output.join(name), &actual.output.stdout).unwrap();
        exports.push(serde_json::from_slice::<Value>(&actual.output.stdout).unwrap());
    }
    let actual = json!({
        "selected_profiles":selected.len(),"bounded_merge_commands":commands.len(),
        "reference_merge_commands":1,"export_commands":exports.len(),
        "max_inputs_per_bounded_merge":maximum,"every_selected_raw_consumed_once":conserved,
        "complete_exports_equal":exports[0]==exports[1],
        "original_raw_unchanged":before==prepared::tree(&raw,prepared::PROFILES,deadline).unwrap(),
        "original_source_unchanged":source_before==prepared::tree(&mini.source,source_limits,deadline).unwrap() && mini.unchanged_source().unwrap()
    });
    assert_eq!(
        fs::read_dir(output.join("commands")).unwrap().count(),
        commands.len() + 3
    );
    fs::write(output.join("graph.json"), format!("{}\n", json!(graph))).unwrap();
    fs::write(output.join("observed.json"), format!("{actual}\n")).unwrap();
    assert_eq!(
        actual,
        case["expected"],
        "{}; retained {}",
        case["id"],
        output.display()
    );
}

pub fn assert_mini_preflight(mini: &Mini, case: &Value) {
    use arboresce_xtask::{collection_prepared as prepared, inventory::sha256};
    use std::time::{Duration, Instant};
    let limits = prepared::Limits {
        files: 16384,
        per_file: 1 << 30,
        total: 4 << 30,
        depth: 32,
    };
    let deadline = Instant::now() + Duration::from_secs(180);
    let before = prepared::tree(&mini.owner, limits, deadline).unwrap();
    let outer_before = prepared::tree(&mini.outer, limits, deadline).unwrap();
    let output = mini.owner.join(case["id"].as_str().unwrap());
    fs::create_dir(&output).unwrap();
    let recipe = &case["recipe"];
    let mut manifest = mini.manifest.clone();
    let mut roles = mini.source.join("roles.json");
    match recipe["mode"].as_str().unwrap() {
        "missing-image" => {
            let missing = recipe["missing_program"]
                .as_str()
                .unwrap()
                .replace("$case", output.to_str().unwrap());
            for field in recipe["replace_selected_fields"].as_array().unwrap() {
                manifest["tools"][recipe["tool"].as_str().unwrap()][field.as_str().unwrap()] =
                    missing.clone().into();
            }
            assert!(!Path::new(&missing).exists());
        }
        "wrong-sysroot" => {
            let root = PathBuf::from(
                recipe["declared_sysroot"]
                    .as_str()
                    .unwrap()
                    .replace("$case", output.to_str().unwrap()),
            );
            for (name, entry) in recipe["entries"].as_object().unwrap() {
                let path = root.join(name);
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(path, entry["file"].as_str().unwrap()).unwrap();
            }
            manifest["sysroot"] = json!(root);
        }
        "changed-resources" => {
            let first = manifest["resources"]
                .as_object()
                .unwrap()
                .keys()
                .next()
                .unwrap()
                .clone();
            assert_ne!(manifest["resources"][&first]["sha256"], recipe["sha256"]);
            manifest["resources"][first]["sha256"] = recipe["sha256"].clone();
        }
        "invalid-roles" => {
            roles = PathBuf::from(
                recipe["roles_path"]
                    .as_str()
                    .unwrap()
                    .replace("$case", output.to_str().unwrap()),
            );
            fs::write(&roles, recipe["invalid_roles_utf8"].as_str().unwrap()).unwrap();
        }
        _ => unreachable!(),
    }
    prepared::validate(&manifest).unwrap();
    let manifest_path = output.join("prepared.json");
    fs::write(&manifest_path, format!("{manifest}\n")).unwrap();
    let collected = output.join("collection");
    let streams = output.join("invocation");
    fs::create_dir(&streams).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_arboresce-verify"));
    command
        .args([
            mini.source.as_os_str(),
            roles.as_os_str(),
            mini.source.join("tests.json").as_os_str(),
            manifest_path.as_os_str(),
            collected.as_os_str(),
        ])
        .current_dir(&mini.source)
        .env(measurement::ENV, &mini.outer);
    if std::env::var_os(measurement::ENV).is_none() {
        command.env(
            "LLVM_PROFILE_FILE",
            manifest["outer"]["environment"]["LLVM_PROFILE_FILE"]
                .as_str()
                .unwrap(),
        );
    }
    let local = if std::env::var_os(measurement::ENV).is_none() {
        Some(measurement::Request::open(&mini.outer).unwrap())
    } else {
        None
    };
    let actual = collection_native::capture(&mut command, &streams, None, local.as_ref()).unwrap();
    let trace = fs::read_to_string(streams.join("trace")).unwrap();
    let observation: Value = serde_json::from_str(
        trace
            .lines()
            .find_map(|l| l.strip_prefix("xtask observation "))
            .unwrap(),
    )
    .unwrap();
    let pid = observation["pid"].as_u64().unwrap();
    let launch_bytes = fs::read(mini.outer.join("launches").join(format!("{pid}.json"))).unwrap();
    let launch: Value = serde_json::from_slice(&launch_bytes).unwrap();
    let exit: Value = serde_json::from_slice(
        &fs::read(mini.outer.join("exits").join(format!("{pid}.json"))).unwrap(),
    )
    .unwrap();
    assert_eq!(launch["pid"], pid);
    assert_eq!(exit["pid"], pid);
    assert_eq!(exit["exit"], json!(actual.status.code()));
    assert!(exit["signal"].is_null());
    assert_eq!(exit["launch_sha256"], sha256(&launch_bytes));
    let mut commands = fs::read_dir(collected.join("commands"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    commands.sort();
    for directory in &commands {
        let completion: Value =
            serde_json::from_slice(&fs::read(directory.join("completion.json")).unwrap()).unwrap();
        assert_eq!(completion["exit"], 0);
        assert!(completion["signal"].is_null());
        assert!(completion["error"].is_null());
    }
    let stage = match commands.len() {
        0 => "observe-tools",
        1 => "observe-runtime",
        6 => "compare-prepared",
        7 => "inventory",
        _ => panic!("unexpected preflight command inventory"),
    };
    if !commands.is_empty() {
        let first: Value =
            serde_json::from_slice(&fs::read(commands[0].join("launch-command.json")).unwrap())
                .unwrap();
        assert_eq!(
            first["arguments"],
            json!([b"--print".as_slice(), b"sysroot".as_slice()])
        );
        assert_eq!(
            fs::read_to_string(commands[0].join("stdout"))
                .unwrap()
                .trim(),
            mini.manifest["sysroot"].as_str().unwrap()
        );
    }
    if commands.len() == 7 {
        let last: Value =
            serde_json::from_slice(&fs::read(commands[6].join("launch-command.json")).unwrap())
                .unwrap();
        assert_eq!(
            last["arguments"],
            json!(
                arboresce_xtask::Action::Architecture
                    .cargo_args()
                    .iter()
                    .map(|a| a.as_bytes())
                    .collect::<Vec<_>>()
            )
        );
        assert!(collected.join("prepared-before.json").is_file());
    }
    let after = prepared::tree(&mini.owner, limits, deadline).unwrap();
    let outer_after = prepared::tree(&mini.outer, limits, deadline).unwrap();
    let old_bytes_unchanged = before
        .as_object()
        .unwrap()
        .iter()
        .all(|(name, row)| after[name] == *row)
        && outer_before
            .as_object()
            .unwrap()
            .iter()
            .all(|(name, row)| outer_after[name] == *row)
        && mini.unchanged_source().unwrap();
    let observed = json!({"exit":actual.status.code(),"stdout":String::from_utf8_lossy(&actual.stdout),"stderr":String::from_utf8_lossy(&actual.stderr),"stage":stage,"tool_commands":commands.len(),"no_later_launches":!collected.join("context").exists()&&!collected.join("normal-target").exists()&&!collected.join("result.json").exists(),"baseline_source_and_evidence_unchanged":old_bytes_unchanged});
    fs::write(output.join("baseline-before.json"), format!("{before}\n")).unwrap();
    fs::write(output.join("baseline-after.json"), format!("{after}\n")).unwrap();
    fs::write(output.join("observed.json"), format!("{observed}\n")).unwrap();
    assert_eq!(
        observed,
        case["expected"],
        "{}; retained {}",
        case["id"],
        output.display()
    );
}
