use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt, fs,
    io::Read,
    path::{Path, PathBuf},
};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::strict_json;

const MAX_FILES: usize = 4096;
const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 256 * 1024 * 1024;
const MAX_DEPTH: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InventoryError {
    Input,
    Files,
    Roles,
    Targets,
    Declaration,
}

impl fmt::Display for InventoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Input => "invalid inventory input",
            Self::Files => "inventory file discovery failed",
            Self::Roles => "inventory source role mismatch",
            Self::Targets => "inventory Cargo target mismatch",
            Self::Declaration => "unproved declaration-only source",
        })
    }
}
impl Error for InventoryError {}

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, InventoryError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or(InventoryError::Input)
}

fn array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>, InventoryError> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or(InventoryError::Input)
}

fn relative(value: &str) -> Result<&str, InventoryError> {
    if value.is_empty()
        || value.starts_with('/')
        || value.contains('\\')
        || value.chars().any(char::is_control)
        || value.split('/').any(|part| matches!(part, "" | "." | ".."))
    {
        return Err(InventoryError::Input);
    }
    Ok(value)
}

/// Normalize a filesystem path value without inventing a lossy UTF-8 name.
pub fn portable_path(path: &Path) -> Result<&str, InventoryError> {
    let name = path.to_str().ok_or(InventoryError::Files)?;
    relative(name).map_err(|_| InventoryError::Files)
}

/// Reopened generated paths must not traverse a symlinked ancestor.
fn real_file(root: &Path, path: &str) -> Result<PathBuf, InventoryError> {
    relative(path)?;
    let mut full = root.to_owned();
    for part in Path::new(path).components() {
        full.push(part);
        if fs::symlink_metadata(&full)
            .map_err(|_| InventoryError::Files)?
            .file_type()
            .is_symlink()
        {
            return Err(InventoryError::Files);
        }
    }
    if !full.is_file() {
        return Err(InventoryError::Files);
    }
    Ok(full)
}

fn read(path: &Path) -> Result<Vec<u8>, InventoryError> {
    let metadata = fs::metadata(path).map_err(|_| InventoryError::Files)?;
    if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
        return Err(InventoryError::Files);
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|_| InventoryError::Files)?
        .take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| InventoryError::Files)?;
    if bytes.len() as u64 != metadata.len() {
        return Err(InventoryError::Files);
    }
    Ok(bytes)
}

#[derive(Default)]
struct Files {
    rows: BTreeMap<String, Value>,
    total: u64,
}

impl Files {
    fn add(&mut self, root: &Path, name: &str) -> Result<(), InventoryError> {
        if Path::new(name).components().count() > MAX_DEPTH + 1 {
            return Err(InventoryError::Files);
        }
        let path = real_file(root, name)?;
        let bytes = read(&path)?;
        let size = bytes.len() as u64;
        let added = if self.rows.contains_key(name) {
            0
        } else {
            size
        };
        if size > MAX_FILE_BYTES
            || self.total + added > MAX_TOTAL_BYTES
            || (!self.rows.contains_key(name) && self.rows.len() == MAX_FILES)
        {
            return Err(InventoryError::Files);
        }
        let row = json!({"sha256":sha256(&bytes),"size":size});
        if let Some(previous) = self.rows.get(name) {
            if previous != &row {
                return Err(InventoryError::Files);
            }
        } else {
            self.total += added;
            self.rows.insert(name.to_owned(), row);
        }
        Ok(())
    }
}

struct CargoInventory {
    packages: BTreeMap<String, PathBuf>,
    sources: BTreeSet<String>,
    targets: BTreeMap<(String, String), (String, String)>,
    directories: BTreeSet<PathBuf>,
}

fn cargo_file(root: &Path, value: &str) -> Result<String, InventoryError> {
    let path = Path::new(value)
        .strip_prefix(root)
        .ok()
        .and_then(Path::to_str)
        .ok_or(InventoryError::Targets)?;
    relative(path).map_err(|_| InventoryError::Targets)?;
    real_file(root, path).map_err(|_| InventoryError::Targets)?;
    Ok(path.to_owned())
}

