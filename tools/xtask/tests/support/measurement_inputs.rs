//! Measured preparation for independently frozen native receipt inputs.

pub struct RunnerFailureInput {
    pub mini: super::collection_mini::Mini,
    pub result:
        arboresce_xtask::collection_inputs::Result<arboresce_xtask::collection_runner::Captured>,
}

impl DiagnosticInput {
    /// Reuse actual prepared tool identities; this is not synthetic preparation.
    pub fn runner_failure(
        &self,
        workspace_files: &Value,
        input: &Value,
    ) -> io::Result<RunnerFailureInput> {
        use arboresce_xtask::{collection_runner::Runner, measurement::Request};
        let mini = super::collection_mini::Mini::run(
            &serde_json::json!({"mini_workspace_files":workspace_files}),
            &serde_json::json!({"id":"diagnostic-retain-exclusive-success","mutation":false}),
        )?;
        if !mini.actual.status.success() {
            return Err(io::Error::other(
                "actual collector mini preparation/collection failed",
            ));
        }
        let request = Request::open(&self.input.root)?;
        self.before_prepare(&serde_json::json!({"recipe":input["failure_recipe"]}))?;
        let output = self.input.owner.path.join("runner");
        fs::create_dir(&output)?;
        let mut runner =
            Runner::new(&mini.source, &output, &mini.manifest).map_err(io::Error::other)?;
        runner.request = Some(request);
        runner.context = Some(self.input.root.clone());
        let program = runner.program("true").map_err(io::Error::other)?;
        let result = runner.run(
            &program,
            &[],
            30,
            &std::collections::BTreeMap::new(),
            &output.join("target"),
        );
        Ok(RunnerFailureInput { mini, result })
    }
}

impl DiagnosticInput {
    /// Save the actual API envelope separately from any expected projection.
    pub fn save_envelope(&self, frame: &Value) -> io::Result<()> {
        fs::write(
            self.input.owner.path.join("observed-envelope.json"),
            format!("{frame}\n"),
        )
    }

    pub fn after_prepare(&self, recipe: &Value) -> io::Result<()> {
        if recipe["recipe"] == "unlink-owned-selected-program-after-prepare" {
            fs::remove_file(&self.selected)?;
        }
        Ok(())
    }

    pub fn restore_launch_permissions(&self, recipe: &Value) -> io::Result<()> {
        if recipe["recipe"] == "readonly-launch-directory" {
            fs::set_permissions(
                self.input.root.join("launches"),
                fs::Permissions::from_mode(0o700),
            )?;
        }
        Ok(())
    }
}

/// Construct actual representation-unit input, never an expected observation.
pub fn representation_packet(base: &Value, input: &Value) -> Value {
    let mut packet = base.clone();
    let padding = &input["command_path_padding"];
    let mut path = vec![padding["prefix_byte"].as_u64().unwrap() as u8];
    path.extend(std::iter::repeat_n(
        padding["fill_byte"].as_u64().unwrap() as u8,
        padding["fill_count"].as_u64().unwrap() as usize,
    ));
    for item in path
        .iter_mut()
        .rev()
        .take(padding["last_short_count"].as_u64().unwrap() as usize)
    {
        *item = padding["short_byte"].as_u64().unwrap() as u8;
    }
    packet["command"]["path"] = serde_json::json!(path);
    packet
}

pub fn controlled_write_failure(input: &Value) -> arboresce_xtask::measurement::WriteFailure {
    let fault = match input["recipe"].as_str().unwrap() {
        "record-write-zero" | "zero-length-write" => Fault::Zero,
        _ => Fault::Flush,
    };
    arboresce_xtask::measurement::write_payload(
        &mut FaultWriter::new(Vec::new(), fault),
        input["payload"].as_str().unwrap().as_bytes(),
    )
    .expect_err("controlled writer recipe")
}

pub struct RetentionInput {
    pub artifact: PathBuf,
    pub target: PathBuf,
    pub trace_path: PathBuf,
    parent: PathBuf,
    trace: fs::File,
    recipe: Value,
}

impl RetentionInput {
    pub fn new(setup: &DiagnosticInput, input: &Value) -> io::Result<Self> {
        let parent = setup.input.owner.path.join("diagnostics");
        fs::create_dir(&parent)?;
        let artifact = parent.join("launch-failure.json");
        let target = setup.input.owner.path.join("preserved-target");
        let trace_path = setup.input.owner.path.join("trace");
        let trace = fs::File::create_new(&trace_path)?;
        match input["recipe"].as_str().unwrap() {
            "existing-artifact" => fs::write(&artifact, input["existing_bytes"].as_str().unwrap())?,
            "symlink-artifact" => {
                fs::write(&target, input["existing_bytes"].as_str().unwrap())?;
                symlink(&target, &artifact)?;
            }
            "readonly-artifact-parent" => {
                fs::set_permissions(&parent, fs::Permissions::from_mode(0o500))?
            }
            _ => {}
        }
        Ok(Self {
            artifact,
            target,
            trace_path,
            parent,
            trace,
            recipe: input.clone(),
        })
    }

