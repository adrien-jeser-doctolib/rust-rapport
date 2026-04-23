//! Formats `cargo clippy --message-format json` output for GitHub Actions.
//!
//! Read diagnostics from a [`BufRead`] stream, filter errors and warnings,
//! and write the result to a [`Write`] sink using one of the three [`Mode`]
//! variants.
//!
//! ```no_run
//! use std::io;
//! use rust_rapport::{Mode, run};
//!
//! # fn main() -> Result<(), rust_rapport::Error> {
//! run(Mode::GithubSummary, io::stdin().lock(), io::stdout().lock(), io::stderr().lock())?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::needless_collect))]

use clap::ValueEnum;
use std::collections::BTreeSet;
use std::io::{BufRead, Write};

mod level;
mod output;
mod print;

use level::Level;
use output::Output;

/// Output mode selected at the CLI.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, ValueEnum)]
#[non_exhaustive]
pub enum Mode {
    /// Markdown table for `$GITHUB_STEP_SUMMARY`.
    GithubSummary,
    /// GitHub workflow commands for inline PR annotations.
    GithubPrAnnotation,
    /// Plain rendered diagnostics.
    Human,
}

/// Errors returned by [`run`].
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// An I/O error while reading input or writing output.
    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// Reads clippy JSON from `reader`, formats according to `mode`, writes to `writer`.
///
/// Per-line JSON parse errors are logged to `error_writer` and the run continues;
/// only I/O failures on `reader` or `writer` are fatal.
///
/// # Errors
/// Returns [`Error::Io`] if reading from `reader`, writing to `writer`, or
/// writing to `error_writer` fails.
pub fn run<R, W, E>(mode: Mode, reader: R, mut writer: W, mut error_writer: E) -> Result<(), Error>
where
    R: BufRead,
    W: Write,
    E: Write,
{
    let (diagnostics, any_success) = parse(reader, &mut error_writer)?;
    let text = match mode {
        Mode::GithubSummary => print::github_summary(&diagnostics, any_success),
        Mode::GithubPrAnnotation => print::github_pr_annotation(&diagnostics),
        Mode::Human => print::human(&diagnostics),
    };
    writeln!(writer, "{text}")?;
    Ok(())
}

fn parse<R, E>(reader: R, error_writer: &mut E) -> Result<(Vec<Output>, bool), Error>
where
    R: BufRead,
    E: Write,
{
    let mut any_success = false;
    let mut set: BTreeSet<Output> = BTreeSet::new();
    for (idx, line) in reader.lines().enumerate() {
        let line = line?;
        match serde_json::from_str::<Output>(&line) {
            Ok(output) => {
                if output.success() {
                    any_success = true;
                }
                if output.is_level(&Level::Error) || output.is_level(&Level::Warning) {
                    set.insert(output);
                }
            }
            Err(e) => {
                writeln!(error_writer, "rust-rapport: line {}: invalid JSON: {e}", idx + 1)?;
            }
        }
    }
    Ok((set.into_iter().collect(), any_success))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const WARN_LINE: &str = r#"{"reason":"compiler-message","manifest_path":"/x/Cargo.toml","message":{"code":{"code":"unused"},"level":"warning","message":"unused variable: x","spans":[{"file_name":"src/main.rs","line_start":1,"line_end":1,"column_start":5,"column_end":6}],"rendered":"warning: unused"}}"#;
    const ERROR_LINE: &str = r#"{"reason":"compiler-message","manifest_path":"/x/Cargo.toml","message":{"code":{"code":"E0001"},"level":"error","message":"boom","spans":[{"file_name":"src/lib.rs","line_start":2,"line_end":2,"column_start":1,"column_end":2}],"rendered":"error: boom"}}"#;
    const NOTE_LINE: &str = r#"{"reason":"compiler-message","manifest_path":"/x/Cargo.toml","message":{"code":null,"level":"note","message":"a note","spans":[],"rendered":"note: a note"}}"#;
    const BUILD_OK: &str = r#"{"reason":"build-finished","success":true}"#;
    const BUILD_KO: &str = r#"{"reason":"build-finished","success":false}"#;

    fn run_capture(mode: Mode, input: &str) -> (String, String) {
        let mut out = Vec::new();
        let mut err = Vec::new();
        run(mode, Cursor::new(input), &mut out, &mut err).expect("run should succeed");
        (String::from_utf8(out).expect("utf8 stdout"), String::from_utf8(err).expect("utf8 stderr"))
    }

    #[test]
    fn summary_happy_when_build_success_and_no_diagnostics() {
        let input = format!("{BUILD_OK}\n");
        let (out, err) = run_capture(Mode::GithubSummary, &input);
        assert!(out.contains("Cargo is Happy"), "got: {out}");
        assert!(err.is_empty());
    }

    #[test]
    fn summary_sad_when_no_build_success_and_no_diagnostics() {
        let input = format!("{BUILD_KO}\n");
        let (out, _) = run_capture(Mode::GithubSummary, &input);
        assert!(out.contains("Cargo is Sad"), "got: {out}");
    }

    #[test]
    fn summary_table_has_header_in_returned_text() {
        let input = format!("{WARN_LINE}\n{BUILD_KO}\n");
        let (out, _) = run_capture(Mode::GithubSummary, &input);
        assert!(out.contains("| Level | Location | Rule | Message |"), "missing header: {out}");
        assert!(out.contains("| --- | --- | --- | --- |"), "missing separator: {out}");
        assert!(out.contains("⚠️ warning"), "missing row: {out}");
    }

    #[test]
    fn diagnostics_are_deduplicated_and_deterministic() {
        let input = format!("{WARN_LINE}\n{WARN_LINE}\n{ERROR_LINE}\n");
        let (out, _) = run_capture(Mode::GithubPrAnnotation, &input);
        let lines: Vec<&str> = out.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 2, "expected 2 unique annotations, got: {out}");
        let (out2, _) = run_capture(Mode::GithubPrAnnotation, &input);
        assert_eq!(out, out2, "output must be deterministic");
    }

    #[test]
    fn note_level_is_filtered_out() {
        let input = format!("{NOTE_LINE}\n{BUILD_KO}\n");
        let (out, _) = run_capture(Mode::GithubPrAnnotation, &input);
        assert!(out.trim().is_empty(), "notes should be filtered: {out}");
    }

    #[test]
    fn malformed_json_is_reported_on_stderr_and_skipped() {
        let input = format!("{WARN_LINE}\nnot json\n{ERROR_LINE}\n{BUILD_KO}\n");
        let (out, err) = run_capture(Mode::GithubPrAnnotation, &input);
        assert!(err.contains("line 2"), "expected line number in stderr: {err}");
        assert!(err.contains("invalid JSON"), "expected diagnostic: {err}");
        let non_empty: Vec<&str> = out.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(non_empty.len(), 2, "valid lines should still render: {out}");
    }

    #[test]
    fn human_mode_emits_rendered_text() {
        let input = format!("{WARN_LINE}\n{ERROR_LINE}\n");
        let (out, _) = run_capture(Mode::Human, &input);
        assert!(out.contains("warning: unused"));
        assert!(out.contains("error: boom"));
    }

    #[test]
    fn error_display_includes_source() {
        let io_err = std::io::Error::other("broken pipe");
        let e = Error::from(io_err);
        assert!(e.to_string().contains("I/O error"));
        assert!(e.to_string().contains("broken pipe"));
    }
}
