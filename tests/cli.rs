//! Integration tests: drive the `rust-rapport` binary with fixture files via stdin.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::needless_collect)]

use assert_cmd::Command;
use predicates::str;
use std::fs;
use std::path::PathBuf;

fn fixture(name: &str) -> String {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures");
    p.push(name);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

fn cmd(mode: &str) -> Command {
    let mut c = Command::cargo_bin("rust-rapport").expect("binary built");
    c.arg(mode);
    c
}

#[test]
fn clean_build_produces_happy_summary() {
    cmd("github-summary")
        .write_stdin(fixture("clippy_clean.json"))
        .assert()
        .success()
        .stdout(str::contains("Cargo is Happy"));
}

#[test]
fn failed_build_without_diagnostics_produces_sad_summary() {
    cmd("github-summary")
        .write_stdin(fixture("clippy_failed.json"))
        .assert()
        .success()
        .stdout(str::contains("Cargo is Sad"));
}

#[test]
fn warnings_and_errors_are_deduplicated_in_annotations() {
    let output = cmd("github-pr-annotation")
        .write_stdin(fixture("clippy_warnings.json"))
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 2, "expected 2 unique annotations, got: {stdout}");
    assert!(lines.iter().any(|l| l.starts_with("::warning ")));
    assert!(lines.iter().any(|l| l.starts_with("::error ")));
}

#[test]
fn warnings_summary_contains_table_header_and_rows() {
    cmd("github-summary")
        .write_stdin(fixture("clippy_warnings.json"))
        .assert()
        .success()
        .stdout(str::contains("| Type | Message |"))
        .stdout(str::contains("| ---- | ------- |"))
        .stdout(str::contains("warning"))
        .stdout(str::contains("error"));
}

#[test]
fn malformed_line_is_logged_on_stderr_and_valid_lines_still_render() {
    let output = cmd("github-pr-annotation")
        .write_stdin(fixture("clippy_malformed.json"))
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    let stderr = String::from_utf8(output.stderr).expect("utf8");

    assert!(stderr.contains("line 2"), "stderr missing line number: {stderr}");
    assert!(stderr.contains("invalid JSON"), "stderr missing diagnostic: {stderr}");
    let valid: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(valid.len(), 2, "valid lines should still render: {stdout}");
}

#[test]
fn human_mode_emits_rendered_text() {
    cmd("human")
        .write_stdin(fixture("clippy_warnings.json"))
        .assert()
        .success()
        .stdout(str::contains("warning: unused variable"))
        .stdout(str::contains("error[E0425]"));
}
