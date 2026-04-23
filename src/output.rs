//! Data structures mirroring `cargo --message-format json` output.

use serde::{Deserialize, Serialize};

use crate::level::Level;

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Output {
    reason: Option<String>,
    manifest_path: Option<String>,
    message: Option<Message>,
    success: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[allow(clippy::struct_field_names)]
pub struct Message {
    code: Option<Code>,
    level: Option<String>,
    message: Option<String>,
    spans: Vec<Span>,
    rendered: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Code {
    code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span {
    file_name: Option<String>,
    line_start: Option<usize>,
    line_end: Option<usize>,
    column_start: Option<usize>,
    column_end: Option<usize>,
}

impl Output {
    pub fn success(&self) -> bool {
        self.success.unwrap_or_default()
    }

    pub fn level(&self) -> Option<&str> {
        self.message.as_ref().and_then(|m| m.level.as_deref())
    }

    pub fn line_start(&self) -> Option<usize> {
        self.first_span().and_then(|s| s.line_start)
    }

    pub fn line_end(&self) -> Option<usize> {
        self.first_span().and_then(|s| s.line_end)
    }

    pub fn column_start(&self) -> Option<usize> {
        self.first_span().and_then(|s| s.column_start)
    }

    pub fn column_end(&self) -> Option<usize> {
        self.first_span().and_then(|s| s.column_end)
    }

    pub fn file_name(&self) -> Option<&str> {
        self.first_span().and_then(|s| s.file_name.as_deref())
    }

    pub fn is_level(&self, level: &Level) -> bool {
        Level::from_str(self.level().unwrap_or_default()).is_some_and(|l| l == *level)
    }

    pub fn rendered(&self) -> Option<&str> {
        self.message.as_ref().and_then(|m| m.rendered.as_deref())
    }

    pub fn message(&self) -> Option<String> {
        self.message.as_ref().and_then(|m| m.message.as_ref()).cloned()
    }

    fn first_span(&self) -> Option<&Span> {
        self.message.as_ref().and_then(|m| m.spans.first())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WARNING_JSON: &str = r#"{
        "reason": "compiler-message",
        "manifest_path": "/x/Cargo.toml",
        "message": {
            "code": { "code": "unused_variables" },
            "level": "warning",
            "message": "unused variable: `foo`",
            "spans": [{
                "file_name": "src/main.rs",
                "line_start": 10,
                "line_end": 11,
                "column_start": 5,
                "column_end": 8
            }],
            "rendered": "warning: unused variable"
        }
    }"#;

    const BUILD_FINISHED_OK: &str = r#"{"reason":"build-finished","success":true}"#;
    const BUILD_FINISHED_KO: &str = r#"{"reason":"build-finished","success":false}"#;

    fn parse(s: &str) -> Output {
        serde_json::from_str(s).expect("valid JSON")
    }

    #[test]
    fn column_start_returns_column_start_not_column_end() {
        let o = parse(WARNING_JSON);
        assert_eq!(o.column_start(), Some(5));
        assert_eq!(o.column_end(), Some(8));
        assert_ne!(o.column_start(), o.column_end());
    }

    #[test]
    fn accessors_read_first_span() {
        let o = parse(WARNING_JSON);
        assert_eq!(o.line_start(), Some(10));
        assert_eq!(o.line_end(), Some(11));
        assert_eq!(o.file_name(), Some("src/main.rs"));
        assert_eq!(o.level(), Some("warning"));
        assert_eq!(o.rendered(), Some("warning: unused variable"));
        assert_eq!(o.message().as_deref(), Some("unused variable: `foo`"));
    }

    #[test]
    fn is_level_dispatches_correctly() {
        let o = parse(WARNING_JSON);
        assert!(o.is_level(&Level::Warning));
        assert!(!o.is_level(&Level::Error));
    }

    #[test]
    fn success_reflects_build_finished_payload() {
        assert!(parse(BUILD_FINISHED_OK).success());
        assert!(!parse(BUILD_FINISHED_KO).success());
    }

    #[test]
    fn accessors_handle_missing_message_gracefully() {
        let o = parse(BUILD_FINISHED_OK);
        assert_eq!(o.level(), None);
        assert_eq!(o.file_name(), None);
        assert_eq!(o.column_start(), None);
        assert_eq!(o.rendered(), None);
        assert_eq!(o.message(), None);
        assert!(!o.is_level(&Level::Warning));
        assert!(!o.is_level(&Level::Error));
    }
}
