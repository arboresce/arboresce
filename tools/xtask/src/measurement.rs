//! Explicit, immutable receipts for owned native processes in one selected run.

use std::{
    ffi::{OsStr, OsString},
    fs,
    io::{self, Read, Write},
    os::unix::{ffi::OsStrExt, fs::PermissionsExt, process::ExitStatusExt},
    path::{Path, PathBuf},
    process::{Child, Command},
};

use serde_json::{Value, json};

use crate::{inventory::sha256, process, strict_json};

pub const ENV: &str = "ARBORESCE_NATIVE_CONTEXT";
const CONTEXT_LIMIT: u64 = 32768;
const FILE_LIMIT: u64 = 67108864;

fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "invalid native measurement input",
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WritePhase {
    Create,
    Write,
    Flush,
}

impl WritePhase {
    pub fn name(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Write => "write",
            Self::Flush => "flush",
        }
    }
}

#[derive(Debug)]
pub struct WriteFailure {
    phase: WritePhase,
    error: io::Error,
}

impl WriteFailure {
    pub fn phase(&self) -> WritePhase {
        self.phase
    }

    pub fn error(&self) -> &io::Error {
        &self.error
    }

    pub fn into_error(self) -> io::Error {
        self.error
    }
}

/// The caller owns the writer; this boundary does not attest an OS write.
pub fn write_payload(writer: &mut impl Write, bytes: &[u8]) -> Result<(), WriteFailure> {
    writer.write_all(bytes).map_err(|error| WriteFailure {
        phase: WritePhase::Write,
        error,
    })?;
    writer.flush().map_err(|error| WriteFailure {
        phase: WritePhase::Flush,
        error,
    })
}

fn write_exclusive(path: &Path, bytes: &[u8]) -> Result<(), WriteFailure> {
    let mut file = fs::File::create_new(path).map_err(|error| WriteFailure {
        phase: WritePhase::Create,
        error,
    })?;
    write_payload(&mut file, bytes)
}

pub fn io_cause(error: &io::Error) -> Value {
    json!({
        "error_kind":format!("{:?}", error.kind()),
        "raw_os_error":error.raw_os_error(),
    })
}

/// Error composition is separate from evidence that a child actually ran.
#[derive(Debug)]
pub struct FailureCauses {
    primary: io::Error,
    cleanup: Option<io::Result<std::process::ExitStatus>>,
}

impl FailureCauses {
    pub fn new(primary: io::Error, cleanup: Option<io::Result<std::process::ExitStatus>>) -> Self {
        Self { primary, cleanup }
    }

    pub fn primary(&self) -> Value {
        io_cause(&self.primary)
    }

    pub fn cleanup(&self) -> Value {
        match &self.cleanup {
            None => Value::Null,
            Some(Ok(status)) => json!({
                "state":"reaped", "exit":status.code(), "signal":status.signal(),
                "error_kind":null, "raw_os_error":null,
            }),
            Some(Err(error)) => json!({
                "state":"failed", "exit":null, "signal":null,
                "error_kind":format!("{:?}",error.kind()),
                "raw_os_error":error.raw_os_error(),
            }),
        }
    }

