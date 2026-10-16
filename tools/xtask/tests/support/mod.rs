use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

pub mod native_trace {
    use super::{PreparedRoot, collection_native};
    use arboresce_xtask::{inventory::sha256, measurement, process};
    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};
    use std::{
        ffi::OsString,
        fs,
        io::{self, Write},
        os::unix::{
            ffi::{OsStrExt, OsStringExt},
            fs::PermissionsExt,
            process::ExitStatusExt,
        },
        path::{Path, PathBuf},
        process::{Command, Stdio},
        time::{Duration, Instant},
    };

    struct Trace {
        file: fs::File,
        writes: u64,
        accepted: u64,
        max_writes: u64,
        max_write_bytes: Option<u64>,
        fail_after: Option<u64>,
        failed_once: bool,
        failure_at: Option<u64>,
        io_failed: bool,
        first_post_error: &'static str,
        flushes: u64,
        flush_error: bool,
        flush_delay: Duration,
        delayed_flushes_returned: u64,
    }

    impl Trace {
        fn fail(&mut self, kind: io::ErrorKind) -> io::Error {
            self.io_failed = true;
            io::Error::new(kind, "controlled trace transport failure")
        }
    }

    impl Write for Trace {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.writes += 1;
            if self.writes > self.max_writes {
                return Err(self.fail(io::ErrorKind::BrokenPipe));
            }
            let mut count = bytes.len();
            if !self.failed_once
                && let Some(boundary) = self.fail_after
            {
                if self.accepted >= boundary {
                    self.failed_once = true;
                    self.failure_at = Some(self.accepted);
                    return Err(self.fail(io::ErrorKind::BrokenPipe));
                }
                count = count.min((boundary - self.accepted) as usize);
            }
            if let Some(limit) = self.max_write_bytes {
                count = count.min(limit as usize);
            }
            let written = match self.file.write(&bytes[..count]) {
                Ok(written) => written,
                Err(error) => {
                    self.io_failed = true;
                    return Err(error);
                }
            };
            if written > 0 && self.io_failed && self.first_post_error == "none" {
                self.first_post_error = if bytes.starts_with(b"xtask cleanup observation ") {
                    "cleanup"
                } else {
                    "other"
                };
            }
            self.accepted += written as u64;
            Ok(written)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flushes += 1;
            if self.flush_error {
                let kind = if self.flushes == 1 {
                    io::ErrorKind::BrokenPipe
                } else {
                    io::ErrorKind::PermissionDenied
                };
                return Err(self.fail(kind));
            }
            if !self.flush_delay.is_zero() {
                std::thread::sleep(self.flush_delay);
                self.delayed_flushes_returned += 1;
            }
            self.file.flush()
        }
    }

    fn input(recipe: &Value) -> Vec<u8> {
        if let Some(bytes) = recipe["bytes"].as_array() {
            bytes
                .iter()
                .map(|byte| byte.as_u64().unwrap() as u8)
                .collect()
        } else {
            vec![
                recipe["repeat_byte"].as_u64().unwrap() as u8;
                recipe["count"].as_u64().unwrap() as usize
            ]
        }
    }

    fn consume(trace: &[u8], position: &mut usize, bytes: &[u8]) -> bool {
        if trace
            .get(*position..)
            .is_some_and(|tail| tail.starts_with(bytes))
        {
            *position += bytes.len();
            true
        } else {
            false
        }
    }

    // Compare the physical frame directly with actual captured bytes, without
    // constructing another multi-million-element JSON value or copying its frame.
    fn complete_frame(trace: &[u8], pid: u32, stdout: &[u8], stderr: &[u8]) -> Option<usize> {
        let prefix = format!(
            "xtask native exited pid={pid} exit=Some(0)\nxtask native pid={pid} exit=Some(0)\nxtask observation {{\"exit\":0,\"pid\":{pid},\"stderr\":["
        );
        let mut position = 0;
        if !consume(trace, &mut position, prefix.as_bytes()) {
            return None;
        }
        let decimal: Vec<_> = (0..=255_u8).map(|byte| byte.to_string()).collect();
        for (stream, suffix) in [
            (stderr, b"],\"stdout\":[".as_slice()),
            (stdout, b"],\"within_budget\":true}\n".as_slice()),
        ] {
            for (index, byte) in stream.iter().enumerate() {
                if (index > 0 && !consume(trace, &mut position, b","))
                    || !consume(trace, &mut position, decimal[*byte as usize].as_bytes())
                {
                    return None;
                }
            }
            if !consume(trace, &mut position, suffix) {
                return None;
            }
        }
        Some(position)
    }

    fn normalized_trace(trace: &[u8], pid: u32) -> Value {
        let prefix = |pid| {
            format!(
                "xtask native exited pid={pid} exit=Some(0)\nxtask native pid={pid} exit=Some(0)\nxtask observation {{\"exit\":0,\"pid\":{pid},"
            )
        };
        let actual = prefix(pid);
        assert!(trace.starts_with(actual.as_bytes()));
        let normalized = prefix(0);
        let mut digest = Sha256::new();
        digest.update(normalized.as_bytes());
        digest.update(&trace[actual.len()..]);
        json!({"normalized_pid":0,"bytes":normalized.len()+trace.len()-actual.len(),
            "sha256":format!("{:x}",digest.finalize())})
    }

    fn retained() -> io::Result<PathBuf> {
        let owner = PreparedRoot::new()?;
        let retained = owner.path.with_extension("trace-evidence");
        fs::rename(&owner.path, &retained)?;
        println!("xtask retained native trace {}", retained.display());
        Ok(retained)
    }

    struct RestorePermissions {
        path: PathBuf,
        original: fs::Permissions,
        active: bool,
    }

    impl RestorePermissions {
        fn restore(&mut self) -> io::Result<()> {
            if self.active {
                fs::set_permissions(&self.path, self.original.clone())?;
                self.active = false;
            }
            Ok(())
        }
    }

    impl Drop for RestorePermissions {
        fn drop(&mut self) {
            let _ = self.restore();
        }
    }

    fn wait_prepared(child: &mut std::process::Child) -> io::Result<std::process::ExitStatus> {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if let Some(status) = child.try_wait()? {
                return Ok(status);
            }
            if Instant::now() >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "trace peer preparation late",
                ));
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// The assertion receives the actual error and retained physical diagnostic.
    pub fn preparation(recipe: &Value) -> io::Result<(PathBuf, io::Error)> {
        let retained = retained()?;
        let fallback = collection_native::context(&retained, "fallback")?;
        let request = measurement::Request::open(&fallback)?;
        let error = observe_at(recipe, &retained, Some(&request))
            .err()
            .ok_or_else(|| io::Error::other("preparation fault unexpectedly succeeded"))?;
        Ok((retained, error))
    }

    /// Build observed physical facts only; no expected answer enters this helper.
    pub fn preparation_observation(
        retained: &Path,
        error: &io::Error,
        recipe: &Value,
    ) -> io::Result<Value> {
        let diagnostic: Value =
            serde_json::from_slice(&fs::read(retained.join("preparation-error.json")).unwrap())
                .unwrap();
        let pid = diagnostic["actual_child_pid"].as_u64().unwrap();
        let binding = |context: &Path| {
            let launch_path = context.join("launches").join(format!("{pid}.json"));
            let exit_path = context.join("exits").join(format!("{pid}.json"));
            if !launch_path.is_file() || !exit_path.is_file() {
                return false;
            }
            let launch_bytes = fs::read(&launch_path).unwrap();
            let launch: Value = serde_json::from_slice(&launch_bytes).unwrap();
            let exit: Value = serde_json::from_slice(&fs::read(&exit_path).unwrap()).unwrap();
            let executable = PathBuf::from(OsString::from_vec(
                launch["executable"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|b| b.as_u64().unwrap() as u8)
                    .collect(),
            ));
            let program = if recipe["program"] == "/bin/sleep" {
                "/bin/sleep"
            } else {
                env!("CARGO_BIN_EXE_arboresce-xtask-test-peer")
            };
            launch["version"] == "native-launch-v1"
                && exit["version"] == "native-exit-v1"
                && launch["pid"] == pid
                && exit["pid"] == pid
                && launch["parent_pid"] == std::process::id()
                && launch["program"] == json!(program.as_bytes())
                && launch["arguments"]
                    == json!(
                        recipe["arguments"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|s| s.as_str().unwrap().as_bytes())
                            .collect::<Vec<_>>()
                    )
                && launch["executable_sha256"] == sha256(&fs::read(executable).unwrap())
                && launch["context_path"]
                    == json!(context.canonicalize().unwrap().as_os_str().as_bytes())
                && launch["context_sha256"]
                    == sha256(&fs::read(context.join("context.json")).unwrap())
                && exit["launch_sha256"] == sha256(&launch_bytes)
                && exit["exit"] == diagnostic["exit"]
                && exit["signal"] == diagnostic["signal"]
                && [
                    "parent_pid",
                    "run_id",
                    "source_sha256",
                    "context_path",
                    "context_sha256",
                ]
                .iter()
                .all(|key| launch[key] == exit[key])
        };
        let mut observed = diagnostic.clone();
        observed["error_kind"] = json!(format!("{:?}", error.kind()));
        observed["original_error_preserved"] = json!(
            observed["error_kind"] == diagnostic["error_kind"]
                && json!(error.raw_os_error()) == diagnostic["raw_os_error"]
        );
        observed["observe_entered"] = json!(retained.join("observe-entered").exists());
        observed["actual_fallback_launch_exit_binding"] =
            json!(binding(&retained.join("fallback")));
        observed["actual_explicit_launch_exit_binding"] = json!(binding(&retained.join("context")));
        observed["explicit_launch_present"] = json!(
            retained
                .join("context/launches")
                .join(format!("{pid}.json"))
                .is_file()
        );
        observed["explicit_completion_present"] = json!(
            retained
                .join("context/exits")
                .join(format!("{pid}.json"))
                .is_file()
        );
        observed["within_cleanup_budget"] = json!(
            diagnostic["preparation_ns"].as_u64().unwrap()
                <= recipe["cleanup_budget_ns"].as_u64().unwrap()
        );
        observed["permissions_restored"] = json!(
            diagnostic["permissions_restored"] == true
                && fs::metadata(retained.join("context/launches"))
                    .unwrap()
                    .permissions()
                    .mode() as u64
                    == diagnostic["original_launch_mode"].as_u64().unwrap()
        );
        observed["original_exit_directory_absent"] =
            json!(!retained.join("context/exits").exists());
        observed["preserved_exit_directory_present"] =
            json!(retained.join("context/exits-preserved").is_dir());
        observed["trace_path_is_directory"] = json!(retained.join("trace").is_dir());
        for name in ["stdout", "stderr"] {
            let bytes = fs::read(retained.join(name)).unwrap();
            observed[name] = json!({"bytes":bytes.len(),"sha256":sha256(&bytes)});
        }
        let summary =
            json!({"observed":observed,"retained":retained,"actual_error":error.to_string()});
        fs::write(
            retained.join("preparation-summary.json"),
            format!("{summary}\n"),
        )
        .unwrap();
        Ok(summary)
    }

    /// Inputs only: the peer, observer and fault transport never receive expected results.
    pub fn observe(recipe: &Value) -> io::Result<Value> {
        observe_at(recipe, &retained()?, None)
    }

    fn observe_at(
        recipe: &Value,
        retained: &Path,
        fallback: Option<&measurement::Request>,
    ) -> io::Result<Value> {
        let ambient_before =
            arboresce_xtask::collection_prepared::environment().map_err(io::Error::other)?;
        let stdout_input = input(&recipe["stdout"]);
        let stderr_input = input(&recipe["stderr"]);
        fs::write(retained.join("input-stdout"), &stdout_input)?;
        fs::write(retained.join("input-stderr"), &stderr_input)?;
        let stdout = retained.join("stdout");
        let stderr = retained.join("stderr");
        let explicit = collection_native::context(retained, "context")?;
        let request = measurement::Request::open(&explicit)?;
        let fault = recipe["preparation_fault"].as_str();
        let mut command = Command::new(if recipe["program"] == "/bin/sleep" {
            "/bin/sleep"
        } else {
            env!("CARGO_BIN_EXE_arboresce-xtask-test-peer")
        });
        if let Some(arguments) = recipe["arguments"].as_array() {
            command.args(arguments.iter().map(|argument| argument.as_str().unwrap()));
        }
        command
            .current_dir(retained)
            .env("ARBORESCE_XTASK_TEST_PEER", "trace-streams")
            .env("ARBORESCE_TRACE_STDOUT", retained.join("input-stdout"))
            .env("ARBORESCE_TRACE_STDERR", retained.join("input-stderr"))
            .stdin(Stdio::null())
            .stdout(Stdio::from(fs::File::create_new(&stdout)?))
            .stderr(Stdio::from(fs::File::create_new(&stderr)?));
        let prepared = request.prepare(&mut command)?;
        let fallback_prepared = fallback
            .map(|request| request.prepare(&mut command))
            .transpose()?;
        let launches = explicit.join("launches");
        let mut permissions = RestorePermissions {
            original: fs::metadata(&launches)?.permissions(),
            path: launches,
            active: false,
        };
        let mut write_denial_proven = false;
        if fault == Some("launch-denied") {
            permissions.active = true;
            fs::set_permissions(&permissions.path, fs::Permissions::from_mode(0o555))?;
            write_denial_proven = fs::File::create_new(permissions.path.join("denial-proof"))
                .is_err_and(|error| error.kind() == io::ErrorKind::PermissionDenied);
            if !write_denial_proven {
                return Err(io::Error::other(
                    "actual local launch write denial not established",
                ));
            }
        }
        if fault == Some("trace-conflict") {
            fs::create_dir(retained.join("trace"))?;
        }
        let mut child = measurement::spawn(&mut command)?;
        let completion_path = explicit.join("exits").join(format!("{}.json", child.id()));
        let preparation_started = Instant::now();
        let mut running_before_record = None;
        let preparation = (|| {
            if let Some(prepared) = fallback_prepared {
                prepared.record(&mut child)?;
            }
            if fault.is_some() && fault != Some("launch-denied") {
                wait_prepared(&mut child)?;
            }
            running_before_record = Some(child.try_wait()?.is_none());
            prepared.record(&mut child)?;
            // Reap without completing either receipt. Only observe may complete B.
            let status = wait_prepared(&mut child)?;
            if fault == Some("missing-exits") {
                fs::rename(explicit.join("exits"), explicit.join("exits-preserved"))?;
            }
            if completion_path.exists() || fs::read_dir(explicit.join("exits"))?.count() != 0 {
                return Err(io::Error::other(
                    "explicit trace completion existed before observe",
                ));
            }
            let trace = Trace {
                file: fs::File::create_new(retained.join("trace"))?,
                writes: 0,
                accepted: 0,
                max_writes: recipe["max_trace_writes"].as_u64().unwrap_or(9000),
                max_write_bytes: recipe["max_write_bytes"].as_u64(),
                fail_after: recipe["fail_once_after_bytes"].as_u64(),
                failed_once: false,
                failure_at: None,
                io_failed: false,
                first_post_error: "none",
                flushes: 0,
                flush_error: !recipe["flush_error_once"].is_null(),
                flush_delay: Duration::from_millis(recipe["flush_delay_ms"].as_u64().unwrap_or(0)),
                delayed_flushes_returned: 0,
            };
            permissions.restore()?;
            fs::write(retained.join("observe-entered"), b"ready to observe\n")?;
            Ok((status, trace))
        })();
        let (status, mut trace) = match preparation {
            Ok(prepared) => prepared,
            Err(error) => {
                let terminated = process::terminate(&mut child);
                let restored = permissions.restore();
                let explicit_completion = request.complete(&mut child);
                let fallback_completion = fallback
                    .map(|request| request.complete(&mut child))
                    .transpose();
                let ambient_completion = measurement::complete(&mut child);
                let status = child.try_wait().ok().flatten();
                let diagnostic = json!({"error_kind":format!("{:?}",error.kind()),
                    "raw_os_error":error.raw_os_error(),"actual_child_pid":child.id(),
                    "exit":status.as_ref().and_then(|status|status.code()),
                    "signal":status.as_ref().and_then(|status|status.signal()),"reaped":status.is_some(),
                    "termination_attempted":true,"termination_ok":terminated.is_ok(),
                    "termination_error":terminated.as_ref().err().map(|e|format!("{:?}",e.kind())),
                    "explicit_completion_attempted":true,"explicit_completion_ok":explicit_completion.is_ok(),
                    "explicit_completion_error":explicit_completion.as_ref().err().map(|e|format!("{:?}",e.kind())),
                    "fallback_completion_attempted":fallback.is_some(),"fallback_completion_ok":fallback_completion.is_ok(),
                    "fallback_completion_error":fallback_completion.as_ref().err().map(|e|format!("{:?}",e.kind())),
                    "ambient_completion_attempted":true,"ambient_completion_ok":ambient_completion.is_ok(),
                    "ambient_completion_error":ambient_completion.as_ref().err().map(|e|format!("{:?}",e.kind())),
                    "preparation_ns":preparation_started.elapsed().as_nanos(),
                    "child_was_running_before_record":running_before_record,
                    "write_denial_proven":write_denial_proven,
                    "original_launch_mode":permissions.original.mode(),
                    "permissions_restored":restored.is_ok() && fs::metadata(&permissions.path).is_ok_and(|m|m.permissions().mode()==permissions.original.mode()),
                    "ambient_environment_unchanged":arboresce_xtask::collection_prepared::environment().is_ok_and(|after|after==ambient_before)});
                let _ = fs::write(
                    retained.join("preparation-error.json"),
                    format!("{diagnostic}\n"),
                );
                return Err(error);
            }
        };
        let started = Instant::now();
        let result = process::observe(
            &mut child,
            &stdout,
            &stderr,
            process::Observation {
                budget: Duration::from_millis(
                    recipe["observation_budget_ms"].as_u64().unwrap_or(30000),
                ),
                request: Some(request),
                ..process::Observation::default()
            },
            started,
            &mut trace,
        );
        let elapsed = started.elapsed();
        // Complete ambient A only after observe has had its sole chance to create B.
        let completion_bytes = fs::read(&completion_path);
        let fallback_completed = fallback
            .map(|request| request.complete(&mut child))
            .transpose();
        let ambient = measurement::complete(&mut child);
        fallback_completed?;
        ambient?;
        let completion: Option<Value> = completion_bytes
            .as_ref()
            .ok()
            .map(|bytes| serde_json::from_slice(bytes))
            .transpose()?;
        let launch_bytes = fs::read(
            explicit
                .join("launches")
                .join(format!("{}.json", child.id())),
        )?;
        let launch: Value = serde_json::from_slice(&launch_bytes)?;
        let launch_matches = completion.as_ref().is_some_and(|completion| {
            completion["version"] == "native-exit-v1"
                && launch["version"] == "native-launch-v1"
                && completion["pid"] == child.id()
                && launch["pid"] == child.id()
                && completion["launch_sha256"] == sha256(&launch_bytes)
                && completion["exit"] == json!(status.code())
                && completion["signal"] == json!(status.signal())
                && [
                    "parent_pid",
                    "run_id",
                    "source_sha256",
                    "context_path",
                    "context_sha256",
                ]
                .iter()
                .all(|key| completion[key] == launch[key])
        });
        let stdout_bytes = fs::read(&stdout)?;
        let stderr_bytes = fs::read(&stderr)?;
        let trace_bytes = fs::read(retained.join("trace"))?;
        let frame = complete_frame(&trace_bytes, child.id(), &stdout_bytes, &stderr_bytes);
        let mut observed = json!({"operation_ok":result.is_ok(),
            "error_kind":result.as_ref().err().map(|error|format!("{:?}",error.kind())),
            "exit":status.code(),"signal":status.signal(),
            "reaped":child.try_wait()?.is_some(),
            "explicit_completion_present":completion.is_some(),
            "explicit_launch_binding_matches":launch_matches,
            "stdout":{"bytes":stdout_bytes.len(),"sha256":sha256(&stdout_bytes)},
            "stderr":{"bytes":stderr_bytes.len(),"sha256":sha256(&stderr_bytes)},
            "observation_bytes_complete":frame.is_some(),
            "first_successful_post_io_error_write":trace.first_post_error,
            "flush_calls":trace.flushes,
            "trace_write_budget_respected":trace.writes <= trace.max_writes});
        if result.is_ok() && frame == Some(trace_bytes.len()) {
            observed["normalized_trace"] = normalized_trace(&trace_bytes, child.id());
        }
        let proof = json!({"actual_child_pid":child.id(),"retained":retained,
            "explicit_completion_absent_before_observe":true,
            "launch":launch,"completion":completion,
            "ambient_completion_succeeded":true,
            "ambient_environment_unchanged":arboresce_xtask::collection_prepared::environment().map_err(io::Error::other)?==ambient_before,
            "write_calls":trace.writes,"accepted_trace_bytes":trace.accepted,
            "one_shot_failure_after_actual_bytes":trace.failure_at,
            "delayed_flushes_returned":trace.delayed_flushes_returned,
            "observation_elapsed_ms":elapsed.as_millis(),
            "captured_inputs_exact":stdout_bytes==stdout_input && stderr_bytes==stderr_input,
            "trace":{"bytes":trace_bytes.len(),"sha256":sha256(&trace_bytes)}});
        let summary = json!({"observed":observed,"proof":proof});
        fs::write(retained.join("summary.json"), format!("{summary}\n"))?;
        Ok(summary)
    }
}

