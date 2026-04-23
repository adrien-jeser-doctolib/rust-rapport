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
//! let mut stdout = io::stdout().lock();
//! let mut stderr = io::stderr().lock();
//! let report = run(Mode::Github, io::stdin().lock(), &mut stdout, &mut stderr)?;
//! std::process::exit(if report.is_failure() { 1 } else { 0 });
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
    /// Convenience mode for GitHub Actions: appends the Markdown summary to
    /// `$GITHUB_STEP_SUMMARY`, emits PR workflow commands on stdout, and lets
    /// the caller set the process exit status based on the [`RunReport`]
    /// returned by [`run`] (non-zero when clippy reported errors or the build
    /// failed). Equivalent to the legacy `tee >(github-summary) >(github-pr-annotation)`
    /// bash incantation, without the `PIPESTATUS` dance.
    Github,
    /// Markdown table for `$GITHUB_STEP_SUMMARY`.
    GithubSummary,
    /// GitHub workflow commands for inline PR annotations.
    GithubPrAnnotation,
    /// Plain rendered diagnostics.
    Human,
}

/// Summary of what [`run`] observed in the clippy stream.
///
/// Returned so the caller can choose an exit code that mirrors clippy's
/// (non-zero when the build failed or at least one error-level diagnostic
/// was reported).
#[derive(Debug, Default, Clone, Copy)]
pub struct RunReport {
    /// Number of unique error-level diagnostics.
    pub errors: usize,
    /// Number of unique warning-level diagnostics.
    pub warnings: usize,
    /// At least one `build-finished` message reported `success: true`.
    pub any_success: bool,
    /// At least one `build-finished` message reported `success: false`.
    pub any_failure: bool,
}

impl RunReport {
    /// `true` if clippy would have exited non-zero: either the build was
    /// marked as failed or at least one error-level diagnostic was emitted.
    #[must_use]
    pub const fn is_failure(&self) -> bool {
        self.errors > 0 || self.any_failure
    }
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
/// only I/O failures on `reader` or `writer` are fatal. In [`Mode::Github`] the
/// summary is appended to the file at `$GITHUB_STEP_SUMMARY` (if set) instead
/// of being written to `writer`; annotations still go to `writer`.
///
/// # Errors
/// Returns [`Error::Io`] if reading from `reader`, writing to `writer`, or
/// writing to `error_writer` / `$GITHUB_STEP_SUMMARY` fails.
pub fn run<R, W, E>(
    mode: Mode,
    reader: R,
    writer: &mut W,
    error_writer: &mut E,
) -> Result<RunReport, Error>
where
    R: BufRead,
    W: Write + ?Sized,
    E: Write + ?Sized,
{
    let (diagnostics, report) = parse(reader, error_writer)?;
    match mode {
        Mode::Github => {
            write_github_summary(
                &print::github_summary(&diagnostics, report.any_success),
                error_writer,
            )?;
            let annot = print::github_pr_annotation(&diagnostics);
            if !annot.is_empty() {
                writeln!(writer, "{annot}")?;
            }
        }
        Mode::GithubSummary => {
            writeln!(writer, "{}", print::github_summary(&diagnostics, report.any_success))?;
        }
        Mode::GithubPrAnnotation => {
            writeln!(writer, "{}", print::github_pr_annotation(&diagnostics))?;
        }
        Mode::Human => writeln!(writer, "{}", print::human(&diagnostics))?,
    }
    Ok(report)
}

/// Append `summary` to the file pointed to by `$GITHUB_STEP_SUMMARY`. Falls
/// back to `error_writer` when the environment variable is unset — useful for
/// local previews of the GitHub rendering.
fn write_github_summary<E: Write + ?Sized>(
    summary: &str,
    error_writer: &mut E,
) -> Result<(), Error> {
    if let Some(path) = std::env::var_os("GITHUB_STEP_SUMMARY") {
        writeln!(std::fs::OpenOptions::new().append(true).create(true).open(path)?, "{summary}")?;
    } else {
        writeln!(error_writer, "{summary}")?;
    }
    Ok(())
}

fn parse<R, E>(reader: R, error_writer: &mut E) -> Result<(Vec<Output>, RunReport), Error>
where
    R: BufRead,
    E: Write + ?Sized,
{
    let lines = reader.lines().collect::<Result<Vec<_>, _>>()?;
    let (diagnostics, report) = lines.into_iter().enumerate().try_fold(
        (BTreeSet::<Output>::new(), RunReport::default()),
        |(mut set, mut report), (idx, line)| -> Result<_, Error> {
            match serde_json::from_str::<Output>(&line) {
                Ok(output) => {
                    match output.build_success() {
                        Some(true) => report.any_success = true,
                        Some(false) => report.any_failure = true,
                        None => {}
                    }
                    if output.is_level(&Level::Error) || output.is_level(&Level::Warning) {
                        set.insert(output);
                    }
                }
                Err(e) => {
                    writeln!(error_writer, "rust-rapport: line {}: invalid JSON: {e}", idx + 1)?;
                }
            }
            Ok((set, report))
        },
    )?;
    // Count distinct findings by level from the deduplicated set.
    let errors = diagnostics.iter().filter(|o| o.is_level(&Level::Error)).count();
    let warnings = diagnostics.iter().filter(|o| o.is_level(&Level::Warning)).count();
    Ok((diagnostics.into_iter().collect(), RunReport { errors, warnings, ..report }))
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

    /// Returns the `RunReport` built from `input` via a non-Github mode
    /// (avoids touching `$GITHUB_STEP_SUMMARY` in unit tests — that behaviour
    /// is covered by the integration tests in `tests/cli.rs`).
    fn run_report(input: &str) -> RunReport {
        let mut out = Vec::new();
        let mut err = Vec::new();
        run(Mode::GithubPrAnnotation, Cursor::new(input), &mut out, &mut err).expect("run")
    }

    #[test]
    fn report_counts_errors_and_warnings_distinctly() {
        let input = format!("{WARN_LINE}\n{ERROR_LINE}\n");
        let r = run_report(&input);
        assert_eq!(r.warnings, 1);
        assert_eq!(r.errors, 1);
        assert!(r.is_failure(), "any error must flip is_failure");
    }

    #[test]
    fn report_is_not_failure_when_only_warnings() {
        let r = run_report(&format!("{WARN_LINE}\n{BUILD_OK}\n"));
        assert!(!r.is_failure());
    }

    #[test]
    fn report_picks_up_build_success_flag() {
        let r = run_report(&format!("{BUILD_OK}\n"));
        assert!(r.any_success);
        assert!(!r.any_failure);
    }

    #[test]
    fn report_flags_failure_on_build_finished_false_without_diagnostics() {
        let r = run_report(&format!("{BUILD_KO}\n"));
        assert!(r.any_failure);
        assert!(r.is_failure());
    }
}
