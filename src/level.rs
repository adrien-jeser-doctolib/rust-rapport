//! Diagnostic severity level parsed from the clippy `level` string.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_levels() {
        assert_eq!(Level::from_str("error"), Some(Level::Error));
        assert_eq!(Level::from_str("warning"), Some(Level::Warning));
    }

    #[test]
    fn returns_none_for_unknown() {
        assert_eq!(Level::from_str("note"), None);
        assert_eq!(Level::from_str("help"), None);
        assert_eq!(Level::from_str(""), None);
        assert_eq!(Level::from_str("ERROR"), None);
    }
}