pub mod qualified_source {
    use super::{PreparedRoot, qualified_number};
    use serde_json::Value;
    use std::{fs, io};

    pub fn fixture() -> Value {
        serde_json::from_str(include_str!(
            "../../../../tests/architecture/fixtures/qualified-source-dependencies-v1/cases.json"
        ))
        .unwrap()
    }

    /// Actual input metadata and source placement, without expected decisions.
    pub fn metadata(
        root: &PreparedRoot,
        input: &Value,
        source_case: Option<&Value>,
    ) -> io::Result<Value> {
        let mut metadata = qualified_number::metadata(root, input)?;
        if let Some(row) = source_case {
            let directory = root.path.join("crates/arboresce-domain");
            let path = match row["placement"].as_str().unwrap_or("main") {
                "unimported" => directory.join("src/unimported.rs"),
                "non-rs-target" => {
                    let path = directory.join("src/actual.txt");
                    let package = metadata["packages"]
                        .as_array_mut()
                        .unwrap()
                        .iter_mut()
                        .find(|p| p["id"] == "arboresce-domain")
                        .unwrap();
                    package["targets"][0]["src_path"] = serde_json::json!(path);
                    path
                }
                _ => directory.join("src/lib.rs"),
            };
            fs::write(path, row["source"].as_str().unwrap())?;
        }
        Ok(metadata)
    }
}