    /// Existing callers retain the original cleanup-error return convention.
    pub fn into_legacy_error(self) -> io::Error {
        match self.cleanup {
            Some(Err(error)) => error,
            _ => self.primary,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticScope {
    ActualRuntime,
    ControlledWriterWithActualCleanup,
    RepresentationUnit,
}

impl DiagnosticScope {
    fn name(self) -> &'static str {
        match self {
            Self::ActualRuntime => "actual_runtime",
            Self::ControlledWriterWithActualCleanup => "controlled_writer_with_actual_cleanup",
            Self::RepresentationUnit => "representation_unit",
        }
    }
}

#[derive(Debug)]
pub struct LaunchFailure {
    stage: &'static str,
    parent_pid: u32,
    child_pid: Option<u32>,
    request: Option<Value>,
    receipt_path: Option<PathBuf>,
    causes: FailureCauses,
}

impl LaunchFailure {
    fn before(stage: &'static str, error: io::Error, request: Option<&Request>) -> Self {
        Self {
            stage,
            parent_pid: std::process::id(),
            child_pid: None,
            request: request.map(|r| r.identity.clone()),
            receipt_path: None,
            causes: FailureCauses::new(error, None),
        }
    }

    pub fn observation(&self) -> Value {
        json!({
            "stage":self.stage,
            "error_kind":format!("{:?}",self.causes.primary.kind()),
            "raw_os_error":self.causes.primary.raw_os_error(),
            "parent_pid":self.parent_pid,
            "child_pid":self.child_pid,
            "request":self.request,
            "receipt_path":self.receipt_path.as_ref().map(|p|p.as_os_str().as_bytes()),
            "cleanup":self.causes.cleanup(),
        })
    }

    pub fn envelope(
        &self,
        command_intent: &Path,
        command_bytes: &[u8],
        scope: DiagnosticScope,
    ) -> Value {
        json!({
            "version":"native-launch-failure-v1",
            "scope":scope.name(),
            "command":{"path":command_intent.as_os_str().as_bytes(),"sha256":sha256(command_bytes)},
            "failure":self.observation(),
            "retention_error":null,
        })
    }

    pub fn into_legacy_error(self) -> io::Error {
        self.causes.into_legacy_error()
    }
}

pub const DIAGNOSTIC_LIMIT: usize = 65536;

#[derive(Debug)]
pub struct RetentionFailure {
    stage: &'static str,
    error: io::Error,
}

impl RetentionFailure {
    pub fn stage(&self) -> &'static str {
        self.stage
    }

    pub fn error(&self) -> &io::Error {
        &self.error
    }

    fn trace(error: WriteFailure) -> Self {
        Self {
            stage: match error.phase {
                WritePhase::Create => "create",
                WritePhase::Write => "trace_write",
                WritePhase::Flush => "trace_flush",
            },
            error: error.error,
        }
    }
}

fn raw_path_hash(value: &Value) -> io::Result<Value> {
    if value.is_null() {
        return Ok(Value::Null);
    }
    let bytes = value
        .as_array()
        .ok_or_else(invalid)?
        .iter()
        .map(|v| {
            v.as_u64()
                .and_then(|n| u8::try_from(n).ok())
                .ok_or_else(invalid)
        })
        .collect::<io::Result<Vec<_>>>()?;
    Ok(json!(sha256(&bytes)))
}

fn fallback(frame: &Value, bytes: &[u8], error: &RetentionFailure) -> io::Result<Value> {
    let mut failure = frame["failure"].as_object().ok_or_else(invalid)?.clone();
    let receipt = failure.remove("receipt_path").ok_or_else(invalid)?;
    failure.insert("receipt_path_sha256".into(), raw_path_hash(&receipt)?);
    if !failure.get("request").ok_or_else(invalid)?.is_null() {
        let mut request = failure["request"].as_object().ok_or_else(invalid)?.clone();
        let path = request.remove("context_path").ok_or_else(invalid)?;
        request.insert("context_path_sha256".into(), raw_path_hash(&path)?);
        failure.insert("request".into(), Value::Object(request));
    }
    Ok(json!({
        "version":"native-launch-failure-fallback-v1",
        "scope":frame["scope"],
        "command":{"path_sha256":raw_path_hash(&frame["command"]["path"])?,"sha256":frame["command"]["sha256"]},
        "diagnostic_sha256":sha256(bytes), "diagnostic_bytes":bytes.len(),
        "failure":failure,
        "retention_error":{"stage":error.stage,"error_kind":format!("{:?}",error.error.kind()),"raw_os_error":error.error.raw_os_error()},
    }))
}

fn retain_fallback(
    frame: &Value,
    bytes: &[u8],
    trace: &mut impl Write,
    error: RetentionFailure,
) -> Result<(), RetentionFailure> {
    let value = fallback(frame, bytes, &error).map_err(|cause| RetentionFailure {
        stage: "fallback",
        error: cause,
    })?;
    let compact = format!("{value}\n").into_bytes();
    if compact.len() > DIAGNOSTIC_LIMIT {
        return Err(RetentionFailure {
            stage: "fallback",
            error: invalid(),
        });
    }
    write_payload(trace, &compact).map_err(RetentionFailure::trace)?;
    Err(error)
}

/// Retain a library-produced frame or an explicitly scoped representation unit.
pub fn retain(
    frame: &Value,
    artifact: &Path,
    trace: &mut impl Write,
) -> Result<(), RetentionFailure> {
    retain_with_writer(frame, artifact, trace, std::convert::identity)
}

/// Exclusive file creation precedes the caller-owned writer boundary.
pub fn retain_with_writer<W: Write>(
    frame: &Value,
    artifact: &Path,
    trace: &mut impl Write,
    wrap: impl FnOnce(fs::File) -> W,
) -> Result<(), RetentionFailure> {
    let bytes = format!("{frame}\n").into_bytes();
    if bytes.len() > DIAGNOSTIC_LIMIT {
        return retain_fallback(
            frame,
            &bytes,
            trace,
            RetentionFailure {
                stage: "size",
                error: invalid(),
            },
        );
    }
    let result = fs::File::create_new(artifact)
        .map_err(|error| WriteFailure {
            phase: WritePhase::Create,
            error,
        })
        .and_then(|file| write_payload(&mut wrap(file), &bytes));
    if let Err(error) = result {
        return retain_fallback(
            frame,
            &bytes,
            trace,
            RetentionFailure {
                stage: error.phase.name(),
                error: error.error,
            },
        );
    }
    write_payload(trace, &bytes).map_err(RetentionFailure::trace)
}

fn regular(path: &Path, limit: u64) -> io::Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err(invalid());
    }
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(limit + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(invalid());
    }
    Ok(bytes)
}

