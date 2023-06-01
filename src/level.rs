#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Level {
    Error,
    Warning,
}

impl Level {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "error" => Some(Self::Error),
            "warning" => Some(Self::Warning),
            _ => None,
        }
    }
}
