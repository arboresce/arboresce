//! Conserve supplied observed facts. Physical intake remains the caller's duty.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

use crate::{
    collection_artifacts,
    collection_inputs::{CollectError, Result},
    coverage::Metric,
};

const BAD: CollectError = CollectError::Conservation;

fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key).and_then(Value::as_str).ok_or(BAD)
}

fn number(v: &Value, key: &str) -> Result<u64> {
    v.get(key).and_then(Value::as_u64).ok_or(BAD)
}

fn rows<'a>(v: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    v.get(key).and_then(Value::as_array).ok_or(BAD)
}

fn hex(text: &str, size: usize) -> bool {
    text.len() == size
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn digest<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    let value = text(v, key)?;
    if !hex(value, 64) {
        return Err(BAD);
    }
    Ok(value)
}

fn bytes(v: &Value) -> Result<Vec<u8>> {
    v.as_array()
        .ok_or(BAD)?
        .iter()
        .map(|b| b.as_u64().and_then(|n| u8::try_from(n).ok()).ok_or(BAD))
        .collect()
}

fn pid(v: &Value, key: &str) -> Result<u64> {
    let value = number(v, key)?;
    if value == 0 || value > u64::from(u32::MAX) {
        return Err(BAD);
    }
    Ok(value)
}

fn names(v: &Value) -> Result<BTreeSet<String>> {
    let mut result = BTreeSet::new();
    for value in v.as_array().ok_or(BAD)? {
        let name = value.as_str().ok_or(BAD)?;
        if name.is_empty() || !result.insert(name.to_owned()) {
            return Err(BAD);
        }
    }
    Ok(result)
}

fn context(record: &Value, identity: &Value) -> Result<()> {
    for key in ["run_id", "source_sha256", "context_sha256", "context_path"] {
        if record.get(key) != identity.get(key) {
            return Err(BAD);
        }
    }
    Ok(())
}

fn wrappers<'a>(v: &'a Value, key: &str, version: &str) -> Result<BTreeMap<u64, &'a Value>> {
    let mut result = BTreeMap::new();
    for wrapper in rows(v, key)? {
        digest(wrapper, "sha256")?;
        let record = wrapper.get("record").ok_or(BAD)?;
        if text(record, "version")? != version
            || result.insert(pid(record, "pid")?, wrapper).is_some()
        {
            return Err(BAD);
        }
    }
    Ok(result)
}

fn status(exit: &Value) -> Result<()> {
    match (exit.get("exit"), exit.get("signal")) {
        (Some(code), Some(Value::Null)) if code.as_u64().is_some_and(|n| n <= 255) => Ok(()),
        (Some(Value::Null), Some(signal)) if signal.as_u64().is_some_and(|n| n > 0) => Ok(()),
        _ => Err(BAD),
    }
}

fn parent_chains(
    launches: &BTreeMap<u64, &Value>,
    owned: &BTreeSet<u64>,
    driver: u64,
) -> Result<()> {
    for (child, wrapper) in launches {
        let mut seen = BTreeSet::from([*child]);
        let mut parent = pid(&wrapper["record"], "parent_pid")?;
        while parent != driver {
            if !owned.contains(&parent) || !seen.insert(parent) {
                return Err(BAD);
            }
            parent = pid(&launches.get(&parent).ok_or(BAD)?["record"], "parent_pid")?;
        }
    }
    Ok(())
}

fn command_arguments(role: &str) -> Result<Value> {
    let args: &[&str] = match role {
        "list" => &["--list", "--format=terse"],
        "run" => &["--show-output", "--test-threads=1", "--color=never"],
        _ => return Err(BAD),
    };
    Ok(json!(args.iter().map(|s| s.as_bytes()).collect::<Vec<_>>()))
}

fn probe_arguments(test: &str) -> Value {
    json!([
        b"--exact".as_slice(),
        test.as_bytes(),
        b"--nocapture".as_slice(),
        b"--test-threads=1".as_slice()
    ])
}

