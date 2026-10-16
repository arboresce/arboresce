//! Retained actual commands with pinned tools and explicit nested receipts.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    os::unix::{ffi::OsStrExt, process::ExitStatusExt},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    time::{Duration, Instant},
};

use serde_json::{Value, json};

use crate::{
    collection_inputs::{CollectError, Result},
    collection_prepared::{self, ENVIRONMENT},
    inventory::sha256,
    measurement::{self, Request},
    process::{self, Observation},
};

pub struct Captured {
    pub pid: u32,
    pub output: Output,
}

pub struct Runner {
    root: PathBuf,
    logs: PathBuf,
    prepared: Value,
    next: u64,
    pub target: PathBuf,
    pub request: Option<Request>,
    pub context: Option<PathBuf>,
    pub deadline: Option<Instant>,
    pub service_input: Option<PathBuf>,
}

pub fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    fs::File::create_new(path)
        .and_then(|mut file| file.write_all(bytes).and_then(|()| file.flush()))
        .map_err(|_| CollectError::Intake)
}

pub fn record(path: &Path, value: &Value) -> Result<()> {
    write(path, format!("{value}\n").as_bytes())
}

impl Runner {
    pub fn new(root: &Path, output: &Path, prepared: &Value) -> Result<Self> {
        let logs = output.join("commands");
        fs::create_dir(&logs).map_err(|_| CollectError::Intake)?;
        Ok(Self {
            root: root.to_owned(),
            logs,
            prepared: prepared.clone(),
            next: 0,
            target: output.join("target"),
            request: None,
            context: None,
            deadline: None,
            service_input: None,
        })
    }

    pub fn program(&self, name: &str) -> Result<PathBuf> {
        self.prepared["tools"][name]["program"]
            .as_str()
            .map(PathBuf::from)
            .ok_or(CollectError::Prepared)
    }

    fn command(
        &self,
        program: &Path,
        args: &[String],
        environment: &BTreeMap<String, String>,
        target: &Path,
    ) -> Result<Command> {
        let mut command = Command::new(program);
        command
            .env("LC_ALL", "C")
            .env("LANG", "C")
            .env("TZ", "UTC")
            .args(args)
            .current_dir(&self.root)
            .stdin(Stdio::null());
        for key in ENVIRONMENT {
            command.env_remove(key);
        }
        let mut paths = Vec::new();
        let mut seen = BTreeSet::new();
        // Preserve the selected executable spelling, including tool dispatch basenames.
        for key in ["cargo", "cargo-llvm-cov", "true", "sleep", "mkfifo"] {
            let program = self.program(key)?;
            let parent = program.parent().ok_or(CollectError::Prepared)?.to_owned();
            if seen.insert(parent.clone()) {
                paths.push(parent);
            }
        }
        command.env(
            "PATH",
            std::env::join_paths(paths).map_err(|_| CollectError::Prepared)?,
        );
        command
            .env("RUSTUP_TOOLCHAIN", "1.99.0")
            .env("RUSTUP_AUTO_INSTALL", "0")
            .env("CARGO_NET_OFFLINE", "true")
            .env(
                "CARGO_HOME",
                self.prepared["cargo_home"]
                    .as_str()
                    .ok_or(CollectError::Prepared)?,
            )
            .env("CARGO_TARGET_DIR", target)
            .env("CARGO_BUILD_BUILD_DIR", target)
            .env("CARGO", self.program("cargo")?)
            .env("RUSTC", self.program("rustc")?)
            .env("RUSTDOC", self.program("rustdoc")?)
            .env("RUSTFMT", self.program("rustfmt")?)
            .env("CLIPPY_DRIVER_PATH", self.program("clippy-driver")?)
            .env("LLVM_COV", self.program("llvm-cov")?)
            .env("LLVM_PROFDATA", self.program("llvm-profdata")?);
        command.envs(environment);
        crate::collection_services::environment(&mut command, self.service_input.as_deref());
        if let Some(context) = &self.context {
            command.env(measurement::ENV, context);
        }
        Ok(command)
    }

