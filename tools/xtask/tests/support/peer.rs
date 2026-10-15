//! Controlled test-only Cargo peer. It cannot qualify actual Cargo execution.

use std::process::ExitCode;

use arboresce_xtask::emit;

fn main() -> ExitCode {
    let mode = std::env::var("ARBORESCE_XTASK_TEST_PEER").unwrap_or_default();
    let (stdout, stderr, exit) = match mode.as_str() {
        "pass" => ("child stdout\n", "child stderr\n", 0),
        "fail" => ("child stdout\n", "child stderr\n", 101),
        "metadata" => {
            let path = std::env::var_os("ARBORESCE_XTASK_TEST_METADATA").unwrap_or_default();
            let bytes = std::fs::read(path).unwrap_or_default();
            let complete = emit(&mut std::io::stdout().lock(), &bytes);
            return ExitCode::from(if complete { 0 } else { 1 });
        }
        "metadata-invalid" => ("not-json\n", "", 0),
        "metadata-warning" => ("not-json\n", "child stderr\n", 0),
        _ => ("", "test peer mode missing\n", 1),
    };
    let complete = emit(&mut std::io::stdout().lock(), stdout.as_bytes())
        && emit(&mut std::io::stderr().lock(), stderr.as_bytes());
    ExitCode::from(if complete { exit } else { 1 })
}
