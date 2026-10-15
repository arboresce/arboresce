use std::process::ExitCode;

use arboresce_xtask::{Action, USAGE, emit, execute};

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let code = match Action::parse(&args) {
        Some(action) => execute(action),
        None => {
            if emit(&mut std::io::stderr().lock(), USAGE.as_bytes()) {
                2
            } else {
                1
            }
        }
    };
    ExitCode::from(code)
}
