use crossterm::style::Color;

use crate::font::{Face, Style};

pub const SECONDS_START: usize = 5;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Large,
    Medium,
    Small,
    Auto,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Palette {
    #[default]
    BlackRed,
    Purple,
    Matrix,
    Amber,
    Monochrome,
    Rainbow,
}

impl Palette {
    pub fn all() -> [Palette; 6] {
        [
            Palette::BlackRed,
            Palette::Purple,
            Palette::Matrix,
            Palette::Amber,
            Palette::Monochrome,
            Palette::Rainbow,
        ]
    }

    pub fn id(self) -> &'static str {
        match self {
            Palette::BlackRed => "black-red",
            Palette::Purple => "purple",
            Palette::Matrix => "matrix",
            Palette::Amber => "amber",
            Palette::Monochrome => "monochrome",
            Palette::Rainbow => "rainbow",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Palette::BlackRed => {
                "black-red: bold red digits on dark framing with soft coral seconds"
            }
            Palette::Purple => "purple: vivid purple digits on black with light lavender seconds",
            Palette::Matrix => "matrix: classic matrix green phosphor display",
            Palette::Amber => "amber: warm amber CRT phosphor display",
            Palette::Monochrome => "monochrome: crisp high-contrast white and silver monochrome",
            Palette::Rainbow => "rainbow: vibrant multi-color spectral digits",
        }
    }

    pub fn from_str(s: &str) -> Option<Palette> {
        match s.to_lowercase().as_str() {
            "black-red" | "black_red" | "red" | "crimson" | "default" => Some(Palette::BlackRed),
            "purple" => Some(Palette::Purple),
            "matrix" | "matrix-green" | "matrix_green" | "green" => Some(Palette::Matrix),
            "amber" | "amber-crt" | "amber_crt" | "crt" => Some(Palette::Amber),
            "monochrome" | "mono" | "gray" | "grey" | "white" => Some(Palette::Monochrome),
            "rainbow" | "spectral" | "colors" | "color" => Some(Palette::Rainbow),
            _ => None,
        }
    }

    pub fn next(self) -> Palette {
        match self {
            Palette::BlackRed => Palette::Purple,
            Palette::Purple => Palette::Matrix,
            Palette::Matrix => Palette::Amber,
            Palette::Amber => Palette::Monochrome,
            Palette::Monochrome => Palette::Rainbow,
            Palette::Rainbow => Palette::BlackRed,
        }
    }

    pub fn border(self) -> Color {
        match self {
            Palette::BlackRed => Color::Rgb {
                r: 127,
                g: 29,
                b: 29,
            },
            Palette::Purple => Color::Rgb {
                r: 88,
                g: 28,
                b: 135,
            },
            Palette::Matrix => Color::Rgb {
                r: 20,
                g: 80,
                b: 20,
            },
            Palette::Amber => Color::Rgb {
                r: 120,
                g: 60,
                b: 0,
            },
            Palette::Monochrome => Color::Rgb {
                r: 75,
                g: 85,
                b: 99,
            },
            Palette::Rainbow => Color::Rgb {
                r: 99,
                g: 102,
                b: 241,
            },
        }
    }

    pub fn separator(self) -> Color {
        match self {
            Palette::BlackRed => Color::Rgb {
                r: 153,
                g: 27,
                b: 27,
            },
            Palette::Purple => Color::Rgb {
                r: 88,
                g: 28,
                b: 135,
            },
            Palette::Matrix => Color::Rgb {
                r: 34,
                g: 120,
                b: 34,
            },
            Palette::Amber => Color::Rgb {
                r: 140,
                g: 70,
                b: 0,
            },
            Palette::Monochrome => Color::Rgb {
                r: 107,
                g: 114,
                b: 128,
            },
            Palette::Rainbow => Color::Rgb {
                r: 129,
                g: 140,
                b: 248,
            },
        }
    }

    pub fn date(self) -> Color {
        match self {
            Palette::BlackRed => Color::Rgb {
                r: 248,
                g: 113,
                b: 113,
            },
            Palette::Purple => Color::Rgb {
                r: 192,
                g: 132,
                b: 252,
            },
            Palette::Matrix => Color::Rgb {
                r: 50,
                g: 205,
                b: 50,
            },
            Palette::Amber => Color::Rgb {
                r: 251,
                g: 191,
                b: 36,
            },
            Palette::Monochrome => Color::Rgb {
                r: 209,
                g: 213,
                b: 219,
            },
            Palette::Rainbow => Color::Rgb {
                r: 253,
                g: 224,
                b: 71,
            },
        }
    }

    pub fn hours_minutes(self) -> Color {
        match self {
            Palette::BlackRed => Color::Rgb {
                r: 239,
                g: 68,
                b: 68,
            },
            Palette::Purple => Color::Rgb {
                r: 168,
                g: 85,
                b: 247,
            },
            Palette::Matrix => Color::Rgb {
                r: 34,
                g: 197,
                b: 94,
            },
            Palette::Amber => Color::Rgb {
                r: 245,
                g: 158,
                b: 11,
            },
            Palette::Monochrome => Color::Rgb {
                r: 255,
                g: 255,
                b: 255,
            },
            Palette::Rainbow => Color::Rgb {
                r: 239,
                g: 68,
                b: 68,
            },
        }
    }

    pub fn seconds(self) -> Color {
        match self {
            Palette::BlackRed => Color::Rgb {
                r: 252,
                g: 165,
                b: 165,
            },
            Palette::Purple => Color::Rgb {
                r: 216,
                g: 180,
                b: 254,
            },
            Palette::Matrix => Color::Rgb {
                r: 134,
                g: 239,
                b: 172,
            },
            Palette::Amber => Color::Rgb {
                r: 254,
                g: 215,
                b: 170,
            },
            Palette::Monochrome => Color::Rgb {
                r: 156,
                g: 163,
                b: 175,
            },
            Palette::Rainbow => Color::Rgb {
                r: 139,
                g: 92,
                b: 246,
            },
        }
    }

    pub fn time_color(self, index: usize) -> Color {
        match self {
            Palette::Rainbow => {
                const RAINBOW_COLORS: [Color; 8] = [
                    Color::Rgb {
                        r: 239,
                        g: 68,
                        b: 68,
                    },
                    Color::Rgb {
                        r: 249,
                        g: 115,
                        b: 22,
                    },
                    Color::Rgb {
                        r: 234,
                        g: 179,
                        b: 8,
                    },
                    Color::Rgb {
                        r: 34,
                        g: 197,
                        b: 94,
                    },
                    Color::Rgb {
                        r: 6,
                        g: 182,
                        b: 212,
                    },
                    Color::Rgb {
                        r: 59,
                        g: 130,
                        b: 246,
                    },
                    Color::Rgb {
                        r: 139,
                        g: 92,
                        b: 246,
                    },
                    Color::Rgb {
                        r: 236,
                        g: 72,
                        b: 153,
                    },
                ];
                RAINBOW_COLORS[index % RAINBOW_COLORS.len()]
            }
            _ => {
                if index < SECONDS_START {
                    self.hours_minutes()
                } else {
                    self.seconds()
                }
            }
        }
    }
}

