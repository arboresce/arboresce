//! Explicit, bounded native PostgreSQL inputs; never ambient service discovery.

use crate::{
    collection_inputs::{CollectError, Result},
    collection_prepared as prepared,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs,
    os::unix::{
        ffi::OsStrExt,
        fs::{MetadataExt, PermissionsExt},
    },
    path::{Component, Path, PathBuf},
    process::Command,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

pub const SELECTOR: &str = "ARBORESCE_POSTGRES_INPUT";
pub const MAX_INPUT: u64 = 2 << 20;

pub const MAX_COHORT_MANIFEST: u64 = 4096;

fn allocation() -> Value {
    json!([
        {"package":"arboresce-postgres","kind":"test","name":"authority_current","service":"current"},
        {"package":"arboresce-postgres","kind":"test","name":"authority_races","service":"legacy"},
        {"package":"arboresce-postgres","kind":"test","name":"qualification","service":"legacy"}
    ])
}

/// A separate opt-in shape; the original single-service validator is unchanged.
pub fn validate_cohorts(value: &Value) -> Result<()> {
    let normalized = |v: &Value| {
        absolute(v)
            && v.as_str()
                .is_some_and(|s| Path::new(s).components().collect::<PathBuf>().to_str() == Some(s))
    };
    if !keys(value, &["version", "services", "allocation"])
        || value["version"] != "postgres-harness-cohorts-v1"
        || !keys(&value["services"], &["legacy", "current"])
        || !normalized(&value["services"]["legacy"])
        || !normalized(&value["services"]["current"])
        || value["services"]["legacy"] == value["services"]["current"]
        || value["allocation"] != allocation()
    {
        return Err(CollectError::Prepared);
    }
    Ok(())
}

/// Only one complete, registered package/kind/name tuple selects the new cohort.
pub fn cohort_route(object: &Value) -> &'static str {
    if object["package"] == "arboresce-postgres"
        && object["kind"] == "test"
        && object["name"] == "authority_current"
    {
        "current"
    } else {
        "legacy"
    }
}

fn overlap(a: &Path, b: &Path) -> bool {
    a.starts_with(b) || b.starts_with(a)
}

/// Structural separation complements, and never replaces, actual input intake.
pub fn validate_cohort_separation(services: &Value) -> Result<()> {
    if !keys(services, &["legacy", "current"]) {
        return Err(CollectError::Prepared);
    }
    let a = &services["legacy"]["record"];
    let b = &services["current"]["record"];
    validate(a)?;
    validate(b)?;
    for key in ["server_pid", "system_identifier", "socket", "data"] {
        if a[key] == b[key] {
            return Err(CollectError::Prepared);
        }
    }
    if a["readiness"]["path"] == b["readiness"]["path"] || a["tools"] != b["tools"] {
        return Err(CollectError::Prepared);
    }
    let data_a = Path::new(a["data"].as_str().ok_or(CollectError::Prepared)?);
    let data_b = Path::new(b["data"].as_str().ok_or(CollectError::Prepared)?);
    if overlap(data_a, data_b)
        || overlap(
            data_a.parent().ok_or(CollectError::Prepared)?,
            data_b.parent().ok_or(CollectError::Prepared)?,
        )
        || overlap(
            Path::new(a["socket"].as_str().ok_or(CollectError::Prepared)?),
            Path::new(b["socket"].as_str().ok_or(CollectError::Prepared)?),
        )
    {
        return Err(CollectError::Prepared);
    }
    Ok(())
}

pub struct Selection {
    pub legacy: Value,
    pub cohorts: Option<Value>,
}

impl Selection {
    pub fn path_for(&self, object: &Value) -> Result<PathBuf> {
        let service = match &self.cohorts {
            Some(cohorts) => &cohorts["services"][cohort_route(object)],
            None => &self.legacy,
        };
        service["path"]
            .as_str()
            .map(PathBuf::from)
            .ok_or(CollectError::Prepared)
    }
}

