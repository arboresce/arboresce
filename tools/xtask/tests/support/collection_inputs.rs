//! Measured preparation of literal collector inputs and owned native fixtures.

use super::{PreparedRoot, report_inputs};
use arboresce_xtask::{measurement, process};
use serde_json::{Map, Value, json};
use std::{
    fs, io,
    io::Write,
    os::unix::fs::symlink,
    path::PathBuf,
    time::{Duration, Instant},
};

pub fn hex(value: &Value, key: &str) -> io::Result<Vec<u8>> {
    let (pairs, remainder) = value[key]
        .as_str()
        .ok_or_else(invalid)?
        .as_bytes()
        .as_chunks::<2>();
    if !remainder.is_empty() {
        return Err(invalid());
    }
    pairs
        .iter()
        .map(|pair| {
            std::str::from_utf8(pair)
                .ok()
                .and_then(|text| u8::from_str_radix(text, 16).ok())
                .ok_or_else(invalid)
        })
        .collect()
}

pub fn read(owner: &PreparedRoot, case: &Value) -> io::Result<PathBuf> {
    let recipe = &case["recipe"];
    let path = owner.path.join("bounded-input");
    if recipe["ancestor_symlink"] == true {
        let real = owner.path.join("real");
        fs::create_dir(&real)?;
        fs::write(real.join("input"), recipe["file"].as_str().unwrap())?;
        symlink(real, &path)?;
        return Ok(path.join("input"));
    }
    if let Some(text) = recipe["file"].as_str() {
        fs::write(&path, text)?;
    } else if recipe["fifo"] == true {
        let mut command = measurement::command("mkfifo", None)?;
        command.arg(&path);
        if !process::output(&mut command)?.status.success() {
            return Err(invalid());
        }
    } else if let Some(target) = recipe["symlink"].as_str() {
        symlink(target, &path)?;
    } else if recipe["directory"] == true {
        fs::create_dir(&path)?;
    } else {
        let size = recipe["total_bytes"].as_u64().ok_or_else(invalid)?;
        if !(1..=67_108_864).contains(&size) {
            return Err(invalid());
        }
        let mut bytes = recipe["prefix"]
            .as_str()
            .ok_or_else(invalid)?
            .as_bytes()
            .to_vec();
        bytes.resize(usize::try_from(size - 1).map_err(|_| invalid())?, b' ');
        bytes.push(b'\n');
        fs::write(&path, bytes)?;
    }
    Ok(path)
}

pub fn object_change(owner: &PreparedRoot, case: &Value) -> io::Result<(PathBuf, Value)> {
    let path = owner.path.join("object");
    fs::write(&path, case["initial"].as_str().unwrap())?;
    let identity = arboresce_xtask::collection_prepared::fingerprint(
        &path,
        64 << 20,
        Instant::now() + Duration::from_secs(30),
    )
    .map_err(io::Error::other)?;
    let recipe = &case["recipe"];
    if let Some(text) = recipe["append"].as_str() {
        fs::OpenOptions::new()
            .append(true)
            .open(&path)?
            .write_all(text.as_bytes())?;
    } else if let Some(text) = recipe["replace"].as_str() {
        fs::write(&path, text)?;
    } else if recipe["remove"] == true {
        fs::remove_file(&path)?;
    }
    println!(
        "xtask object freshness {}",
        json!({"before":identity,"actual_after_bytes":fs::read(&path).ok(),"case":case["id"]})
    );
    Ok((path, identity["sha256"].clone()))
}

pub struct DeadlineOutput {
    pub bytes: Vec<u8>,
    pub deadline: Instant,
}
impl Write for DeadlineOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        while Instant::now() < self.deadline {
            std::thread::sleep(Duration::from_millis(1));
        }
        Ok(())
    }
}

fn invalid() -> io::Error {
    io::Error::other("collector fixture preparation failed")
}

