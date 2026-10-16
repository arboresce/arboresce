use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{
    Action,
    architecture::{GuardError, Layer, check_dependency, check_source_selected},
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
    let mut command =
        crate::measurement::command("cargo", Some(&root)).map_err(|_| RunError::Command)?;
    command.args(Action::Architecture.cargo_args());
    let output = crate::process::output(&mut command).map_err(|_| RunError::Command)?;
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
        "arboresce-application" => Ok(Layer::Application),
        "arboresce-postgres" => Ok(Layer::Adapter),
        "arboresce-xtask" => Ok(Layer::Tool),
        _ => Err(GuardError::Dependency),
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum DependencyProfile {
    Initial,
    JcsNumberV1,
    JcsSourceV1,
    JcsSourcePostgresV1,
}

impl DependencyProfile {
    fn for_package(package: &Value) -> Result<Self, GuardError> {
        let Some(settings) = package.get("metadata").and_then(|m| m.get("arboresce")) else {
            return Ok(Self::Initial);
        };
        let settings = settings.as_object().ok_or(GuardError::InvalidInput)?;
        match settings.get("architecture-profile") {
            None => Ok(Self::Initial),
            Some(Value::String(name)) if name == "initial-v1" => Ok(Self::Initial),
            Some(Value::String(name))
                if name == "jcs-number-v1"
                    && text(package, "name")? == "arboresce-domain"
                    && text(package, "version")? == "0.0.0" =>
            {
                Ok(Self::JcsNumberV1)
            }
            Some(Value::String(name))
                if name == "jcs-source-v1"
                    && text(package, "name")? == "arboresce-domain"
                    && text(package, "version")? == "0.0.0" =>
            {
                Ok(Self::JcsSourceV1)
            }
            Some(Value::String(name))
                if name == "jcs-source-postgres-v1"
                    && text(package, "name")? == "arboresce-domain"
                    && text(package, "version")? == "0.0.0" =>
            {
                Ok(Self::JcsSourcePostgresV1)
            }
            _ => Err(GuardError::Dependency),
        }
    }

    fn check(self, layer: Layer, package: &Value, kind: &str) -> Result<(), GuardError> {
        if self == Self::Initial || kind == "dev" {
            return check_dependency(
                layer,
                text(package, "name")?,
                text(package, "version")?,
                kind,
            );
        }
        match (kind, text(package, "name")?, text(package, "version")?) {
            ("normal", "serde_json_canonicalizer", "0.4.1") | ("normal", "sha2", "0.10.9") => {
                Ok(())
            }
            ("normal", "serde", "1.0.229") | ("normal", "serde_json", "1.0.151")
                if matches!(self, Self::JcsSourceV1 | Self::JcsSourcePostgresV1) =>
            {
                Ok(())
            }
            _ => Err(GuardError::Dependency),
        }
    }
}

type CargoRows<'a> = BTreeMap<&'a str, &'a Value>;

fn package_identity(package: &Value) -> Result<String, GuardError> {
    Ok(format!(
        "{}@{}",
        text(package, "name")?,
        text(package, "version")?
    ))
}

fn selected_features(node: &Value) -> Result<Vec<&str>, GuardError> {
    let raw = array(node, "features")?;
    let features: BTreeSet<_> = raw
        .iter()
        .map(|value| value.as_str().ok_or(GuardError::InvalidInput))
        .collect::<Result<_, _>>()?;
    if features.len() != raw.len() {
        return Err(GuardError::InvalidInput);
    }
    Ok(features.into_iter().collect())
}

fn selected_edges<'a>(
    node: &'a Value,
    packages: &CargoRows<'_>,
    skip_dev: bool,
) -> Result<(Vec<Value>, Vec<&'a str>), GuardError> {
    let mut edges = BTreeMap::new();
    let mut targets = Vec::new();
    for dependency in array(node, "deps")? {
        let raw_kinds = array(dependency, "dep_kinds")?;
        if raw_kinds.is_empty() {
            return Err(GuardError::InvalidInput);
        }
        if skip_dev && raw_kinds.iter().all(|entry| entry["kind"] == "dev") {
            continue;
        }
        let id = text(dependency, "pkg")?;
        let package = packages.get(id).ok_or(GuardError::InvalidInput)?;
        let mut kinds = Vec::new();
        for entry in raw_kinds {
            let kind = entry.get("kind").ok_or(GuardError::InvalidInput)?;
            let target = entry.get("target").ok_or(GuardError::InvalidInput)?;
            if !(kind.is_null() || kind.is_string()) || !(target.is_null() || target.is_string()) {
                return Err(GuardError::InvalidInput);
            }
            kinds.push(json!({"kind":kind,"target":target}));
        }
        kinds.sort_by_cached_key(Value::to_string);
        let name = text(dependency, "name")?;
        let edge = json!({"name":name,"package":package_identity(package)?,"kinds":kinds});
        if edges.insert(name, edge).is_some() {
            return Err(GuardError::Dependency);
        }
        targets.push(id);
    }
    Ok((edges.into_values().collect(), targets))
}

