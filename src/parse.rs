use std::error::Error;
use std::fmt;

/// Failure to parse a named value such as a theme id, palette id or hour
/// format. Carries the accepted values so callers can print a self-explanatory
/// message without duplicating the list of valid options.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ParseError {
    kind: &'static str,
    value: String,
    expected: String,
}

impl ParseError {
    pub fn new(kind: &'static str, value: &str, expected: impl Into<String>) -> Self {
        Self {
            kind,
            value: value.to_string(),
            expected: expected.into(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown {} `{}` (expected one of: {})",
            self.kind, self.value, self.expected
        )
    }
}

impl Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_lists_kind_value_and_expectations() {
        let e = ParseError::new("theme", "retro", "large, small");
        assert_eq!(
            e.to_string(),
            "unknown theme `retro` (expected one of: large, small)"
        );
    }

    #[test]
    fn is_a_std_error() {
        fn assert_error<E: Error>(_: &E) {}
        assert_error(&ParseError::new("palette", "neon", "red"));
    }
}