/// Both shapes share the inherited 2-MiB outer intake ceiling. Once identified,
/// the opt-in manifest has its separate 4-KiB limit and two complete observers.
pub fn observe_selection(path: &Path, deadline: Instant) -> Result<Selection> {
    if !absolute(&json!(path.to_str().ok_or(CollectError::Input)?)) {
        return Err(CollectError::Input);
    }
    let (value, hash) = prepared::read_json_until(path, MAX_INPUT, deadline)?;
    if value["version"] == "postgres-harness-service-v1" {
        return Ok(Selection {
            legacy: observe(path, deadline)?,
            cohorts: None,
        });
    }
    validate_cohorts(&value)?;
    let physical = fs::symlink_metadata(path).map_err(|_| CollectError::Intake)?;
    if physical.nlink() != 1 {
        return Err(CollectError::Intake);
    }
    let manifest = prepared::fingerprint(path, MAX_COHORT_MANIFEST, deadline)?;
    if manifest["sha256"] != hash {
        return Err(CollectError::Changed);
    }
    let mut services = json!({});
    let mut ownership = json!({});
    for name in ["legacy", "current"] {
        let input = Path::new(
            value["services"][name]
                .as_str()
                .ok_or(CollectError::Prepared)?,
        );
        let observed = observe(input, deadline)?;
        let record = &observed["record"];
        let data = Path::new(record["data"].as_str().ok_or(CollectError::Prepared)?);
        let paths = [
            ("descriptor", input),
            ("data", data),
            ("control", data.parent().ok_or(CollectError::Prepared)?),
            (
                "socket",
                Path::new(record["socket"].as_str().ok_or(CollectError::Prepared)?),
            ),
            (
                "readiness",
                Path::new(
                    record["readiness"]["path"]
                        .as_str()
                        .ok_or(CollectError::Prepared)?,
                ),
            ),
        ];
        let mut physical = json!({});
        for (role, selected) in paths {
            let metadata = fs::symlink_metadata(selected).map_err(|_| CollectError::Intake)?;
            if selected.canonicalize().map_err(|_| CollectError::Intake)? != selected
                || metadata.file_type().is_symlink()
                || (role == "control"
                    && (!metadata.is_dir() || metadata.permissions().mode() & 0o777 != 0o700))
            {
                return Err(CollectError::Prepared);
            }
            physical[role] = json!({"device":metadata.dev(),"inode":metadata.ino(),"mode":metadata.permissions().mode()&0o777});
        }
        ownership[name] = physical;
        services[name] = observed;
    }
    validate_cohort_separation(&services)?;
    for role in ["descriptor", "data", "control", "socket", "readiness"] {
        let a = &ownership["legacy"][role];
        let b = &ownership["current"][role];
        if a["device"] == b["device"] && a["inode"] == b["inode"] {
            return Err(CollectError::Prepared);
        }
    }
    let after = fs::symlink_metadata(path).map_err(|_| CollectError::Changed)?;
    if after.nlink() != 1
        || after.dev() != physical.dev()
        || after.ino() != physical.ino()
        || after.len() != physical.len()
        || after.modified().map_err(|_| CollectError::Changed)?
            != physical.modified().map_err(|_| CollectError::Changed)?
    {
        return Err(CollectError::Changed);
    }
    let legacy = services["legacy"].clone();
    Ok(Selection {
        legacy,
        cohorts: Some(
            json!({"manifest":{"path":path,"sha256":hash,"record":value},
        "services":services,"ownership":ownership}),
        ),
    })
}

fn absolute(value: &Value) -> bool {
    value.as_str().is_some_and(|s| {
        let p = Path::new(s);
        p.is_absolute()
            && !s.as_bytes().contains(&0)
            && p.components()
                .all(|c| matches!(c, Component::RootDir | Component::Normal(_)))
    })
}

fn digest(value: &Value) -> bool {
    value.as_str().is_some_and(|s| {
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}

fn keys(value: &Value, expected: &[&str]) -> bool {
    value.as_object().is_some_and(|m| {
        m.keys().map(String::as_str).collect::<BTreeSet<_>>() == expected.iter().copied().collect()
    })
}

fn lease(value: &Value) -> Result<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| CollectError::Prepared)?
        .as_millis();
    if u128::from(
        value["lease_deadline_unix_ms"]
            .as_u64()
            .ok_or(CollectError::Prepared)?,
    ) <= now
    {
        return Err(CollectError::Prepared);
    }
    Ok(())
}