    pub fn retain(
        &mut self,
        frame: &Value,
    ) -> Result<(), arboresce_xtask::measurement::RetentionFailure> {
        use arboresce_xtask::measurement;
        match self.recipe["recipe"].as_str().unwrap() {
            "trace-zero-write" => measurement::retain(
                frame,
                &self.artifact,
                &mut FaultWriter::new(&mut self.trace, Fault::Zero),
            ),
            "trace-flush-broken-pipe" => measurement::retain(
                frame,
                &self.artifact,
                &mut FaultWriter::new(&mut self.trace, Fault::Flush),
            ),
            "controlled-artifact-write" => {
                measurement::retain_with_writer(frame, &self.artifact, &mut self.trace, |file| {
                    FaultWriter::new(
                        file,
                        Fault::Prefix(
                            self.recipe["artifact_writer"]["prefix_bytes"]
                                .as_u64()
                                .unwrap() as usize,
                        ),
                    )
                })
            }
            "controlled-artifact-flush" => {
                measurement::retain_with_writer(frame, &self.artifact, &mut self.trace, |file| {
                    FaultWriter::new(file, Fault::Flush)
                })
            }
            _ => measurement::retain(frame, &self.artifact, &mut self.trace),
        }
    }

    pub fn restore_permissions(&self) -> io::Result<()> {
        if self.recipe["recipe"] == "readonly-artifact-parent" {
            fs::set_permissions(&self.parent, fs::Permissions::from_mode(0o700))?;
        }
        Ok(())
    }
}

/// Owned setup accepts recipes only; expected observations stay in assertions.
pub struct DiagnosticInput {
    pub input: Input,
    pub command: Command,
    pub intent: PathBuf,
    pub selected: PathBuf,
}

impl DiagnosticInput {
    pub fn new(base: &Value, recipe: &Value) -> io::Result<Self> {
        let mut input = Input::new(base, &serde_json::json!({}))?;
        input.owner.preserve();
        fs::write(
            input.owner.path.join("context-original.json"),
            fs::read(input.root.join("context.json"))?,
        )?;
        let mode = recipe["recipe"].as_str().unwrap();
        let selected = input.owner.path.join("selected-program");
        let mut program = OsString::from(recipe["program"].as_str().unwrap_or("/usr/bin/true"));
        let mut cwd = input.owner.path.clone();
        match mode {
            "missing-owned-program" => program = selected.clone().into_os_string(),
            "nonexecutable-owned-file" => {
                fs::write(&selected, recipe["file_bytes"].as_str().unwrap())?;
                fs::set_permissions(&selected, fs::Permissions::from_mode(0o600))?;
                program = selected.clone().into_os_string();
            }
            "unlink-owned-selected-program-after-prepare" => {
                symlink(recipe["selected_program"].as_str().unwrap(), &selected)?;
                program = selected.clone().into_os_string();
            }
            "missing-cwd" => cwd.push("missing"),
            _ => {}
        }
        let mut command = Command::new(program);
        command.current_dir(cwd);
        if let Some(arguments) = recipe["arguments"].as_array() {
            command.args(arguments.iter().map(|a| a.as_str().unwrap()));
        }
        let intent = input.owner.path.join("launch-command.json");
        use std::os::unix::ffi::OsStrExt;
        let bytes = format!(
            "{}\n",
            serde_json::json!({
                "program":command.get_program().as_bytes(),
                "arguments":command.get_args().map(|a|a.as_bytes()).collect::<Vec<_>>(),
                "cwd":command.get_current_dir().unwrap().as_os_str().as_bytes(),
            })
        );
        fs::write(&intent, bytes)?;
        Ok(Self {
            input,
            command,
            intent,
            selected,
        })
    }

    pub fn change_context(&self, recipe: &Value) -> io::Result<()> {
        let path = self.input.root.join("context.json");
        let mut value: Value =
            serde_json::from_slice(&fs::read(&path)?).map_err(io::Error::other)?;
        value["run_id"] = recipe["replacement_run_id"].clone();
        fs::write(path, value.to_string())
    }