/// Bind the complete selected closure, including conditional/build edges and
/// workspace-unified features. This internal tool encoding is not product JCS.
fn check_qualified_graph(
    profile: DependencyProfile,
    owner: &Value,
    packages: &CargoRows<'_>,
    nodes: &CargoRows<'_>,
) -> Result<(), GuardError> {
    let expected_sha256 = match profile {
        DependencyProfile::JcsNumberV1 => {
            "d6d37e664f5971fa19c2d73ae893a46d74a5d89302083987d0f39c5d9f4dd014"
        }
        DependencyProfile::JcsSourceV1 => {
            "5f819d934d2598938c580eafa2b79186a9b34351e82b6976cf27e7dec00beb8f"
        }
        DependencyProfile::JcsSourcePostgresV1 => {
            "cea739f5ca926bf0c0b53244c3063d2052d11020cfa889007f619b41e4d1b776"
        }
        DependencyProfile::Initial => return Err(GuardError::Dependency),
    };
    let (roots, mut pending) = selected_edges(owner, packages, true)?;
    let mut seen = BTreeSet::new();
    let mut records = BTreeMap::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        if seen.len() > 23 {
            return Err(GuardError::Dependency);
        }
        let package = packages.get(id).ok_or(GuardError::InvalidInput)?;
        let node = nodes.get(id).ok_or(GuardError::InvalidInput)?;
        let source = package.get("source").ok_or(GuardError::InvalidInput)?;
        if !(source.is_null() || source.is_string()) {
            return Err(GuardError::InvalidInput);
        }
        let identity = package_identity(package)?;
        let (dependencies, targets) = selected_edges(node, packages, false)?;
        let record = json!({"package":identity,"source":source,
            "features":selected_features(node)?,"dependencies":dependencies});
        if records.insert(identity, record).is_some() {
            return Err(GuardError::Dependency);
        }
        pending.extend(targets);
    }
    let graph = json!({"owner_features":selected_features(owner)?,"roots":roots,
        "packages":records.into_values().collect::<Vec<_>>()});
    let bytes = serde_json::to_vec(&graph).map_err(|_| GuardError::InvalidInput)?;
    if format!("{:x}", Sha256::digest(bytes)) != expected_sha256 {
        return Err(GuardError::Dependency);
    }
    Ok(())
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
        let profile = DependencyProfile::for_package(package)?;
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
                profile.check(layer, target, kind)?;
            }
        }
        if profile != DependencyProfile::Initial {
            check_qualified_graph(profile, node, &packages, &nodes)?;
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
            check_file(source, layer, profile)?;
        }
        scan_sources(
            package_root,
            &package_root.join("src"),
            layer,
            profile,
            &mut files,
        )?;
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
    profile: DependencyProfile,
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
            scan_sources(root, &path, layer, profile, sources)?;
        } else if path.extension().is_some_and(|x| x == "rs") {
            if sources.len() >= MAX_SOURCE_FILES && !sources.contains(&path) {
                return Err(GuardError::InvalidInput);
            }
            if sources.insert(path.clone()) {
                check_file(&path, layer, profile)?;
            }
        }
    }
    Ok(())
}

fn check_file(path: &Path, layer: Layer, profile: DependencyProfile) -> Result<(), GuardError> {
    let mut source = String::new();
    fs::File::open(path)
        .map_err(|_| GuardError::InvalidInput)?
        .take(MAX_SOURCE_BYTES + 1)
        .read_to_string(&mut source)
        .map_err(|_| GuardError::InvalidInput)?;
    check_source_selected(
        layer,
        &source,
        matches!(
            profile,
            DependencyProfile::JcsSourceV1 | DependencyProfile::JcsSourcePostgresV1
        ),
    )
}
