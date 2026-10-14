use crossterm::style::Color;

use crate::font::{Face, Style};

pub const SECONDS_START: usize = 5;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Large,
    Medium,
    Small,
}

impl Theme {
    pub fn all() -> [Theme; 3] {
        [Theme::Large, Theme::Medium, Theme::Small]
    }

    pub fn id(self) -> &'static str {
        match self {
            Theme::Large => "large",
            Theme::Medium => "medium",
            Theme::Small => "small",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Theme::Large => "large: double-width block digits for wide terminals",
            Theme::Medium => "medium: single-width block digits for standard terminals",
            Theme::Small => "small: compact 3x5 digits for small terminals",
        }
    }

    pub fn hotkey(self) -> char {
        match self {
            Theme::Large => '1',
            Theme::Medium => '2',
            Theme::Small => '3',
        }
    }

    pub fn from_hotkey(key: char) -> Option<Theme> {
        Theme::all().into_iter().find(|t| t.hotkey() == key)
    }

    pub fn from_str(s: &str) -> Option<Theme> {
        match s.to_lowercase().as_str() {
            "large" | "modern" | "big" => Some(Theme::Large),
            "medium" | "classic" | "mid" => Some(Theme::Medium),
            "small" | "compact" | "mini" => Some(Theme::Small),
            _ => None,
        }
    }

    pub fn style(self) -> Style {
        match self {
            Theme::Large => Style {
                face: Face::Block,
                scale_x: 2,
                gap_x: 2,
            },
            Theme::Medium => Style {
                face: Face::Block,
                scale_x: 1,
                gap_x: 1,
            },
            Theme::Small => Style {
                face: Face::Compact,
                scale_x: 1,
                gap_x: 1,
            },
        }
    }

    // Black-and-purple palette: vivid purple digits on the
    // terminal's (black) background, dark purple frame.
    pub fn border(self) -> Color {
        Color::Rgb {
            r: 88,
            g: 28,
            b: 135,
        }
    }

    pub fn separator(self) -> Color {
        Color::Rgb {
            r: 88,
            g: 28,
            b: 135,
        }
    }

    pub fn date(self) -> Color {
        Color::Rgb {
            r: 192,
            g: 132,
            b: 252,
        }
    }

    pub fn hours_minutes(self) -> Color {
        Color::Rgb {
            r: 168,
            g: 85,
            b: 247,
        }
    }

    pub fn seconds(self) -> Color {
        Color::Rgb {
            r: 216,
            g: 180,
            b: 254,
        }
    }

    pub fn time_color(self, index: usize) -> Color {
        if index < SECONDS_START {
            self.hours_minutes()
        } else {
            self.seconds()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ids_case_insensitively() {
        assert_eq!(Theme::from_str("large"), Some(Theme::Large));
        assert_eq!(Theme::from_str("Medium"), Some(Theme::Medium));
        assert_eq!(Theme::from_str("SMALL"), Some(Theme::Small));
        assert_eq!(Theme::from_str("modern"), Some(Theme::Large));
        assert_eq!(Theme::from_str("classic"), Some(Theme::Medium));
        assert_eq!(Theme::from_str("compact"), Some(Theme::Small));
        assert_eq!(Theme::from_str("retro"), None);
        assert_eq!(Theme::from_str(""), None);
    }

    #[test]
    fn hotkeys_cover_all_themes_uniquely() {
        let keys: Vec<char> = Theme::all().iter().map(|t| t.hotkey()).collect();
        assert_eq!(keys.len(), 3);
        for t in Theme::all() {
            assert_eq!(Theme::from_hotkey(t.hotkey()), Some(t));
        }
        assert_eq!(Theme::from_hotkey('q'), None);
        for reserved in ['s', 'h', 'b', 'd', 'q'] {
            assert!(
                Theme::from_hotkey(reserved).is_none(),
                "theme hotkey collides with action key `{reserved}`"
            );
            assert!(
                Theme::from_hotkey(reserved.to_ascii_uppercase()).is_none(),
                "theme hotkey collides with action key `{}`",
                reserved.to_ascii_uppercase()
            );
        }
    }

    #[test]
    fn styles_match_documented_geometry() {
        assert_eq!(Theme::Large.style().face.glyph_h(), 7);
        assert_eq!(Theme::Medium.style().face.glyph_h(), 7);
        assert_eq!(Theme::Small.style().face.glyph_h(), 5);
        assert_eq!(line_width_of(Theme::Large, 8), 8 * 10 + 7 * 2);
        assert_eq!(line_width_of(Theme::Medium, 8), 8 * 5 + 7);
        assert_eq!(line_width_of(Theme::Small, 8), 8 * 3 + 7);
    }

    #[test]
    fn splits_hours_minutes_from_seconds() {
        for theme in Theme::all() {
            for i in 0..SECONDS_START {
                assert_eq!(theme.time_color(i), theme.hours_minutes());
            }
            for i in SECONDS_START..8 {
                assert_eq!(theme.time_color(i), theme.seconds());
            }
        }
    }

    fn line_width_of(theme: Theme, n: usize) -> usize {
        crate::font::line_width(n, theme.style())
    }
}
