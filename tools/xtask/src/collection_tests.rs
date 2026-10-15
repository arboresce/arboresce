//! Exact selected/executed libtest names and suite-scoped literal events.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::collection_inputs::{CollectError, MAX_PROTOCOL_BYTES, Result};

const PREFIXES: [&str; 3] = ["xtask case=", "identifier case=", "collector case="];

fn name(value: &str) -> bool {
    !value.is_empty() && !value.chars().any(char::is_whitespace)
}

pub fn list(raw: &str) -> Result<Vec<String>> {
    if raw.len() > MAX_PROTOCOL_BYTES || (!raw.is_empty() && !raw.ends_with('\n')) {
        return Err(CollectError::Selection);
    }
    let mut names = BTreeSet::new();
    for line in raw.split_terminator('\n') {
        let selected = line.strip_suffix(": test").ok_or(CollectError::Selection)?;
        if !name(selected) || !names.insert(selected.to_owned()) {
            return Err(CollectError::Selection);
        }
    }
    Ok(names.into_iter().collect())
}

struct Test {
    prefix: Option<String>,
    ids: BTreeSet<String>,
}

fn metadata(value: &Value) -> Result<BTreeMap<String, Test>> {
    let mut tests = BTreeMap::new();
    let mut all_ids = BTreeSet::new();
    for value in value.as_array().ok_or(CollectError::Input)? {
        if value.as_object().ok_or(CollectError::Input)?.len() != 3 {
            return Err(CollectError::Input);
        }
        let test_name = value["name"].as_str().ok_or(CollectError::Input)?;
        if !name(test_name) {
            return Err(CollectError::Input);
        }
        let prefix = match value.get("prefix").ok_or(CollectError::Input)? {
            Value::Null => None,
            Value::String(text) if PREFIXES.contains(&text.as_str()) => Some(text.clone()),
            _ => return Err(CollectError::Input),
        };
        let mut ids = BTreeSet::new();
        for value in value["case_ids"].as_array().ok_or(CollectError::Input)? {
            let id = value.as_str().ok_or(CollectError::Input)?;
            if !name(id)
                || prefix.is_none()
                || !ids.insert(id.to_owned())
                || !all_ids.insert((prefix.clone(), id.to_owned()))
            {
                return Err(CollectError::Input);
            }
        }
        if tests
            .insert(test_name.to_owned(), Test { prefix, ids })
            .is_some()
        {
            return Err(CollectError::Input);
        }
    }
    Ok(tests)
}

/// Validate the exact reviewed registry shape before selecting any harness.
pub fn selected_names(expected: &Value) -> Result<Vec<String>> {
    Ok(metadata(expected)?.into_keys().collect())
}

fn diagnostic(line: &str) -> Result<()> {
    if line.starts_with("test ")
        || line.starts_with("running ")
        || line.starts_with("successes:")
        || line.starts_with("failures:")
        || PREFIXES.iter().any(|prefix| line.starts_with(prefix))
    {
        return Err(CollectError::Execution);
    }
    Ok(())
}

fn results(raw: &str, tests: &BTreeMap<String, Test>) -> Result<()> {
    let count = tests.len();
    let noun = if count == 1 { "test" } else { "tests" };
    let body = raw
        .strip_prefix(&format!("\nrunning {count} {noun}\n"))
        .ok_or(CollectError::Execution)?;
    let mut lines = body.lines();
    for test in tests.keys() {
        let line = lines.next().ok_or(CollectError::Execution)?;
        let remaining = line
            .strip_prefix(&format!("test {test} ... "))
            .ok_or(CollectError::Execution)?;
        if remaining == "ok" {
            continue;
        }
        if matches!(remaining, "FAILED" | "ignored") {
            return Err(CollectError::Execution);
        }
        diagnostic(remaining)?;
        loop {
            let line = lines.next().ok_or(CollectError::Execution)?;
            if line == "ok" {
                break;
            }
            diagnostic(line)?;
        }
    }
    if lines.any(|line| !line.is_empty()) {
        return Err(CollectError::Execution);
    }
    Ok(())
}

