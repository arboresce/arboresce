//! Controlled test-only Cargo peer. It cannot qualify actual Cargo execution.

use std::process::ExitCode;

use arboresce_xtask::emit;

fn collector_probe() -> std::io::Result<serde_json::Value> {
    use arboresce_xtask::{collection_prepared as prepared, measurement, process};
    use serde_json::{Value, json};
    use std::{
        fs, io,
        os::unix::process::ExitStatusExt,
        path::Path,
        process::Command,
        time::{Duration, Instant},
    };
    let input: Value = serde_json::from_slice(&fs::read(
        std::env::var_os("ARBORESCE_COLLECTOR_PROBE").unwrap_or_default(),
    )?)?;
    let case = &input["case"];
    let output = Path::new(input["output"].as_str().unwrap());
    let ambient = prepared::environment().map_err(io::Error::other)?;
    let a = std::path::PathBuf::from(std::env::var_os(measurement::ENV).unwrap_or_default());
    if case["action"] == "outer-negative" {
        let deadline = Instant::now() + Duration::from_secs(2);
        while !output.join("ready").exists() {
            if Instant::now() >= deadline {
                return Err(io::Error::other("outer probe preparation deadline"));
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        let mut declaration = json!({"outer":{"context":a,"environment":ambient}});
        if case["mode"] == "undeclared" {
            declaration["outer"] = Value::Null;
        }
        if case["mode"] == "mismatch" {
            declaration["outer"]["environment"]["LLVM_PROFILE_FILE"] =
                "/different/outer-%p-%12m.profraw".into();
        }
        let observed = match prepared::outer(&declaration) {
            Ok(()) => json!({"value":true}),
            Err(error) => json!({"error":error.to_string()}),
        };
        return Ok(
            json!({"observed":observed,"proof":{"ambient_before":ambient,"ambient_after":prepared::environment().map_err(io::Error::other)?,"pid":std::process::id(),"declaration":declaration}}),
        );
    }
    let limits = prepared::Limits {
        files: 100,
        per_file: 65536,
        total: 1 << 20,
        depth: 1,
    };
    let before = prepared::tree(&a, limits, Instant::now() + Duration::from_secs(30))
        .map_err(io::Error::other)?;
    let b = Path::new(input["B"].as_str().unwrap());
    let request = measurement::Request::open(b)?;
    let recipe = &case["recipe"];
    let mut command = Command::new(if case["mode"] == "output-limit" {
        input["peer"].as_str().unwrap()
    } else {
        recipe["program"].as_str().unwrap()
    });
    command.current_dir(output);
    for arg in recipe["arguments"].as_array().unwrap() {
        command.arg(arg.as_str().unwrap());
    }
    let raw = Path::new(input["raw"].as_str().unwrap());
    command.env("LLVM_PROFILE_FILE", raw.join("arboresce-%p-%12m.profraw"));
    if case["mode"] == "output-limit" {
        let path = output.join("oversized-native-input");
        fs::write(&path, vec![b'x'; 8388609])?;
        command
            .env("ARBORESCE_XTASK_TEST_PEER", "metadata")
            .env("ARBORESCE_XTASK_TEST_METADATA", path);
    }
    let stdout = output.join("child-stdout");
    let stderr = output.join("child-stderr");
    command
        .stdout(std::process::Stdio::from(fs::File::create_new(&stdout)?))
        .stderr(std::process::Stdio::from(fs::File::create_new(&stderr)?));
    let started = Instant::now();
    let mut trace = fs::File::create_new(output.join("child-trace"))?;
    let mut child = measurement::spawn_with(&mut command, &request)?;
    let result = process::observe(
        &mut child,
        &stdout,
        &stderr,
        process::Observation {
            budget: Duration::from_millis(recipe["budget_ms"].as_u64().unwrap()),
            poll: Duration::from_millis(recipe["poll_ms"].as_u64().unwrap()),
            initial_delay: Duration::from_millis(
                recipe["initial_observation_delay_ms"].as_u64().unwrap(),
            ),
            request: Some(request),
        },
        started,
        &mut trace,
    );
    let status = child
        .try_wait()?
        .ok_or_else(|| io::Error::other("collector probe child still running"))?;
    let after = prepared::tree(&a, limits, Instant::now() + Duration::from_secs(30))
        .map_err(io::Error::other)?;
    let entries = |root: &Path, name: &str| -> io::Result<Vec<Value>> {
        fs::read_dir(root.join(name))?
            .map(|entry| {
                serde_json::from_slice(&fs::read(entry?.path())?).map_err(io::Error::other)
            })
            .collect()
    };
    let launches = entries(b, "launches")?;
    let exits = entries(b, "exits")?;
    let observed = json!({"a_launches":entries(&a,"launches")?.len(),"a_exits":entries(&a,"exits")?.len(),"b_launches":launches.len(),"b_exits":exits.len(),"ambient_context_unchanged":ambient==prepared::environment().map_err(io::Error::other)?,"same_actual_pid":launches[0]["pid"]==child.id()&&exits[0]["pid"]==child.id(),"child_reaped":true,"operation_ok":result.is_ok(),"error_kind":result.as_ref().err().map(|error|format!("{:?}",error.kind())),"exit":status.code(),"signal":status.signal(),"stdout_size":fs::metadata(&stdout)?.len(),"stderr_size":fs::metadata(&stderr)?.len(),"invocations":launches.len(),"A_bytes_unchanged":before==after});
    let profiles = prepared::tree(
        raw,
        prepared::PROFILES,
        Instant::now() + Duration::from_secs(30),
    )
    .map_err(io::Error::other)?;
    Ok(
        json!({"observed":observed,"proof":{"pid":std::process::id(),"actual_child_pid":child.id(),"launches":launches,"exits":exits,"A_before":before,"A_after":after,"ambient":ambient,"raw":profiles,"output":output}}),
    )
}

fn publication_probe() -> std::io::Result<serde_json::Value> {
    use arboresce_xtask::{collection_prepared as prepared, measurement};
    use serde_json::{Value, json};
    use std::{
        fs, io,
        os::unix::fs::MetadataExt,
        path::Path,
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    };
    let input: Value = serde_json::from_slice(&fs::read(
        std::env::var_os("ARBORESCE_PUBLICATION_INPUT").unwrap_or_default(),
    )?)?;
    let root = Path::new(input["output"].as_str().unwrap());
    let recipe = &input["recipe"];
    let handshake = Duration::from_millis(
        input["profile"]["parent_handshake_budget_ms"]
            .as_u64()
            .unwrap(),
    );
    let ready_deadline = Instant::now() + handshake;
    while !root.join("release").exists() {
        if Instant::now() >= ready_deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "publication preparation missing",
            ));
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    let context = std::env::var_os(measurement::ENV).unwrap_or_default();
    let context = Path::new(&context);
    let ambient = prepared::environment().map_err(io::Error::other)?;
    let declaration = json!({"outer":{"context":context,"environment":ambient}});
    let started = Instant::now();
    let deadline = started + Duration::from_millis(recipe["caller_budget_ms"].as_u64().unwrap());
    let entered_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos();
    fs::write(
        root.join("entered.json"),
        format!(
            "{}\n",
            json!({"pid":std::process::id(),"unix_ns":entered_ns})
        ),
    )?;
    let mut selected = None;
    let actual = if recipe["selection_observer"] == false {
        prepared::outer_until(&declaration, deadline)
    } else {
        prepared::outer_until_with(&declaration, deadline, || {
            let path = context
                .join("launches")
                .join(format!("{}.json", std::process::id()));
            let metadata = fs::symlink_metadata(&path)?;
            let at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(io::Error::other)?
                .as_nanos();
            let record = json!({"pid":std::process::id(),"unix_ns":at,"elapsed_ns":started.elapsed().as_nanos(),"device":metadata.dev(),"inode":metadata.ino(),"bytes":metadata.len()});
            fs::write(root.join("selected.json"), format!("{record}\n"))?;
            selected = Some(record);
            if recipe["observer_behavior"] == "error" {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "controlled observer failure",
                ));
            }
            if recipe["observer_behavior"] == "delay" {
                std::thread::sleep(Duration::from_millis(
                    recipe["observer_delay_ms"].as_u64().unwrap(),
                ));
            }
            if recipe["parent_wait_for"] == "selected" {
                let handoff_deadline = Instant::now() + handshake;
                while !root.join("continue").exists() {
                    if Instant::now() >= handoff_deadline {
                        return Err(io::Error::new(
                            io::ErrorKind::TimedOut,
                            "publication mutation missing",
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
            Ok(())
        })
    };
    let completed_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos();
    Ok(
        json!({"pid":std::process::id(),"observed":{"accepted":actual.is_ok(),"error":actual.err().map(|e|e.to_string())},"proof":{"entered_unix_ns":entered_ns,"completed_unix_ns":completed_ns,"elapsed_ns":started.elapsed().as_nanos(),"selected":selected,"ambient_before":ambient,"ambient_after":prepared::environment().map_err(io::Error::other)?}}),
    )
}

fn main() -> ExitCode {
    let mode = std::env::var("ARBORESCE_XTASK_TEST_PEER").unwrap_or_default();
    let (stdout, stderr, exit) = match mode.as_str() {
        "publication-probe" => {
            return match publication_probe() {
                Ok(value) => ExitCode::from(
                    if emit(
                        &mut std::io::stdout().lock(),
                        format!("{value}\n").as_bytes(),
                    ) {
                        0
                    } else {
                        1
                    },
                ),
                Err(error) => {
                    emit(
                        &mut std::io::stderr().lock(),
                        format!("publication probe failed: {error}\n").as_bytes(),
                    );
                    ExitCode::from(1)
                }
            };
        }
        "service-environment" => {
            use std::os::unix::ffi::OsStrExt;
            let mut keys: Vec<_> = std::env::vars_os()
                .filter(|(key, _)| key.as_bytes().starts_with(b"PG"))
                .map(|(key, _)| key.to_string_lossy().into_owned())
                .collect();
            keys.sort();
            let value = serde_json::json!({"selected":std::env::var(arboresce_xtask::collection_services::SELECTOR).ok(),"pg_keys":keys});
            return ExitCode::from(
                if emit(
                    &mut std::io::stdout().lock(),
                    format!("{value}\n").as_bytes(),
                ) {
                    0
                } else {
                    1
                },
            );
        }
        "trace-streams" => {
            let result = (|| -> std::io::Result<bool> {
                let stdout =
                    std::fs::read(std::env::var_os("ARBORESCE_TRACE_STDOUT").unwrap_or_default())?;
                let stderr =
                    std::fs::read(std::env::var_os("ARBORESCE_TRACE_STDERR").unwrap_or_default())?;
                Ok(emit(&mut std::io::stdout().lock(), &stdout)
                    && emit(&mut std::io::stderr().lock(), &stderr))
            })();
            return ExitCode::from(if matches!(result, Ok(true)) { 0 } else { 1 });
        }
        "collector-probe" => {
            return match collector_probe() {
                Ok(value) => ExitCode::from(
                    if emit(
                        &mut std::io::stdout().lock(),
                        format!("{value}\n").as_bytes(),
                    ) {
                        0
                    } else {
                        1
                    },
                ),
                Err(error) => {
                    emit(
                        &mut std::io::stderr().lock(),
                        format!("collector probe failed: {error}\n").as_bytes(),
                    );
                    ExitCode::from(1)
                }
            };
        }
        "pass" => ("child stdout\n", "child stderr\n", 0),
        "fail" => ("child stdout\n", "child stderr\n", 101),
        "metadata" => {
            let path = std::env::var_os("ARBORESCE_XTASK_TEST_METADATA").unwrap_or_default();
            let bytes = std::fs::read(path).unwrap_or_default();
            let complete = emit(&mut std::io::stdout().lock(), &bytes);
            return ExitCode::from(if complete { 0 } else { 1 });
        }
        "measured-status"
        | "measured-selected-status"
        | "measured-output"
        | "measured-missing-output"
        | "measured-broken-trace" => {
            let program = std::env::var_os("ARBORESCE_NATIVE_PROGRAM").unwrap_or_default();
            let mut command = if mode == "measured-selected-status" {
                match arboresce_xtask::measurement::command(program, None) {
                    Ok(command) => command,
                    Err(_) => {
                        emit(
                            &mut std::io::stderr().lock(),
                            b"native measurement failed\n",
                        );
                        return ExitCode::from(1);
                    }
                }
            } else {
                std::process::Command::new(program)
            };
            let result = if mode == "measured-output" {
                command.arg("measurement child");
                arboresce_xtask::process::output(&mut command).map(|output| {
                    let complete = emit(&mut std::io::stdout().lock(), &output.stdout)
                        && emit(&mut std::io::stderr().lock(), &output.stderr);
                    if complete {
                        output.status.code().unwrap_or(1)
                    } else {
                        1
                    }
                })
            } else if matches!(
                mode.as_str(),
                "measured-missing-output" | "measured-broken-trace"
            ) {
                (|| -> std::io::Result<i32> {
                    let root = std::path::PathBuf::from(
                        std::env::var_os(arboresce_xtask::measurement::ENV).unwrap_or_default(),
                    );
                    let stdout = root.join("observer-stdout");
                    let stderr = root.join("observer-stderr");
                    command
                        .stdout(std::process::Stdio::from(std::fs::File::create_new(
                            &stdout,
                        )?))
                        .stderr(std::process::Stdio::from(std::fs::File::create_new(
                            &stderr,
                        )?));
                    let started = std::time::Instant::now();
                    let mut child = arboresce_xtask::measurement::spawn(&mut command)?;
                    if mode == "measured-missing-output" {
                        std::fs::remove_file(&stdout)?;
                    }
                    let (reader, mut writer) = std::io::pipe()?;
                    drop(reader);
                    let result = if mode == "measured-broken-trace" {
                        arboresce_xtask::process::observe(
                            &mut child,
                            &stdout,
                            &stderr,
                            arboresce_xtask::process::Observation::default(),
                            started,
                            &mut writer,
                        )
                    } else {
                        arboresce_xtask::process::observe(
                            &mut child,
                            &stdout,
                            &stderr,
                            arboresce_xtask::process::Observation::default(),
                            started,
                            &mut Vec::new(),
                        )
                    };
                    result.map(|output| output.status.code().unwrap_or(1))
                })()
            } else {
                arboresce_xtask::measurement::spawn(&mut command).and_then(|mut child| {
                    arboresce_xtask::process::wait(&mut child, std::time::Duration::from_secs(30))
                        .map(|status| status.code().unwrap_or(1))
                })
            };
            return ExitCode::from(match result {
                Ok(code) => u8::try_from(code).unwrap_or(1),
                Err(_) => {
                    emit(
                        &mut std::io::stderr().lock(),
                        b"native measurement failed\n",
                    );
                    1
                }
            });
        }
        "metadata-invalid" => ("not-json\n", "", 0),
        "metadata-warning" => ("not-json\n", "child stderr\n", 0),
        _ => ("", "test peer mode missing\n", 1),
    };
    let complete = emit(&mut std::io::stdout().lock(), stdout.as_bytes())
        && emit(&mut std::io::stderr().lock(), stderr.as_bytes());
    ExitCode::from(if complete { exit } else { 1 })
}
