//! Explicit, bounded native PostgreSQL inputs; never ambient service discovery.

use crate::{
    collection_inputs::{CollectError, Result},
    collection_prepared as prepared,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::{Component, Path},
    process::Command,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

pub const SELECTOR: &str = "ARBORESCE_POSTGRES_INPUT";
pub const MAX_INPUT: u64 = 2 << 20;

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
