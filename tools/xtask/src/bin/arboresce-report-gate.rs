use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    ExitCode::from(arboresce_xtask::report_command::run(
        &args,
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
    ))
}