fn cargo_inventory(root: &Path, metadata: &Value) -> Result<CargoInventory, InventoryError> {
    if Path::new(text(metadata, "workspace_root")?)
        .canonicalize()
        .map_err(|_| InventoryError::Targets)?
        != root
    {
        return Err(InventoryError::Targets);
    }
    let mut by_id = BTreeMap::new();
    for package in array(metadata, "packages")? {
        if by_id.insert(text(package, "id")?, package).is_some() {
            return Err(InventoryError::Targets);
        }
    }
    let members = array(metadata, "workspace_members")?;
    if members.is_empty() {
        return Err(InventoryError::Targets);
    }
    let mut result = CargoInventory {
        packages: BTreeMap::new(),
        sources: BTreeSet::new(),
        targets: BTreeMap::new(),
        directories: BTreeSet::new(),
    };
    let mut seen = BTreeSet::new();
    for member in members {
        let id = member.as_str().ok_or(InventoryError::Input)?;
        if !seen.insert(id) {
            return Err(InventoryError::Targets);
        }
        let package = by_id.get(id).ok_or(InventoryError::Targets)?;
        let name = text(package, "name")?;
        let manifest = cargo_file(root, text(package, "manifest_path")?)?;
        let directory = Path::new(&manifest)
            .parent()
            .ok_or(InventoryError::Targets)?
            .to_owned();
        if result
            .packages
            .insert(name.to_owned(), directory.clone())
            .is_some()
        {
            return Err(InventoryError::Targets);
        }
        let targets = array(package, "targets")?;
        if targets.is_empty() {
            return Err(InventoryError::Targets);
        }
        let mut seen_targets = BTreeSet::new();
        for target in targets {
            let target_name = text(target, "name")?;
            let kinds = array(target, "kind")?;
            let kind = match kinds.as_slice() {
                [value] => match value.as_str() {
                    Some("lib" | "rlib" | "dylib" | "cdylib" | "staticlib" | "proc-macro") => {
                        "library"
                    }
                    Some("bin" | "example" | "custom-build") => "binary",
                    Some("test" | "bench") => "test",
                    _ => return Err(InventoryError::Targets),
                },
                _ => return Err(InventoryError::Targets),
            };
            if !seen_targets.insert((kind, target_name)) {
                return Err(InventoryError::Targets);
            }
            let source = cargo_file(root, text(target, "src_path")?)?;
            if !Path::new(&source).starts_with(&directory) {
                return Err(InventoryError::Targets);
            }
            let parent = Path::new(&source).parent().ok_or(InventoryError::Targets)?;
            if parent != directory {
                result.directories.insert(parent.to_owned());
            }
            result.sources.insert(source.clone());
            if kind != "test"
                && result
                    .targets
                    .insert(
                        (kind.to_owned(), target_name.to_owned()),
                        (name.to_owned(), source),
                    )
                    .is_some()
            {
                return Err(InventoryError::Targets);
            }
        }
    }
    Ok(result)
}

fn excluded(path: &Path, cargo: &CargoInventory) -> bool {
    if path == Path::new(".git") {
        return true;
    }
    // A declared source/package tree wins over a coincidentally named cache.
    if cargo.packages.values().any(|root| root.starts_with(path))
        || cargo.directories.iter().any(|root| root.starts_with(path))
        || cargo.sources.iter().any(|source| Path::new(source) == path)
    {
        return false;
    }
    [Path::new(""), Path::new("contracts/validation")]
        .into_iter()
        .chain(cargo.packages.values().map(PathBuf::as_path))
        .any(|root| {
            path.strip_prefix(root).is_ok_and(|rest| {
                matches!(
                    rest.to_str(),
                    Some(
                        "target"
                            | "build"
                            | "dist"
                            | ".venv"
                            | "node_modules"
                            | "__pycache__"
                            | ".pytest_cache"
                            | ".ruff_cache"
                            | ".mypy_cache"
                    )
                )
            })
        })
}