fn directory(path: &Path) -> io::Result<()> {
    if !fs::symlink_metadata(path)?.is_dir() {
        return Err(invalid());
    }
    Ok(())
}

fn parsed(bytes: &[u8]) -> io::Result<Value> {
    let text = std::str::from_utf8(bytes).map_err(|_| invalid())?;
    strict_json::parse(text).ok_or_else(invalid)
}

fn write_record(path: &Path, record: &Value, repeat: bool) -> io::Result<()> {
    let bytes = format!("{record}\n").into_bytes();
    match write_exclusive(path, &bytes) {
        Ok(()) => Ok(()),
        Err(failure)
            if repeat
                && failure.phase == WritePhase::Create
                && failure.error.kind() == io::ErrorKind::AlreadyExists =>
        {
            let error = failure.error;
            if regular(path, FILE_LIMIT)? == bytes {
                Ok(())
            } else {
                Err(error)
            }
        }
        Err(failure) => Err(failure.error),
    }
}

#[derive(Clone)]
pub struct Request {
    root: PathBuf,
    identity: Value,
}

impl Request {
    pub fn open_diagnosed(root: &Path) -> Result<Self, LaunchFailure> {
        Self::open(root).map_err(|error| LaunchFailure::before("select_context", error, None))
    }

    pub fn open(root: &Path) -> io::Result<Self> {
        directory(root)?;
        let root = root.canonicalize()?;
        directory(&root.join("launches"))?;
        directory(&root.join("exits"))?;
        let bytes = regular(&root.join("context.json"), CONTEXT_LIMIT)?;
        let context = parsed(&bytes)?;
        let object = context.as_object().ok_or_else(invalid)?;
        if object.len() != 3 || context["version"] != "native-measurement-v1" {
            return Err(invalid());
        }
        for (key, length) in [("run_id", 32), ("source_sha256", 64)] {
            let value = context[key].as_str().ok_or_else(invalid)?;
            if value.len() != length
                || !value
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err(invalid());
            }
        }
        let identity = json!({
            "context_path":root.as_os_str().as_bytes(),
            "context_sha256":sha256(&bytes),
            "run_id":context["run_id"], "source_sha256":context["source_sha256"],
        });
        Ok(Self { root, identity })
    }

    fn current(&self) -> io::Result<()> {
        if Self::open(&self.root)?.identity != self.identity {
            return Err(invalid());
        }
        Ok(())
    }

    fn common(&self, pid: u32) -> Value {
        let mut record = self.identity.clone();
        record["pid"] = pid.into();
        record["parent_pid"] = std::process::id().into();
        record
    }

    /// Prepare and spawn must use the same unchanged Command, without argv0 overrides.
    pub fn prepare(&self, command: &mut Command) -> io::Result<PreparedLaunch> {
        self.prepare_diagnosed(command)
            .map_err(LaunchFailure::into_legacy_error)
    }

    pub fn prepare_diagnosed(
        &self,
        command: &mut Command,
    ) -> Result<PreparedLaunch, LaunchFailure> {
        let failure = |stage, error| LaunchFailure::before(stage, error, Some(self));
        self.current().map_err(|e| failure("prepare_context", e))?;
        let cwd = effective_cwd(command).map_err(|e| failure("prepare_cwd", e))?;
        let program = command.get_program().to_owned();
        if !program.as_bytes().contains(&b'/') {
            return Err(failure("prepare_program", invalid()));
        }
        let executable = cwd
            .join(&program)
            .canonicalize()
            .map_err(|e| failure("prepare_executable", e))?;
        if fs::metadata(&executable)
            .map_err(|e| failure("prepare_metadata", e))?
            .permissions()
            .mode()
            & 0o111
            == 0
        {
            return Err(failure("prepare_permissions", invalid()));
        }
        let bytes = regular(&executable, FILE_LIMIT).map_err(|e| failure("prepare_bytes", e))?;
        let fields = serde_json::Map::from_iter([
            (
                "executable".into(),
                json!(executable.as_os_str().as_bytes()),
            ),
            ("executable_sha256".into(), json!(sha256(&bytes))),
            ("program".into(), json!(program.as_bytes())),
            (
                "arguments".into(),
                json!(command.get_args().map(|a| a.as_bytes()).collect::<Vec<_>>()),
            ),
            ("cwd".into(), json!(cwd.as_os_str().as_bytes())),
        ]);
        Ok(PreparedLaunch {
            request: self.clone(),
            fields,
        })
    }

    /// Derive status from this actual Child; callers cannot supply a synthetic exit.
    pub fn complete(&self, child: &mut Child) -> io::Result<()> {
        self.current()?;
        let status = child.try_wait()?.ok_or_else(|| {
            io::Error::new(io::ErrorKind::WouldBlock, "native child still running")
        })?;
        let common = self.common(child.id());
        let launch_bytes = regular(
            &self
                .root
                .join("launches")
                .join(format!("{}.json", child.id())),
            FILE_LIMIT,
        )?;
        let launch = parsed(&launch_bytes)?;
        if launch["version"] != "native-launch-v1"
            || common
                .as_object()
                .ok_or_else(invalid)?
                .iter()
                .any(|(key, value)| launch.get(key) != Some(value))
        {
            return Err(invalid());
        }
        let mut record = common;
        record["version"] = "native-exit-v1".into();
        record["launch_sha256"] = sha256(&launch_bytes).into();
        record["exit"] = json!(status.code());
        record["signal"] = json!(status.signal());
        write_record(
            &self.root.join("exits").join(format!("{}.json", child.id())),
            &record,
            true,
        )
    }
}