    /// The phase budget includes hashing, spawn, actual status, output and receipts.
    pub fn run(
        &mut self,
        program: &Path,
        args: &[String],
        budget: u64,
        environment: &BTreeMap<String, String>,
        target: &Path,
    ) -> Result<Captured> {
        let bad = CollectError::Command;
        let started = Instant::now();
        let maximum = started + Duration::from_secs(budget);
        let deadline = self.deadline.map_or(maximum, |phase| phase.min(maximum));
        collection_prepared::check_time(deadline)?;
        let directory = self.logs.join(format!("{:04}", self.next));
        self.next += 1;
        fs::create_dir(&directory).map_err(|_| bad)?;
        let canonical = program.canonicalize().map_err(|_| bad)?;
        let identity = collection_prepared::fingerprint(&canonical, 64 << 20, deadline)?;
        for selected in self.prepared["tools"]
            .as_object()
            .ok_or(CollectError::Prepared)?
            .values()
        {
            if selected["program"].as_str() == program.to_str()
                && (selected["sha256"] != identity["sha256"]
                    || selected["executable"].as_str() != canonical.to_str())
            {
                return Err(CollectError::Changed);
            }
        }
        let mut command = self.command(program, args, environment, target)?;
        let stdout = directory.join("stdout");
        let stderr = directory.join("stderr");
        command
            .stdout(Stdio::from(fs::File::create_new(&stdout).map_err(|_| bad)?))
            .stderr(Stdio::from(fs::File::create_new(&stderr).map_err(|_| bad)?));
        let intent_path = directory.join("launch-command.json");
        let intent = json!({
            "program":program.as_os_str().as_bytes(),"executable":canonical.as_os_str().as_bytes(),
            "executable_sha256":identity["sha256"],"arguments":args.iter().map(|s|s.as_bytes()).collect::<Vec<_>>(),
            "cwd":self.root.as_os_str().as_bytes(),"budget_seconds":budget,"observation_budget_ns":deadline.duration_since(started).as_nanos(),
            "environment":command.get_envs().map(|(key,v)|(key.to_string_lossy().into_owned(),v.map(|v|v.to_string_lossy().into_owned()))).collect::<BTreeMap<_,_>>()
        });
        let intent_bytes = format!("{intent}\n").into_bytes();
        write(&intent_path, &intent_bytes)?;
        let mut trace = fs::File::create_new(directory.join("trace")).map_err(|_| bad)?;
        let mut child = match measurement::spawn_diagnosed(&mut command, self.request.as_ref()) {
            Ok(child) => child,
            Err(failure) => {
                let frame = failure.envelope(
                    &intent_path,
                    &intent_bytes,
                    measurement::DiagnosticScope::ActualRuntime,
                );
                measurement::retain(&frame, &directory.join("launch-failure.json"), &mut trace)
                    .map_err(|_| bad)?;
                return Err(bad);
            }
        };
        let pid = child.id();
        let result = process::observe(
            &mut child,
            &stdout,
            &stderr,
            Observation {
                budget: deadline.duration_since(started),
                request: self.request.clone(),
                ..Observation::default()
            },
            started,
            &mut trace,
        );
        let status = child.try_wait().map_err(|_| bad)?.ok_or(bad)?;
        let value = match &result {
            Ok(output) => json!({"pid":pid,"exit":status.code(),"signal":status.signal(),
                "stdout_sha256":sha256(&output.stdout),"stderr_sha256":sha256(&output.stderr),"error":null}),
            Err(error) => {
                json!({"pid":pid,"exit":status.code(),"signal":status.signal(),"error":error.to_string(),"error_kind":format!("{:?}",error.kind())})
            }
        };
        record(&directory.join("completion.json"), &value)?;
        trace.flush().map_err(|_| bad)?;
        if Instant::now() >= deadline {
            return Err(bad);
        }
        Ok(Captured {
            pid,
            output: result.map_err(|_| bad)?,
        })
    }

    pub fn tool(
        &mut self,
        name: &str,
        args: &[&str],
        budget: u64,
        environment: &BTreeMap<String, String>,
        target: &Path,
    ) -> Result<Captured> {
        let program = self.program(name)?;
        self.run(
            &program,
            &args.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>(),
            budget,
            environment,
            target,
        )
    }

    pub fn success(
        &mut self,
        name: &str,
        args: &[&str],
        budget: u64,
        environment: &BTreeMap<String, String>,
        target: &Path,
    ) -> Result<Captured> {
        let actual = self.tool(name, args, budget, environment, target)?;
        if !actual.output.status.success() {
            return Err(CollectError::Command);
        }
        Ok(actual)
    }
}