fn walk(
    root: &Path,
    directory: &Path,
    depth: usize,
    cargo: &CargoInventory,
    files: &mut Files,
) -> Result<(), InventoryError> {
    if depth > MAX_DEPTH {
        return Err(InventoryError::Files);
    }
    for entry in fs::read_dir(root.join(directory)).map_err(|_| InventoryError::Files)? {
        let entry = entry.map_err(|_| InventoryError::Files)?;
        let path = directory.join(entry.file_name());
        let name = portable_path(&path)?;
        if excluded(&path, cargo) {
            continue;
        }
        let kind = entry.file_type().map_err(|_| InventoryError::Files)?;
        if kind.is_dir() {
            walk(root, &path, depth + 1, cargo, files)?;
        } else {
            files.add(root, name)?;
        }
    }
    Ok(())
}

fn data_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(path) => {
            path.qself.is_none()
                && path.path.leading_colon.is_none()
                && path.path.segments.len() == 1
                && path.path.segments.iter().all(|part| match &part.arguments {
                    syn::PathArguments::None => true,
                    syn::PathArguments::AngleBracketed(args) => args
                        .args
                        .iter()
                        .all(|arg| matches!(arg, syn::GenericArgument::Type(ty) if data_type(ty))),
                    _ => false,
                })
        }
        syn::Type::Reference(value) => value.mutability.is_none() && data_type(&value.elem),
        syn::Type::Array(value) => {
            data_type(&value.elem)
                && matches!(&value.len, syn::Expr::Lit(n) if n.attrs.is_empty() && matches!(n.lit, syn::Lit::Int(_)))
        }
        syn::Type::Slice(value) => data_type(&value.elem),
        syn::Type::Tuple(value) => value.elems.iter().all(data_type),
        _ => false,
    }
}

fn data_expr(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Lit(value) => value.attrs.is_empty(),
        syn::Expr::Array(value) => value.attrs.is_empty() && value.elems.iter().all(data_expr),
        syn::Expr::Repeat(value) => {
            value.attrs.is_empty()
                && data_expr(&value.expr)
                && matches!(&*value.len, syn::Expr::Lit(n) if n.attrs.is_empty() && matches!(n.lit, syn::Lit::Int(_)))
        }
        syn::Expr::Reference(value) => {
            value.attrs.is_empty() && value.mutability.is_none() && data_expr(&value.expr)
        }
        syn::Expr::Struct(value) => {
            value.attrs.is_empty()
                && value.qself.is_none()
                && value.rest.is_none()
                && value.path.get_ident().is_some()
                && value
                    .fields
                    .iter()
                    .all(|field| field.attrs.is_empty() && data_expr(&field.expr))
        }
        syn::Expr::Path(value) => {
            value.attrs.is_empty() && value.qself.is_none() && value.path.is_ident("None")
        }
        syn::Expr::Call(value) => {
            value.attrs.is_empty()
                && matches!(&*value.func, syn::Expr::Path(p) if p.attrs.is_empty() && p.qself.is_none() && p.path.is_ident("Some"))
                && value.args.len() == 1
                && value.args.iter().all(data_expr)
        }
        _ => false,
    }
}

fn declaration(bytes: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return false;
    };
    let Ok(file) = syn::parse_file(text) else {
        return false;
    };
    file.attrs.is_empty()
        && !file.items.is_empty()
        && file.items.iter().all(|item| match item {
            syn::Item::Struct(value) => {
                value.attrs.is_empty()
                    && value.generics.params.is_empty()
                    && value.generics.where_clause.is_none()
                    && value
                        .fields
                        .iter()
                        .all(|field| field.attrs.is_empty() && data_type(&field.ty))
            }
            syn::Item::Const(value) => {
                value.attrs.is_empty()
                    && value.generics.params.is_empty()
                    && value.generics.where_clause.is_none()
                    && data_type(&value.ty)
                    && data_expr(&value.expr)
            }
            _ => false,
        })
}

