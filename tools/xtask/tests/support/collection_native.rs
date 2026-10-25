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
    capture_with(
        command,
        output,
        closed,
        launch,
        std::time::Duration::from_secs(300),
        |_| Ok(()),
    )
}

fn capture_with(
    command: &mut Command,
    output: &Path,
    closed: Option<&str>,
    launch: Option<&measurement::Request>,
    budget: std::time::Duration,
    after_record: impl FnOnce(&mut std::process::Child) -> io::Result<()>,
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
        after_record(&mut child)
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
            budget,
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

fn publication_wait(
    path: &Path,
    child: &mut std::process::Child,
    deadline: Instant,
) -> io::Result<()> {
    while !path.exists() {
        if child.try_wait()?.is_some() || Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "publication acknowledgement missing",
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    Ok(())
}

/// Exercise parent failures only after observing this real peer's normal exit.
pub fn parent_preparation_fault(fault: &str, profile: &Value) -> io::Result<Value> {
    use std::time::Duration;

    let mut owner = PreparedRoot::new()?;
    owner.preserve();
    let root = &owner.path;
    let local = context(root, "parent-fault-context")?;
    let request = measurement::Request::open(&local)?;
    let context_path = local.join("context.json");
    let original_context = fs::read(&context_path)?;
    fs::write(root.join("original-context.json"), &original_context)?;
    let ambient = arboresce_xtask::collection_prepared::environment().map_err(io::Error::other)?;
    let ambient_path = std::env::var_os(measurement::ENV).map(PathBuf::from);
    let ambient_context = ambient_path
        .as_ref()
        .map(|path| fs::read(path.join("context.json")))
        .transpose()?;
    let mut command = Command::new(env!("CARGO_BIN_EXE_arboresce-xtask-test-peer"));
    command
        .current_dir(root)
        .env("ARBORESCE_XTASK_TEST_PEER", "pass")
        .env(measurement::ENV, &local);
    let budget = Duration::from_millis(profile["native_observer_budget_ms"].as_u64().unwrap());
    let started = Instant::now();
    let deadline = started + budget;
    let handshake_deadline = deadline.min(
        started + Duration::from_millis(profile["parent_handshake_budget_ms"].as_u64().unwrap()),
    );
    let mut pid = None;
    let mut actual_status = None;
    let mut original_launch = None;
    let mut fault_started = None;
    let result = capture_with(&mut command, root, None, Some(&request), budget, |child| {
        pid = Some(child.id());
        let bytes = fs::read(local.join("launches").join(format!("{}.json", child.id())))?;
        fs::write(root.join("original-launch.json"), &bytes)?;
        original_launch = Some(bytes);
        let status = process::wait(child, deadline.saturating_duration_since(Instant::now()))?;
        actual_status = Some(status);
        if status.code() != Some(0) || status.signal().is_some() {
            return Err(io::Error::other("parent fault peer did not exit normally"));
        }
        fault_started = Some(Instant::now());
        match fault {
            "callback-error" => Err(io::Error::other("controlled parent callback failure")),
            "missing-acknowledgement" => publication_wait(
                &root.join("missing-acknowledgement"),
                child,
                handshake_deadline,
            ),
            "conflicting-local-exit" => {
                fs::write(
                    local.join("exits").join(format!("{}.json", child.id())),
                    b"{}\n",
                )?;
                Err(io::Error::other("controlled conflicting local completion"))
            }
            _ => Err(io::Error::other("unknown parent preparation fault")),
        }
    });
    // Common capture cleanup has finished before these evidence assertions.
    let elapsed = started.elapsed();
    let cleanup_elapsed = fault_started.unwrap().elapsed();
    let error = result.expect_err("parent preparation fault unexpectedly succeeded");
    let pid = pid.unwrap();
    let status = actual_status.unwrap();
    let original_launch = original_launch.unwrap();
    let launch_path = local.join("launches").join(format!("{pid}.json"));
    let exit_bytes = fs::read(local.join("exits").join(format!("{pid}.json"))).unwrap();
    let exit: Value = serde_json::from_slice(&exit_bytes).unwrap();
    let exit_state = if exit["version"] == "native-exit-v1" {
        assert_eq!(exit["pid"], pid);
        assert_eq!(exit["exit"], status.code().unwrap());
        assert!(exit["signal"].is_null());
        assert_eq!(
            exit["launch_sha256"],
            arboresce_xtask::inventory::sha256(&original_launch)
        );
        "native"
    } else {
        assert_eq!(exit_bytes, b"{}\n");
        "conflicting-file"
    };
    let trace = fs::read_to_string(root.join("trace")).unwrap();
    let mut lines = trace.lines();
    let setup: Value = serde_json::from_str(
        lines
            .next()
            .unwrap()
            .strip_prefix("xtask capture setup ")
            .unwrap(),
    )
    .unwrap();
    assert!(lines.next().is_none());
    assert_eq!(setup["pid"], pid);
    assert_eq!(setup["exit"], status.code().unwrap());
    assert!(setup["signal"].is_null());
    let ambient_after =
        arboresce_xtask::collection_prepared::environment().map_err(io::Error::other)?;
    let ambient_context_after = ambient_path
        .as_ref()
        .map(|path| fs::read(path.join("context.json")))
        .transpose()?;
    let observed = json!({
        "child_exit":status.code(),"child_signal":status.signal(),
        "reaped":setup["reaped"],"local_completion":setup["local_completion"],
        "ambient_completion":setup["ambient_completion"],
        "launch_unchanged":fs::read(&launch_path).unwrap()==original_launch,
        "context_unchanged":fs::read(&context_path).unwrap()==original_context,
        "ambient_unchanged":ambient==ambient_after && ambient_context==ambient_context_after,
        "local_exit_state":exit_state,
        "stdout_utf8":fs::read_to_string(root.join("stdout")).unwrap(),
        "stderr_utf8":fs::read_to_string(root.join("stderr")).unwrap(),
        "within_original_budget":elapsed<=budget && cleanup_elapsed<=Duration::from_millis(profile["cleanup_budget_ms"].as_u64().unwrap()),
        "parent_error_kind":format!("{:?}",error.kind()),"setup_error_kind":setup["error_kind"]
    });
    let proof = json!({"fault":fault,"pid":pid,"observed":observed,"setup":setup,
        "elapsed_ns":elapsed.as_nanos(),"post_fault_ns":cleanup_elapsed.as_nanos(),
        "ambient_before":ambient,"ambient_after":ambient_after});
    fs::write(root.join("parent-fault-proof.json"), format!("{proof}\n"))?;
    println!("xtask retained parent fault {}", root.display());
    Ok(observed)
}

fn publication_bytes(kind: &str, original: &[u8]) -> io::Result<Vec<u8>> {
    if kind == "empty" {
        return Ok(Vec::new());
    }
    if kind == "partial" {
        return Ok(original[..original.len() / 2].to_vec());
    }
    if kind == "without-lf" {
        return Ok(original[..original.len() - 1].to_vec());
    }
    if kind == "malformed" {
        return Ok(b"{\n".to_vec());
    }
    if kind == "invalid-utf8" {
        return Ok(vec![255, b'\n']);
    }
    if matches!(kind, "exact-64k" | "over-64k") {
        let length = if kind == "exact-64k" { 65536 } else { 65537 };
        let mut bytes = original[..original.len() - 1].to_vec();
        assert!(bytes.len() < length);
        bytes.resize(length - 1, b' ');
        bytes.push(b'\n');
        assert_eq!(bytes.len(), length);
        assert_eq!(
            serde_json::from_slice::<Value>(&bytes)?,
            serde_json::from_slice::<Value>(original)?
        );
        return Ok(bytes);
    }
    if kind == "trailing-value" {
        let mut bytes = original.to_vec();
        bytes.extend_from_slice(b"{}\n");
        return Ok(bytes);
    }
    if kind == "complete" {
        return Ok(original.to_vec());
    }
    let mut value: Value = serde_json::from_slice(original)?;
    match kind {
        "wrong-pid" => value["pid"] = json!(value["pid"].as_u64().unwrap() + 1),
        "wrong-program" => value["program"] = json!(b"/different/program".as_slice()),
        "wrong-argv" => value["arguments"] = json!([b"different argument".as_slice()]),
        "wrong-cwd" => value["cwd"] = json!(b"/different/cwd".as_slice()),
        "wrong-context-hash" | "wrong-run" | "wrong-source" | "wrong-image-hash" => {
            let field = match kind {
                "wrong-context-hash" => "context_sha256",
                "wrong-run" => "run_id",
                "wrong-source" => "source_sha256",
                _ => "executable_sha256",
            };
            let mut bytes = value[field].as_str().unwrap().as_bytes().to_vec();
            bytes[0] = if bytes[0] == b'0' { b'1' } else { b'0' };
            value[field] = String::from_utf8(bytes).unwrap().into();
        }
        _ => return Err(io::Error::other("unknown publication input")),
    }
    Ok(format!("{value}\n").into_bytes())
}

/// Controlled owned publication mutations preserve the real parent receipt.
pub fn publication(case: &Value, profile: &Value) -> io::Result<Value> {
    use arboresce_xtask::inventory::sha256;
    use std::{
        os::unix::fs::{MetadataExt, symlink},
        time::{Duration, SystemTime, UNIX_EPOCH},
    };
    let owner = PreparedRoot::new()?;
    let root = owner.path.clone();
    std::mem::forget(owner);
    let b = context(&root, "publication-context")?;
    let request = measurement::Request::open(&b)?;
    let context_path = b.join("context.json");
    let context_original = fs::read(&context_path)?;
    fs::write(root.join("original-context.json"), &context_original)?;
    let recipe = &case["recipe"];
    // Expected answers never enter the peer's input or determine its result.
    let input = json!({"recipe":recipe,"profile":profile,"output":root});
    let input_path = root.join("publication-input.json");
    fs::write(&input_path, format!("{input}\n"))?;
    let ambient = arboresce_xtask::collection_prepared::environment().map_err(io::Error::other)?;
    let mut command = Command::new(env!("CARGO_BIN_EXE_arboresce-xtask-test-peer"));
    command
        .current_dir(&root)
        .env("ARBORESCE_XTASK_TEST_PEER", "publication-probe")
        .env("ARBORESCE_PUBLICATION_INPUT", &input_path)
        .env(measurement::ENV, &b);
    if std::env::var_os("LLVM_PROFILE_FILE").is_none() {
        command.env("LLVM_PROFILE_FILE", root.join("probe-%p-%12m.profraw"));
    }
    let mut proof = json!({"case":case["id"],"output":root});
    let started = Instant::now();
    let budget = Duration::from_millis(profile["native_observer_budget_ms"].as_u64().unwrap());
    let native_deadline = started + budget;
    let handshake_deadline = native_deadline.min(
        started + Duration::from_millis(profile["parent_handshake_budget_ms"].as_u64().unwrap()),
    );
    let result = capture_with(&mut command, &root, None, Some(&request), budget, |child| {
        let receipt = b.join("launches").join(format!("{}.json", child.id()));
        let original = fs::read(&receipt)?;
        let saved = root.join("original-launch.json");
        fs::rename(&receipt, &saved)?;
        let setup = (|| {
            let initial = recipe["initial"].as_str().unwrap();
            match initial {
                "missing" => {}
                "symlink" => symlink(&saved, &receipt)?,
                "directory" => fs::create_dir(&receipt)?,
                "fifo" => {
                    let mut mkfifo = measurement::command("mkfifo", Some(&root))?;
                    mkfifo.arg(&receipt);
                    let (_, actual) = observe_identity(&mut mkfifo)?;
                    if !actual.status.success() || !actual.stderr.is_empty() {
                        return Err(io::Error::other("publication FIFO preparation failed"));
                    }
                }
                _ => fs::write(&receipt, publication_bytes(initial, &original)?)?,
            }
            if receipt.is_file() && !fs::symlink_metadata(&receipt)?.is_symlink() {
                fs::copy(&receipt, root.join("initial-receipt.bytes"))?;
            }
            if let Ok(metadata) = fs::symlink_metadata(&receipt) {
                proof["initial_path"] =
                    json!({"device":metadata.dev(),"inode":metadata.ino(),"bytes":metadata.len()});
            }
            fs::write(root.join("release"), b"ready")?;
            publication_wait(&root.join("entered.json"), child, handshake_deadline)?;
            if recipe["parent_wait_for"] == "selected" {
                publication_wait(&root.join("selected.json"), child, handshake_deadline)?;
            }
            let action = recipe["action"].as_str().unwrap();
            let delay_started = Instant::now();
            if action == "repair-after-child-exit" {
                process::wait(
                    child,
                    native_deadline.saturating_duration_since(Instant::now()),
                )?;
            } else if action != "none" {
                std::thread::sleep(Duration::from_millis(
                    recipe["publication_delay_ms"].as_u64().unwrap(),
                ));
                assert!(
                    delay_started.elapsed().as_millis()
                        >= recipe["publication_delay_ms"].as_u64().unwrap() as u128
                );
            }
            if Instant::now() >= native_deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "publication preparation late",
                ));
            }
            proof["parent_delay_ns"] = json!(delay_started.elapsed().as_nanos());
            let action_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(io::Error::other)?
                .as_nanos();
            match action {
                "none" => {}
                "publish" | "repair-after-child-exit" => fs::write(&receipt, &original)?,
                "append" => fs::OpenOptions::new()
                    .append(true)
                    .open(&receipt)?
                    .write_all(&original[original.len() / 2..])?,
                "append-lf" => fs::OpenOptions::new()
                    .append(true)
                    .open(&receipt)?
                    .write_all(b"\n")?,
                "context-and-append" => {
                    let mut changed: Value = serde_json::from_slice(&context_original)?;
                    changed["run_id"] = "b".repeat(32).into();
                    let changed = format!("{changed}\n");
                    fs::write(&context_path, &changed)?;
                    fs::write(root.join("mutated-context.json"), &changed)?;
                    fs::OpenOptions::new()
                        .append(true)
                        .open(&receipt)?
                        .write_all(&original[original.len() / 2..])?;
                }
                "replace-complete" | "unlink" | "replace-symlink" | "replace-directory" => {
                    // Complete the selected old inode too. Reading that inode
                    // alone must not hide a changed or absent selected path.
                    fs::OpenOptions::new()
                        .append(true)
                        .open(&receipt)?
                        .write_all(&original[original.len() / 2..])?;
                    let selected_old = root.join("selected-old-complete.json");
                    let old = fs::symlink_metadata(&receipt)?;
                    fs::rename(&receipt, &selected_old)?;
                    match action {
                        "replace-complete" => fs::write(&receipt, &original)?,
                        "replace-symlink" => symlink(&selected_old, &receipt)?,
                        "replace-directory" => fs::create_dir(&receipt)?,
                        _ => {}
                    }
                    proof["selected_old_inode"] = json!({"device":old.dev(),"inode":old.ino(),"bytes":fs::read(&selected_old)?.len()});
                }
                _ => return Err(io::Error::other("unknown publication action")),
            }
            proof["action"] = json!({"name":action,"unix_ns":action_at});
            if recipe["parent_wait_for"] == "selected" {
                fs::write(root.join("continue"), b"ready")?;
            }
            let status = process::wait(
                child,
                native_deadline.saturating_duration_since(Instant::now()),
            )?;
            proof["actual_child_pid"] = child.id().into();
            proof["actual_status"] = json!({"exit":status.code(),"signal":status.signal()});
            Ok(())
        })();
        if setup.is_err() {
            // Restore only after the real child is no longer running, including
            // a failed parent preparation; common capture retains every cause.
            process::terminate(child)?;
        }
        fs::write(&context_path, &context_original)?;
        if let Ok(metadata) = fs::symlink_metadata(&receipt) {
            if metadata.is_dir() {
                fs::rename(&receipt, root.join("retained-receipt-directory"))?;
            } else {
                fs::rename(&receipt, root.join("retained-mutated-receipt"))?;
            }
        }
        fs::write(&receipt, &original)?;
        proof["original_launch_sha256"] = sha256(&original).into();
        proof["restored_launch_sha256"] = sha256(&fs::read(&receipt)?).into();
        proof["context_restored_after_child_completion"] =
            (fs::read(&context_path)? == context_original).into();
        setup
    })?;
    if !result.status.success() || !result.stderr.is_empty() {
        return Err(io::Error::other(format!(
            "publication peer failed; retained {}",
            root.display()
        )));
    }
    let actual: Value = serde_json::from_slice(&result.stdout)?;
    assert_eq!(actual["pid"], proof["actual_child_pid"]);
    assert_eq!(
        actual["proof"]["ambient_before"],
        actual["proof"]["ambient_after"]
    );
    if recipe["parent_wait_for"] == "selected" {
        let selected = &actual["proof"]["selected"];
        assert_eq!(selected["pid"], actual["pid"]);
        assert_eq!(selected["device"], proof["initial_path"]["device"]);
        assert_eq!(selected["inode"], proof["initial_path"]["inode"]);
        assert!(
            selected["unix_ns"].as_u64().unwrap() <= proof["action"]["unix_ns"].as_u64().unwrap()
        );
    }
    if recipe["action"] == "publish" && recipe["publication_delay_ms"].as_u64().unwrap() > 1000 {
        assert!(actual["proof"]["elapsed_ns"].as_u64().unwrap() >= 1_000_000_000);
        assert!(
            actual["proof"]["elapsed_ns"].as_u64().unwrap()
                < recipe["caller_budget_ms"].as_u64().unwrap() * 1_000_000
        );
    }
    assert_eq!(
        ambient,
        arboresce_xtask::collection_prepared::environment().map_err(io::Error::other)?
    );
    let launch: Value = serde_json::from_slice(&fs::read(
        b.join("launches")
            .join(format!("{}.json", actual["pid"].as_u64().unwrap())),
    )?)?;
    let exit: Value = serde_json::from_slice(&fs::read(
        b.join("exits")
            .join(format!("{}.json", actual["pid"].as_u64().unwrap())),
    )?)?;
    assert_eq!(launch["pid"], actual["pid"]);
    assert_eq!(exit["pid"], actual["pid"]);
    assert_eq!(exit["exit"], result.status.code().unwrap());
    assert_eq!(exit["launch_sha256"], proof["original_launch_sha256"]);
    assert_eq!(
        proof["original_launch_sha256"],
        proof["restored_launch_sha256"]
    );
    proof["peer"] = actual.clone();
    proof["ambient_unchanged"] = true.into();
    proof["launch"] = launch;
    proof["exit"] = exit;
    fs::write(root.join("publication-proof.json"), format!("{proof}\n"))?;
    println!("xtask retained publication {}", root.display());
    let mut observed = actual["observed"].clone();
    observed["actual_peer_exit"] = json!(result.status.code());
    observed["actual_peer_signal"] = json!(result.status.signal());
    Ok(observed)
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
