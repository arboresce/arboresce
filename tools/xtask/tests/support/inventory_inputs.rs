use std::{
    fs, io,
    os::unix::{fs::symlink, net::UnixListener},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use serde_json::Value;

static NEXT: AtomicU64 = AtomicU64::new(0);

fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "invalid inventory fixture recipe",
    )
}

fn text<'a>(value: &'a Value, key: &str) -> io::Result<&'a str> {
    value.get(key).and_then(Value::as_str).ok_or_else(invalid)
}

fn path(root: &Path, name: &str) -> io::Result<PathBuf> {
    if name.is_empty()
        || name.contains('\\')
        || Path::new(name)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(invalid());
    }
    let mut result = root.to_owned();
    for part in Path::new(name).components() {
        result.push(part);
        if fs::symlink_metadata(&result).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err(invalid());
        }
    }
    Ok(result)
}

fn write(root: &Path, name: &str, bytes: &[u8]) -> io::Result<()> {
    let path = path(root, name)?;
    fs::create_dir_all(path.parent().ok_or_else(invalid)?)?;
    fs::write(path, bytes)
}

fn patch(value: &mut Value, changes: &Value) -> io::Result<()> {
    for change in changes.as_array().ok_or_else(invalid)? {
        let pointer = text(change, "pointer")?;
        if change.get("remove") == Some(&Value::Bool(true)) {
            let (parent, key) = pointer.rsplit_once('/').ok_or_else(invalid)?;
            value
                .pointer_mut(parent)
                .and_then(Value::as_object_mut)
                .and_then(|map| map.remove(key))
                .ok_or_else(invalid)?;
        } else {
            *value.pointer_mut(pointer).ok_or_else(invalid)? =
                change.get("value").ok_or_else(invalid)?.clone();
        }
    }
    Ok(())
}

/// Materialize only the frozen literal model recipes. Never calculate answers.
pub struct Prepared {
    pub root: PathBuf,
    pub profile: String,
    pub metadata: String,
    sockets: Vec<UnixListener>,
}

impl Drop for Prepared {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

pub fn prepare(fixture: &Value, case: &Value) -> io::Result<Prepared> {
    // Short owned paths also permit real Unix sockets on macOS.
    let root = Path::new("/tmp").join(format!(
        "arboresce-inventory-model-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root)?;
    let mut result = Prepared {
        root: root.canonicalize()?,
        profile: String::new(),
        metadata: String::new(),
        sockets: Vec::new(),
    };
    for (name, bytes) in fixture["base_files"].as_object().ok_or_else(invalid)? {
        write(
            &result.root,
            name,
            bytes.as_str().ok_or_else(invalid)?.as_bytes(),
        )?;
    }
    for op in case["file_operations"].as_array().ok_or_else(invalid)? {
        match text(op, "op")? {
            "write" => write(
                &result.root,
                text(op, "path")?,
                text(op, "text")?.as_bytes(),
            )?,
            "remove" => fs::remove_file(path(&result.root, text(op, "path")?)?)?,
            "rename" => fs::rename(
                path(&result.root, text(op, "path")?)?,
                path(&result.root, text(op, "to")?)?,
            )?,
            "symlink" => {
                let target = text(op, "target")?;
                path(&result.root, target)?;
                symlink(target, path(&result.root, text(op, "path")?)?)?;
            }
            "sparse" => {
                let size = op["size"]
                    .as_u64()
                    .filter(|n| *n <= 67_108_865)
                    .ok_or_else(invalid)?;
                fs::File::create_new(path(&result.root, text(op, "path")?)?)?.set_len(size)?;
            }
            "many_files" => {
                let count = op["count"]
                    .as_u64()
                    .filter(|n| *n <= 4097)
                    .ok_or_else(invalid)?;
                for index in 0..count {
                    write(&result.root, &format!("extras/item-{index}"), b"")?;
                }
            }
            "deep_directory" => {
                let depth = op["depth"]
                    .as_u64()
                    .filter(|n| *n <= 33)
                    .ok_or_else(invalid)?;
                write(
                    &result.root,
                    &format!("{}data", "deep/".repeat(depth as usize)),
                    b"",
                )?;
            }
            "unix_socket" => result
                .sockets
                .push(UnixListener::bind(path(&result.root, text(op, "path")?)?)?),
            _ => return Err(invalid()),
        }
    }
    let mut profile = fixture["base_profile"].clone();
    patch(&mut profile, &case["profile_patch"])?;
    if let Some(names) = case.get("remove_profile") {
        for name in names.as_array().ok_or_else(invalid)? {
            profile
                .as_object_mut()
                .and_then(|map| map.remove(name.as_str()?))
                .ok_or_else(invalid)?;
        }
    }
    let mut metadata = fixture["base_metadata"].clone();
    patch(&mut metadata, &case["metadata_patch"])?;
    result.profile = match case.get("profile_raw") {
        Some(raw) => raw.as_str().ok_or_else(invalid)?.to_owned(),
        None => profile.to_string(),
    };
    result.metadata = match case.get("metadata_raw") {
        Some(raw) => raw.as_str().ok_or_else(invalid)?.to_owned(),
        None => metadata.to_string(),
    }
    .replace("$ROOT", result.root.to_str().ok_or_else(invalid)?);
    Ok(result)
}
