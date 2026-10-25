//! Explicit installed-tool identity, bounded physical input and outer admission.

use std::{
    collections::BTreeSet,
    fs,
    io::{Read, Seek, SeekFrom},
    os::unix::{ffi::OsStrExt, fs::MetadataExt},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::{
    collection_inputs::{CollectError, Result},
    inventory, measurement, strict_json,
};

pub const PROFILE: &str = "rust-1.99.0-macos-arm64-default-v1";
pub const TOOLS: [&str; 15] = [
    "cargo",
    "rustc",
    "rustdoc",
    "cargo-fmt",
    "rustfmt",
    "cargo-clippy",
    "clippy-driver",
    "cargo-llvm-cov",
    "llvm-cov",
    "llvm-profdata",
    "true",
    "false",
    "echo",
    "sleep",
    "mkfifo",
];
pub const ENVIRONMENT: [&str; 22] = [
    "ARBORESCE_NATIVE_CONTEXT",
    "LLVM_PROFILE_FILE",
    "RUSTFLAGS",
    "RUSTDOCFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "CARGO_ENCODED_RUSTDOCFLAGS",
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "CARGO_BUILD_TARGET",
    "CARGO_LLVM_COV_TARGET_DIR",
    "CARGO_LLVM_COV_BUILD_DIR",
    "CARGO_LLVM_COV",
    "CARGO_LLVM_COV_SHOW_ENV",
    "__CARGO_LLVM_COV_RUSTC_WRAPPER",
    "__CARGO_LLVM_COV_RUSTC_WRAPPER_RUSTFLAGS",
    "__CARGO_LLVM_COV_RUSTC_WRAPPER_CRATE_NAMES",
    "RUSTC",
    "RUSTDOC",
    "RUSTFMT",
    "CARGO_BUILD_RUSTC",
    "CARGO_BUILD_RUSTDOC",
    "CLIPPY_DRIVER_PATH",
];

#[derive(Clone, Copy)]
pub struct Limits {
    pub files: usize,
    pub per_file: u64,
    pub total: u64,
    pub depth: usize,
}

pub const RESOURCES: Limits = Limits {
    files: 16384,
    per_file: 1 << 30,
    total: 4 << 30,
    depth: 32,
};
pub const PROFILES: Limits = Limits {
    files: 4096,
    per_file: 64 << 20,
    total: 4 << 30,
    depth: 0,
};

fn absolute(path: &str) -> bool {
    path.strip_prefix('/')
        .is_some_and(|p| inventory::portable_path(Path::new(p)).is_ok())
}

fn hex(value: &Value) -> bool {
    value.as_str().is_some_and(|s| {
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}

pub fn check_time(deadline: Instant) -> Result<()> {
    if Instant::now() >= deadline {
        Err(CollectError::Intake)
    } else {
        Ok(())
    }
}

fn regular(path: &Path, limit: u64, deadline: Instant, capture: bool) -> Result<(Value, Vec<u8>)> {
    let bad = CollectError::Intake;
    check_time(deadline)?;
    let absolute = std::env::current_dir().map_err(|_| bad)?.join(path);
    for ancestor in absolute.ancestors().skip(1) {
        if !fs::symlink_metadata(ancestor).is_ok_and(|m| m.is_dir()) {
            return Err(bad);
        }
    }
    let before = fs::symlink_metadata(path).map_err(|_| bad)?;
    if !before.is_file() || before.len() > limit {
        return Err(bad);
    }
    let mut file = fs::File::open(path).map_err(|_| bad)?;
    let opened = file.metadata().map_err(|_| bad)?;
    if !opened.is_file() || opened.dev() != before.dev() || opened.ino() != before.ino() {
        return Err(bad);
    }
    let mut digest = Sha256::new();
    let mut size = 0u64;
    let mut buffer = [0u8; 65536];
    let mut bytes = Vec::new();
    loop {
        check_time(deadline)?;
        let count = file.read(&mut buffer).map_err(|_| bad)?;
        if count == 0 {
            break;
        }
        size = size.checked_add(count as u64).ok_or(bad)?;
        if size > limit {
            return Err(bad);
        }
        digest.update(&buffer[..count]);
        if capture {
            bytes.extend_from_slice(&buffer[..count]);
        }
    }
    let after = fs::symlink_metadata(path).map_err(|_| bad)?;
    if !after.is_file()
        || size != before.len()
        || after.len() != before.len()
        || after.dev() != before.dev()
        || after.ino() != before.ino()
        || after.modified().map_err(|_| bad)? != before.modified().map_err(|_| bad)?
    {
        return Err(bad);
    }
    check_time(deadline)?;
    Ok((
        json!({"sha256":format!("{:x}",digest.finalize()),"size":size}),
        bytes,
    ))
}

/// Stream one stable regular descriptor without following symlinked ancestors.
pub fn fingerprint(path: &Path, limit: u64, deadline: Instant) -> Result<Value> {
    regular(path, limit, deadline, false).map(|(identity, _)| identity)
}

pub fn read_bytes(path: &Path, limit: u64, deadline: Instant) -> Result<(Vec<u8>, String)> {
    let (identity, bytes) = regular(path, limit, deadline, true)?;
    Ok((
        bytes,
        identity["sha256"]
            .as_str()
            .ok_or(CollectError::Intake)?
            .to_owned(),
    ))
}

pub fn read_text(path: &Path, limit: u64, deadline: Instant) -> Result<(String, String)> {
    let (bytes, hash) = read_bytes(path, limit, deadline)?;
    Ok((
        String::from_utf8(bytes).map_err(|_| CollectError::Input)?,
        hash,
    ))
}

pub fn unchanged_object(path: &Path, hash: &Value, deadline: Instant) -> Result<()> {
    let actual = fingerprint(path, 64 << 20, deadline).map_err(|_| CollectError::Changed)?;
    if actual["sha256"] != *hash {
        return Err(CollectError::Changed);
    }
    Ok(())
}

/// Bind the inventoried resources to the selected compiler's actual runtime.
pub fn runtime(value: &Value, actual_sysroot: &str) -> Result<()> {
    let bad = CollectError::Prepared;
    let root = value["sysroot"].as_str().ok_or(bad)?;
    if actual_sysroot != format!("{root}\n") {
        return Err(bad);
    }
    for name in [
        "cargo",
        "rustc",
        "rustdoc",
        "cargo-fmt",
        "rustfmt",
        "cargo-clippy",
        "clippy-driver",
        "llvm-cov",
        "llvm-profdata",
    ] {
        let directory = if matches!(name, "llvm-cov" | "llvm-profdata") {
            "lib/rustlib/aarch64-apple-darwin/bin"
        } else {
            "bin"
        };
        let expected = format!("{root}/{directory}/{name}");
        if value["tools"][name]["program"] != expected
            || value["tools"][name]["executable"] != expected
        {
            return Err(bad);
        }
    }
    Ok(())
}

fn walk(
    root: &Path,
    directory: &Path,
    depth: usize,
    limits: Limits,
    deadline: Instant,
    files: &mut Map<String, Value>,
    total: &mut u64,
) -> Result<()> {
    let bad = CollectError::Intake;
    check_time(deadline)?;
    if depth > limits.depth {
        return Err(bad);
    }
    for entry in fs::read_dir(root.join(directory)).map_err(|_| bad)? {
        let entry = entry.map_err(|_| bad)?;
        let path = directory.join(entry.file_name());
        let relative = inventory::portable_path(&path).map_err(|_| bad)?.to_owned();
        let kind = entry.file_type().map_err(|_| bad)?;
        if kind.is_dir() {
            walk(root, &path, depth + 1, limits, deadline, files, total)?;
        } else {
            if !kind.is_file() || files.len() >= limits.files {
                return Err(bad);
            }
            let row = fingerprint(&root.join(&path), limits.per_file, deadline)?;
            *total = total
                .checked_add(row["size"].as_u64().ok_or(bad)?)
                .ok_or(bad)?;
            if *total > limits.total || files.insert(relative, row).is_some() {
                return Err(bad);
            }
        }
    }
    Ok(())
}

/// Use the same walker and inclusive limits for real and small adversarial inputs.
pub fn tree(root: &Path, limits: Limits, deadline: Instant) -> Result<Value> {
    if !fs::symlink_metadata(root).is_ok_and(|m| m.is_dir()) {
        return Err(CollectError::Intake);
    }
    let mut files = Map::new();
    walk(root, Path::new(""), 0, limits, deadline, &mut files, &mut 0)?;
    check_time(deadline)?;
    Ok(Value::Object(files))
}

fn identities(value: &Value, limits: Limits, resources: bool) -> Result<()> {
    let bad = CollectError::Prepared;
    let values = value.as_object().ok_or(bad)?;
    if values.len() > limits.files {
        return Err(bad);
    }
    let mut total = 0u64;
    let mut roots = BTreeSet::new();
    for (path, row) in values {
        if (resources && inventory::portable_path(Path::new(path)).is_err())
            || (!resources && !absolute(path))
            || !hex(&row["sha256"])
        {
            return Err(bad);
        }
        let size = row["size"].as_u64().ok_or(bad)?;
        if size > limits.per_file {
            return Err(bad);
        }
        total = total.checked_add(size).ok_or(bad)?;
        if total > limits.total {
            return Err(bad);
        }
        if resources {
            let (root, _) = path.split_once('/').ok_or(bad)?;
            if !["bin", "lib", "libexec"].contains(&root) {
                return Err(bad);
            }
            roots.insert(root);
        }
    }
    if resources && roots != BTreeSet::from(["bin", "lib", "libexec"]) {
        return Err(bad);
    }
    Ok(())
}

/// Validate supplied preparation facts, before reopening the actual installed set.
pub fn validate(value: &Value) -> Result<()> {
    let bad = CollectError::Prepared;
    if value
        .as_object()
        .ok_or(bad)?
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        != BTreeSet::from([
            "version",
            "profile",
            "sysroot",
            "cargo_home",
            "tools",
            "resources",
            "configs",
            "versions",
            "outer",
        ])
        || value["version"] != "native-prepared-tools-v1"
        || value["profile"] != PROFILE
        || !value["sysroot"].as_str().is_some_and(absolute)
        || !value["cargo_home"].as_str().is_some_and(absolute)
    {
        return Err(bad);
    }
    let tools = value["tools"].as_object().ok_or(bad)?;
    if tools.keys().map(String::as_str).collect::<BTreeSet<_>>() != BTreeSet::from(TOOLS) {
        return Err(bad);
    }
    for row in tools.values() {
        if row.as_object().ok_or(bad)?.len() != 3
            || !hex(&row["sha256"])
            || !row["program"].as_str().is_some_and(absolute)
            || !row["executable"].as_str().is_some_and(absolute)
        {
            return Err(bad);
        }
    }
    identities(&value["resources"], RESOURCES, true)?;
    identities(
        &value["configs"],
        Limits {
            files: 4096,
            per_file: 64 << 20,
            total: 256 << 20,
            depth: 32,
        },
        false,
    )?;
    let versions = &value["versions"];
    let llvm = versions["rust_llvm"].as_str().ok_or(bad)?;
    if versions.as_object().ok_or(bad)?.len() != 7
        || versions["rustc"] != "1.99.0"
        || versions["cargo"] != "1.99.0"
        || versions["cargo-llvm-cov"] != "0.9.1"
        || versions["host"] != "aarch64-apple-darwin"
        || versions["llvm-cov"] != llvm
        || versions["llvm-profdata"] != llvm
        || llvm.is_empty()
        || llvm
            .split('.')
            .any(|p| p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(bad);
    }
    if !value["outer"].is_null() {
        let outer = &value["outer"];
        let path = outer["context"].as_str().ok_or(bad)?;
        let environment = outer["environment"].as_object().ok_or(bad)?;
        if outer.as_object().ok_or(bad)?.len() != 2
            || !absolute(path)
            || environment.get(measurement::ENV).and_then(Value::as_str) != Some(path)
            || !environment
                .get("LLVM_PROFILE_FILE")
                .and_then(Value::as_str)
                .is_some_and(absolute)
            || environment
                .iter()
                .any(|(key, v)| !ENVIRONMENT.contains(&key.as_str()) || !v.is_string())
        {
            return Err(bad);
        }
    }
    Ok(())
}

pub fn environment() -> Result<Value> {
    let mut values = Map::new();
    for key in ENVIRONMENT {
        if let Some(value) = std::env::var_os(key) {
            values.insert(
                key.to_owned(),
                value.to_str().ok_or(CollectError::Nesting)?.into(),
            );
        }
    }
    Ok(Value::Object(values))
}

pub fn read_json_until(path: &Path, limit: u64, deadline: Instant) -> Result<(Value, String)> {
    let (raw, hash) = read_text(path, limit, deadline)?;
    let value = strict_json::parse(&raw).ok_or(CollectError::Input)?;
    check_time(deadline)?;
    Ok((value, hash))
}

pub fn read_json(path: &Path, limit: u64) -> Result<(Value, String)> {
    read_json_until(path, limit, Instant::now() + Duration::from_secs(180))
}

/// Validate this running collector's actual outer launch, never its own final exit.
pub fn outer(prepared: &Value) -> Result<()> {
    outer_until(prepared, Instant::now() + Duration::from_secs(180))
}

pub fn outer_until(prepared: &Value, deadline: Instant) -> Result<()> {
    outer_until_with(prepared, deadline, || Ok(()))
}

struct LaunchFrame {
    path: PathBuf,
    file: fs::File,
    identity: fs::Metadata,
}

impl LaunchFrame {
    fn current(&self, deadline: Instant) -> Result<fs::Metadata> {
        let bad = CollectError::Nesting;
        check_time(deadline).map_err(|_| bad)?;
        for ancestor in self.path.ancestors().skip(1) {
            if !fs::symlink_metadata(ancestor).is_ok_and(|metadata| metadata.is_dir()) {
                return Err(bad);
            }
        }
        let path = fs::symlink_metadata(&self.path).map_err(|_| bad)?;
        let file = self.file.metadata().map_err(|_| bad)?;
        if !path.is_file()
            || !file.is_file()
            || path.dev() != self.identity.dev()
            || path.ino() != self.identity.ino()
            || file.dev() != self.identity.dev()
            || file.ino() != self.identity.ino()
            || path.len() > 65536
            || file.len() > 65536
        {
            return Err(bad);
        }
        check_time(deadline).map_err(|_| bad)?;
        Ok(file)
    }

    fn unchanged(&self, deadline: Instant) -> Result<()> {
        let bad = CollectError::Nesting;
        let file = self.current(deadline)?;
        let path = fs::symlink_metadata(&self.path).map_err(|_| bad)?;
        if !path.is_file()
            || path.dev() != self.identity.dev()
            || path.ino() != self.identity.ino()
            || file.len() != self.identity.len()
            || path.len() != self.identity.len()
            || file.modified().map_err(|_| bad)? != self.identity.modified().map_err(|_| bad)?
            || path.modified().map_err(|_| bad)? != self.identity.modified().map_err(|_| bad)?
        {
            return Err(bad);
        }
        check_time(deadline).map_err(|_| bad)
    }
}

fn publication_pause(deadline: Instant) -> Result<()> {
    check_time(deadline).map_err(|_| CollectError::Nesting)?;
    std::thread::sleep(
        Duration::from_millis(1).min(deadline.saturating_duration_since(Instant::now())),
    );
    Ok(())
}

fn launch_frame(
    path: PathBuf,
    deadline: Instant,
    selected: impl FnOnce() -> std::io::Result<()>,
) -> Result<(Value, LaunchFrame)> {
    let bad = CollectError::Nesting;
    let (file, identity) = loop {
        check_time(deadline).map_err(|_| bad)?;
        match fs::symlink_metadata(&path) {
            Ok(identity) => {
                if !identity.is_file() || identity.len() > 65536 {
                    return Err(bad);
                }
                let file = fs::File::open(&path).map_err(|_| bad)?;
                break (file, identity);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                publication_pause(deadline)?;
            }
            Err(_) => return Err(bad),
        }
    };
    let mut frame = LaunchFrame {
        path,
        file,
        identity,
    };
    frame.current(deadline)?;
    selected().map_err(|_| bad)?;
    loop {
        let current = frame.current(deadline)?;
        if current.len() != 0 {
            frame.file.seek(SeekFrom::End(-1)).map_err(|_| bad)?;
            let mut last = [0];
            frame.file.read_exact(&mut last).map_err(|_| bad)?;
            if last[0] == b'\n' {
                frame.identity = frame.current(deadline)?;
                frame.file.seek(SeekFrom::Start(0)).map_err(|_| bad)?;
                let mut bytes = Vec::new();
                Read::by_ref(&mut frame.file)
                    .take(65536)
                    .read_to_end(&mut bytes)
                    .map_err(|_| bad)?;
                frame.unchanged(deadline)?;
                if bytes.len() as u64 != frame.identity.len() || bytes.last() != Some(&b'\n') {
                    return Err(bad);
                }
                let text = std::str::from_utf8(&bytes).map_err(|_| bad)?;
                let value = strict_json::parse(text).ok_or(bad)?;
                check_time(deadline).map_err(|_| bad)?;
                return Ok((value, frame));
            }
        }
        publication_pause(deadline)?;
    }
}

/// Observe selection only; the callback cannot supply bytes or authorize a launch.
pub fn outer_until_with(
    prepared: &Value,
    deadline: Instant,
    selected: impl FnOnce() -> std::io::Result<()>,
) -> Result<()> {
    let entered = Instant::now();
    check_time(deadline)?;
    let deadline = deadline.min(entered + Duration::from_millis(1000));
    let bad = CollectError::Nesting;
    let actual = environment()?;
    let declared = &prepared["outer"];
    if declared.is_null() {
        return if actual.as_object().ok_or(bad)?.is_empty() {
            Ok(())
        } else {
            Err(bad)
        };
    }
    if actual != declared["environment"] {
        return Err(bad);
    }
    let path = Path::new(declared["context"].as_str().ok_or(bad)?);
    measurement::Request::open(path).map_err(|_| bad)?;
    let path = path.canonicalize().map_err(|_| bad)?;
    let (context, hash) =
        read_json_until(&path.join("context.json"), 32768, deadline).map_err(|_| bad)?;
    let (launch, frame) = launch_frame(
        path.join("launches")
            .join(format!("{}.json", std::process::id())),
        deadline,
        selected,
    )?;
    let executable = std::env::current_exe()
        .and_then(|p| p.canonicalize())
        .map_err(|_| bad)?;
    let identity = fingerprint(&executable, 64 << 20, deadline).map_err(|_| bad)?;
    let arguments = std::env::args_os().collect::<Vec<_>>();
    let cwd = std::env::current_dir()
        .and_then(|p| p.canonicalize())
        .map_err(|_| bad)?;
    if launch["version"] != "native-launch-v1"
        || launch["pid"] != std::process::id()
        || launch["context_path"] != json!(path.as_os_str().as_bytes())
        || launch["context_sha256"] != hash
        || launch["run_id"] != context["run_id"]
        || launch["source_sha256"] != context["source_sha256"]
        || launch["executable"] != json!(executable.as_os_str().as_bytes())
        || launch["executable_sha256"] != identity["sha256"]
        || launch["cwd"] != json!(cwd.as_os_str().as_bytes())
        || launch["program"] != json!(arguments.first().ok_or(bad)?.as_bytes())
        || launch["arguments"]
            != json!(
                arguments
                    .iter()
                    .skip(1)
                    .map(|s| s.as_bytes())
                    .collect::<Vec<_>>()
            )
    {
        return Err(bad);
    }
    let (_, current_hash) =
        read_json_until(&path.join("context.json"), 32768, deadline).map_err(|_| bad)?;
    if current_hash != hash {
        return Err(bad);
    }
    frame.unchanged(deadline)
}

/// Discover every present config again so additions and removals are observable.
pub fn configs(root: &Path, cargo_home: &Path, deadline: Instant) -> Result<Value> {
    let mut paths = BTreeSet::new();
    for directory in root
        .ancestors()
        .map(|p| p.join(".cargo"))
        .chain([cargo_home.to_owned()])
    {
        for name in ["config", "config.toml"] {
            let path = directory.join(name);
            match fs::symlink_metadata(&path) {
                Ok(_) => {
                    paths.insert(path);
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => return Err(CollectError::Intake),
            }
        }
    }
    let mut result = Map::new();
    for path in paths {
        let row = fingerprint(&path, 64 << 20, deadline)?;
        let canonical = path.canonicalize().map_err(|_| CollectError::Intake)?;
        result.insert(
            canonical.to_str().ok_or(CollectError::Intake)?.to_owned(),
            row,
        );
    }
    Ok(Value::Object(result))
}

pub fn resources(sysroot: &Path, deadline: Instant) -> Result<Value> {
    let mut result = Map::new();
    for name in ["bin", "lib", "libexec"] {
        let values = tree(&sysroot.join(name), RESOURCES, deadline)?;
        for (path, row) in values.as_object().ok_or(CollectError::Intake)? {
            result.insert(format!("{name}/{path}"), row.clone());
        }
    }
    let result = Value::Object(result);
    identities(&result, RESOURCES, true)?;
    Ok(result)
}

/// Observe the selected spelling and its actual canonical bytes without fallback.
pub fn tool(program: &Path, deadline: Instant) -> Result<Value> {
    let executable = program.canonicalize().map_err(|_| CollectError::Prepared)?;
    let row = fingerprint(&executable, 64 << 20, deadline)?;
    Ok(
        json!({"program":program.to_str().ok_or(CollectError::Prepared)?,
        "executable":executable.to_str().ok_or(CollectError::Prepared)?,"sha256":row["sha256"]}),
    )
}

pub fn paths(sysroot: &Path, plugin: &Path, deadline: Instant) -> Result<Value> {
    let mut tools = Map::new();
    for name in TOOLS {
        let program: PathBuf = match name {
            "cargo-llvm-cov" => plugin.to_owned(),
            "llvm-cov" | "llvm-profdata" => sysroot
                .join("lib/rustlib/aarch64-apple-darwin/bin")
                .join(name),
            "true" | "false" | "echo" | "sleep" | "mkfifo" => {
                measurement::selected_program(&std::process::Command::new(name))
                    .map_err(|_| CollectError::Prepared)?
            }
            _ => sysroot.join("bin").join(name),
        };
        tools.insert(name.to_owned(), tool(&program, deadline)?);
    }
    Ok(Value::Object(tools))
}
