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
        .stdout(str::contains("| Level | Location | Rule | Message |"))
        .stdout(str::contains("| --- | --- | --- | --- |"))
        .stdout(str::contains("⚠️ warning"))
        .stdout(str::contains("❌ error"));
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

#[test]
fn github_mode_writes_summary_to_env_file_and_annotations_to_stdout() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("rust-rapport-it-{}.md", std::process::id()));
    let _ = std::fs::remove_file(&path);

    // clippy_warnings.json contains an error → mode `github` exits 1.
    cmd("github")
        .env("GITHUB_STEP_SUMMARY", &path)
        .write_stdin(fixture("clippy_warnings.json"))
        .assert()
        .code(1)
        .stdout(str::contains("::warning "))
        .stdout(str::contains("::error "));

    let summary = std::fs::read_to_string(&path).expect("summary file");
    assert!(
        summary.contains("| Level | Location | Rule | Message |"),
        "summary missing: {summary}"
    );
    assert!(summary.contains("⚠️ warning"));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn github_mode_without_env_var_falls_back_to_stderr_for_summary() {
    cmd("github")
        .env_remove("GITHUB_STEP_SUMMARY")
        .write_stdin(fixture("clippy_warnings.json"))
        .assert()
        .code(1)
        .stderr(str::contains("| Level | Location | Rule | Message |"));
}

#[test]
fn github_mode_exits_nonzero_on_build_failure() {
    // clippy_failed.json reports a build failure → mode `github` must exit 1.
    cmd("github")
        .env_remove("GITHUB_STEP_SUMMARY")
        .write_stdin(fixture("clippy_failed.json"))
        .assert()
        .code(1);
}

#[test]
fn github_mode_exits_zero_on_clean_build() {
    cmd("github")
        .env_remove("GITHUB_STEP_SUMMARY")
        .write_stdin(fixture("clippy_clean.json"))
        .assert()
        .success();
}

#[test]
fn non_github_modes_still_exit_zero_even_on_build_failure() {
    // `github-summary` etc. are pure formatters — they never flip the exit code.
    cmd("github-summary").write_stdin(fixture("clippy_failed.json")).assert().success();
}