pub mod qualified_number {
    use super::PreparedRoot;
    use serde_json::{Value, json};
    use std::{fs, io};

    pub fn fixture() -> Value {
        serde_json::from_str(include_str!(
            "../../../../tests/architecture/fixtures/qualified-number-dependencies-v1/cases.json"
        ))
        .unwrap()
    }

    /// Materialize input facts only. Expected acceptance never enters metadata.
    pub fn metadata(root: &PreparedRoot, input: &Value) -> io::Result<Value> {
        let mut metadata = root.metadata()?;
        // Every registered owner exists so a wrong-owner case reaches policy.
        let directory = root.path.join("tools/xtask");
        fs::create_dir_all(directory.join("src"))?;
        fs::write(
            directory.join("Cargo.toml"),
            "[package]\nname=\"arboresce-xtask\"\nversion=\"0.0.0\"\n",
        )?;
        fs::write(directory.join("src/lib.rs"), "pub fn n() -> u8 { 1 }\n")?;
        metadata["packages"].as_array_mut().unwrap().push(json!({"id":"arboresce-xtask",
            "name":"arboresce-xtask","version":"0.0.0","manifest_path":directory.join("Cargo.toml"),
            "targets":[{"name":"arboresce-xtask","kind":["lib"],"src_path":directory.join("src/lib.rs")}]}));
        metadata["workspace_members"]
            .as_array_mut()
            .unwrap()
            .push(json!("arboresce-xtask"));
        metadata["resolve"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id":"arboresce-xtask","deps":[]}));
        for package in input["graph"]["packages"].as_array().unwrap() {
            let id = package["package"].as_str().unwrap();
            let declared = &input["declared_packages"][id];
            metadata["packages"]
                .as_array_mut()
                .unwrap()
                .push(json!({"id":id,
                "name":declared["name"],"version":declared["version"],"source":package["source"]}));
            let deps: Vec<_> = package["dependencies"].as_array().unwrap().iter()
                .map(|edge| json!({"name":edge["name"],"pkg":edge["package"],"dep_kinds":edge["kinds"]})).collect();
            metadata["resolve"]["nodes"]
                .as_array_mut()
                .unwrap()
                .push(json!({"id":id,"features":package["features"],"deps":deps}));
        }
        let owner = input["owner"].as_str().unwrap();
        let package = metadata["packages"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|p| p["id"] == owner)
            .unwrap();
        if input["profile_present"] == true {
            package["metadata"] = json!({"arboresce":{"architecture-profile":input["profile"]}});
        }
        let node = metadata["resolve"]["nodes"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|p| p["id"] == owner)
            .unwrap();
        node["features"] = input["graph"]["owner_features"].clone();
        let mut deps: Vec<_> = input["graph"]["roots"]
            .as_array()
            .unwrap()
            .iter()
            .map(
                |edge| json!({"name":edge["name"],"pkg":edge["package"],"dep_kinds":edge["kinds"]}),
            )
            .collect();
        // Match the actual controlled fixture decoder without a normal root.
        deps.push(json!({"name":"serde_json","pkg":"serde_json@1.0.151","dep_kinds":[{"kind":"dev","target":null}]}));
        node["deps"] = json!(deps);
        Ok(metadata)
    }
}

pub struct PreparedRoot {
    pub path: PathBuf,
    preserve: bool,
}

impl PreparedRoot {
    pub fn new() -> io::Result<Self> {
        let parent = Path::new(env!("CARGO_TARGET_TMPDIR"));
        fs::create_dir_all(parent)?;
        let path = parent.join(format!(
            "arboresce-xtask-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        Ok(Self {
            path,
            preserve: false,
        })
    }