fn events(raw: &str, test: &Test, pid: u32) -> Result<Vec<String>> {
    // The formatter adds one LF after each captured stdout buffer.
    let captured = raw.strip_suffix('\n').ok_or(CollectError::Execution)?;
    let mut ids = BTreeSet::new();
    for line in captured.split_inclusive('\n') {
        let plain = line.strip_suffix('\n').unwrap_or(line);
        if let Some(prefix) = PREFIXES.iter().find(|prefix| plain.starts_with(**prefix)) {
            if !line.ends_with('\n') || test.prefix.as_deref() != Some(*prefix) {
                return Err(CollectError::Execution);
            }
            let content = plain
                .strip_prefix(*prefix)
                .and_then(|text| text.strip_suffix(&format!(" pid={pid}")))
                .ok_or(CollectError::Execution)?;
            if !test.ids.contains(content) || !ids.insert(content.to_owned()) {
                return Err(CollectError::Execution);
            }
        }
    }
    if ids != test.ids {
        return Err(CollectError::Execution);
    }
    Ok(ids.into_iter().collect())
}

fn outputs(
    raw: &str,
    tests: &BTreeMap<String, Test>,
    pid: u32,
) -> Result<BTreeMap<String, Vec<String>>> {
    let mut observed = BTreeMap::new();
    let mut current: Option<&str> = None;
    let mut body = String::new();
    for line in raw.split_inclusive('\n') {
        if let Some(header) = line.strip_prefix("---- ") {
            if let Some(previous) = current.take() {
                let test = tests.get(previous).ok_or(CollectError::Execution)?;
                observed.insert(previous.to_owned(), events(&body, test, pid)?);
                body.clear();
            }
            let selected = header
                .strip_suffix(" stdout ----\n")
                .ok_or(CollectError::Execution)?;
            if !tests.contains_key(selected) || observed.contains_key(selected) {
                return Err(CollectError::Execution);
            }
            current = Some(selected);
        } else if current.is_some() {
            body.push_str(line);
        } else if line != "\n" {
            return Err(CollectError::Execution);
        }
    }
    if let Some(previous) = current {
        let test = tests.get(previous).ok_or(CollectError::Execution)?;
        observed.insert(previous.to_owned(), events(&body, test, pid)?);
    }
    for (test_name, test) in tests {
        let actual = observed.entry(test_name.clone()).or_default();
        if actual.iter().cloned().collect::<BTreeSet<_>>() != test.ids {
            return Err(CollectError::Execution);
        }
    }
    Ok(observed)
}

fn summary(raw: &str, tests: &BTreeMap<String, Test>) -> Result<()> {
    let names = tests
        .keys()
        .map(|name| format!("    {name}\n"))
        .collect::<String>();
    let prefix = format!(
        "{names}\ntest result: ok. {} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in ",
        tests.len()
    );
    let duration = raw
        .strip_prefix(&prefix)
        .and_then(|rest| rest.strip_suffix("s\n\n"))
        .ok_or(CollectError::Execution)?;
    let (seconds, fraction) = duration.split_once('.').ok_or(CollectError::Execution)?;
    if seconds.is_empty()
        || fraction.is_empty()
        || !seconds.bytes().all(|b| b.is_ascii_digit())
        || !fraction.bytes().all(|b| b.is_ascii_digit())
        || seconds.parse::<u64>().is_err()
        || fraction.parse::<u64>().is_err()
    {
        return Err(CollectError::Execution);
    }
    Ok(())
}

/// Compare complete execution with admitted per-test metadata and actual PID.
pub fn execution(raw: &str, expected: &Value, pid: u32) -> Result<BTreeMap<String, Vec<String>>> {
    let tests = metadata(expected)?;
    if raw.len() > MAX_PROTOCOL_BYTES || !raw.ends_with('\n') {
        return Err(CollectError::Execution);
    }
    let pieces: Vec<_> = raw.split("\nsuccesses:\n").collect();
    let [run, captured, tail] = pieces.as_slice() else {
        return Err(CollectError::Execution);
    };
    results(run, &tests)?;
    summary(tail, &tests)?;
    outputs(captured, &tests, pid)
}
