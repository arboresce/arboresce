//! Explicit local commands and architecture checks, without product authority.

pub mod architecture;
pub mod coverage;
pub mod inventory;
pub mod inventory_command;
pub mod measurement;
pub mod process;
pub mod report_command;
mod strict_json;
pub mod workspace;

use std::{ffi::OsString, io::Write};

pub const USAGE: &str = "usage: cargo run --locked --offline -p arboresce-xtask -- <architecture|fmt|check|test|clippy>\n";
pub const COMMAND_FAILURE: &str = "verification command failed\n";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Architecture,
    Format,
    Check,
    Test,
    Clippy,
    Help,
}

impl Action {
    pub fn parse(args: &[OsString]) -> Option<Self> {
        let [argument] = args else { return None };
        Some(match argument.to_str()? {
            "architecture" => Self::Architecture,
            "fmt" => Self::Format,
            "check" => Self::Check,
            "test" => Self::Test,
            "clippy" => Self::Clippy,
            "help" | "--help" | "-h" => Self::Help,
            _ => return None,
        })
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Architecture => "architecture",
            Self::Format => "fmt",
            Self::Check => "check",
            Self::Test => "test",
            Self::Clippy => "clippy",
            Self::Help => "help",
        }
    }

    pub const fn operation_budget(self) -> std::time::Duration {
        std::time::Duration::from_secs(match self {
            Self::Format => 60,
            Self::Test => 300,
            _ => 180,
        })
    }

    pub const fn cargo_args(self) -> &'static [&'static str] {
        match self {
            Self::Architecture => &["metadata", "--locked", "--offline", "--format-version", "1"],
            Self::Format => &["fmt", "--all", "--", "--check"],
            Self::Check => &[
                "check",
                "--locked",
                "--offline",
                "--workspace",
                "--all-targets",
            ],
            Self::Test => &[
                "test",
                "--locked",
                "--offline",
                "--workspace",
                "--",
                "--nocapture",
            ],
            Self::Clippy => &[
                "clippy",
                "--locked",
                "--offline",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
            Self::Help => &[],
        }
    }
}

/// Complete owned frames require both a successful write and flush.
pub fn emit(writer: &mut impl Write, bytes: &[u8]) -> bool {
    writer
        .write_all(bytes)
        .and_then(|()| writer.flush())
        .is_ok()
}

fn failed() -> u8 {
    emit(&mut std::io::stderr().lock(), COMMAND_FAILURE.as_bytes());
    1
}

/// Run only an explicit action; Cargo forwards its own streams and exit.
pub fn execute(action: Action) -> u8 {
    if action == Action::Help {
        return if emit(&mut std::io::stdout().lock(), USAGE.as_bytes()) {
            0
        } else {
            failed()
        };
    }
    if action == Action::Architecture {
        return match workspace::check_current() {
            Ok(()) => {
                if emit(
                    &mut std::io::stdout().lock(),
                    b"architecture checks passed\n",
                ) {
                    0
                } else {
                    failed()
                }
            }
            Err(workspace::RunError::Child(code)) => code,
            Err(workspace::RunError::Command) => failed(),
            Err(workspace::RunError::Guard(error)) => {
                emit(
                    &mut std::io::stderr().lock(),
                    format!("arboresce-xtask: {error}\n").as_bytes(),
                );
                1
            }
        };
    }
    match measurement::command("cargo", None)
        .and_then(|mut command| measurement::spawn(command.args(action.cargo_args())))
        .and_then(|mut child| process::wait(&mut child, action.operation_budget()))
    {
        Ok(status) => match status.code().and_then(|code| u8::try_from(code).ok()) {
            Some(code) => code,
            None => failed(),
        },
        Err(_) => failed(),
    }
}