    /// Keep selected native evidence at its original byte-bound paths.
    pub fn preserve(&mut self) {
        self.preserve = true;
    }

    /// Generate syntax-policy inputs, not compiled or executable product code.
    pub fn metadata(&self) -> io::Result<serde_json::Value> {
        let mut packages = Vec::new();
        let mut members = Vec::new();
        let mut nodes = Vec::new();
        for (name, source) in [
            ("arboresce", "pub fn n() { println!(\"Arboresce\"); }"),
            ("arboresce-domain", "pub fn n() -> u8 { 1 }"),
        ] {
            let directory = self.path.join("crates").join(name);
            fs::create_dir_all(directory.join("src"))?;
            fs::write(
                directory.join("Cargo.toml"),
                format!("[package]\nname=\"{name}\"\nversion=\"0.0.0\"\nedition=\"2024\"\n"),
            )?;
            fs::write(directory.join("src/lib.rs"), source)?;
            packages.push(serde_json::json!({
                "id":name,"name":name,"version":"0.0.0","manifest_path":directory.join("Cargo.toml"),
                "targets":[{"name":name,"kind":["lib"],"src_path":directory.join("src/lib.rs")}]
            }));
            members.push(name);
            nodes.push(serde_json::json!({"id":name,"deps":[]}));
        }
        Ok(
            serde_json::json!({"workspace_root":self.path,"packages":packages,"workspace_members":members,"resolve":{"nodes":nodes}}),
        )
    }

