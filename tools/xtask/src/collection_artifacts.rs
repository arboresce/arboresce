//! Reconcile actual Cargo output with complete reviewed typed selections.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

use crate::{
    collection_inputs::{CollectError, MAX_PROTOCOL_BYTES, Result},
    collection_tests, strict_json,
};

const BAD: CollectError = CollectError::Artifacts;
type Key = (String, String, String);

#[derive(Debug)]
pub struct Exercise {
    pub arguments: Vec<String>,
    pub exit: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// Exact ordinary-program vectors; absence retains the existing native test path.
pub fn exercises(binary: &Value) -> Result<Vec<Exercise>> {
    let binary = binary.as_object().ok_or(BAD)?;
    let Some(vectors) = binary.get("exercises") else {
        return Ok(Vec::new());
    };
    vectors
        .as_array()
        .ok_or(BAD)?
        .iter()
        .map(|vector| {
            let record = vector.as_object().ok_or(BAD)?;
            if record.keys().map(String::as_str).collect::<BTreeSet<_>>()
                != BTreeSet::from(["arguments", "exit", "stdout_utf8", "stderr_utf8"])
            {
                return Err(BAD);
            }
            let arguments = rows(vector, "arguments")?;
            if arguments.len() > 256 {
                return Err(BAD);
            }
            let arguments = arguments
                .iter()
                .map(|argument| {
                    let argument = argument.as_str().ok_or(BAD)?;
                    if argument.len() > 4096 || argument.contains('\0') {
                        return Err(BAD);
                    }
                    Ok(argument.to_owned())
                })
                .collect::<Result<Vec<_>>>()?;
            let exit = vector["exit"].as_u64().filter(|v| *v <= 255).ok_or(BAD)? as i32;
            let stdout = text(vector, "stdout_utf8")?.as_bytes();
            let stderr = text(vector, "stderr_utf8")?.as_bytes();
            if stdout.len() > MAX_PROTOCOL_BYTES || stderr.len() > MAX_PROTOCOL_BYTES {
                return Err(BAD);
            }
            Ok(Exercise {
                arguments,
                exit,
                stdout: stdout.to_vec(),
                stderr: stderr.to_vec(),
            })
        })
        .collect()
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value.get(key).and_then(Value::as_str).ok_or(BAD)
}

fn rows<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array).ok_or(BAD)
}

fn portable(path: &str, absolute: bool) -> bool {
    let path = if absolute {
        let Some(path) = path.strip_prefix('/') else {
            return false;
        };
        path
    } else {
        path
    };
    !path.is_empty()
        && !path.contains('\\')
        && !path.chars().any(char::is_control)
        && path.split('/').all(|p| !matches!(p, "" | "." | ".."))
}

fn names(value: &Value) -> Result<BTreeSet<String>> {
    let mut result = BTreeSet::new();
    for value in value.as_array().ok_or(BAD)? {
        if !result.insert(value.as_str().ok_or(BAD)?.to_owned()) {
            return Err(BAD);
        }
    }
    Ok(result)
}

fn key(package: &str, value: &Value) -> Result<Key> {
    Ok((
        package.to_owned(),
        text(value, "kind")?.to_owned(),
        text(value, "name")?.to_owned(),
    ))
}

fn target_key(package: &str, target: &Value) -> Result<Key> {
    let [kind] = rows(target, "kind")?.as_slice() else {
        return Err(BAD);
    };
    let kind = kind.as_str().ok_or(BAD)?;
    if !matches!(kind, "lib" | "bin" | "test") {
        return Err(BAD);
    }
    Ok((
        package.to_owned(),
        kind.to_owned(),
        text(target, "name")?.to_owned(),
    ))
}

struct Selection<'a> {
    packages: BTreeMap<&'a str, &'a Value>,
    owned: BTreeMap<&'a str, &'a str>,
    features: BTreeMap<&'a str, BTreeSet<String>>,
    targets: BTreeMap<Key, &'a Value>,
    harnesses: BTreeMap<Key, &'a Value>,
    binaries: BTreeMap<Key, &'a Value>,
}

