use std::{
    ffi::OsString,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

use crate::{
    Action, emit,
    inventory::{self, InventoryError},
    process::{self, Observation},
    report_command::read_input,
    strict_json, workspace,
};

pub const USAGE: &str = "usage: arboresce-source-inventory <workspace> <roles.json>\n";
pub const OUTPUT_FAILURE: &str = "source inventory output failed\n";
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> io::Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "arboresce-inventory-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        Ok(Self(path))
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn metadata(root: &Path, trace: &mut impl Write) -> io::Result<Output> {
    let scratch = Scratch::new()?;
    let stdout = scratch.0.join("stdout");
    let stderr = scratch.0.join("stderr");
    let mut command = Command::new("cargo");
    command
        .args(Action::Architecture.cargo_args())
        .current_dir(root)
        .stdout(Stdio::from(fs::File::create_new(&stdout)?))
        .stderr(Stdio::from(fs::File::create_new(&stderr)?));
    let started = Instant::now();
    let mut child = command.spawn()?;
    process::observe(
        &mut child,
        &stdout,
        &stderr,
        Observation::default(),
        started,
        trace,
    )
}

fn output_failed(stderr: &mut impl Write) -> u8 {
    emit(stderr, OUTPUT_FAILURE.as_bytes());
    1
}

pub fn run(args: &[OsString], stdout: &mut impl Write, stderr: &mut impl Write) -> u8 {
    if let [flag] = args
        && matches!(flag.to_str(), Some("help" | "--help" | "-h"))
    {
        return if emit(stdout, USAGE.as_bytes()) {
            0
        } else {
            output_failed(stderr)
        };
    }
    let [root, profile] = args else {
        return if emit(stderr, USAGE.as_bytes()) { 2 } else { 1 };
    };
    let input = (|| {
        let root = Path::new(root.to_str().ok_or(InventoryError::Input)?)
            .canonicalize()
            .map_err(|_| InventoryError::Input)?;
        if !root.is_dir() {
            return Err(InventoryError::Input);
        }
        let profile = read_input(Path::new(profile)).map_err(|_| InventoryError::Input)?;
        strict_json::parse(&profile).ok_or(InventoryError::Input)?;
        Ok((root, profile))
    })();
    let (root, profile) = match input {
        Ok(value) => value,
        Err(error) => {
            emit(
                stderr,
                format!("arboresce-source-inventory: {error}\n").as_bytes(),
            );
            return 1;
        }
    };
    let mut trace = Vec::new();
    let captured = match metadata(&root, &mut trace) {
        Ok(output) => output,
        Err(_) => {
            emit(stderr, &trace);
            emit(
                stderr,
                b"arboresce-source-inventory: cargo metadata failed\n",
            );
            return 1;
        }
    };
    if !emit(stderr, &captured.stderr) {
        return output_failed(stderr);
    }
    if !captured.status.success() {
        emit(stderr, &trace);
        emit(
            stderr,
            b"arboresce-source-inventory: cargo metadata failed\n",
        );
        return 1;
    }
    let result = (|| {
        let metadata = std::str::from_utf8(&captured.stdout).map_err(|_| InventoryError::Input)?;
        workspace::check_workspace(&root, metadata).map_err(|_| InventoryError::Targets)?;
        inventory::snapshot(&root, &profile, metadata)
    })();
    match result {
        Ok(mut value) => {
            value["metadata_trace"] = String::from_utf8_lossy(&trace).as_ref().into();
            if emit(stdout, format!("{value}\n").as_bytes()) {
                0
            } else {
                emit(stderr, &trace);
                output_failed(stderr)
            }
        }
        Err(error) => {
            emit(stderr, &trace);
            emit(
                stderr,
                format!("arboresce-source-inventory: {error}\n").as_bytes(),
            );
            1
        }
    }
}