    #[cfg(unix)]
    pub fn cargo_peer(&self) -> io::Result<PathBuf> {
        let directory = self.path.join("bin");
        fs::create_dir(&directory)?;
        std::os::unix::fs::symlink(
            env!("CARGO_BIN_EXE_arboresce-xtask-test-peer"),
            directory.join("cargo"),
        )?;
        Ok(directory)
    }
}

impl Drop for PreparedRoot {
    fn drop(&mut self) {
        if !self.preserve {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

#[derive(Clone, Copy)]
pub enum ClosedStream {
    Stdout,
    Stderr,
}

pub struct Observation {
    pub budget: Duration,
    pub poll: Duration,
    pub initial_delay: Duration,
    pub closed: Option<ClosedStream>,
}

impl Default for Observation {
    fn default() -> Self {
        Self {
            budget: Duration::from_secs(30),
            poll: Duration::from_millis(5),
            initial_delay: Duration::ZERO,
            closed: None,
        }
    }
}

fn closed_pipe() -> io::Result<Stdio> {
    let (reader, writer) = io::pipe()?;
    drop(reader);
    Ok(Stdio::from(writer))
}

/// Actual native processes share one bounded observation/collection path.
/// Regular task-owned files avoid blocking pipe-reader joins and inherited pipes.
pub fn observe(command: &mut Command) -> io::Result<Output> {
    observe_with(command, Observation::default())
}

pub fn observe_with(command: &mut Command, options: Observation) -> io::Result<Output> {
    observe_identity_with(command, options).map(|(_, output)| output)
}

pub fn observe_identity(command: &mut Command) -> io::Result<(u32, Output)> {
    observe_identity_with(command, Observation::default())
}

fn observe_identity_with(command: &mut Command, options: Observation) -> io::Result<(u32, Output)> {
    let prepared = PreparedRoot::new()?;
    let retained = prepared.path.with_extension("observation-evidence");
    fs::rename(&prepared.path, &retained)?;
    println!("xtask retained native observation {}", retained.display());
    let stdout_path = retained.join("stdout");
    let stderr_path = retained.join("stderr");
    let stdout = fs::File::create(&stdout_path)?;
    let stderr = fs::File::create(&stderr_path)?;
    let mut trace = fs::File::create_new(retained.join("trace"))?;
    command.stdout(if matches!(options.closed, Some(ClosedStream::Stdout)) {
        closed_pipe()?
    } else {
        Stdio::from(stdout)
    });
    command.stderr(if matches!(options.closed, Some(ClosedStream::Stderr)) {
        closed_pipe()?
    } else {
        Stdio::from(stderr)
    });
    let started = Instant::now();
    let mut child = arboresce_xtask::measurement::spawn(command)?;
    let pid = child.id();
    arboresce_xtask::process::observe(
        &mut child,
        &stdout_path,
        &stderr_path,
        arboresce_xtask::process::Observation {
            budget: options.budget,
            poll: options.poll,
            initial_delay: options.initial_delay,
            request: None,
        },
        started,
        &mut trace,
    )
    .map(|output| (pid, output))
}

pub struct FailedOutput {
    pub write_fails: bool,
    pub bytes: Vec<u8>,
}
impl Write for FailedOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.write_fails {
            Err(io::Error::other("write failed"))
        } else {
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("flush failed"))
    }
}

/// A one-shot trace fault and optional real delay at a complete line boundary.
pub struct ObservationTrace {
    pub bytes: Vec<u8>,
    pub fail_after_lines: Option<usize>,
    pub delay_after_line: usize,
    pub delay: Duration,
    failed: bool,
    lines: usize,
}

impl ObservationTrace {
    pub fn new(fail_after_lines: Option<usize>, delay_after_line: usize, delay: Duration) -> Self {
        Self {
            bytes: Vec::new(),
            fail_after_lines,
            delay_after_line,
            delay,
            failed: false,
            lines: 0,
        }
    }
}

impl Write for ObservationTrace {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if !self.failed && self.fail_after_lines == Some(self.lines) {
            self.failed = true;
            return Err(io::Error::other("controlled trace write failure"));
        }
        self.bytes.extend_from_slice(bytes);
        for byte in bytes {
            if *byte == b'\n' {
                self.lines += 1;
                if self.lines == self.delay_after_line {
                    std::thread::sleep(self.delay);
                }
            }
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
#[cfg(unix)]
pub mod inventory_inputs;
pub mod report_inputs;

pub mod collection_inputs;
pub mod collection_mini;
pub mod collection_native;
pub mod measurement_inputs;