impl<'a> Selection<'a> {
    fn open(root: &str, metadata: &'a Value, registry: &'a Value) -> Result<Self> {
        if registry["version"] != "rust-test-suites-v1"
            || registry["profile"] != "rust-1.99.0-macos-arm64-default-v1"
            || metadata["workspace_root"] != root
        {
            return Err(BAD);
        }
        let mut packages = BTreeMap::new();
        for package in rows(metadata, "packages")? {
            if packages.insert(text(package, "id")?, package).is_some() {
                return Err(BAD);
            }
        }
        let mut owned = BTreeMap::new();
        let mut targets = BTreeMap::new();
        for member in rows(metadata, "workspace_members")? {
            let id = member.as_str().ok_or(BAD)?;
            let package = packages.get(id).ok_or(BAD)?;
            let name = text(package, "name")?;
            if owned.insert(id, name).is_some() {
                return Err(BAD);
            }
            for target in rows(package, "targets")? {
                let key = target_key(name, target)?;
                if targets.insert(key, target).is_some() {
                    return Err(BAD);
                }
            }
        }
        let expected_features = registry["features"].as_object().ok_or(BAD)?;
        if owned.is_empty()
            || expected_features
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>()
                != owned.values().copied().collect()
        {
            return Err(BAD);
        }
        let mut features = BTreeMap::new();
        for node in rows(&metadata["resolve"], "nodes")? {
            let id = text(node, "id")?;
            if let Some(name) = owned.get(id) {
                let actual = names(&node["features"])?;
                if actual != names(expected_features.get(*name).ok_or(BAD)?)?
                    || features.insert(*name, actual).is_some()
                {
                    return Err(BAD);
                }
            }
        }
        if features.len() != owned.len() {
            return Err(BAD);
        }
        let mut harnesses = BTreeMap::new();
        let mut required = 0;
        let mut case_ids = BTreeSet::new();
        for harness in rows(registry, "harnesses")? {
            let key = key(text(harness, "package")?, harness)?;
            let selected = collection_tests::selected_names(&harness["tests"]).map_err(|_| BAD)?;
            let needed = harness["required"].as_bool().ok_or(BAD)?;
            if needed != !selected.is_empty()
                || !targets.contains_key(&key)
                || harnesses.insert(key, harness).is_some()
            {
                return Err(BAD);
            }
            required += usize::from(needed);
            for test in rows(harness, "tests")? {
                for id in rows(test, "case_ids")? {
                    if !case_ids
                        .insert((test["prefix"].clone().to_string(), id.clone().to_string()))
                    {
                        return Err(BAD);
                    }
                }
            }
        }
        if required == 0 || harnesses.keys().ne(targets.keys()) {
            return Err(BAD);
        }
        let mut binaries = BTreeMap::new();
        for binary in rows(registry, "native_binaries")? {
            exercises(binary)?;
            let key = key(text(binary, "package")?, binary)?;
            let target = targets.get(&key).ok_or(BAD)?;
            let sources = names(&binary["sources"])?;
            let source = text(target, "src_path")?
                .strip_prefix(&format!("{root}/"))
                .ok_or(BAD)?;
            if key.1 != "bin"
                || !sources.contains(source)
                || sources.iter().any(|p| !portable(p, false))
                || binaries.insert(key, binary).is_some()
            {
                return Err(BAD);
            }
        }
        if binaries.keys().cloned().collect::<BTreeSet<_>>()
            != targets
                .keys()
                .filter(|key| key.1 == "bin")
                .cloned()
                .collect()
        {
            return Err(BAD);
        }
        let mut docs = BTreeSet::new();
        for doc in rows(registry, "doctests")? {
            if doc["tests"].as_u64() != Some(0) || !docs.insert(text(doc, "package")?) {
                return Err(BAD);
            }
        }
        if docs != owned.values().copied().collect() {
            return Err(BAD);
        }
        Ok(Self {
            packages,
            owned,
            features,
            targets,
            harnesses,
            binaries,
        })
    }
}

