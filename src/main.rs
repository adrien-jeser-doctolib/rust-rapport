//! Binary entrypoint: parses CLI args and delegates to [`rust_rapport::run`].

#![allow(clippy::print_stderr)]

use clap::Parser;
use rust_rapport::{Mode, run};
use std::io;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(value_enum)]
    mode: Mode,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    match run(cli.mode, stdin.lock(), &mut stdout, &mut stderr) {
        // In `Mode::Github` we mirror clippy's verdict in the exit code.
        // The other modes are pure formatters — they never fail the pipeline.
        Ok(report) if cli.mode == Mode::Github && report.is_failure() => ExitCode::from(1),
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("rust-rapport: {e}");
            ExitCode::from(2)
        }
    }
}
