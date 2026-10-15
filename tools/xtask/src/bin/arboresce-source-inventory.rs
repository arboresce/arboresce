use std::process::ExitCode;

fn main() -> ExitCode {
    ExitCode::from(arboresce_xtask::inventory_command::run(
        &std::env::args_os().skip(1).collect::<Vec<_>>(),
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
    ))
}
