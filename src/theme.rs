use crossterm::style::Color;

use crate::font::{Face, Style};

/// First char index of the seconds part in "HH:MM:SS".
pub const SECONDS_START: usize = 5;

/// Selectable clock theme: typeface rendering + color palette.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    /// Current look: large double-width block digits, white/cyan.
    Modern,
    /// Old look: original single-width block digits, white/cyan.
    Classic,
    /// Compact plain-ASCII 3×3 digits, green phosphor.
    Compact,
}

impl Theme {
    pub fn all() -> [Theme; 3] {
        [Theme::Modern, Theme::Classic, Theme::Compact]
    }

    /// CLI id: `--theme <id>`.
    pub fn id(self) -> &'static str {
        match self {
            Theme::Modern => "modern",
            Theme::Classic => "classic",
            Theme::Compact => "compact",
        }
    }

    /// Short human-readable description for `--list-themes`.
    pub fn description(self) -> &'static str {
        match self {
            Theme::Modern => "current: large double-width block digits",
            Theme::Classic => "old: original single-width block digits",
            Theme::Compact => "compact 3x3 plain-ASCII digits",
        }
    }

    /// Key (`1`/`2`/`3`) that selects this theme live.
    pub fn hotkey(self) -> char {
        match self {
            Theme::Modern => '1',
            Theme::Classic => '2',
            Theme::Compact => '3',
        }
    }

    pub fn from_hotkey(key: char) -> Option<Theme> {
        Theme::all().into_iter().find(|t| t.hotkey() == key)
    }

    /// Case-insensitive CLI parsing.
    pub fn from_str(s: &str) -> Option<Theme> {
        let lower = s.to_lowercase();
        Theme::all()
            .into_iter()
            .find(|t| t.id() == lower)
    }

    /// Rendering parameters for this theme.
    pub fn style(self) -> Style {
        match self {
            Theme::Modern => Style {
                face: Face::Block,
                scale_x: 2,
                gap_x: 2,
            },
            Theme::Classic => Style {
                face: Face::Block,
                scale_x: 1,
                gap_x: 1,
            },
            Theme::Compact => Style {
                face: Face::Compact,
                scale_x: 1,
                gap_x: 1,
            },
        }
    }

    pub fn border(self) -> Color {
        Color::DarkGrey
    }

    pub fn separator(self) -> Color {
        Color::DarkGrey
    }

    pub fn date(self) -> Color {
        Color::Grey
    }

    pub fn hours_minutes(self) -> Color {
        match self {
            Theme::Compact => Color::Green,
            _ => Color::White,
        }
    }

    pub fn seconds(self) -> Color {
        match self {
            Theme::Compact => Color::DarkGreen,
            _ => Color::Cyan,
        }
    }

    /// "HH:MM:SS" -> chars before `SECONDS_START` in the main color,
    /// the second colon and seconds in the accent color.
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
        assert_eq!(Theme::from_str("modern"), Some(Theme::Modern));
        assert_eq!(Theme::from_str("Classic"), Some(Theme::Classic));
        assert_eq!(Theme::from_str("COMPACT"), Some(Theme::Compact));
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
    }

    #[test]
    fn styles_match_documented_geometry() {
        assert_eq!(Theme::Modern.style().face.glyph_h(), 7);
        assert_eq!(line_width_of(Theme::Modern, 8), 8 * 10 + 7 * 2);
        assert_eq!(line_width_of(Theme::Classic, 8), 8 * 5 + 7);
        assert_eq!(line_width_of(Theme::Compact, 8), 8 * 3 + 7);
        // Compact is actually narrower than the old look.
        assert!(line_width_of(Theme::Compact, 8) < line_width_of(Theme::Classic, 8));
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