/// Two inert filesystem namespaces exercise selection, never database readiness.
pub fn cohorts(
    owner: &PreparedRoot,
    manifest_input: &Value,
    observations: &Value,
    input: &Value,
) -> io::Result<Value> {
    use arboresce_xtask::{collection_services, inventory::sha256};
    use std::{
        os::unix::fs::{DirBuilderExt, MetadataExt},
        sync::atomic::Ordering,
        time::{SystemTime, UNIX_EPOCH},
    };
    struct Sockets(Vec<PathBuf>);
    impl Drop for Sockets {
        fn drop(&mut self) {
            for path in &self.0 {
                let _ = fs::remove_dir(path);
            }
        }
    }
    let root = owner.path.canonicalize()?;
    let native = root.join("native");
    fs::create_dir(&native)?;
    let mut native_files = Map::new();
    let mut tools = json!({});
    for name in ["postgres", "pg_ctl"] {
        let path = native.join(name);
        fs::write(&path, b"inert fixture tool\n")?;
        let hash = sha256(b"inert fixture tool\n");
        native_files.insert(
            path.to_str().ok_or_else(invalid)?.to_owned(),
            json!({"sha256":hash,"size":19}),
        );
        tools[name] = json!({"path":path,"sha256":hash});
    }
    let mut sockets = Sockets(Vec::new());
    let now: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_millis()
        .try_into()
        .map_err(io::Error::other)?;
    let mut manifest = manifest_input.clone();
    let mut descriptor_paths = Map::new();
    for name in ["legacy", "current"] {
        let base = root.join(name);
        fs::DirBuilder::new().mode(0o700).create(&base)?;
        let data = base.join("data");
        fs::DirBuilder::new().mode(0o700).create(&data)?;
        let socket = PathBuf::from(format!(
            "/private/tmp/cohort-{}-{}",
            std::process::id(),
            super::NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::DirBuilder::new().mode(0o700).create(&socket)?;
        sockets.0.push(socket.clone());
        let readiness = base.join("readiness.json");
        fs::write(&readiness, b"{}")?;
        let mut files = native_files.clone();
        for file in ["postgresql.conf", "pg_hba.conf", "postgresql.auto.conf"] {
            let path = data.join(file);
            fs::write(&path, b"inert configuration\n")?;
            files.insert(
                path.to_str().ok_or_else(invalid)?.to_owned(),
                json!({"sha256":sha256(b"inert configuration\n"),"size":20}),
            );
        }
        let mut record = observations[name]["record"].clone();
        record["socket"] = json!(socket);
        record["data"] = json!(data);
        record["lease_deadline_unix_ms"] = json!(now + 60000);
        record["readiness"] = json!({"path":readiness,"sha256":sha256(b"{}")});
        record["tools"] = tools.clone();
        record["files"] = json!(files);
        let descriptor = base.join("service.json");
        fs::write(&descriptor, serde_json::to_vec(&record)?)?;
        manifest["services"][name] = json!(descriptor);
        descriptor_paths.insert(name.to_owned(), json!(descriptor));
    }
    let selected = root.join("cohorts.json");
    let recipe = input["recipe"].as_str().ok_or_else(invalid)?;
    if recipe == "service-alias" {
        manifest["services"]["current"] = manifest["services"]["legacy"].clone();
    }
    fs::write(&selected, serde_json::to_vec(&manifest)?)?;
    if recipe == "manifest-link-cycle" {
        let original = fs::metadata(&selected)?;
        let bytes = fs::read(&selected)?;
        let before = collection_services::observe_selection(
            &selected,
            Instant::now() + Duration::from_secs(2),
        );
        let alias = root.join("manifest-alias.json");
        fs::hard_link(&selected, &alias)?;
        let linked = fs::metadata(&selected)?;
        let alias_metadata = fs::metadata(&alias)?;
        let during = collection_services::observe_selection(
            &selected,
            Instant::now() + Duration::from_secs(2),
        );
        let linked_bytes = fs::read(&selected)?;
        let alias_bytes = fs::read(&alias)?;
        fs::remove_file(&alias)?;
        let restored = fs::metadata(&selected)?;
        let after = collection_services::observe_selection(
            &selected,
            Instant::now() + Duration::from_secs(2),
        );
        let identity = |m: &fs::Metadata| (m.dev(), m.ino());
        return Ok(json!({
            "accepted":[before.is_ok(),during.is_ok(),after.is_ok()],
            "link_counts":[original.nlink(),linked.nlink(),restored.nlink()],
            "same_inode":identity(&original)==identity(&linked)
                &&identity(&linked)==identity(&alias_metadata)
                &&identity(&alias_metadata)==identity(&restored),
            "same_bytes":bytes==linked_bytes&&bytes==alias_bytes&&bytes==fs::read(&selected)?
        }));
    }
    match recipe {
        "manifest-symlink" => {
            let target = root.join("manifest-target.json");
            fs::rename(&selected, &target)?;
            symlink(target, &selected)?;
        }
        "manifest-directory" => {
            fs::remove_file(&selected)?;
            fs::create_dir(&selected)?;
        }
        "manifest-one-over" => {
            let mut bytes = fs::read(&selected)?;
            bytes.resize(4097, b' ');
            fs::write(&selected, bytes)?;
        }
        "regular"
        | "service-alias"
        | "service-byte-change"
        | "secondary-config-change"
        | "manifest-byte-change" => (),
        _ => return Err(invalid()),
    }
    let changed = matches!(
        recipe,
        "service-byte-change" | "secondary-config-change" | "manifest-byte-change"
    );
    let before = if changed {
        Some(
            collection_services::observe_selection(
                &selected,
                Instant::now() + Duration::from_secs(2),
            )
            .map_err(io::Error::other)?,
        )
    } else {
        None
    };
    match recipe {
        "service-byte-change" => fs::OpenOptions::new()
            .append(true)
            .open(descriptor_paths["current"].as_str().ok_or_else(invalid)?)?
            .write_all(b" ")?,
        "secondary-config-change" => {
            fs::write(root.join("current/data/postgresql.conf"), b"changed\n")?
        }
        "manifest-byte-change" => fs::OpenOptions::new()
            .append(true)
            .open(&selected)?
            .write_all(b" ")?,
        _ => (),
    }
    let after =
        collection_services::observe_selection(&selected, Instant::now() + Duration::from_secs(2));
    let same = after.as_ref().is_ok_and(|a| {
        before
            .as_ref()
            .is_none_or(|b| a.legacy == b.legacy && a.cohorts == b.cohorts)
    });
    Ok(json!({"accepted":same,"changed_after_initial":before.is_some() &&!same}))
}

pub fn prepared(base: &Value, case: &Value) -> io::Result<Value> {
    if case["action"] == "prepared" {
        return report_inputs::replace(base, &case["patch"]).map_err(|_| invalid());
    }
    let recipe = &case["recipe"];
    let count = recipe["files"].as_u64().ok_or_else(invalid)?;
    if count > 16385 {
        return Err(invalid());
    }
    let first = recipe["first_paths"].as_array().ok_or_else(invalid)?;
    let prefix = recipe["remaining_prefix"].as_str().ok_or_else(invalid)?;
    let mut resources = Map::new();
    for index in 0..count {
        let path = if let Some(value) = first.get(index as usize) {
            value.as_str().ok_or_else(invalid)?.to_owned()
        } else {
            format!("{prefix}{index}")
        };
        resources.insert(path, json!({"sha256":"2".repeat(64),"size":recipe["size"]}));
    }
    let mut result = base.clone();
    result["resources"] = Value::Object(resources);
    Ok(result)
}

pub fn physical(owner: &PreparedRoot, case: &Value) -> io::Result<PathBuf> {
    let root = owner.path.join("collector-intake");
    if case["missing_root"] == true {
        return Ok(root);
    }
    if case["root_file"] == true {
        fs::write(&root, b"file")?;
        return Ok(root);
    }
    fs::create_dir(&root)?;
    for (name, recipe) in case["entries"].as_object().ok_or_else(invalid)? {
        let path = root.join(name);
        fs::create_dir_all(path.parent().ok_or_else(invalid)?)?;
        if let Some(text) = recipe["file"].as_str() {
            fs::write(path, text)?;
        } else if let Some(target) = recipe["symlink"].as_str() {
            symlink(target, path)?;
        } else if recipe["fifo"] == true {
            let mut command = measurement::command("mkfifo", None)?;
            command.arg(path);
            if !process::output(&mut command)?.status.success() {
                return Err(invalid());
            }
        } else {
            return Err(invalid());
        }
    }
    Ok(root)
}
pub fn exercise_binary(case: &serde_json::Value) -> serde_json::Value {
    let mut binary = case["binary"].clone();
    for stream in ["stdout", "stderr"] {
        if let Some(count) = case["recipe"][format!("{stream}_bytes")].as_u64() {
            let repeat = case["recipe"][format!("{stream}_repeat")].as_str().unwrap();
            assert_eq!(repeat.len(), 1);
            assert!(count <= 8_388_609);
            binary["exercises"][0][format!("{stream}_utf8")] = repeat.repeat(count as usize).into();
        }
    }
    binary
}

pub fn literal_bytes(value: &Value) -> Vec<u8> {
    if let Some(text) = value.as_str() {
        text.as_bytes().to_vec()
    } else {
        value["bytes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|byte| u8::try_from(byte.as_u64().unwrap()).unwrap())
            .collect()
    }
}

/// Expand only independently admitted finite capture recipes, without an oracle.
pub fn export_parts(input: &Value) -> Vec<Vec<u8>> {
    let parts = input["parts"].as_array().unwrap();
    assert!(parts.len() <= 3);
    parts
        .iter()
        .map(|part| {
            let mut bytes = literal_bytes(&part["utf8"]);
            if let Some(size) = part["pad_to_bytes"].as_u64() {
                let size = usize::try_from(size).unwrap();
                assert!(size >= bytes.len() && size <= 8_388_609);
                bytes.resize(size, b' ');
            }
            if let Some(suffix) = part["suffix_bytes"].as_array() {
                bytes.extend(
                    suffix
                        .iter()
                        .map(|byte| u8::try_from(byte.as_u64().unwrap()).unwrap()),
                );
            }
            bytes
        })
        .collect()
}

/// Prepare physical component inputs; never execute the literal image payloads.
pub fn boundary(owner: &PreparedRoot, case: &Value) -> io::Result<Value> {
    use arboresce_xtask::{collection, collection_prepared as prepared, inventory::sha256};
    use std::os::unix::fs::PermissionsExt;
    let root = owner.path.canonicalize()?;
    let recipe = &case["recipe"];
    let deadline = if recipe["expired_deadline"] == true {
        Instant::now()
    } else {
        Instant::now() + Duration::from_secs(30)
    };
    let result = match case["action"].as_str().ok_or_else(invalid)? {
        "fresh" => {
            let source = root.join("source");
            fs::create_dir(&source)?;
            let output = match recipe["mode"].as_str().unwrap() {
                "fresh" => root.join("output"),
                "exists" => {
                    fs::create_dir(root.join("output"))?;
                    root.join("output")
                }
                "inside-source" => source.join("output"),
                "ancestor" => root.clone(),
                "missing-parent" => root.join("missing/output"),
                "filesystem-root" => PathBuf::from("/"),
                _ => return Err(invalid()),
            };
            collection::fresh(&source, output.to_str().unwrap())
                .map(|actual| json!(actual == output && actual.is_dir()))
        }
        "objects" => {
            let actual = root.join("actual-object");
            let path = if recipe["mode"] == "symlink" {
                root.join("object-link")
            } else {
                actual.clone()
            };
            if recipe["mode"] != "missing" {
                fs::write(&actual, recipe["payload_utf8"].as_str().unwrap())?;
                fs::set_permissions(
                    &actual,
                    fs::Permissions::from_mode(
                        u32::from_str_radix(recipe["mode_octal"].as_str().unwrap(), 8)
                            .map_err(|_| invalid())?,
                    ),
                )?;
            }
            if recipe["mode"] == "symlink" {
                symlink(recipe["symlink_target"].as_str().unwrap(), &path)?;
            }
            let mut selection = recipe["selection"].clone();
            if let Some(rows) = selection["objects"].as_array_mut() {
                for row in rows {
                    if row["path"] == "$object" {
                        row["path"] = json!(path);
                    }
                }
            }
            collection::objects(&selection, deadline).map(|rows| {
                json!(
                    rows.iter()
                        .map(|row| json!({"sha256":row["sha256"],"test_count":row["test_count"]}))
                        .collect::<Vec<_>>()
                )
            })
        }
        "receipts" => {
            let directory = root.join("receipts");
            fs::create_dir(&directory)?;
            let path = directory.join(recipe["filename"].as_str().unwrap());
            if recipe["mode"] == "directory" {
                fs::create_dir(path)?;
            } else {
                fs::write(path, recipe["content"].as_str().unwrap())?;
            }
            collection::receipt_directory(&directory, "native-launch-v1", deadline).map(|rows| {
                json!(
                    rows.iter()
                        .map(|row| row["record"].clone())
                        .collect::<Vec<_>>()
                )
            })
        }
        "profile-name" => {
            collection::profile_pid(recipe["name"].as_str().unwrap()).map(|pid| json!(pid))
        }
        "dependency-malformed" => {
            let mut object = recipe["object"].clone();
            if let Some(path) = object["path"].as_str() {
                object["path"] = path.replace("$owner", root.to_str().unwrap()).into();
            }
            Err(collection::binary_dependency(&object, deadline)
                .err()
                .ok_or_else(invalid)?)
        }
        "compiled-sources" => {
            let source = physical(owner, recipe)?.canonicalize()?;
            let target = source.join("image");
            let dependency = recipe["dependency"]
                .as_str()
                .unwrap()
                .replace("$target", target.to_str().unwrap());
            collection::compiled_sources(&source, &recipe["snapshot"], &target, &dependency)
                .map(|sources| json!(sources))
        }
        "binary-counts" => {
            let actual = recipe["actual_sources"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_owned())
                .collect();
            let report = recipe["report_utf8"]
                .as_str()
                .unwrap()
                .replace("$root", root.to_str().unwrap());
            collection::binary_counts(&root, &actual, &report).map(|counts| json!(counts))
        }
        "preflight" => {
            let fixture = physical(owner, recipe)?.canonicalize()?;
            if let Some(base) = recipe.get("prepared") {
                let mut base = base.clone();
                let context = fixture.join("different-declared-context");
                base["outer"] = json!({"context":context,"environment":{(measurement::ENV):context,"LLVM_PROFILE_FILE":fixture.join("outer-%p-%12m.profraw")}});
                fs::write(fixture.join("manifest.json"), format!("{base}\n"))?;
            }
            let arguments = recipe["args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| {
                    std::ffi::OsString::from(
                        v.as_str()
                            .unwrap()
                            .replace("$fixture", fixture.to_str().unwrap()),
                    )
                })
                .collect::<Vec<_>>();
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let exit = collection::run(&arguments, &mut stdout, &mut stderr);
            let retained = prepared::tree(
                &fixture,
                prepared::Limits {
                    files: 100,
                    per_file: 65536,
                    total: 1 << 20,
                    depth: 8,
                },
                deadline,
            )
            .map_err(io::Error::other)?;
            let tool_commands = retained
                .as_object()
                .unwrap()
                .keys()
                .filter(|p| p.contains("/commands/") && p.ends_with("/launch-command.json"))
                .count();
            Ok(
                json!({"exit":exit,"stdout":String::from_utf8_lossy(&stdout),"stderr":String::from_utf8_lossy(&stderr),"tool_commands":tool_commands}),
            )
        }
        "dependency" => {
            let image = &recipe["image"];
            let debug = root.join("debug");
            fs::create_dir(&debug)?;
            let path = debug.join(image["name"].as_str().unwrap());
            fs::write(&path, image["payload_utf8"].as_str().unwrap())?;
            fs::set_permissions(
                &path,
                fs::Permissions::from_mode(
                    u32::from_str_radix(image["mode_octal"].as_str().unwrap(), 8)
                        .map_err(|_| invalid())?,
                ),
            )?;
            if sha256(&fs::read(&path)?) != image["sha256"] {
                return Err(invalid());
            }
            let deps = debug.join("deps");
            if recipe["mode"] != "missing-directory" {
                fs::create_dir(&deps)?;
                for (name, entry) in recipe["entries"].as_object().unwrap() {
                    let candidate = deps.join(name);
                    if let Some(text) = entry["file"].as_str() {
                        fs::write(&candidate, text)?;
                        if let Some(size) = entry["size"].as_u64() {
                            fs::OpenOptions::new()
                                .write(true)
                                .open(candidate)?
                                .set_len(size)?;
                        }
                    } else {
                        symlink(entry["symlink"].as_str().unwrap(), candidate)?;
                    }
                }
                if let Some(count) = recipe["additional_files"]["count"].as_u64() {
                    for index in 0..count {
                        fs::write(
                            deps.join(format!(
                                "{}{index}",
                                recipe["additional_files"]["prefix"].as_str().unwrap()
                            )),
                            recipe["additional_files"]["payload_utf8"].as_str().unwrap(),
                        )?;
                    }
                }
                if fs::read_dir(&deps)?.count() as u64 != recipe["total_entries"].as_u64().unwrap()
                {
                    return Err(invalid());
                }
            }
            collection::binary_dependency(&json!({"path":path,"name":image["name"],"sha256":image["sha256"]}),deadline)
                .map(|(candidate, dependency)| json!({"candidate":candidate.file_name().unwrap().to_str().unwrap(),"dependency":dependency}))
        }
        "configs" => {
            let source = root.join("source");
            let home = root.join("home");
            fs::create_dir_all(source.join(".cargo"))?;
            fs::create_dir(root.join(".cargo"))?;
            fs::create_dir(&home)?;
            fs::write(root.join(".cargo/config"), b"ancestor\n")?;
            fs::write(home.join("config.toml"), b"home\n")?;
            if recipe["mode"] == "nonregular" {
                fs::create_dir(source.join(".cargo/config.toml"))?;
                prepared::configs(&source, &home, deadline)
            } else {
                let initial =
                    prepared::configs(&source, &home, deadline).map_err(io::Error::other)?;
                let local = |value: &Value| {
                    value
                        .as_object()
                        .unwrap()
                        .iter()
                        .filter_map(|(name, _)| {
                            std::path::Path::new(name)
                                .strip_prefix(&root)
                                .ok()
                                .map(|p| p.to_str().unwrap().to_owned())
                        })
                        .collect::<Vec<_>>()
                };
                let external = |value: &Value| {
                    value
                        .as_object()
                        .unwrap()
                        .iter()
                        .filter(|(name, _)| !std::path::Path::new(name).starts_with(&root))
                        .map(|(name, row)| (name.clone(), row.clone()))
                        .collect::<Map<_, _>>()
                };
                let added = source.join(".cargo/config.toml");
                fs::write(&added, b"added\n")?;
                let after_add =
                    prepared::configs(&source, &home, deadline).map_err(io::Error::other)?;
                fs::write(home.join("config.toml"), b"changed home\n")?;
                let after_change =
                    prepared::configs(&source, &home, deadline).map_err(io::Error::other)?;
                fs::remove_file(&added)?;
                let after_remove =
                    prepared::configs(&source, &home, deadline).map_err(io::Error::other)?;
                let home_key = home.join("config.toml").to_str().unwrap().to_owned();
                let external_unchanged = [&after_add, &after_change, &after_remove]
                    .iter()
                    .all(|value| external(value) == external(&initial));
                Ok(
                    json!({"initial":local(&initial),"added":after_add.as_object().unwrap().len()==initial.as_object().unwrap().len()+1&&after_add.get(added.to_str().unwrap()).is_some(),"content_changed":after_add[&home_key]!=after_change[&home_key],"removed":after_remove.get(added.to_str().unwrap()).is_none()&&local(&after_remove)==local(&initial),"external_unchanged":external_unchanged}),
                )
            }
        }
        "json" => {
            let path = root.join("input.json");
            fs::write(&path, literal_bytes(&recipe["raw"]))?;
            prepared::read_json(&path, 65536).map(|(value, _)| value)
        }
        _ => return Err(invalid()),
    };
    let observed = match result {
        Ok(value) => json!({"value":value}),
        Err(error) => json!({"error":error.to_string()}),
    };
    println!(
        "xtask physical boundary {}",
        json!({"case":case["id"],"root":root,"observed":observed})
    );
    Ok(observed)
}

pub fn preparation_failure(owner: &PreparedRoot, case: &Value) -> io::Result<Value> {
    let recipe = &case["recipe"];
    let input = &recipe["input"];
    let result = match recipe["target"].as_str().unwrap() {
        "prepared" => prepared(&json!({"resources":{}}), input).map(|_| ()),
        "physical" => physical(owner, input).map(|_| ()),
        "read" => read(owner, input).map(|_| ()),
        "hex" => hex(input, "data").map(|_| ()),
        _ => return Err(invalid()),
    };
    let error = result.err().ok_or_else(invalid)?;
    Ok(json!({"error_kind":format!("{:?}",error.kind()),"message":error.to_string()}))
}

/// Filesystem-only service fixtures never assert that their synthetic tools run.
pub fn service(owner: &PreparedRoot, fixture: &Value, case: &Value) -> io::Result<Value> {
    use arboresce_xtask::{collection_services, inventory::sha256};
    use std::{
        os::unix::fs::{DirBuilderExt, PermissionsExt},
        sync::atomic::Ordering,
        time::{SystemTime, UNIX_EPOCH},
    };
    struct Socket(PathBuf);
    impl Drop for Socket {
        fn drop(&mut self) {
            let _ = fs::remove_dir(&self.0);
        }
    }
    let root = owner.path.canonicalize()?;
    let socket = Socket(PathBuf::from(format!(
        "/private/tmp/arboresce-input-{}-{}",
        std::process::id(),
        super::NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::DirBuilder::new().mode(0o700).create(&socket.0)?;
    let data = root.join("data");
    fs::DirBuilder::new().mode(0o700).create(&data)?;
    let native = root.join("native");
    fs::create_dir(&native)?;
    let readiness = root.join("readiness.json");
    fs::write(&readiness, b"{}")?;
    let mut files = Map::new();
    let mut value = fixture["base_input"].clone();
    value["socket"] = json!(socket.0);
    value["data"] = json!(data);
    let now: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_millis()
        .try_into()
        .map_err(io::Error::other)?;
    value["lease_deadline_unix_ms"] = json!(now + 60000);
    value["readiness"] = json!({"path":readiness,"sha256":sha256(b"{}")});
    for (path, bytes) in [
        (native.join("postgres"), b"x".as_slice()),
        (native.join("pg_ctl"), b"x".as_slice()),
        (data.join("postgresql.conf"), b"x".as_slice()),
        (data.join("pg_hba.conf"), b"x".as_slice()),
        (data.join("postgresql.auto.conf"), b"x".as_slice()),
    ] {
        fs::write(&path, bytes)?;
        files.insert(
            path.to_str().ok_or_else(invalid)?.to_owned(),
            json!({"sha256":sha256(bytes),"size":bytes.len()}),
        );
    }
    files.insert(
        readiness.to_str().ok_or_else(invalid)?.to_owned(),
        json!({"sha256":sha256(b"{}"),"size":2}),
    );
    value["files"] = Value::Object(files);
    for name in ["postgres", "pg_ctl"] {
        value["tools"][name] = json!({"path":native.join(name),"sha256":sha256(b"x")});
    }
    let input = root.join("service.json");
    fs::write(&input, serde_json::to_vec(&value)?)?;
    let paths = [
        ("input", input.clone()),
        ("native", native.clone()),
        ("postgres", native.join("postgres")),
        ("pg_ctl", native.join("pg_ctl")),
        ("configuration", data.join("postgresql.conf")),
        ("hba", data.join("pg_hba.conf")),
        ("auto", data.join("postgresql.auto.conf")),
        ("readiness", readiness.clone()),
        ("data", data.clone()),
        ("socket", socket.0.clone()),
    ];
    let physical_before = service_files(&paths)?;
    let before = collection_services::observe(&input, Instant::now() + Duration::from_secs(2))
        .map_err(io::Error::other)?;
    let mut environment = Value::Null;
    if let Some(size) = case["target_bytes"].as_u64() {
        let mut bytes = serde_json::to_vec(&value)?;
        let size: usize = size.try_into().map_err(io::Error::other)?;
        if size < bytes.len() || size > 2_097_153 {
            return Err(invalid());
        }
        bytes.resize(size, b' ');
        fs::write(&input, bytes)?;
    } else {
        match case["recipe"].as_str().ok_or_else(invalid)? {
            "none" => (),
            "remove-input" => fs::remove_file(&input)?,
            "change-tool" => fs::write(native.join("postgres"), b"y")?,
            "remove-tool" => fs::remove_file(native.join("postgres"))?,
            "change-configuration" => fs::write(data.join("postgresql.conf"), b"y")?,
            "change-readiness" => fs::write(&readiness, b"[]")?,
            "change-profile" => {
                let mut bytes = serde_json::to_vec(&value)?;
                bytes.push(b' ');
                fs::write(&input, bytes)?;
            }
            "symlink-tool" => {
                fs::rename(native.join("postgres"), native.join("original-postgres"))?;
                symlink(native.join("original-postgres"), native.join("postgres"))?;
            }
            "symlink-ancestor" => {
                let real = root.join("original-native");
                fs::rename(&native, &real)?;
                symlink(real, &native)?;
            }
            "data-mode-0755" => fs::set_permissions(&data, fs::Permissions::from_mode(0o755))?,
            "socket-mode-0755" => {
                fs::set_permissions(&socket.0, fs::Permissions::from_mode(0o755))?
            }
            "expire-lease" => {
                value["lease_deadline_unix_ms"] = json!(now - 1);
                fs::write(&input, serde_json::to_vec(&value)?)?;
            }
            "explicit-environment" => {
                let mut command =
                    std::process::Command::new(env!("CARGO_BIN_EXE_arboresce-xtask-test-peer"));
                command
                    .env("ARBORESCE_XTASK_TEST_PEER", "service-environment")
                    .env("PGHOST", "unselected")
                    .env("PGDATABASE", "unselected")
                    .env(collection_services::SELECTOR, "/unselected/input.json");
                collection_services::environment(&mut command, Some(&input));
                let output = super::observe(&mut command)?;
                if !output.status.success() {
                    return Err(invalid());
                }
                environment = serde_json::from_slice(&output.stdout)?;
            }
            _ => return Err(invalid()),
        }
    }
    let after_deadline = Instant::now() + Duration::from_secs(2);
    let after = collection_services::observe(&input, after_deadline);
    if Instant::now() >= after_deadline {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "service fixture observation exceeded its deadline",
        ));
    }
    let physical_after = service_files(&paths)?;
    let unchanged_required = case["recipe"] == "change-profile";
    let accepted = match &after {
        Ok(observed) => !unchanged_required || observed == &before,
        Err(_) => false,
    };
    Ok(
        json!({"accepted":accepted,"before":before,"after":after.as_ref().ok(),
        "error":after.as_ref().err().map(ToString::to_string),
        "error_class":after.as_ref().err().map(|error|format!("{error:?}")),
        "physical_before":physical_before,"physical_after":physical_after,
        "environment":environment,"input_path":input}),
    )
}

fn service_files(paths: &[(&str, PathBuf)]) -> io::Result<Value> {
    use arboresce_xtask::inventory::sha256;
    use std::os::unix::fs::PermissionsExt;
    let mut rows = Map::new();
    for (name, path) in paths {
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                rows.insert((*name).to_owned(), json!({"path":path,"kind":"absent"}));
                continue;
            }
            Err(error) => return Err(error),
        };
        let mut row = json!({"path":path,"mode":metadata.permissions().mode() & 0o777});
        if metadata.is_file() {
            if metadata.len() > 2_097_153 {
                return Err(invalid());
            }
            let bytes = fs::read(path)?;
            row["kind"] = json!("file");
            row["size"] = json!(bytes.len());
            row["sha256"] = json!(sha256(&bytes));
            if *name == "input" && bytes.len() <= 2_097_152 {
                let profile: Value = serde_json::from_slice(&bytes)?;
                row["lease_deadline_unix_ms"] = profile["lease_deadline_unix_ms"].clone();
            }
        } else if metadata.is_symlink() {
            row["kind"] = json!("symlink");
            row["target"] = json!(fs::read_link(path)?);
        } else {
            row["kind"] = json!("directory");
        }
        rows.insert((*name).to_owned(), row);
    }
    Ok(Value::Object(rows))
}
