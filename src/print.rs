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

    let mut table =
        String::from("| Level | Location | Rule | Message |\n| --- | --- | --- | --- |\n");
    let body = outputs
        .iter()
        .map(|o| {
            format!(
                "| {} | {} | {} | {} |",
                level_badge(o.level()),
                location(o),
                code_cell(o.code()),
                short_message(o),
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
            // Short, code-like identifier displayed as the annotation title.
            // Falls back to the first line of the diagnostic message when the
            // compiler did not emit a lint code (deprecated-lint warnings, etc).
            let title = o
                .code()
                .map(str::to_owned)
                .or_else(|| o.message().map(first_line))
                .unwrap_or_else(|| "diagnostic".to_owned());

            let opts = [
                o.file_name().map(|f| format!("file={}", escape_property(f))),
                o.line_start().map(|l| format!("line={l}")),
                o.line_end().map(|l| format!("endLine={l}")),
                o.column_start().map(|c| format!("col={c}")),
                o.column_end().map(|c| format!("endColumn={c}")),
                Some(format!("title={}", escape_property(&title))),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(",");

            let level = o.level().unwrap_or("notice");
            // Body is the full rustc rendering — it carries the `help:` hints,
            // the `-/+` suggested replacements, and the note about which lint
            // is implied. We only strip the redundant `  --> file:line:col`
            // pointer since GitHub already knows the location from the
            // `file=/line=/col=` properties above.
            let body_text = o.rendered().map_or_else(
                || o.message().unwrap_or_else(|| "No message".to_owned()),
                strip_location_pointer,
            );
            let body = escape_data(&body_text);

            if opts.is_empty() {
                format!("::{level}::{body}")
            } else {
                format!("::{level} {opts}::{body}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Escape a GitHub Actions workflow command *property value* (key=value).
/// `%`, `\r`, `\n`, `:`, and `,` all need to be percent-encoded.
fn escape_property(s: &str) -> String {
    s.replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
        .replace(':', "%3A")
        .replace(',', "%2C")
}

/// Escape a GitHub Actions workflow command *data* payload (after `::`).
/// Only `%`, `\r`, `\n` are required to be escaped here.
fn escape_data(s: &str) -> String {
    s.replace('%', "%25").replace('\r', "%0D").replace('\n', "%0A")
}

fn first_line(s: impl AsRef<str>) -> String {
    s.as_ref().lines().next().unwrap_or("").to_owned()
}

/// Drop the `  --> file:line:col` pointer line from a rustc-rendered
/// diagnostic — it duplicates the `file=/line=/col=` annotation properties.
/// Everything else (source excerpts, `help:` hints, `-/+` suggestions, notes)
/// is preserved.
fn strip_location_pointer(rendered: &str) -> String {
    rendered.lines().filter(|l| !l.trim_start().starts_with("--> ")).collect::<Vec<_>>().join("\n")
}

fn level_badge(level: Option<&str>) -> &'static str {
    match level {
        Some("error") => "❌ error",
        Some("warning") => "⚠️ warning",
        Some("note") => "ℹ️ note",
        Some("help") => "💡 help",
        _ => "❔ unknown",
    }
}

fn location(o: &Output) -> String {
    match (o.file_name(), o.line_start()) {
        (Some(f), Some(l)) => format!("`{f}:{l}`"),
        (Some(f), None) => format!("`{f}`"),
        _ => "—".to_owned(),
    }
}

fn code_cell(code: Option<&str>) -> String {
    code.map_or_else(|| "—".to_owned(), |c| format!("`{c}`"))
}

fn short_message(o: &Output) -> String {
    o.message()
        .map(first_line)
        .or_else(|| o.rendered().map(first_line))
        .unwrap_or_else(|| "No message".to_owned())
        // Keep pipes from breaking the Markdown table.
        .replace('|', "\\|")
}

#[cfg(test)]
mod tests {
    use super::*;

    const WARNING_JSON: &str = r#"{
        "reason":"compiler-message",
        "manifest_path":"/x/Cargo.toml",
        "message":{
            "code":{"code":"clippy::unused_return"},
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
        assert!(github_summary(&[], true).contains("Cargo is Happy"));
    }

    #[test]
    fn summary_empty_failure_prints_sad() {
        assert!(github_summary(&[], false).contains("Cargo is Sad"));
    }

    #[test]
    fn summary_renders_four_columns() {
        let s = github_summary(&[warning()], false);
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines[0], "| Level | Location | Rule | Message |");
        assert_eq!(lines[1], "| --- | --- | --- | --- |");
        assert!(lines[2].starts_with("| ⚠️ warning | "));
        assert!(lines[2].contains("`src/main.rs:1`"), "location missing: {s}");
        assert!(lines[2].contains("`clippy::unused_return`"), "rule code missing: {s}");
        assert!(lines[2].contains("unused variable `x`"), "message missing: {s}");
    }

    #[test]
    fn summary_uses_unknown_badge_when_level_absent() {
        let no_level: Output = serde_json::from_str(
            r#"{"reason":"compiler-message","manifest_path":null,"message":{"code":null,"level":null,"message":"m","spans":[{"file_name":"a","line_start":1,"line_end":1,"column_start":1,"column_end":2}],"rendered":"r"},"success":null}"#,
        )
        .expect("valid");
        let s = github_summary(&[no_level], false);
        assert!(s.contains("❔ unknown"), "got: {s}");
    }

    #[test]
    fn annotation_uses_code_as_title_and_preserves_full_body() {
        let s = github_pr_annotation(&[warning()]);
        // Lint code `::` must be percent-encoded to survive the workflow command
        // parser (commas and colons are structural separators).
        assert!(s.contains("title=clippy%3A%3Aunused_return"), "code not in title: {s}");
        // Body keeps the full `rendered` payload so users still see the
        // `help:` suggestion / `-/+` diff hints clippy emits.
        assert!(s.contains("help: prefix with _"), "help hint missing from body: {s}");
    }

    #[test]
    fn annotation_strips_redundant_location_pointer() {
        let with_pointer: Output = serde_json::from_str(
            r#"{"reason":"compiler-message","manifest_path":null,"message":{"code":{"code":"c"},"level":"warning","message":"m","spans":[{"file_name":"x.rs","line_start":1,"line_end":1,"column_start":1,"column_end":2}],"rendered":"warning: m\n  --> x.rs:1:1\nhelp: do the thing"},"success":null}"#,
        )
        .expect("valid");
        let s = github_pr_annotation(&[with_pointer]);
        // `-->` pointer duplicates file=/line=/col= properties, should be gone.
        assert!(!s.contains("--> x.rs"), "location pointer leaked: {s}");
        // Everything else survives.
        assert!(s.contains("help: do the thing"), "help hint stripped: {s}");
    }

    #[test]
    fn annotation_escapes_special_chars_in_title() {
        let o: Output = serde_json::from_str(
            r#"{"reason":"compiler-message","manifest_path":null,"message":{"code":{"code":"foo,bar"},"level":"warning","message":"m","spans":[{"file_name":"a,b.rs","line_start":1,"line_end":1,"column_start":1,"column_end":2}],"rendered":"r"},"success":null}"#,
        )
        .expect("valid");
        let s = github_pr_annotation(&[o]);
        assert!(s.contains("file=a%2Cb.rs"), "comma in path not escaped: {s}");
        assert!(s.contains("title=foo%2Cbar"), "comma in title not escaped: {s}");
    }

    #[test]
    fn annotation_falls_back_to_message_as_title_when_no_code() {
        let o: Output = serde_json::from_str(
            r#"{"reason":"compiler-message","manifest_path":null,"message":{"code":null,"level":"warning","message":"deprecated lint renamed","spans":[],"rendered":"r"},"success":null}"#,
        )
        .expect("valid");
        let s = github_pr_annotation(&[o]);
        assert!(s.contains("title=deprecated lint renamed"), "fallback title missing: {s}");
    }

    #[test]
    fn annotation_encodes_newlines_in_body() {
        let o: Output = serde_json::from_str(
            r#"{"reason":"compiler-message","manifest_path":null,"message":{"code":{"code":"c"},"level":"warning","message":"m","spans":[{"file_name":"x","line_start":1,"line_end":1,"column_start":1,"column_end":2}],"rendered":"line1\nline2"},"success":null}"#,
        )
        .expect("valid");
        let s = github_pr_annotation(&[o]);
        assert!(s.contains("%0A"), "newline not encoded: {s}");
        assert!(!s.contains('\n'), "raw newline must not appear: {s:?}");
    }

    #[test]
    fn human_collects_rendered_text() {
        let s = human(&[warning()]);
        assert!(s.contains("warning: unused"));
    }

    // Regression: deprecated-lint warnings have empty `spans`. They must survive
    // all three formatters.
    const DEPRECATED_LINT_JSON: &str = r#"{
        "reason":"compiler-message",
        "manifest_path":"/x/Cargo.toml",
        "message":{
            "code":{"code":"renamed_and_removed_lints"},
            "level":"warning",
            "message":"the lint `clippy::cognitive_complexity` has been renamed",
            "spans":[],
            "rendered":"warning: the lint has been renamed"
        }
    }"#;

    fn deprecated_lint() -> Output {
        serde_json::from_str(DEPRECATED_LINT_JSON).expect("valid JSON")
    }

    #[test]
    fn human_retains_warnings_without_spans() {
        assert!(human(&[deprecated_lint()]).contains("renamed"));
    }

    #[test]
    fn summary_retains_warnings_without_spans() {
        let s = github_summary(&[deprecated_lint()], false);
        assert!(s.contains("⚠️ warning"), "level badge missing: {s}");
        assert!(s.contains("has been renamed"), "message missing: {s}");
        // No span → location is an em-dash placeholder.
        assert!(s.contains("| — |"), "placeholder location missing: {s}");
    }

    #[test]
    fn annotation_retains_warnings_without_spans() {
        let s = github_pr_annotation(&[deprecated_lint()]);
        assert!(!s.trim().is_empty());
        assert!(s.starts_with("::warning"));
        assert!(s.contains("title=renamed_and_removed_lints"));
    }
}
