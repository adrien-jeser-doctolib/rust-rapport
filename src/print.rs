//! Formatters that turn a slice of [`Output`] into a printable `String`.

use crate::output::Output;

pub fn human(outputs: &[Output]) -> String {
    outputs.iter().filter_map(Output::rendered).collect()
}

pub fn github_summary(outputs: &[Output], any_success: bool) -> String {
    if outputs.is_empty() {
        return if any_success {
            "\u{1f980} Cargo is Happy !".to_owned()
        } else {
            "\u{1f612} Cargo is Sad !".to_owned()
        };
    }

    let mut table = String::from("| Type | Message |\n| ---- | ------- |\n");
    let body = outputs
        .iter()
        .map(|o| {
            format!(
                "| {} | {} |",
                o.level().unwrap_or("Unknown"),
                o.message()
                    .unwrap_or_else(|| "No message".to_owned())
                    .lines()
                    .take(1)
                    .collect::<String>(),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    table.push_str(&body);
    table
}

pub fn github_pr_annotation(outputs: &[Output]) -> String {
    outputs
        .iter()
        .map(|o| {
            let opts = [
                o.file_name().map(|f| format!("file={f}")),
                o.line_start().map(|l| format!("line={l}")),
                o.line_end().map(|l| format!("endLine={l}")),
                o.column_start().map(|c| format!("col={c}")),
                o.column_end().map(|c| format!("endColumn={c}")),
                o.message().map(|t| format!("title={t}")),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(",");

            let level = o.level().unwrap_or("notice");
            let body = o.rendered().unwrap_or("No message").replace('\n', "%0A");
            if opts.is_empty() {
                format!("::{level}::{body}")
            } else {
                format!("::{level} {opts}::{body}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    const WARNING_JSON: &str = r#"{
        "reason":"compiler-message",
        "manifest_path":"/x/Cargo.toml",
        "message":{
            "code":{"code":"unused"},
            "level":"warning",
            "message":"unused variable `x`",
            "spans":[{"file_name":"src/main.rs","line_start":1,"line_end":1,"column_start":5,"column_end":6}],
            "rendered":"warning: unused\nhelp: prefix with _"
        }
    }"#;

    fn warning() -> Output {
        serde_json::from_str(WARNING_JSON).expect("valid JSON")
    }

    #[test]
    fn summary_empty_success_prints_happy() {
        let s = github_summary(&[], true);
        assert!(s.contains("Cargo is Happy"));
    }

    #[test]
    fn summary_empty_failure_prints_sad() {
        let s = github_summary(&[], false);
        assert!(s.contains("Cargo is Sad"));
    }

    #[test]
    fn summary_contains_header_and_row_in_one_string() {
        let outs = [warning()];
        let s = github_summary(&outs, false);
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines[0], "| Type | Message |");
        assert_eq!(lines[1], "| ---- | ------- |");
        assert!(lines[2].starts_with("| warning | "));
        assert!(lines[2].contains("unused variable `x`"));
    }

    #[test]
    fn summary_uses_unknown_fallback_not_unknow() {
        let no_level: Output = serde_json::from_str(
            r#"{"reason":"compiler-message","manifest_path":null,"message":{"code":null,"level":null,"message":"m","spans":[{"file_name":"a","line_start":1,"line_end":1,"column_start":1,"column_end":2}],"rendered":"r"},"success":null}"#,
        )
        .expect("valid");
        let s = github_summary(&[no_level], false);
        assert!(s.contains("| Unknown |"), "got: {s}");
        assert!(!s.contains("Unknow "), "typo `Unknow` resurfaced: {s}");
    }

    #[test]
    fn annotation_encodes_newlines_and_includes_all_coords() {
        let outs = [warning()];
        let s = github_pr_annotation(&outs);
        assert!(s.starts_with("::warning "));
        assert!(s.contains("file=src/main.rs"));
        assert!(s.contains("line=1"));
        assert!(s.contains("endLine=1"));
        assert!(s.contains("col=5"));
        assert!(s.contains("endColumn=6"));
        assert!(s.contains("%0A"), "newline not encoded: {s}");
        assert!(!s.contains('\n'), "raw newline must not appear inside an annotation: {s:?}");
    }

    #[test]
    fn annotation_without_opts_emits_valid_form() {
        let minimal: Output = serde_json::from_str(
            r#"{"reason":"compiler-message","manifest_path":null,"message":{"code":null,"level":"warning","message":null,"spans":[{"file_name":"x","line_start":null,"line_end":null,"column_start":null,"column_end":null}],"rendered":"w"},"success":null}"#,
        )
        .expect("valid");
        let s = github_pr_annotation(&[minimal]);
        assert!(s.starts_with("::warning "), "got: {s}");
        assert!(!s.contains(":: ::"), "invalid double-colon form: {s}");
    }

    #[test]
    fn human_collects_rendered_text() {
        let outs = [warning()];
        let s = human(&outs);
        assert!(s.contains("warning: unused"));
    }

    // Regression: clippy emits renamed/deprecated-lint warnings with `spans: []`
    // (no file). The upstream filter in `lib.rs` already keeps only Error/Warning;
    // the formatters must not drop warnings that happen to have no span.
    const DEPRECATED_LINT_JSON: &str = r#"{
        "reason":"compiler-message",
        "manifest_path":"/x/Cargo.toml",
        "message":{
            "code":{"code":"renamed_and_removed_lints"},
            "level":"warning",
            "message":"the lint `clippy::cognitive_complexity` has been renamed to `clippy::cognitive-complexity`",
            "spans":[],
            "rendered":"warning: the lint `clippy::cognitive_complexity` has been renamed"
        }
    }"#;

    fn deprecated_lint() -> Output {
        serde_json::from_str(DEPRECATED_LINT_JSON).expect("valid JSON")
    }

    #[test]
    fn human_retains_warnings_without_spans() {
        let s = human(&[deprecated_lint()]);
        assert!(s.contains("renamed"), "deprecated-lint warning was dropped: {s:?}");
    }

    #[test]
    fn summary_retains_warnings_without_spans() {
        let s = github_summary(&[deprecated_lint()], false);
        assert!(s.contains("| warning | "), "warning row missing: {s}");
        assert!(s.contains("has been renamed"), "deprecated-lint message missing: {s}");
    }

    #[test]
    fn annotation_retains_warnings_without_spans() {
        let s = github_pr_annotation(&[deprecated_lint()]);
        assert!(!s.trim().is_empty(), "deprecated-lint annotation was dropped");
        assert!(s.starts_with("::warning"), "got: {s}");
        assert!(s.contains("has been renamed"), "rendered body missing: {s}");
    }
}
