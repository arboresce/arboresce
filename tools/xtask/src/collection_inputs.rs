//! Bounded text protocols used by the explicit native collector.

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CollectError {
    Input,
    Environment,
    Artifacts,
    Selection,
    Execution,
    Changed,
    Conservation,
    Prepared,
    Intake,
    Nesting,
    Command,
}

impl fmt::Display for CollectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Input => "invalid collector input",
            Self::Environment => "invalid instrumentation environment",
            Self::Artifacts => "collector artifact mismatch",
            Self::Selection => "collector test selection mismatch",
            Self::Execution => "collector test execution mismatch",
            Self::Changed => "collector inputs changed",
            Self::Conservation => "collector process/profile mismatch",
            Self::Prepared => "invalid prepared collector tools",
            Self::Intake => "collector file intake failed",
            Self::Nesting => "invalid collector nesting",
            Self::Command => "collector command failed",
        })
    }
}
impl Error for CollectError {}

pub type Result<T> = std::result::Result<T, CollectError>;
pub const MAX_PROTOCOL_BYTES: usize = 8 * 1024 * 1024;

fn shell_value(raw: &str) -> Result<String> {
    let mut quoted = false;
    let mut bytes = Vec::new();
    let mut input = raw.bytes();
    while let Some(byte) = input.next() {
        match byte {
            b'\'' => quoted = !quoted,
            b'\0' | b'\n' | b'\r' => return Err(CollectError::Environment),
            _ if quoted => bytes.push(byte),
            b'\\' => {
                if input.next() != Some(b'\'') {
                    return Err(CollectError::Environment);
                }
                bytes.push(b'\'');
            }
            _ if byte.is_ascii_alphanumeric() || b"_./:%,=-".contains(&byte) => {
                bytes.push(byte);
            }
            _ => return Err(CollectError::Environment),
        }
    }
    if quoted {
        return Err(CollectError::Environment);
    }
    String::from_utf8(bytes).map_err(|_| CollectError::Environment)
}

/// Parse selected show-env output without evaluating shell text.
pub fn environment(
    raw: &str,
    target: &str,
    wrapper: &str,
    crate_names: &BTreeSet<String>,
    workspace_name: &str,
) -> Result<BTreeMap<String, String>> {
    if raw.len() > MAX_PROTOCOL_BYTES || !raw.ends_with('\n') {
        return Err(CollectError::Environment);
    }
    let mut values = BTreeMap::new();
    for line in raw.split_terminator('\n') {
        let (key, value) = line.split_once('=').ok_or(CollectError::Environment)?;
        if values.insert(key.to_owned(), shell_value(value)?).is_some() {
            return Err(CollectError::Environment);
        }
    }
    let required = [
        (
            "LLVM_PROFILE_FILE",
            format!("{target}/{workspace_name}-%p-%12m.profraw"),
        ),
        ("__CARGO_LLVM_COV_RUSTC_WRAPPER", "1".to_owned()),
        (
            "__CARGO_LLVM_COV_RUSTC_WRAPPER_RUSTFLAGS",
            "-C\u{1f}instrument-coverage\u{1f}--cfg=coverage".to_owned(),
        ),
        ("RUSTC_WRAPPER", wrapper.to_owned()),
        ("CARGO_LLVM_COV", "1".to_owned()),
        ("CARGO_LLVM_COV_SHOW_ENV", "1".to_owned()),
        ("CARGO_LLVM_COV_TARGET_DIR", target.to_owned()),
        ("CARGO_LLVM_COV_BUILD_DIR", target.to_owned()),
    ];
    if values.len() != 9
        || required
            .iter()
            .any(|(key, value)| values.get(*key) != Some(value))
    {
        return Err(CollectError::Environment);
    }
    let names = values
        .get("__CARGO_LLVM_COV_RUSTC_WRAPPER_CRATE_NAMES")
        .ok_or(CollectError::Environment)?;
    let mut actual = BTreeSet::new();
    for name in names.split(',') {
        if name.is_empty() || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
            return Err(CollectError::Environment);
        }
        actual.insert(name.to_owned());
    }
    if actual.is_empty() || &actual != crate_names {
        return Err(CollectError::Environment);
    }
    Ok(values)
}

fn continued(raw: &str) -> Result<String> {
    if raw.len() > MAX_PROTOCOL_BYTES || !raw.ends_with('\n') {
        return Err(CollectError::Artifacts);
    }
    let mut output = String::new();
    let mut input = raw.chars();
    while let Some(c) = input.next() {
        if c == '\\' {
            match input.next() {
                Some('\n') => output.push(' '),
                Some(next) => {
                    output.push('\\');
                    output.push(next);
                }
                None => return Err(CollectError::Artifacts),
            }
        } else {
            output.push(c);
        }
    }
    if !output.ends_with('\n') {
        return Err(CollectError::Artifacts);
    }
    Ok(output)
}

fn dependency_words(line: &str) -> Result<BTreeSet<String>> {
    let mut input = line.chars();
    let mut token = String::new();
    let mut dependencies = BTreeSet::new();
    while let Some(c) = input.next() {
        match c {
            '\\' => {
                let escaped = input.next().ok_or(CollectError::Artifacts)?;
                if !matches!(escaped, ' ' | ':' | '#' | '\\') {
                    return Err(CollectError::Artifacts);
                }
                token.push(escaped);
            }
            ' ' | '\t' => {
                if !token.is_empty() && !dependencies.insert(std::mem::take(&mut token)) {
                    return Err(CollectError::Artifacts);
                }
            }
            _ if c.is_control() => return Err(CollectError::Artifacts),
            _ => token.push(c),
        }
    }
    if !token.is_empty() && !dependencies.insert(token) {
        return Err(CollectError::Artifacts);
    }
    Ok(dependencies)
}

/// Select the complete dependency rule for the actual compiled object.
pub fn dependencies(raw: &str, object: &str) -> Result<BTreeSet<String>> {
    let text = continued(raw)?;
    // Rust writes the known object target raw, even when its path has spaces
    // or colons. Never guess its boundary by splitting those characters.
    let raw_target = format!("{object}:");
    let escaped = object
        .chars()
        .map(|c| {
            if matches!(c, ' ' | ':' | '#' | '\\') {
                format!("\\{c}")
            } else {
                c.to_string()
            }
        })
        .collect::<String>();
    let escaped_target = format!("{escaped}:");
    let mut selected = None;
    for line in text.split_terminator('\n') {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(right) = line
            .strip_prefix(&raw_target)
            .or_else(|| line.strip_prefix(&escaped_target))
            && selected.replace(dependency_words(right)?).is_some()
        {
            return Err(CollectError::Artifacts);
        }
    }
    let selected = selected.ok_or(CollectError::Artifacts)?;
    if selected.is_empty() {
        return Err(CollectError::Artifacts);
    }
    Ok(selected)
}