pub struct PreparedLaunch {
    request: Request,
    fields: serde_json::Map<String, Value>,
}

impl PreparedLaunch {
    /// Record the actual Child spawned from the prepared Command unchanged.
    /// A failed receipt kills/reaps only that Child before returning failure.
    pub fn record(self, child: &mut Child) -> io::Result<()> {
        self.record_diagnosed(child)
            .map_err(LaunchFailure::into_legacy_error)
    }

    fn failed(&self, child: &mut Child, stage: &'static str, error: io::Error) -> LaunchFailure {
        LaunchFailure {
            stage,
            parent_pid: std::process::id(),
            child_pid: Some(child.id()),
            request: Some(self.request.identity.clone()),
            receipt_path: Some(
                self.request
                    .root
                    .join("launches")
                    .join(format!("{}.json", child.id())),
            ),
            causes: FailureCauses::new(error, Some(process::terminate(child))),
        }
    }

    /// Failure-only writer boundary; it cannot attest a receipt or successful launch.
    pub fn record_failure(self, child: &mut Child, failure: WriteFailure) -> LaunchFailure {
        let stage = match failure.phase {
            WritePhase::Create => "record_create",
            WritePhase::Write => "record_write",
            WritePhase::Flush => "record_flush",
        };
        self.failed(child, stage, failure.error)
    }

