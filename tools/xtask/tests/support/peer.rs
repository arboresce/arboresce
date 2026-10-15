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
        "measured-status"
        | "measured-selected-status"
        | "measured-output"
        | "measured-missing-output"
        | "measured-broken-trace" => {
            let program = std::env::var_os("ARBORESCE_NATIVE_PROGRAM").unwrap_or_default();
            let mut command = if mode == "measured-selected-status" {
                match arboresce_xtask::measurement::command(program, None) {
                    Ok(command) => command,
                    Err(_) => {
                        emit(
                            &mut std::io::stderr().lock(),
                            b"native measurement failed\n",
                        );
                        return ExitCode::from(1);
                    }
                }
            } else {
                std::process::Command::new(program)
            };
            let result = if mode == "measured-output" {
                command.arg("measurement child");
                arboresce_xtask::process::output(&mut command).map(|output| {
                    let complete = emit(&mut std::io::stdout().lock(), &output.stdout)
                        && emit(&mut std::io::stderr().lock(), &output.stderr);
                    if complete {
                        output.status.code().unwrap_or(1)
                    } else {
                        1
                    }
                })
            } else if matches!(
                mode.as_str(),
                "measured-missing-output" | "measured-broken-trace"
            ) {
                (|| -> std::io::Result<i32> {
                    let root = std::path::PathBuf::from(
                        std::env::var_os(arboresce_xtask::measurement::ENV).unwrap_or_default(),
                    );
                    let stdout = root.join("observer-stdout");
                    let stderr = root.join("observer-stderr");
                    command
                        .stdout(std::process::Stdio::from(std::fs::File::create_new(
                            &stdout,
                        )?))
                        .stderr(std::process::Stdio::from(std::fs::File::create_new(
                            &stderr,
                        )?));
                    let started = std::time::Instant::now();
                    let mut child = arboresce_xtask::measurement::spawn(&mut command)?;
                    if mode == "measured-missing-output" {
                        std::fs::remove_file(&stdout)?;
                    }
                    let (reader, mut writer) = std::io::pipe()?;
                    drop(reader);
                    let result = if mode == "measured-broken-trace" {
                        arboresce_xtask::process::observe(
                            &mut child,
                            &stdout,
                            &stderr,
                            arboresce_xtask::process::Observation::default(),
                            started,
                            &mut writer,
                        )
                    } else {
                        arboresce_xtask::process::observe(
                            &mut child,
                            &stdout,
                            &stderr,
                            arboresce_xtask::process::Observation::default(),
                            started,
                            &mut Vec::new(),
                        )
                    };
                    result.map(|output| output.status.code().unwrap_or(1))
                })()
            } else {
                arboresce_xtask::measurement::spawn(&mut command).and_then(|mut child| {
                    arboresce_xtask::process::wait(&mut child, std::time::Duration::from_secs(30))
                        .map(|status| status.code().unwrap_or(1))
                })
            };
            return ExitCode::from(match result {
                Ok(code) => u8::try_from(code).unwrap_or(1),
                Err(_) => {
                    emit(
                        &mut std::io::stderr().lock(),
                        b"native measurement failed\n",
                    );
                    1
                }
            });
        }
        "metadata-invalid" => ("not-json\n", "", 0),
        "metadata-warning" => ("not-json\n", "child stderr\n", 0),
        _ => ("", "test peer mode missing\n", 1),
    };
    let complete = emit(&mut std::io::stdout().lock(), stdout.as_bytes())
        && emit(&mut std::io::stderr().lock(), stderr.as_bytes());
    ExitCode::from(if complete { exit } else { 1 })
}
