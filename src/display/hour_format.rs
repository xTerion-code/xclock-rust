#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum HourFormat {
    #[default]
    H24,
    H12,
}

impl HourFormat {
    pub fn is_12h(self) -> bool {
        self == HourFormat::H12
    }

    pub fn toggle(self) -> Self {
        match self {
            HourFormat::H24 => HourFormat::H12,
            HourFormat::H12 => HourFormat::H24,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "12" | "12h" => Some(HourFormat::H12),
            "24" | "24h" => Some(HourFormat::H24),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_24h() {
        assert_eq!(HourFormat::default(), HourFormat::H24);
        assert!(!HourFormat::default().is_12h());
        assert!(HourFormat::H12.is_12h());
    }

    #[test]
    fn toggle_flips_format() {
        assert_eq!(HourFormat::H24.toggle(), HourFormat::H12);
        assert_eq!(HourFormat::H12.toggle(), HourFormat::H24);
    }

    #[test]
    fn parses_known_ids() {
        assert_eq!(HourFormat::from_str("12"), Some(HourFormat::H12));
        assert_eq!(HourFormat::from_str("12h"), Some(HourFormat::H12));
        assert_eq!(HourFormat::from_str("24"), Some(HourFormat::H24));
        assert_eq!(HourFormat::from_str("24H"), Some(HourFormat::H24));
        assert_eq!(HourFormat::from_str("13"), None);
        assert_eq!(HourFormat::from_str(""), None);
    }
}