    pub fn record_diagnosed(self, child: &mut Child) -> Result<(), LaunchFailure> {
        if let Err(error) = self.request.current() {
            return Err(self.failed(child, "record_context", error));
        }
        let mut record = self.request.common(child.id());
        record["version"] = "native-launch-v1".into();
        for (key, value) in &self.fields {
            record[key] = value.clone();
        }
        let path = self
            .request
            .root
            .join("launches")
            .join(format!("{}.json", child.id()));
        write_exclusive(&path, format!("{record}\n").as_bytes())
            .map_err(|failure| self.record_failure(child, failure))
    }

    /// The prepared Command must remain unchanged; owned input files may have changed.
    pub fn spawn_diagnosed(self, command: &mut Command) -> Result<Child, LaunchFailure> {
        let mut child = command
            .spawn()
            .map_err(|e| LaunchFailure::before("spawn", e, Some(&self.request)))?;
        self.record_diagnosed(&mut child)?;
        Ok(child)
    }
}

fn active() -> io::Result<Option<Request>> {
    std::env::var_os(ENV)
        .map(|path| Request::open(Path::new(&path)))
        .transpose()
}

fn effective_cwd(command: &Command) -> io::Result<PathBuf> {
    let parent = std::env::current_dir()?;
    let cwd = parent
        .join(command.get_current_dir().unwrap_or(&parent))
        .canonicalize()?;
    directory(&cwd)?;
    Ok(cwd)
}

/// Select before constructing the spawn Command; never rely on exec PATH fallback.
/// The returned absolute spelling preserves a selected tool symlink's basename.
pub fn selected_program(command: &Command) -> io::Result<PathBuf> {
    let cwd = effective_cwd(command)?;
    let program = command.get_program();
    if program.as_bytes().contains(&b'/') {
        return Ok(cwd.join(program));
    }
    let path = command
        .get_envs()
        .find(|(key, _)| *key == "PATH")
        .map_or_else(
            || std::env::var_os("PATH"),
            |(_, value)| value.map(OsString::from),
        );
    let path = path
        .filter(|s| !s.is_empty())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "native executable PATH missing"))?;
    std::env::split_paths(&path)
        .map(|p| cwd.join(p).join(program))
        .find(|p| fs::metadata(p).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "native executable missing"))
}

/// Existing callers keep ordinary command construction unless measurement is explicit.
pub fn command(program: impl AsRef<OsStr>, root: Option<&Path>) -> io::Result<Command> {
    let mut requested = Command::new(program.as_ref());
    if let Some(root) = root {
        requested.current_dir(root);
    }
    if active()?.is_none() {
        return Ok(requested);
    }
    let mut bound = Command::new(selected_program(&requested)?);
    if let Some(root) = root {
        bound.current_dir(root);
    }
    Ok(bound)
}

/// Default operation has no measurement side effects.
pub fn spawn(command: &mut Command) -> io::Result<Child> {
    spawn_diagnosed(command, None).map_err(LaunchFailure::into_legacy_error)
}

/// An explicit nested request owns this invocation instead of ambient selection.
pub fn spawn_with(command: &mut Command, request: &Request) -> io::Result<Child> {
    spawn_diagnosed(command, Some(request)).map_err(LaunchFailure::into_legacy_error)
}

/// An explicit request takes precedence over ambient selection, including on failure.
pub fn spawn_diagnosed(
    command: &mut Command,
    request: Option<&Request>,
) -> Result<Child, LaunchFailure> {
    let ambient;
    let selected = match request {
        Some(request) => Some(request),
        None => {
            ambient = active().map_err(|e| LaunchFailure::before("select_context", e, None))?;
            ambient.as_ref()
        }
    };
    match selected {
        Some(request) => request.prepare_diagnosed(command)?.spawn_diagnosed(command),
        None => command
            .spawn()
            .map_err(|e| LaunchFailure::before("spawn", e, None)),
    }
}

pub fn complete(child: &mut Child) -> io::Result<()> {
    if let Some(request) = active()? {
        request.complete(child)?;
    }
    Ok(())
}
