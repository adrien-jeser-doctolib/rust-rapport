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
    let stdout = io::stdout();
    let stderr = io::stderr();
    match run(cli.mode, stdin.lock(), stdout.lock(), stderr.lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("rust-rapport: {e}");
            ExitCode::FAILURE
        }
    }
}