impl Theme {
    pub fn all() -> [Theme; 4] {
        [Theme::Large, Theme::Medium, Theme::Small, Theme::Auto]
    }

    pub fn next(self) -> Theme {
        match self {
            Theme::Large => Theme::Medium,
            Theme::Medium => Theme::Small,
            Theme::Small => Theme::Auto,
            Theme::Auto => Theme::Large,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Theme::Large => "large",
            Theme::Medium => "medium",
            Theme::Small => "small",
            Theme::Auto => "auto",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Theme::Large => "large: double-width block digits for wide terminals",
            Theme::Medium => "medium: single-width block digits for standard terminals",
            Theme::Small => "small: compact 3x5 digits for small terminals",
            Theme::Auto => "auto: adaptive theme selected by terminal size",
        }
    }

    pub fn hotkey(self) -> char {
        match self {
            Theme::Large => '1',
            Theme::Medium => '2',
            Theme::Small => '3',
            Theme::Auto => 'a',
        }
    }

    pub fn from_hotkey(key: char) -> Option<Theme> {
        let lower = key.to_ascii_lowercase();
        if lower == '0' {
            return Some(Theme::Auto);
        }
        Theme::all().into_iter().find(|t| t.hotkey() == lower)
    }

    pub fn from_str(s: &str) -> Option<Theme> {
        match s.to_lowercase().as_str() {
            "large" | "modern" | "big" => Some(Theme::Large),
            "medium" | "classic" | "mid" => Some(Theme::Medium),
            "small" | "compact" | "mini" => Some(Theme::Small),
            "auto" | "adaptive" | "fit" => Some(Theme::Auto),
            _ => None,
        }
    }

