//! Actual native collector probes with retained receipts and deterministic streams.

use super::{PreparedRoot, observe_identity};
use arboresce_xtask::{measurement, process};
use serde_json::{Value, json};
use std::{
    ffi::OsString,
    fs, io,
    io::Write,
    os::unix::{ffi::OsStringExt, process::ExitStatusExt},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    time::Instant,
};

pub fn context(root: &Path, name: &str) -> io::Result<PathBuf> {
    let path = root.join(name);
    fs::create_dir(&path)?;
    fs::create_dir(path.join("launches"))?;
    fs::create_dir(path.join("exits"))?;
    fs::write(
        path.join("context.json"),
        format!(
            "{}\n",
            json!({"version":"native-measurement-v1","run_id":"a".repeat(32),"source_sha256":"1".repeat(64)})
        ),
    )?;
    Ok(path)
}

pub fn cli(case: &Value) -> io::Result<Output> {
    let owner = PreparedRoot::new()?;
    let mut command = Command::new(env!("CARGO_BIN_EXE_arboresce-verify"));
    command.current_dir(&owner.path);
    for argument in case["args"].as_array().unwrap() {
        command.arg(if let Some(text) = argument.as_str() {
            OsString::from(text)
        } else {
            OsString::from_vec(
                argument["bytes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|byte| byte.as_u64().unwrap() as u8)
                    .collect(),
            )
        });
    }
    capture(&mut command, &owner.path, case["closed"].as_str(), None)
}

/// Dropping the pipe reader before spawn makes every child write fail deterministically.
pub fn capture(
    command: &mut Command,
    output: &Path,
    closed: Option<&str>,
    launch: Option<&measurement::Request>,
) -> io::Result<Output> {
    let stdout = output.join("stdout");
    let stderr = output.join("stderr");
    for (name, path) in [("stdout", &stdout), ("stderr", &stderr)] {
        let regular = fs::File::create_new(path)?;
        let stream = if closed == Some(name) {
            let (reader, writer) = io::pipe()?;
            drop(reader);
            Stdio::from(writer)
        } else {
            Stdio::from(regular)
        };
        if name == "stdout" {
            command.stdout(stream);
        } else {
            command.stderr(stream);
        }
    }
    let prepared = launch.map(|request| request.prepare(command)).transpose()?;
    let mut trace = fs::File::create_new(output.join("trace"))?;
    let started = Instant::now();
    let mut child = measurement::spawn(command)?;
    let mutation = command
        .get_envs()
        .find(|(key, _)| *key == "ARBORESCE_COLLECTOR_OUTER_MUTATION")
        .and_then(|(_, value)| value);
    // Recording itself may terminate the child. Include that real interval.
    let setup_started = Instant::now();
    let setup = (|| {
        if let Some(prepared) = prepared {
            prepared.record(&mut child)?;
        }
        // Only explicit probes use this parent-owned mutation gate.
        if let Some(mode) = mutation {
            let root = command
                .get_envs()
                .find(|(key, _)| *key == measurement::ENV)
                .and_then(|(_, value)| value)
                .ok_or_else(|| io::Error::other("collector probe context missing"))?;
            let path = Path::new(root)
                .join("launches")
                .join(format!("{}.json", child.id()));
            if mode == "missing-launch" {
                fs::remove_file(&path)?;
            } else if mode == "wrong-identity" {
                let mut record: Value = serde_json::from_slice(&fs::read(&path)?)?;
                record["executable_sha256"] = "0".repeat(64).into();
                fs::write(&path, format!("{record}\n"))?;
            }
            fs::write(output.join("ready"), b"ready")?;
        }
        Ok::<_, io::Error>(())
    })();
    if let Err(error) = setup {
        let stopped = process::terminate(&mut child);
        let cleanup_ns = setup_started.elapsed().as_nanos();
        // Try both receipts even when either completion itself fails.
        let local = launch
            .map(|request| request.complete(&mut child))
            .transpose();
        let ambient = measurement::complete(&mut child);
        let written = writeln!(
            trace,
            "xtask capture setup {}",
            json!({"pid":child.id(),
            "error_kind":format!("{:?}",error.kind()),"reaped":stopped.is_ok(),"cleanup_ns":cleanup_ns,
            "exit":stopped.as_ref().ok().and_then(|s|s.code()),"signal":stopped.as_ref().ok().and_then(|s|s.signal()),
            "local_completion":local.is_ok(),"ambient_completion":ambient.is_ok()})
        );
        stopped?;
        local?;
        ambient?;
        written?;
        return Err(error);
    }
    process::observe(
        &mut child,
        &stdout,
        &stderr,
        process::Observation {
            budget: std::time::Duration::from_secs(300),
            request: if mutation.is_some() {
                None
            } else {
                launch.cloned()
            },
            ..process::Observation::default()
        },
        started,
        &mut trace,
    )
}

pub fn probe(case: &Value) -> io::Result<Value> {
    let owner = PreparedRoot::new()?;
    let root = owner.path.clone();
    std::mem::forget(owner);
    let a = context(&root, "A")?;
    let b = context(&root, "B")?;
    let raw = root.join("B-raw");
    fs::create_dir(&raw)?;
    let input = json!({"case":case,"B":b,"raw":raw,"output":root,
        "peer":env!("CARGO_BIN_EXE_arboresce-xtask-test-peer")});
    let path = root.join("probe.json");
    fs::write(&path, format!("{input}\n"))?;
    let mut command = Command::new(env!("CARGO_BIN_EXE_arboresce-xtask-test-peer"));
    command
        .current_dir(&root)
        .env("ARBORESCE_XTASK_TEST_PEER", "collector-probe")
        .env("ARBORESCE_COLLECTOR_PROBE", &path)
        .env(measurement::ENV, &a);
    let actual = if case["action"] == "outer-negative" {
        command.env(
            "ARBORESCE_COLLECTOR_OUTER_MUTATION",
            case["mode"].as_str().unwrap(),
        );
        // Even an ordinary noninstrumented test has an explicit outer pattern.
        if std::env::var_os("LLVM_PROFILE_FILE").is_none() {
            command.env("LLVM_PROFILE_FILE", raw.join("outer-%p-%12m.profraw"));
        }
        capture(
            &mut command,
            &root,
            None,
            Some(&measurement::Request::open(&a)?),
        )?
    } else {
        let (_, output) = observe_identity(&mut command)?;
        output
    };
    if !actual.status.success() || !actual.stderr.is_empty() {
        return Err(io::Error::other(format!(
            "native collector probe failed: {}",
            json!({"exit":actual.status.code(),"signal":actual.status.signal(),
                "stdout":actual.stdout,"stderr":actual.stderr,"output":root})
        )));
    }
    let result: Value = serde_json::from_slice(&actual.stdout)?;
    println!("xtask collector probe {result}");
    Ok(result["observed"].clone())
}

pub fn pre_spawn(case: &Value) -> io::Result<Value> {
    let owner = PreparedRoot::new()?;
    let request_root = context(&owner.path, "local")?;
    let request = measurement::Request::open(&request_root)?;
    let mut command = if case["recipe"]["mode"] == "collision" {
        fs::write(owner.path.join("stdout"), b"existing\n")?;
        measurement::command("true", None)?
    } else {
        Command::new(owner.path.join("confirmed-missing-program"))
    };
    command.current_dir(&owner.path);
    let error = capture(&mut command, &owner.path, None, Some(&request))
        .err()
        .ok_or_else(|| io::Error::other("pre-spawn failure unexpectedly succeeded"))?;
    let spawned = fs::read_dir(request_root.join("launches"))?
        .next()
        .is_some();
    let observed = json!({"error_kind":format!("{:?}",error.kind()),"spawned":spawned});
    println!(
        "xtask capture preparation {}",
        json!({"owner":owner.path,"observed":observed})
    );
    Ok(observed)
}

pub fn postspawn_io(case: &Value) -> io::Result<Value> {
    use std::os::unix::fs::PermissionsExt;
    let owner = PreparedRoot::new()?;
    let root = owner.path.clone();
    std::mem::forget(owner);
    let before = arboresce_xtask::collection_prepared::environment().map_err(io::Error::other)?;
    let a = context(&root, "A")?;
    fs::create_dir(root.join("ready"))?;
    let recipe = &case["recipe"];
    let mut lookup = Command::new(recipe["program"].as_str().unwrap());
    lookup.current_dir(&root);
    let mut command = Command::new(measurement::selected_program(&lookup)?);
    command
        .current_dir(&root)
        .args(
            recipe["arguments"]
                .as_array()
                .unwrap()
                .iter()
                .map(|a| a.as_str().unwrap()),
        )
        .env(
            "ARBORESCE_COLLECTOR_OUTER_MUTATION",
            recipe["mutation_mode"].as_str().unwrap(),
        );
    if recipe["omit_command_context"] != true {
        command.env(measurement::ENV, &a);
    }
    let request = measurement::Request::open(&a)?;
    let launches_path = a.join("launches");
    let original = fs::metadata(&launches_path)?.permissions();
    let blocked = recipe["block_local_launches"] == true;
    let mut write_denial_proven = false;
    if blocked {
        fs::set_permissions(&launches_path, fs::Permissions::from_mode(0o555))?;
        write_denial_proven = fs::File::create_new(launches_path.join("denial-proof"))
            .is_err_and(|error| error.kind() == io::ErrorKind::PermissionDenied);
    }
    let returned = if !blocked || write_denial_proven {
        capture(&mut command, &root, None, Some(&request))
    } else {
        Err(io::Error::other(
            "actual local launch write denial not established",
        ))
    };
    // Restore the owned directory immediately, before any fallible observation.
    if blocked {
        fs::set_permissions(&launches_path, original.clone())?;
    }
    let error = returned
        .err()
        .ok_or_else(|| io::Error::other("postspawn gate unexpectedly succeeded"))?;
    if blocked && !write_denial_proven {
        return Err(error);
    }
    let trace = fs::read_to_string(root.join("trace"))?;
    let line = trace
        .lines()
        .find_map(|line| line.strip_prefix("xtask capture setup "))
        .ok_or_else(|| io::Error::other("actual setup cleanup trace missing"))?;
    let actual: Value = serde_json::from_str(line)?;
    let launches = fs::read_dir(a.join("launches"))?.collect::<io::Result<Vec<_>>>()?;
    let exits = fs::read_dir(a.join("exits"))?.collect::<io::Result<Vec<_>>>()?;
    if blocked {
        let observed = json!({"capture_error_kind":format!("{:?}",error.kind()),"setup_error_kind":actual["error_kind"],"signal":actual["signal"],"exit":actual["exit"],"reaped":actual["reaped"],"within_cleanup_budget":actual["cleanup_ns"].as_u64().unwrap()<=recipe["cleanup_budget_ns"].as_u64().unwrap(),"write_denial_proven":write_denial_proven,"permissions_restored":fs::metadata(&launches_path)?.permissions().mode()==original.mode(),"local_launches":launches.len(),"local_exits":exits.len(),"local_completion":actual["local_completion"],"ambient_completion":actual["ambient_completion"],"ambient_environment_unchanged":before==arboresce_xtask::collection_prepared::environment().map_err(io::Error::other)?});
        println!(
            "xtask collector denied launch {}",
            json!({"owner":root,"observed":observed,"actual":actual})
        );
        return Ok(observed);
    }
    if launches.len() != 1 || exits.len() != 1 {
        return Err(io::Error::other("actual setup receipt pair missing"));
    }
    let launch_bytes = fs::read(launches[0].path())?;
    let launch: Value = serde_json::from_slice(&launch_bytes)?;
    let exit: Value = serde_json::from_slice(&fs::read(exits[0].path())?)?;
    if exit["signal"] != actual["signal"] || exit["exit"] != actual["exit"] {
        return Err(io::Error::other("actual cleanup status/receipt mismatch"));
    }
    let observed = json!({"error_kind":format!("{:?}",error.kind()),"signal":actual["signal"],"exit":actual["exit"],
        "reaped":actual["reaped"],"within_cleanup_budget":actual["cleanup_ns"].as_u64().unwrap()<=recipe["cleanup_budget_ns"].as_u64().unwrap(),
        "local_launches":launches.len(),"local_exits":exits.len(),"same_actual_pid":actual["pid"]==launch["pid"]&&actual["pid"]==exit["pid"],
        "matching_launch_hash":exit["launch_sha256"]==arboresce_xtask::inventory::sha256(&launch_bytes),
        "local_completion":actual["local_completion"],"ambient_completion":actual["ambient_completion"],
        "ambient_environment_unchanged":before==arboresce_xtask::collection_prepared::environment().map_err(io::Error::other)?});
    println!(
        "xtask collector setup {}",
        json!({"owner":root,"observed":observed,"actual":actual,"launch":launch,"exit":exit})
    );
    Ok(observed)
}
