use serde_json::{Value, json};

#[derive(Debug)]
pub struct InvalidInput;

pub struct Inputs {
    pub root: String,
    pub inventory: String,
    pub report: String,
}

pub fn replace(base: &Value, patch: &Value) -> Result<Value, InvalidInput> {
    let mut result = base.clone();
    for (pointer, replacement) in patch.as_object().ok_or(InvalidInput)? {
        *result.pointer_mut(pointer).ok_or(InvalidInput)? = replacement.clone();
    }
    Ok(result)
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, InvalidInput> {
    value.get(key).and_then(Value::as_str).ok_or(InvalidInput)
}

fn number(value: &Value, key: &str) -> Result<u64, InvalidInput> {
    value.get(key).and_then(Value::as_u64).ok_or(InvalidInput)
}

fn summary(value: &Value) -> Result<Value, InvalidInput> {
    let count = number(value, "count")?;
    let covered = number(value, "covered")?;
    let notcovered = count.checked_sub(covered).ok_or(InvalidInput)?;
    Ok(json!({"lines": {"count": count, "covered": covered},
              "regions": {"count": count, "covered": covered, "notcovered": notcovered}}))
}

pub fn pad(mut value: String, bytes: u64) -> Result<String, InvalidInput> {
    if bytes < value.len() as u64 || bytes > 67_108_865 {
        return Err(InvalidInput);
    }
    value.extend(std::iter::repeat_n(
        ' ',
        (bytes - value.len() as u64) as usize,
    ));
    Ok(value)
}

fn generated(recipe: &Value, root: &str) -> Result<(Value, Value), InvalidInput> {
    let count = number(recipe, "count")?;
    if count > 4097 {
        return Err(InvalidInput);
    }
    let prefix = text(recipe, "source_prefix")?;
    let suffix = text(recipe, "source_suffix")?;
    let package = text(recipe, "package")?;
    let role = text(recipe, "role")?;
    let metric = summary(recipe.get("metric").ok_or(InvalidInput)?)?;
    let total = summary(recipe.get("totals").ok_or(InvalidInput)?)?;
    let mut sources = Vec::new();
    let mut files = Vec::new();
    let mut paths = Vec::new();
    for index in 0..count {
        let path = format!("{prefix}{index}{suffix}");
        sources.push(json!({"path": path, "package": package, "role": role}));
        files.push(json!({"filename": format!("{root}/{path}"), "summary": metric}));
        paths.push(path);
    }
    Ok((
        json!({"version": "rust-report-roles-v1", "sources": sources,
               "targets": [{"kind": "binary", "name": package, "sources": paths}]}),
        json!({"type": "llvm.coverage.json.export", "version": "3.1.0",
               "data": [{"files": files, "totals": total, "functions": []}]}),
    ))
}

/// Build only literal input recipes; never derive production expected results.
pub fn build(fixture: &Value, case: &Value) -> Result<Inputs, InvalidInput> {
    let root = case
        .get("root")
        .and_then(Value::as_str)
        .or_else(|| fixture.get("root").and_then(Value::as_str))
        .ok_or(InvalidInput)?
        .to_owned();
    let (inventory, report) = if let Some(recipe) = case.get("generated_inputs") {
        generated(recipe, &root)?
    } else {
        (
            fixture.get("base_inventory").ok_or(InvalidInput)?.clone(),
            fixture.get("base_report").ok_or(InvalidInput)?.clone(),
        )
    };
    let empty = json!({});
    let inventory = replace(&inventory, case.get("inventory_patch").unwrap_or(&empty))?;
    let report = replace(&report, case.get("report_patch").unwrap_or(&empty))?;
    let mut inventory = if case.get("raw_inventory").is_some() {
        text(case, "raw_inventory")?.to_owned()
    } else {
        serde_json::to_string(&inventory).map_err(|_| InvalidInput)?
    };
    let mut report = if case.get("raw_report").is_some() {
        text(case, "raw_report")?.to_owned()
    } else {
        serde_json::to_string(&report).map_err(|_| InvalidInput)?
    };
    if let Some(recipe) = case.get("padded_input") {
        match text(recipe, "target")? {
            "inventory" => inventory = pad(inventory, number(recipe, "bytes")?)?,
            "report" => report = pad(report, number(recipe, "bytes")?)?,
            _ => return Err(InvalidInput),
        }
    }
    Ok(Inputs {
        root,
        inventory,
        report,
    })
}
