use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;

use crate::{
    Action,
    architecture::{GuardError, Layer, check_dependency, check_source},
};

const MAX_METADATA_BYTES: usize = 8 * 1024 * 1024;
const MAX_SOURCE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_SOURCE_FILES: usize = 4096;

#[derive(Debug)]
pub enum RunError {
    Command,
    Child(u8),
    Guard(GuardError),
}

pub fn check_current() -> Result<(), RunError> {
    let root = std::env::current_dir().map_err(|_| RunError::Command)?;
    let output = Command::new("cargo")
        .args(Action::Architecture.cargo_args())
        .output()
        .map_err(|_| RunError::Command)?;
    let mut stderr = std::io::stderr().lock();
    stderr
        .write_all(&output.stderr)
        .and_then(|()| stderr.flush())
        .map_err(|_| RunError::Command)?;
    let code = output
        .status
        .code()
        .and_then(|code| u8::try_from(code).ok())
        .ok_or(RunError::Command)?;
    if code != 0 {
        return Err(RunError::Child(code));
    }
    let metadata = std::str::from_utf8(&output.stdout)
        .map_err(|_| RunError::Guard(GuardError::InvalidInput))?;
    check_workspace(&root, metadata).map_err(RunError::Guard)
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, GuardError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or(GuardError::InvalidInput)
}
fn array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>, GuardError> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or(GuardError::InvalidInput)
}

/// Current package roles are registered, not inferred from arbitrary names.
pub fn registered_layer(name: &str, version: &str) -> Result<Layer, GuardError> {
    if version != "0.0.0" {
        return Err(GuardError::Dependency);
    }
    match name {
        "arboresce" => Ok(Layer::Proof),
        "arboresce-domain" => Ok(Layer::Domain),
        "arboresce-xtask" => Ok(Layer::Tool),
        _ => Err(GuardError::Dependency),
    }
}

/// Reconcile Cargo's actual resolved graph and each current production source.
pub fn check_workspace(root: &Path, metadata: &str) -> Result<(), GuardError> {
    if metadata.len() > MAX_METADATA_BYTES {
        return Err(GuardError::InvalidInput);
    }
    let root = root.canonicalize().map_err(|_| GuardError::InvalidInput)?;
    let document: Value = serde_json::from_str(metadata).map_err(|_| GuardError::InvalidInput)?;
    if Path::new(text(&document, "workspace_root")?)
        .canonicalize()
        .map_err(|_| GuardError::InvalidInput)?
        != root
    {
        return Err(GuardError::InvalidInput);
    }
    let members = array(&document, "workspace_members")?;
    if members.is_empty() {
        return Err(GuardError::InvalidInput);
    }
    let mut packages = BTreeMap::new();
    for package in array(&document, "packages")? {
        if packages.insert(text(package, "id")?, package).is_some() {
            return Err(GuardError::InvalidInput);
        }
    }
    let resolve = document.get("resolve").ok_or(GuardError::InvalidInput)?;
    let mut nodes = BTreeMap::new();
    for node in array(resolve, "nodes")? {
        if nodes.insert(text(node, "id")?, node).is_some() {
            return Err(GuardError::InvalidInput);
        }
    }
    let mut seen = BTreeSet::new();
    for member in members {
        let id = member.as_str().ok_or(GuardError::InvalidInput)?;
        if !seen.insert(id) {
            return Err(GuardError::InvalidInput);
        }
        let package = packages.get(id).ok_or(GuardError::InvalidInput)?;
        let layer = registered_layer(text(package, "name")?, text(package, "version")?)?;
        let node = nodes.get(id).ok_or(GuardError::InvalidInput)?;
        for dependency in array(node, "deps")? {
            let target = packages
                .get(text(dependency, "pkg")?)
                .ok_or(GuardError::InvalidInput)?;
            let kinds = array(dependency, "dep_kinds")?;
            if kinds.is_empty() {
                return Err(GuardError::InvalidInput);
            }
            for entry in kinds {
                let kind = match entry.get("kind") {
                    Some(Value::Null) => "normal",
                    Some(Value::String(s)) => s,
                    _ => return Err(GuardError::InvalidInput),
                };
                check_dependency(layer, text(target, "name")?, text(target, "version")?, kind)?;
            }
        }
        let manifest = Path::new(text(package, "manifest_path")?)
            .canonicalize()
            .map_err(|_| GuardError::InvalidInput)?;
        if !manifest.starts_with(&root) || !manifest.is_file() {
            return Err(GuardError::InvalidInput);
        }
        let package_root = manifest.parent().ok_or(GuardError::InvalidInput)?;
        let mut files = check_targets(&root, package_root, package)?;
        for source in &files {
            check_file(source, layer)?;
        }
        scan_sources(package_root, &package_root.join("src"), layer, &mut files)?;
        if files.is_empty() {
            return Err(GuardError::InvalidInput);
        }
    }
    Ok(())
}