/// Observe actual files and reconcile supplied Cargo metadata and reviewed roles.
/// Tool provenance and before/after collection remain the caller's obligations.
pub fn snapshot(
    root: &Path,
    raw_profile: &str,
    raw_metadata: &str,
) -> Result<Value, InventoryError> {
    let root = root.canonicalize().map_err(|_| InventoryError::Input)?;
    if !root.is_dir() {
        return Err(InventoryError::Input);
    }
    let profile = strict_json::parse(raw_profile).ok_or(InventoryError::Input)?;
    let metadata = strict_json::parse(raw_metadata).ok_or(InventoryError::Input)?;
    if text(&profile, "version")? != "rust-source-roles-v1" {
        return Err(InventoryError::Input);
    }
    let sources = array(&profile, "sources")?;
    let targets = array(&profile, "targets")?;
    let mut generated = BTreeSet::new();
    for path in array(&profile, "generated_sources")? {
        let path = relative(path.as_str().ok_or(InventoryError::Input)?)?;
        if !generated.insert(path.to_owned()) {
            return Err(InventoryError::Input);
        }
    }
    let cargo = cargo_inventory(&root, &metadata)?;
    let mut files = Files::default();
    walk(&root, Path::new(""), 0, &cargo, &mut files)?;
    for path in &generated {
        files.add(&root, path)?;
    }
    let discovered: BTreeSet<_> = files
        .rows
        .keys()
        .filter(|path| Path::new(path).extension().is_some_and(|ext| ext == "rs"))
        .cloned()
        .chain(cargo.sources.iter().cloned())
        .chain(generated)
        .collect();
    let mut rows = BTreeMap::new();
    let mut measured = BTreeMap::new();
    let mut coverage_sources = Vec::new();
    for source in sources {
        let path = relative(text(source, "path")?)?;
        let package = text(source, "package")?;
        let role = text(source, "role")?;
        let owner = cargo.packages.get(package).ok_or(InventoryError::Roles)?;
        if !files.rows.contains_key(path)
            || !Path::new(path).starts_with(owner)
            || rows.insert(path, source).is_some()
        {
            return Err(InventoryError::Roles);
        }
        match role {
            "measured" | "assertion" => {
                if source.get("declaration_sha256").is_some() {
                    return Err(InventoryError::Roles);
                }
                coverage_sources.push(json!({"path":path,"package":package,"role":role}));
                if role == "measured" {
                    measured.insert(path, package);
                }
            }
            "declaration" => {
                let expected = source
                    .get("declaration_sha256")
                    .and_then(Value::as_str)
                    .ok_or(InventoryError::Declaration)?;
                let bytes = read(&real_file(&root, path)?)?;
                if files.rows[path]["sha256"] != expected
                    || sha256(&bytes) != expected
                    || !declaration(&bytes)
                {
                    return Err(InventoryError::Declaration);
                }
            }
            _ => return Err(InventoryError::Roles),
        }
    }
    if rows.keys().copied().collect::<BTreeSet<_>>()
        != discovered.iter().map(String::as_str).collect()
    {
        return Err(InventoryError::Roles);
    }
    let mut seen_targets = BTreeSet::new();
    for target in targets {
        let key = (
            text(target, "kind")?.to_owned(),
            text(target, "name")?.to_owned(),
        );
        let (package, entry) = cargo.targets.get(&key).ok_or(InventoryError::Targets)?;
        if !seen_targets.insert(key) {
            return Err(InventoryError::Targets);
        }
        let mut names = BTreeSet::new();
        for path in array(target, "sources")? {
            let path = path.as_str().ok_or(InventoryError::Input)?;
            if measured.get(path).copied() != Some(package.as_str()) || !names.insert(path) {
                return Err(InventoryError::Targets);
            }
        }
        if !names.contains(entry.as_str()) {
            return Err(InventoryError::Targets);
        }
    }
    if seen_targets != cargo.targets.keys().cloned().collect() {
        return Err(InventoryError::Targets);
    }
    Ok(json!({
        "version":"rust-source-inventory-v1",
        "profile_sha256":sha256(raw_profile.as_bytes()),
        "files":files.rows,
        "sources":sources,
        "targets":targets,
        "coverage_roles":{"version":"rust-report-roles-v1","sources":coverage_sources,"targets":targets}
    }))
}
