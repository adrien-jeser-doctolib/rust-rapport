use serde::{Deserialize, Serialize};

use crate::level::Level;

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
pub struct Output {
    reason: Option<String>,
    manifest_path: Option<String>,
    message: Option<Message>,
    success: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
pub struct Target {
    kind: Vec<Option<String>>,
    crate_types: Option<String>,
    name_path: Option<String>,
    edition: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Hash, PartialEq, Eq)]
pub struct Message {
    code: Option<Code>,
    level: Option<String>,
    message: Option<String>,
    spans: Vec<Span>,
    rendered: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
pub struct Code {
    code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
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
        self.message
            .as_ref()
            .and_then(|message| message.level.as_ref())
            .map(std::string::String::as_str)
    }

    pub fn line_start(&self) -> Option<usize> {
        self.message
            .as_ref()
            .and_then(|message| message.spans.first())
            .and_then(|span| span.line_start)
    }

    pub fn line_end(&self) -> Option<usize> {
        self.message
            .as_ref()
            .and_then(|message| message.spans.first())
            .and_then(|span| span.line_end)
    }

    pub fn column_start(&self) -> Option<usize> {
        self.message
            .as_ref()
            .and_then(|message| message.spans.first())
            .and_then(|span| span.column_end)
    }

    pub fn column_end(&self) -> Option<usize> {
        self.message
            .as_ref()
            .and_then(|message| message.spans.first())
            .and_then(|span| span.column_end)
    }

    pub fn file_name(&self) -> Option<&str> {
        self.message
            .as_ref()
            .and_then(|message| message.spans.first())
            .and_then(|span| span.file_name.as_ref())
            .map(std::string::String::as_str)
    }

    pub fn is_level(&self, level: &Level) -> bool {
        Level::from_str(self.level().unwrap_or_default())
            .map(|l| l == *level)
            .unwrap_or_default()
    }

    pub fn rendered(&self) -> Option<&str> {
        self.message
            .as_ref()
            .and_then(|message| message.rendered.as_ref())
            .map(std::string::String::as_str)
    }

    pub fn message(&self) -> Option<String> {
        self.message
            .as_ref()
            .and_then(|message| message.message.as_ref())
            .map(std::string::ToString::to_string)
    }
}