    pub fn style(self) -> Style {
        match self {
            Theme::Large | Theme::Auto => Style {
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

    pub fn block_height(self, show_date: bool) -> usize {
        self.style().face.glyph_h() + if show_date { 7 } else { 6 }
    }

    pub fn fits(self, cols: usize, lines: usize, show_seconds: bool, show_date: bool) -> bool {
        let n_chars = if show_seconds { 8 } else { 5 };
        let clock_w = crate::font::line_width(n_chars, self.style());
        let min_cols = clock_w + 4;
        let min_lines = self.block_height(show_date);
        cols >= min_cols && lines >= min_lines
    }

    pub fn auto_fit(cols: usize, lines: usize, show_seconds: bool, show_date: bool) -> Theme {
        if Theme::Large.fits(cols, lines, show_seconds, show_date) {
            Theme::Large
        } else if Theme::Medium.fits(cols, lines, show_seconds, show_date) {
            Theme::Medium
        } else {
            Theme::Small
        }
    }

    pub fn resolve(self, cols: usize, lines: usize, show_seconds: bool, show_date: bool) -> Theme {
        match self {
            Theme::Auto => Theme::auto_fit(cols, lines, show_seconds, show_date),
            concrete => concrete,
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
        assert_eq!(Theme::from_str("auto"), Some(Theme::Auto));
        assert_eq!(Theme::from_str("adaptive"), Some(Theme::Auto));
        assert_eq!(Theme::from_str("fit"), Some(Theme::Auto));
        assert_eq!(Theme::from_str("modern"), Some(Theme::Large));
        assert_eq!(Theme::from_str("classic"), Some(Theme::Medium));
        assert_eq!(Theme::from_str("compact"), Some(Theme::Small));
        assert_eq!(Theme::from_str("retro"), None);
        assert_eq!(Theme::from_str(""), None);
    }

    #[test]
    fn theme_all_and_next_cycling() {
        let themes = Theme::all();
        assert_eq!(themes.len(), 4);
        let mut curr = Theme::Large;
        let expected = [Theme::Medium, Theme::Small, Theme::Auto, Theme::Large];
        for next in expected {
            curr = curr.next();
            assert_eq!(curr, next);
        }
    }

    #[test]
    fn hotkeys_cover_all_themes_uniquely() {
        let keys: Vec<char> = Theme::all().iter().map(|t| t.hotkey()).collect();
        assert_eq!(keys.len(), 4);
        for t in Theme::all() {
            assert_eq!(Theme::from_hotkey(t.hotkey()), Some(t));
        }
        assert_eq!(Theme::from_hotkey('q'), None);
        for reserved in ['s', 'h', 'b', 'd', 'q', 'p', 't'] {
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
    fn auto_fit_selects_by_terminal_dimensions() {
        // Wide terminal accommodates Large
        assert_eq!(Theme::auto_fit(120, 30, true, true), Theme::Large);
        // Standard 80x24 terminal accommodates Medium (Large needs 98 width for 8 chars)
        assert_eq!(Theme::auto_fit(80, 24, true, true), Theme::Medium);
        // Compact 45x14 terminal accommodates Small
        assert_eq!(Theme::auto_fit(45, 14, true, true), Theme::Small);
        // Standard 80x24 terminal with hidden seconds fits Large (5 chars: 58 width + 4 <= 80)
        assert_eq!(Theme::auto_fit(80, 24, false, true), Theme::Large);
        // Wide but vertically constrained terminal (height < 14) falls back to Small
        assert_eq!(Theme::auto_fit(120, 12, true, true), Theme::Small);
        // Resolving concrete themes returns self
        assert_eq!(Theme::Large.resolve(45, 14, true, true), Theme::Large);
        assert_eq!(Theme::Auto.resolve(80, 24, true, true), Theme::Medium);
    }

    #[test]
    fn palette_all_and_next_cycling() {
        let palettes = Palette::all();
        assert_eq!(palettes.len(), 6);
        let mut curr = Palette::BlackRed;
        let expected = [
            Palette::Purple,
            Palette::Matrix,
            Palette::Amber,
            Palette::Monochrome,
            Palette::Rainbow,
            Palette::BlackRed,
        ];
        for next in expected {
            curr = curr.next();
            assert_eq!(curr, next);
        }
    }

    #[test]
    fn palette_parses_ids_and_aliases() {
        assert_eq!(Palette::from_str("black-red"), Some(Palette::BlackRed));
        assert_eq!(Palette::from_str("black_red"), Some(Palette::BlackRed));
        assert_eq!(Palette::from_str("red"), Some(Palette::BlackRed));
        assert_eq!(Palette::from_str("crimson"), Some(Palette::BlackRed));
        assert_eq!(Palette::from_str("default"), Some(Palette::BlackRed));
        assert_eq!(Palette::from_str("purple"), Some(Palette::Purple));
        assert_eq!(Palette::from_str("Matrix"), Some(Palette::Matrix));
        assert_eq!(Palette::from_str("matrix-green"), Some(Palette::Matrix));
        assert_eq!(Palette::from_str("GREEN"), Some(Palette::Matrix));
        assert_eq!(Palette::from_str("amber"), Some(Palette::Amber));
        assert_eq!(Palette::from_str("amber-crt"), Some(Palette::Amber));
        assert_eq!(Palette::from_str("monochrome"), Some(Palette::Monochrome));
        assert_eq!(Palette::from_str("mono"), Some(Palette::Monochrome));
        assert_eq!(Palette::from_str("white"), Some(Palette::Monochrome));
        assert_eq!(Palette::from_str("rainbow"), Some(Palette::Rainbow));
        assert_eq!(Palette::from_str("spectral"), Some(Palette::Rainbow));
        assert_eq!(Palette::from_str("unknown"), None);
    }

    #[test]
    fn rainbow_palette_gives_distinct_colors_per_index() {
        let rainbow = Palette::Rainbow;
        let c0 = rainbow.time_color(0);
        let c1 = rainbow.time_color(1);
        let c2 = rainbow.time_color(2);
        let c3 = rainbow.time_color(3);
        assert_ne!(c0, c1);
        assert_ne!(c1, c2);
        assert_ne!(c2, c3);
    }

    #[test]
    fn splits_hours_minutes_from_seconds() {
        for palette in [
            Palette::BlackRed,
            Palette::Purple,
            Palette::Matrix,
            Palette::Amber,
            Palette::Monochrome,
        ] {
            for i in 0..SECONDS_START {
                assert_eq!(palette.time_color(i), palette.hours_minutes());
            }
            for i in SECONDS_START..8 {
                assert_eq!(palette.time_color(i), palette.seconds());
            }
        }
    }

    fn line_width_of(theme: Theme, n: usize) -> usize {
        crate::font::line_width(n, theme.style())
    }
}
