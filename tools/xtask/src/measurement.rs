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
    match fs::File::create_new(path) {
        Ok(mut file) => file.write_all(&bytes).and_then(|()| file.flush()),
        Err(error) if repeat && error.kind() == io::ErrorKind::AlreadyExists => {
            if regular(path, FILE_LIMIT)? == bytes {
                Ok(())
            } else {
                Err(error)
            }
        }
        Err(error) => Err(error),
    }
}

#[derive(Clone)]
pub struct Request {
    root: PathBuf,
    identity: Value,
}

impl Request {
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
        self.current()?;
        let cwd = effective_cwd(command)?;
        let program = command.get_program().to_owned();
        if !program.as_bytes().contains(&b'/') {
            return Err(invalid());
        }
        let executable = cwd.join(&program).canonicalize()?;
        if fs::metadata(&executable)?.permissions().mode() & 0o111 == 0 {
            return Err(invalid());
        }
        let bytes = regular(&executable, FILE_LIMIT)?;
        let fields = json!({
            "executable":executable.as_os_str().as_bytes(), "executable_sha256":sha256(&bytes),
            "program":program.as_bytes(), "arguments":command.get_args().map(|a|a.as_bytes()).collect::<Vec<_>>(),
            "cwd":cwd.as_os_str().as_bytes(),
        });
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
    fields: Value,
}

impl PreparedLaunch {
    /// Record the actual Child spawned from the prepared Command unchanged.
    /// A failed receipt kills/reaps only that Child before returning failure.
    pub fn record(self, child: &mut Child) -> io::Result<()> {
        let result = (|| {
            self.request.current()?;
            let mut record = self.request.common(child.id());
            record["version"] = "native-launch-v1".into();
            record
                .as_object_mut()
                .ok_or_else(invalid)?
                .extend(self.fields.as_object().ok_or_else(invalid)?.clone());
            write_record(
                &self
                    .request
                    .root
                    .join("launches")
                    .join(format!("{}.json", child.id())),
                &record,
                false,
            )
        })();
        if result.is_err() {
            process::terminate(child)?;
        }
        result
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
    let prepared = active()?.map(|r| r.prepare(command)).transpose()?;
    let mut child = command.spawn()?;
    if let Some(prepared) = prepared {
        prepared.record(&mut child)?;
    }
    Ok(child)
}

pub fn complete(child: &mut Child) -> io::Result<()> {
    if let Some(request) = active()? {
        request.complete(child)?;
    }
    Ok(())
}
