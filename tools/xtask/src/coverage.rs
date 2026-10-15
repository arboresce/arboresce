//! Strict counts for a supplied role inventory and selected LLVM export.
//! Inventory provenance, N/A proof and fresh native collection are separate gates.

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

use serde_json::Value;

use crate::strict_json;

pub const MAX_INPUT_BYTES: usize = strict_json::MAX_BYTES;
const MAX_SOURCES: usize = 4096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GateError {
    InvalidInput,
    Inventory,
    Counts,
    Threshold,
}

impl fmt::Display for GateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidInput => "invalid coverage input",
            Self::Inventory => "coverage inventory mismatch",
            Self::Counts => "invalid coverage counts",
            Self::Threshold => "coverage below strict threshold",
        })
    }
}
impl Error for GateError {}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Metric {
    pub count: u64,
    pub covered: u64,
}

impl Metric {
    pub fn strict(self) -> bool {
        self.count > 0
            && self.covered <= self.count
            && 10 * u128::from(self.covered) > 9 * u128::from(self.count)
    }

    fn from_value(value: &Value, regions: bool) -> Result<Self, GateError> {
        let count = value
            .get("count")
            .and_then(Value::as_u64)
            .ok_or(GateError::Counts)?;
        let covered = value
            .get("covered")
            .and_then(Value::as_u64)
            .ok_or(GateError::Counts)?;
        if count == 0 || covered > count {
            return Err(GateError::Counts);
        }
        if regions && value.get("notcovered").and_then(Value::as_u64) != Some(count - covered) {
            return Err(GateError::Counts);
        }
        Ok(Self { count, covered })
    }

