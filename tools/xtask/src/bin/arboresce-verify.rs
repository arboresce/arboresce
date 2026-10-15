use std::process::ExitCode;

fn main() -> ExitCode {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    ExitCode::from(arboresce_xtask::collection::run(
        &args,
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
    ))
}