fn check_targets(
    root: &Path,
    package_root: &Path,
    package: &Value,
) -> Result<BTreeSet<PathBuf>, GuardError> {
    let targets = array(package, "targets")?;
    if targets.is_empty() {
        return Err(GuardError::InvalidInput);
    }
    let mut names = BTreeSet::new();
    let mut sources = BTreeSet::new();
    for target in targets {
        let name = text(target, "name")?;
        if !names.insert(name) {
            return Err(GuardError::InvalidInput);
        }
        let kinds = array(target, "kind")?;
        if kinds.is_empty() {
            return Err(GuardError::InvalidInput);
        }
        let declared = Path::new(text(target, "src_path")?);
        if fs::symlink_metadata(declared)
            .map_err(|_| GuardError::InvalidInput)?
            .file_type()
            .is_symlink()
        {
            return Err(GuardError::InvalidInput);
        }
        let source = declared
            .canonicalize()
            .map_err(|_| GuardError::InvalidInput)?;
        if !source.is_file() || !source.starts_with(package_root) {
            return Err(GuardError::InvalidInput);
        }
        for kind in kinds {
            match kind.as_str().ok_or(GuardError::InvalidInput)? {
                "lib" | "rlib" | "dylib" | "cdylib" | "staticlib" | "proc-macro" | "bin"
                | "custom-build" => {
                    let registered_peer = text(package, "name")? == "arboresce-xtask"
                        && package_root == root.join("tools/xtask")
                        && name == "arboresce-xtask-test-peer"
                        && kinds == &vec![Value::String("bin".to_owned())]
                        && source == package_root.join("tests/support/peer.rs");
                    if !registered_peer && !source.starts_with(package_root.join("src")) {
                        return Err(GuardError::InvalidInput);
                    }
                    if !registered_peer {
                        sources.insert(source.clone());
                        if sources.len() > MAX_SOURCE_FILES {
                            return Err(GuardError::InvalidInput);
                        }
                    }
                }
                "test" | "bench" | "example" => {}
                _ => return Err(GuardError::InvalidInput),
            }
        }
    }
    Ok(sources)
}

fn scan_sources(
    root: &Path,
    directory: &Path,
    layer: Layer,
    sources: &mut BTreeSet<PathBuf>,
) -> Result<(), GuardError> {
    if fs::symlink_metadata(directory)
        .map_err(|_| GuardError::InvalidInput)?
        .file_type()
        .is_symlink()
    {
        return Err(GuardError::InvalidInput);
    }
    if !directory
        .canonicalize()
        .map_err(|_| GuardError::InvalidInput)?
        .starts_with(root)
    {
        return Err(GuardError::InvalidInput);
    }
    for entry in fs::read_dir(directory).map_err(|_| GuardError::InvalidInput)? {
        let entry = entry.map_err(|_| GuardError::InvalidInput)?;
        let path = entry.path();
        let kind = entry.file_type().map_err(|_| GuardError::InvalidInput)?;
        if kind.is_symlink() {
            return Err(GuardError::InvalidInput);
        }
        if kind.is_dir() {
            scan_sources(root, &path, layer, sources)?;
        } else if path.extension().is_some_and(|x| x == "rs") {
            if sources.len() >= MAX_SOURCE_FILES && !sources.contains(&path) {
                return Err(GuardError::InvalidInput);
            }
            if sources.insert(path.clone()) {
                check_file(&path, layer)?;
            }
        }
    }
    Ok(())
}

fn check_file(path: &Path, layer: Layer) -> Result<(), GuardError> {
    let mut source = String::new();
    fs::File::open(path)
        .map_err(|_| GuardError::InvalidInput)?
        .take(MAX_SOURCE_BYTES + 1)
        .read_to_string(&mut source)
        .map_err(|_| GuardError::InvalidInput)?;
    check_source(layer, &source)
}