    pub fn before_prepare(&self, recipe: &Value) -> io::Result<()> {
        match recipe["recipe"].as_str().unwrap() {
            "missing-context-root" => fs::remove_dir_all(&self.input.root)?,
            "invalid-context-version" => {
                let path = self.input.root.join("context.json");
                let mut value: Value =
                    serde_json::from_slice(&fs::read(&path)?).map_err(io::Error::other)?;
                value["version"] = "invalid".into();
                fs::write(path, value.to_string())?;
            }
            "remove-context-after-open" => fs::remove_file(self.input.root.join("context.json"))?,
            "change-context-after-open" => self.change_context(recipe)?,
            _ => {}
        }
        Ok(())
    }

    pub fn before_record(&self, recipe: &Value, pid: u32) -> io::Result<()> {
        let path = self.input.root.join("launches").join(format!("{pid}.json"));
        match recipe["recipe"].as_str().unwrap() {
            "remove-context-before-record" => {
                fs::remove_file(self.input.root.join("context.json"))?
            }
            "change-context-before-record" => self.change_context(recipe)?,
            "collision-regular" => fs::write(path, recipe["collision_bytes"].as_str().unwrap())?,
            "collision-symlink" => {
                fs::write(&self.selected, recipe["collision_bytes"].as_str().unwrap())?;
                symlink(&self.selected, path)?;
            }
            "readonly-launch-directory" => fs::set_permissions(
                self.input.root.join("launches"),
                fs::Permissions::from_mode(0o500),
            )?,
            _ => {}
        }
        Ok(())
    }
}

pub enum Fault {
    Zero,
    Flush,
    Prefix(usize),
}

pub struct FaultWriter<W> {
    pub inner: W,
    fault: Fault,
}

impl<W> FaultWriter<W> {
    pub fn new(inner: W, fault: Fault) -> Self {
        Self { inner, fault }
    }
}

impl<W: io::Write> io::Write for FaultWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match &mut self.fault {
            Fault::Zero | Fault::Prefix(0) => Ok(0),
            Fault::Flush => self.inner.write(bytes),
            Fault::Prefix(remaining) => {
                let length = bytes.len().min(*remaining);
                let written = self.inner.write(&bytes[..length])?;
                *remaining -= written;
                Ok(written)
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.fault {
            Fault::Flush => Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "controlled writer flush",
            )),
            _ => self.inner.flush(),
        }
    }
}

pub fn constructed_error(input: &Value) -> io::Error {
    match input["constructor"].as_str().unwrap() {
        "raw-os-error" => io::Error::from_raw_os_error(input["value"].as_i64().unwrap() as i32),
        "kind" => io::Error::new(
            match input["value"].as_str().unwrap() {
                "InvalidData" => io::ErrorKind::InvalidData,
                "TimedOut" => io::ErrorKind::TimedOut,
                _ => unreachable!("reviewed error constructor"),
            },
            "controlled cause",
        ),
        _ => unreachable!("reviewed error recipe"),
    }
}

use super::PreparedRoot;
use serde_json::Value;
use std::{
    ffi::OsString,
    fs, io,
    os::unix::{
        ffi::OsStringExt,
        fs::{PermissionsExt, symlink},
    },
    path::PathBuf,
    process::Command,
};

pub struct Input {
    pub owner: PreparedRoot,
    pub root: PathBuf,
}

