use std::{
    ffi::OsString,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

use crate::{
    coverage::{self, GateError, MAX_INPUT_BYTES},
    emit,
};

pub const USAGE: &str =
    "usage: arboresce-report-gate <source-root> <inventory.json> <llvm-export.json>\n";
pub const OUTPUT_FAILURE: &str = "coverage report output failed\n";

pub(crate) fn read_input(path: &Path) -> Result<String, GateError> {
    let metadata = fs::metadata(path).map_err(|_| GateError::InvalidInput)?;
    if !metadata.is_file() || metadata.len() > MAX_INPUT_BYTES as u64 {
        return Err(GateError::InvalidInput);
    }
    let file = File::open(path).map_err(|_| GateError::InvalidInput)?;
    let mut text = String::new();
    file.take(MAX_INPUT_BYTES as u64 + 1)
        .read_to_string(&mut text)
        .map_err(|_| GateError::InvalidInput)?;
    if text.len() > MAX_INPUT_BYTES {
        return Err(GateError::InvalidInput);
    }
    Ok(text)
}

fn failed_output(stderr: &mut impl Write) -> u8 {
    emit(stderr, OUTPUT_FAILURE.as_bytes());
    1
}

pub fn run(args: &[OsString], stdout: &mut impl Write, stderr: &mut impl Write) -> u8 {
    if let [flag] = args
        && matches!(flag.to_str(), Some("help" | "--help" | "-h"))
    {
        return if emit(stdout, USAGE.as_bytes()) {
            0
        } else {
            failed_output(stderr)
        };
    }
    let [root, inventory, report] = args else {
        return if emit(stderr, USAGE.as_bytes()) { 2 } else { 1 };
    };
    let result = (|| {
        let root = root.to_str().ok_or(GateError::InvalidInput)?;
        let inventory = read_input(Path::new(inventory))?;
        let report = read_input(Path::new(report))?;
        coverage::evaluate(root, &inventory, &report)
    })();
    match result {
        Ok(_) => {
            if emit(stdout, b"coverage report counts passed\n") {
                0
            } else {
                failed_output(stderr)
            }
        }
        Err(error) => {
            emit(
                stderr,
                format!("arboresce-report-gate: {error}\n").as_bytes(),
            );
            1
        }
    }
}