fn scopes(
    facts: &Value,
    objects: &BTreeMap<&str, &Value>,
    profiles: &BTreeMap<u64, &Value>,
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for scope in rows(facts, "binary_scopes")? {
        let hash = digest(scope, "object_sha256")?;
        let object = objects.get(hash).ok_or(BAD)?;
        if object["role"] != "binary" || scope["name"] != object["name"] || !seen.insert(hash) {
            return Err(BAD);
        }
        let actual = names(&scope["actual_sources"])?;
        if actual.is_empty()
            || actual != names(&scope["registered_sources"])?
            || actual != names(&object["sources"])?
        {
            return Err(BAD);
        }
        let expected_pids: BTreeSet<_> = profiles
            .iter()
            .filter(|(_, p)| p["object_sha256"] == hash)
            .map(|(pid, _)| *pid)
            .collect();
        let mut selected = BTreeSet::new();
        for selected_pid in rows(scope, "profile_pids")? {
            if !selected.insert(selected_pid.as_u64().ok_or(BAD)?) {
                return Err(BAD);
            }
        }
        if expected_pids.is_empty() || selected != expected_pids {
            return Err(BAD);
        }
        let counts = scope["counts"].as_object().ok_or(BAD)?;
        if counts.keys().cloned().collect::<BTreeSet<_>>() != actual {
            return Err(BAD);
        }
        let mut total = Metric::default();
        for count in counts.values() {
            let metric = Metric {
                count: number(count, "count")?,
                covered: number(count, "covered")?,
            };
            if metric.covered > metric.count
                || number(count, "notcovered")? != metric.count - metric.covered
            {
                return Err(BAD);
            }
            total.count = total.count.checked_add(metric.count).ok_or(BAD)?;
            total.covered = total.covered.checked_add(metric.covered).ok_or(BAD)?;
        }
        if !total.strict() {
            return Err(BAD);
        }
    }
    let binaries: BTreeSet<_> = objects
        .iter()
        .filter(|(_, object)| object["role"] == "binary")
        .map(|(hash, _)| *hash)
        .collect();
    if seen != binaries {
        return Err(BAD);
    }
    Ok(())
}