/// Structural policy only; literal hashes never establish actual readiness.
pub fn validate(value: &Value) -> Result<()> {
    let bad = CollectError::Prepared;
    if !keys(
        value,
        &[
            "version",
            "socket",
            "data",
            "server_pid",
            "postmaster_start",
            "system_identifier",
            "server_version_num",
            "os_build",
            "architecture",
            "lease_deadline_unix_ms",
            "readiness",
            "tools",
            "files",
        ],
    ) || value["version"] != "postgres-harness-service-v1"
        || !absolute(&value["socket"])
        || !absolute(&value["data"])
        || value["socket"].as_str().is_none_or(|s| s.len() > 80)
        || !value["server_pid"]
            .as_u64()
            .is_some_and(|n| n > 0 && n <= u64::from(u32::MAX))
        || !value["system_identifier"].as_str().is_some_and(|s| {
            !s.is_empty()
                && s.len() <= 20
                && s.bytes().all(|b| b.is_ascii_digit())
                && s.parse::<u64>().is_ok()
        })
        || !value["postmaster_start"]
            .as_str()
            .is_some_and(|s| !s.is_empty() && s.len() <= 64)
        || value["server_version_num"] != 170011
        || value["architecture"] != "arm64"
        || !value["os_build"]
            .as_str()
            .is_some_and(|s| !s.is_empty() && s.len() <= 32)
        || !value["lease_deadline_unix_ms"]
            .as_u64()
            .is_some_and(|n| n > 0)
        || !keys(&value["readiness"], &["path", "sha256"])
        || !absolute(&value["readiness"]["path"])
        || !digest(&value["readiness"]["sha256"])
        || !keys(&value["tools"], &["postgres", "pg_ctl"])
    {
        return Err(bad);
    }
    let files = value["files"].as_object().ok_or(bad)?;
    if files.is_empty() || files.len() > 64 {
        return Err(bad);
    }
    let mut total = 0u64;
    for (path, row) in files {
        let size = row["size"].as_u64().ok_or(bad)?;
        if !absolute(&json!(path))
            || !keys(row, &["sha256", "size"])
            || !digest(&row["sha256"])
            || size > (2 << 30)
        {
            return Err(bad);
        }
        total = total.checked_add(size).ok_or(bad)?;
    }
    if total > (8 << 30) {
        return Err(bad);
    }
    for row in value["tools"].as_object().ok_or(bad)?.values() {
        if !keys(row, &["path", "sha256"])
            || !absolute(&row["path"])
            || !digest(&row["sha256"])
            || files
                .get(row["path"].as_str().ok_or(bad)?)
                .is_none_or(|file| file["sha256"] != row["sha256"])
        {
            return Err(bad);
        }
    }
    let data = value["data"].as_str().ok_or(bad)?;
    for name in ["postgresql.conf", "pg_hba.conf", "postgresql.auto.conf"] {
        if !files.contains_key(&format!("{data}/{name}")) {
            return Err(bad);
        }
    }
    Ok(())
}

/// Reopen every bound identity; the returned immutable record enters run identity.
pub fn observe(path: &Path, deadline: Instant) -> Result<Value> {
    if !absolute(&json!(path.to_str().ok_or(CollectError::Input)?)) {
        return Err(CollectError::Input);
    }
    let (value, hash) = prepared::read_json_until(path, MAX_INPUT, deadline)?;
    validate(&value)?;
    lease(&value)?;
    for key in ["data", "socket"] {
        let path = Path::new(value[key].as_str().ok_or(CollectError::Prepared)?);
        let metadata = fs::symlink_metadata(path).map_err(|_| CollectError::Intake)?;
        if !metadata.is_dir()
            || metadata.permissions().mode() & 0o777 != 0o700
            || path.canonicalize().map_err(|_| CollectError::Intake)? != path
        {
            return Err(CollectError::Prepared);
        }
    }
    let readiness = &value["readiness"];
    let actual = prepared::fingerprint(
        Path::new(readiness["path"].as_str().ok_or(CollectError::Prepared)?),
        MAX_INPUT,
        deadline,
    )?;
    if actual["sha256"] != readiness["sha256"] {
        return Err(CollectError::Changed);
    }
    for (path, expected) in value["files"].as_object().ok_or(CollectError::Prepared)? {
        if prepared::fingerprint(Path::new(path), 2 << 30, deadline)? != *expected {
            return Err(CollectError::Changed);
        }
    }
    lease(&value)?;
    Ok(json!({"path":path.to_str().ok_or(CollectError::Input)?,"sha256":hash,"record":value}))
}

/// Apply explicit selection after other command environment configuration.
pub fn environment(command: &mut Command, selected: Option<&Path>) {
    for (key, _) in std::env::vars_os() {
        if key.as_bytes().starts_with(b"PG") {
            command.env_remove(key);
        }
    }
    // Also remove explicit values already placed on this Command.
    let pg: Vec<_> = command
        .get_envs()
        .filter(|(key, _)| key.as_bytes().starts_with(b"PG"))
        .map(|(key, _)| key.to_owned())
        .collect();
    for key in pg {
        command.env_remove(key);
    }
    command.env_remove(SELECTOR);
    if let Some(path) = selected {
        command.env(SELECTOR, path);
    }
}
