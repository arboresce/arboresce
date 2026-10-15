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