/// Validate supplied observed facts only; no supplied hash attests actual intake.
pub fn evaluate(facts: &Value) -> Result<Value> {
    if facts["version"] != "native-collection-facts-v1" {
        return Err(CollectError::Input);
    }
    let before = facts.get("inputs_before").ok_or(BAD)?;
    if before != facts.get("inputs_after").ok_or(BAD)? {
        return Err(CollectError::Changed);
    }
    for key in [
        "source",
        "manifests",
        "tools",
        "resources",
        "configs",
        "native",
    ] {
        before.get(key).and_then(Value::as_object).ok_or(BAD)?;
    }
    let identity = facts.get("context").ok_or(BAD)?;
    if !hex(text(identity, "run_id")?, 32)
        || bytes(&identity["context_path"])?.first() != Some(&b'/')
        || number(identity, "started_ns")? > number(identity, "finished_ns")?
    {
        return Err(BAD);
    }
    digest(identity, "source_sha256")?;
    digest(identity, "context_sha256")?;
    let driver = pid(identity, "driver_pid")?;
    let mut objects = BTreeMap::new();
    let mut paths = BTreeSet::new();
    let mut declared_probes = BTreeMap::new();
    let mut required = 0;
    let mut harnesses = 0;
    for object in rows(facts, "objects")? {
        let hash = digest(object, "sha256")?;
        let path = text(object, "path")?;
        if !path.starts_with('/') || !paths.insert(path) || objects.insert(hash, object).is_some() {
            return Err(BAD);
        }
        match text(object, "role")? {
            "binary" => {
                if object.get("descendant_probes").is_some() {
                    return Err(BAD);
                }
            }
            "harness" => {
                harnesses += 1;
                let is_required = object["required"].as_bool().ok_or(BAD)?;
                if is_required != (number(object, "test_count")? > 0) {
                    return Err(BAD);
                }
                required += usize::from(is_required);
                declared_probes.insert(
                    hash,
                    collection_artifacts::descendant_probes(object).map_err(|_| BAD)?,
                );
            }
            _ => return Err(BAD),
        }
    }
    if required == 0 {
        return Err(BAD);
    }
    let launches = wrappers(facts, "launches", "native-launch-v1")?;
    let exits = wrappers(facts, "exits", "native-exit-v1")?;
    if launches.keys().ne(exits.keys()) || launches.contains_key(&driver) {
        return Err(BAD);
    }
    let tools = before["tools"].as_object().ok_or(BAD)?;
    let mut owned = BTreeSet::new();
    let mut external = 0;
    for (process_id, wrapper) in &launches {
        let launch = &wrapper["record"];
        let exit = &exits[process_id]["record"];
        context(launch, identity)?;
        context(exit, identity)?;
        status(exit)?;
        if exit["parent_pid"] != launch["parent_pid"] || exit["launch_sha256"] != wrapper["sha256"]
        {
            return Err(BAD);
        }
        let hash = digest(launch, "executable_sha256")?;
        if let Some(object) = objects.get(hash) {
            if bytes(&launch["executable"])? != text(object, "path")?.as_bytes() {
                return Err(BAD);
            }
            owned.insert(*process_id);
        } else if tools.values().any(|tool| tool.as_str() == Some(hash)) {
            external += 1;
        } else {
            return Err(BAD);
        }
    }
    parent_chains(&launches, &owned, driver)?;
    let mut commands = BTreeMap::new();
    let mut selection = BTreeSet::new();
    for command in rows(facts, "commands")? {
        let process_id = pid(command, "pid")?;
        let hash = digest(command, "object_sha256")?;
        let object = objects.get(hash).ok_or(BAD)?;
        let role = text(command, "role")?;
        let launch = &launches.get(&process_id).ok_or(BAD)?["record"];
        if object["role"] != "harness"
            || launch["executable_sha256"] != hash
            || launch["arguments"] != command_arguments(role)?
            || launch["parent_pid"] != driver
            || !selection.insert((hash, role))
            || commands.insert(process_id, command).is_some()
        {
            return Err(BAD);
        }
    }
    for (hash, object) in &objects {
        if object["role"] == "harness"
            && (!selection.contains(&(*hash, "list")) || !selection.contains(&(*hash, "run")))
        {
            return Err(BAD);
        }
    }
    let mut profiles = BTreeMap::new();
    let mut raw = serde_json::Map::new();
    let mut observed_probes = BTreeSet::new();
    for profile in rows(facts, "profiles")? {
        let process_id = pid(profile, "pid")?;
        let launch = &launches.get(&process_id).ok_or(BAD)?["record"];
        let hash = digest(profile, "object_sha256")?;
        let object = objects.get(hash).ok_or(BAD)?;
        let filename = text(profile, "name")?;
        let signature = filename
            .strip_prefix(&format!("arboresce-{process_id}-"))
            .and_then(|s| s.strip_suffix(".profraw"))
            .ok_or(BAD)?;
        let size = number(profile, "size")?;
        let modified = number(profile, "modified_ns")?;
        let positive = number(profile, "positive_functions")?;
        if signature.is_empty()
            || !signature
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
            || size == 0
            || modified < number(identity, "started_ns")?
            || modified > number(identity, "finished_ns")?
            || positive > number(profile, "functions")?
            || digest(profile, "sha256")? != digest(profile, "after_sha256")?
            || launch["executable_sha256"] != hash
            || profiles.insert(process_id, profile).is_some()
        {
            return Err(BAD);
        }
        let required_run = commands
            .get(&process_id)
            .is_some_and(|command| command["role"] == "run" && object["required"] == true);
        if (object["role"] == "binary" || required_run) && positive == 0 {
            return Err(BAD);
        }
        if object["role"] == "harness" {
            if exits[&process_id]["record"]["exit"] != 0 {
                return Err(BAD);
            }
            if !commands.contains_key(&process_id) {
                let parent = commands.get(&pid(launch, "parent_pid")?).ok_or(BAD)?;
                let test = declared_probes[hash]
                    .iter()
                    .find(|test| launch["arguments"] == probe_arguments(test))
                    .ok_or(BAD)?;
                if parent["role"] != "run"
                    || parent["object_sha256"] != hash
                    || positive == 0
                    || !observed_probes.insert((hash, test.clone()))
                {
                    return Err(BAD);
                }
            }
        }
        if raw
            .insert(
                filename.to_owned(),
                json!({"sha256":profile["sha256"],"size":size}),
            )
            .is_some()
        {
            return Err(BAD);
        }
    }
    if profiles.keys().copied().collect::<BTreeSet<_>>() != owned
        || facts["raw_after"] != Value::Object(raw)
    {
        return Err(BAD);
    }
    let expected_probes: BTreeSet<_> = declared_probes
        .iter()
        .flat_map(|(hash, tests)| tests.iter().map(move |test| (*hash, test.clone())))
        .collect();
    if observed_probes != expected_probes {
        return Err(BAD);
    }
    scopes(facts, &objects, &profiles)?;
    Ok(json!({
        "objects":objects.len(),"profiles":profiles.len(),
        "native_binaries":objects.len()-harnesses,"harnesses":harnesses,
        "required_suites":required,"external_processes":external,
        "launch_exit_pairs":launches.len()
    }))
}
