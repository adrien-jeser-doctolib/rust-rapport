#![allow(clippy::implicit_return)]
#![allow(clippy::print_stdout)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::std_instead_of_alloc)]

use crate::print::{github_pr_annotation, github_summary, human};
use clap::{Parser, ValueEnum};
use level::Level;
use output::Output;
use std::collections::HashSet;
use std::io;
use std::io::BufRead;

mod level;
mod output;
mod print;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[non_exhaustive]
pub enum Mode {
    GithubSummary,
    GithubPrAnnotation,
    Human,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(value_enum)]
    mode: Mode,
}

fn main() {
    let cli = Cli::parse();
    let stdin = io::stdin();
    let outputs: Vec<_> = stdin
        .lock()
        .lines()
        .flatten()
        .filter_map(|line| {
            serde_json::from_str(&line).ok().and_then(|output: Output| {
                (output.is_level(&Level::Error) || output.is_level(&Level::Warning))
                    .then_some(output)
            })
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    match cli.mode {
        Mode::GithubSummary => {
            println!("{}", github_summary(&outputs));
        }
        Mode::GithubPrAnnotation => {
            println!("{}", github_pr_annotation(&outputs));
        }
        Mode::Human => {
            println!("{}", human(&outputs));
        }
    }
}