    fn add(self, other: Self) -> Result<Self, GateError> {
        Ok(Self {
            count: self
                .count
                .checked_add(other.count)
                .ok_or(GateError::Counts)?,
            covered: self
                .covered
                .checked_add(other.covered)
                .ok_or(GateError::Counts)?,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Counts {
    pub lines: Metric,
    pub regions: Metric,
}

impl Counts {
    fn from_summary(value: &Value) -> Result<Self, GateError> {
        Ok(Self {
            lines: Metric::from_value(value.get("lines").ok_or(GateError::Counts)?, false)?,
            regions: Metric::from_value(value.get("regions").ok_or(GateError::Counts)?, true)?,
        })
    }

    fn add(self, other: Self) -> Result<Self, GateError> {
        Ok(Self {
            lines: self.lines.add(other.lines)?,
            regions: self.regions.add(other.regions)?,
        })
    }
}

#[derive(Debug)]
pub struct Report {
    pub scopes: BTreeMap<String, Counts>,
}

struct Source {
    package: String,
    measured: bool,
}

struct Inventory {
    sources: BTreeMap<String, Source>,
    targets: BTreeMap<String, BTreeSet<String>>,
}

fn named(value: &Value, key: &str) -> Result<String, GateError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or(GateError::Inventory)
}

fn relative(path: &str) -> bool {
    !path.is_empty()
        && !path.contains('\\')
        && !path.chars().any(char::is_control)
        && path.split('/').all(|part| !matches!(part, "" | "." | ".."))
}

fn name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

impl Inventory {
    fn from_value(value: &Value) -> Result<Self, GateError> {
        if value.get("version").and_then(Value::as_str) != Some("rust-report-roles-v1") {
            return Err(GateError::Inventory);
        }
        let rows = value
            .get("sources")
            .and_then(Value::as_array)
            .ok_or(GateError::Inventory)?;
        if rows.is_empty() || rows.len() > MAX_SOURCES {
            return Err(GateError::Inventory);
        }
        let mut sources = BTreeMap::new();
        let mut packages = BTreeSet::new();
        let mut measured_packages = BTreeSet::new();
        for row in rows {
            let path = named(row, "path")?;
            let package = named(row, "package")?;
            let measured = match named(row, "role")?.as_str() {
                "measured" => true,
                "assertion" => false,
                _ => return Err(GateError::Inventory),
            };
            if !relative(&path) || !name(&package) {
                return Err(GateError::Inventory);
            }
            packages.insert(package.clone());
            if measured {
                measured_packages.insert(package.clone());
            }
            if sources.insert(path, Source { package, measured }).is_some() {
                return Err(GateError::Inventory);
            }
        }
        if packages != measured_packages {
            return Err(GateError::Inventory);
        }
        let rows = value
            .get("targets")
            .and_then(Value::as_array)
            .ok_or(GateError::Inventory)?;
        if rows.is_empty() {
            return Err(GateError::Inventory);
        }
        let mut targets = BTreeMap::new();
        for row in rows {
            let kind = named(row, "kind")?;
            let target = named(row, "name")?;
            if !matches!(kind.as_str(), "library" | "binary") || !name(&target) {
                return Err(GateError::Inventory);
            }
            let paths = row
                .get("sources")
                .and_then(Value::as_array)
                .ok_or(GateError::Inventory)?;
            if paths.is_empty() {
                return Err(GateError::Inventory);
            }
            let mut selected = BTreeSet::new();
            for path in paths {
                let path = path.as_str().ok_or(GateError::Inventory)?;
                if !sources.get(path).is_some_and(|source| source.measured)
                    || !selected.insert(path.to_owned())
                {
                    return Err(GateError::Inventory);
                }
            }
            if targets
                .insert(format!("{kind}:{target}"), selected)
                .is_some()
            {
                return Err(GateError::Inventory);
            }
        }
        Ok(Self { sources, targets })
    }
}

fn add_scope(
    scopes: &mut BTreeMap<String, Counts>,
    key: String,
    counts: Counts,
) -> Result<(), GateError> {
    let current = scopes.get(&key).copied().unwrap_or_default();
    scopes.insert(key, current.add(counts)?);
    Ok(())
}

/// Evaluate supplied counts only. The caller must separately qualify the source
/// inventory, declaration N/A, exact report provenance and complete fresh runs.
pub fn evaluate(root: &str, inventory: &str, raw_report: &str) -> Result<Report, GateError> {
    let inventory = strict_json::parse(inventory).ok_or(GateError::InvalidInput)?;
    let report = strict_json::parse(raw_report).ok_or(GateError::InvalidInput)?;
    let inventory = Inventory::from_value(&inventory)?;
    if root != "/" && !root.strip_prefix('/').is_some_and(relative) {
        return Err(GateError::Inventory);
    }
    let prefix = if root == "/" {
        "/".to_owned()
    } else {
        format!("{root}/")
    };
    if report.get("type").and_then(Value::as_str) != Some("llvm.coverage.json.export")
        || report.get("version").and_then(Value::as_str) != Some("3.1.0")
    {
        return Err(GateError::InvalidInput);
    }
    let data = report
        .get("data")
        .and_then(Value::as_array)
        .ok_or(GateError::InvalidInput)?;
    let [data] = data.as_slice() else {
        return Err(GateError::InvalidInput);
    };
    let files = data
        .get("files")
        .and_then(Value::as_array)
        .ok_or(GateError::InvalidInput)?;
    if files.is_empty() {
        return Err(GateError::InvalidInput);
    }
    let mut seen = BTreeSet::new();
    let mut measured = BTreeMap::new();
    let mut total = Counts::default();
    for file in files {
        let path = file
            .get("filename")
            .and_then(Value::as_str)
            .and_then(|path| path.strip_prefix(&prefix))
            .filter(|path| relative(path))
            .ok_or(GateError::Inventory)?;
        let source = inventory.sources.get(path).ok_or(GateError::Inventory)?;
        if !seen.insert(path.to_owned()) {
            return Err(GateError::Inventory);
        }
        let counts = Counts::from_summary(file.get("summary").ok_or(GateError::Counts)?)?;
        total = total.add(counts)?;
        if source.measured {
            measured.insert(path.to_owned(), counts);
        }
    }
    if seen.len() != inventory.sources.len() {
        return Err(GateError::Inventory);
    }
    if total != Counts::from_summary(data.get("totals").ok_or(GateError::Counts)?)? {
        return Err(GateError::Counts);
    }
    let mut scopes = BTreeMap::new();
    for (path, counts) in &measured {
        let source = inventory.sources.get(path).ok_or(GateError::Inventory)?;
        scopes.insert(format!("file:{path}"), *counts);
        add_scope(&mut scopes, "repository:Rust".to_owned(), *counts)?;
        add_scope(&mut scopes, format!("package:{}", source.package), *counts)?;
    }
    for (target, paths) in &inventory.targets {
        for path in paths {
            let counts = measured.get(path).ok_or(GateError::Inventory)?;
            add_scope(&mut scopes, target.clone(), *counts)?;
        }
    }
    if scopes
        .values()
        .any(|counts| !counts.lines.strict() || !counts.regions.strict())
    {
        return Err(GateError::Threshold);
    }
    Ok(Report { scopes })
}