/// Parse the complete actual no-run stream; physical artifacts are opened later.
pub fn select(
    root: &str,
    target: &str,
    metadata: &Value,
    registry: &Value,
    raw: &str,
) -> Result<Value> {
    if !portable(root, true)
        || !portable(target, true)
        || raw.len() > MAX_PROTOCOL_BYTES
        || !raw.ends_with('\n')
    {
        return Err(BAD);
    }
    let selection = Selection::open(root, metadata, registry)?;
    let mut finished = false;
    let mut selected = BTreeMap::new();
    let mut paths = BTreeSet::new();
    for line in raw.split_terminator('\n') {
        let artifact = strict_json::parse(line).ok_or(BAD)?;
        if finished {
            return Err(BAD);
        }
        let reason = text(&artifact, "reason")?;
        if reason == "build-finished" {
            if artifact["success"] != true {
                return Err(BAD);
            }
            finished = true;
            continue;
        }
        let id = text(&artifact, "package_id")?;
        if !selection.packages.contains_key(id) {
            return Err(BAD);
        }
        match reason {
            "compiler-message" => {
                if text(&artifact["message"], "level")? == "error" {
                    return Err(BAD);
                }
                continue;
            }
            "build-script-executed" if !selection.owned.contains_key(id) => continue,
            "compiler-artifact" => {}
            _ => return Err(BAD),
        }
        let Some(package) = selection.owned.get(id) else {
            continue;
        };
        let key = target_key(package, &artifact["target"])?;
        let metadata_target = selection.targets.get(&key).ok_or(BAD)?;
        if artifact["target"]["src_path"] != metadata_target["src_path"]
            || names(&artifact["features"])? != selection.features[package]
            || artifact["fresh"] != false
        {
            return Err(BAD);
        }
        let harness = artifact["profile"]["test"].as_bool().ok_or(BAD)?;
        if artifact["executable"].is_null() && key.1 == "lib" && !harness {
            continue;
        }
        let path = text(&artifact, "executable")?;
        if !portable(path, true)
            || !path.starts_with(&format!("{target}/"))
            || !names(&artifact["filenames"])?.contains(path)
            || !paths.insert(path.to_owned())
        {
            return Err(BAD);
        }
        let source = text(metadata_target, "src_path")?
            .strip_prefix(&format!("{root}/"))
            .ok_or(BAD)?;
        if !portable(source, false) {
            return Err(BAD);
        }
        let role = if harness { "harness" } else { "binary" };
        let mut object = json!({"package":package,"kind":key.1,"name":key.2,"role":role,"path":path,"source":source});
        if harness {
            let registered = selection.harnesses.get(&key).ok_or(BAD)?;
            object["required"] = registered["required"].clone();
            object["tests"] = registered["tests"].clone();
        } else {
            let registered = selection.binaries.get(&key).ok_or(BAD)?;
            object["sources"] = registered["sources"].clone();
            if let Some(exercises) = registered.get("exercises") {
                object["exercises"] = exercises.clone();
            }
        }
        if selected.insert((key, role), object).is_some() {
            return Err(BAD);
        }
    }
    let actual_harnesses: BTreeSet<_> = selected
        .keys()
        .filter(|(_, role)| *role == "harness")
        .map(|(key, _)| key.clone())
        .collect();
    let actual_binaries: BTreeSet<_> = selected
        .keys()
        .filter(|(_, role)| *role == "binary")
        .map(|(key, _)| key.clone())
        .collect();
    if !finished
        || actual_harnesses != selection.harnesses.keys().cloned().collect()
        || actual_binaries != selection.binaries.keys().cloned().collect()
    {
        return Err(BAD);
    }
    Ok(
        json!({"harnesses":actual_harnesses.len(),"binaries":actual_binaries.len(),"objects":selected.into_values().collect::<Vec<_>>()}),
    )
}