impl Input {
    pub fn new(base: &Value, case: &Value) -> io::Result<Self> {
        let owner = PreparedRoot::new()?;
        let mut root = owner.path.join("measurement");
        fs::create_dir(&root)?;
        fs::create_dir(root.join("launches"))?;
        fs::create_dir(root.join("exits"))?;
        let mut context = base.clone();
        if let Some(set) = case["set"].as_object() {
            context.as_object_mut().unwrap().extend(set.clone());
        }
        if let Some(key) = case["remove"].as_str() {
            context.as_object_mut().unwrap().remove(key);
        }
        let mut bytes = case["raw"]
            .as_str()
            .map_or_else(|| context.to_string(), str::to_owned)
            .into_bytes();
        if let Some(count) = case["context_padding_bytes"].as_u64() {
            bytes.resize(count as usize, b' ');
        }
        fs::write(root.join("context.json"), bytes)?;
        match case["prepare"].as_str().unwrap_or("") {
            "missing-root" => fs::remove_dir_all(&root)?,
            "root-file" => {
                fs::remove_dir_all(&root)?;
                fs::write(&root, b"file")?;
            }
            "root-link" => {
                let link = owner.path.join("root-link");
                symlink(&root, &link)?;
                root = link;
            }
            "missing-context" => fs::remove_file(root.join("context.json"))?,
            "context-directory" => {
                fs::remove_file(root.join("context.json"))?;
                fs::create_dir(root.join("context.json"))?;
            }
            "context-link" => {
                let target = owner.path.join("context-target");
                fs::rename(root.join("context.json"), &target)?;
                symlink(target, root.join("context.json"))?;
            }
            "missing-launches" | "missing-exits" => {
                let name = case["prepare"]
                    .as_str()
                    .unwrap()
                    .strip_prefix("missing-")
                    .unwrap();
                fs::remove_dir(root.join(name))?;
            }
            "launches-file" | "exits-file" | "launches-link" | "exits-link" => {
                let recipe = case["prepare"].as_str().unwrap();
                let (name, mode) = recipe.split_once('-').unwrap();
                fs::remove_dir(root.join(name))?;
                if mode == "file" {
                    fs::write(root.join(name), b"file")?;
                } else {
                    let target = owner.path.join("directory-target");
                    fs::create_dir(&target)?;
                    symlink(target, root.join(name))?;
                }
            }
            "" | "context-oversized" => {}
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "unknown measurement preparation",
                ));
            }
        }
        Ok(Self { owner, root })
    }

    pub fn command(&self, case: &Value) -> io::Result<Command> {
        let mode = case["program_mode"].as_str().unwrap_or("");
        let mut program = OsString::from(case["program"].as_str().unwrap_or("/usr/bin/true"));
        let mut cwd = self.owner.path.clone();
        if case["cwd"] == "subdir" || mode == "relative" {
            cwd.push("subdir");
            fs::create_dir(&cwd)?;
        } else if case["cwd"] == "missing" {
            cwd.push("missing");
        }
        match mode {
            "relative" => {
                symlink("/usr/bin/true", cwd.join("tool"))?;
                program = "./tool".into();
            }
            "missing" => program = self.owner.path.join("missing-tool").into_os_string(),
            "directory" => {
                let path = self.owner.path.join("tool-directory");
                fs::create_dir(&path)?;
                program = path.into_os_string();
            }
            "not-executable" | "oversized" | "exact-limit" => {
                let path = self.owner.path.join("tool-file");
                let file = fs::File::create_new(&path)?;
                let count = match mode {
                    "oversized" => 67108865,
                    "exact-limit" => 67108864,
                    _ => 0,
                };
                file.set_len(count)?;
                fs::set_permissions(
                    &path,
                    fs::Permissions::from_mode(if mode == "not-executable" {
                        0o644
                    } else {
                        0o755
                    }),
                )?;
                program = path.into_os_string();
            }
            "path" | "path-link" | "path-empty" | "path-removed" | "path-access-failure" => {
                program = "tool".into()
            }
            "" => {}
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "unknown measurement executable",
                ));
            }
        }
        let mut command = Command::new(program);
        command.current_dir(cwd);
        match mode {
            "path" | "path-link" => {
                let bin = self.owner.path.join("bin");
                fs::create_dir(&bin)?;
                symlink("/usr/bin/true", bin.join("tool"))?;
                let selected = if mode == "path-link" {
                    let link = self.owner.path.join("bin-link");
                    symlink(&bin, &link)?;
                    link
                } else {
                    bin
                };
                command.env("PATH", selected);
            }
            "path-access-failure" => {
                let mut paths = Vec::new();
                for (name, key) in [("first-bin", "first_mode"), ("second-bin", "second_mode")] {
                    let bin = self.owner.path.join(name);
                    fs::create_dir(&bin)?;
                    fs::copy("/usr/bin/true", bin.join("tool"))?;
                    fs::set_permissions(
                        bin.join("tool"),
                        fs::Permissions::from_mode(case[key].as_u64().unwrap() as u32),
                    )?;
                    paths.push(bin);
                }
                command.env(
                    "PATH",
                    std::env::join_paths(paths).map_err(io::Error::other)?,
                );
            }
            "path-empty" => {
                command.env("PATH", "");
            }
            "path-removed" => {
                command.env_remove("PATH");
            }
            _ => {}
        }
        if let Some(arguments) = case["arguments"].as_array() {
            for argument in arguments {
                command.arg(argument.as_str().unwrap());
            }
        }
        if let Some(bytes) = case["argument_bytes"].as_array() {
            command.arg(OsString::from_vec(
                bytes.iter().map(|n| n.as_u64().unwrap() as u8).collect(),
            ));
        }
        if let Some(key) = case["remove_environment"].as_str() {
            command.env_remove(key);
        }
        Ok(command)
    }
}

/// These fixture Commands have only default streams; copy their explicit configuration.
pub fn bind_selected(requested: &Command, selected: &std::path::Path) -> Command {
    let mut bound = Command::new(selected);
    bound.args(requested.get_args());
    if let Some(cwd) = requested.get_current_dir() {
        bound.current_dir(cwd);
    }
    for (key, value) in requested.get_envs() {
        if let Some(value) = value {
            bound.env(key, value);
        } else {
            bound.env_remove(key);
        }
    }
    bound
}
