use crossterm::style::Color;
use std::str::FromStr;

use crate::font::{FaceLayout, Style, line_width};
use crate::parse::ParseError;

/// Columns taken by the two vertical borders of the framed face.
pub const FRAME_COLUMNS: usize = 2;

/// Columns kept free inside the frame so the clock never touches a border.
pub const FRAME_MARGIN: usize = 4;

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb { r, g, b }
}

/// Colour table for one palette. Keeping the values in a `const` table instead
/// of a `match` per channel makes adding a palette a single struct literal and
/// removes any chance of two arms drifting apart.
struct PaletteSpec {
    id: &'static str,
    aliases: &'static [&'static str],
    description: &'static str,
    border: Color,
    separator: Color,
    date: Color,
    hours_minutes: Color,
    seconds: Color,
}

const PALETTES: [PaletteSpec; 6] = [
    PaletteSpec {
        id: "black-red",
        aliases: &["black_red", "red", "crimson", "default"],
        description: "black-red: bold red digits on dark framing with soft coral seconds",
        border: rgb(0x7F, 0x1D, 0x1D),
        separator: rgb(0x99, 0x1B, 0x1B),
        date: rgb(0xF8, 0x71, 0x71),
        hours_minutes: rgb(0xEF, 0x44, 0x44),
        seconds: rgb(0xFC, 0xA5, 0xA5),
    },
    PaletteSpec {
        id: "purple",
        aliases: &[],
        description: "purple: vivid purple digits on black with light lavender seconds",
        border: rgb(0x58, 0x1C, 0x87),
        separator: rgb(0x58, 0x1C, 0x87),
        date: rgb(0xC0, 0x84, 0xFC),
        hours_minutes: rgb(0xA8, 0x55, 0xF7),
        seconds: rgb(0xD8, 0xB4, 0xFE),
    },
    PaletteSpec {
        id: "matrix",
        aliases: &["matrix-green", "matrix_green", "green"],
        description: "matrix: classic matrix green phosphor display",
        border: rgb(0x14, 0x50, 0x14),
        separator: rgb(0x22, 0x78, 0x22),
        date: rgb(0x32, 0xCD, 0x32),
        hours_minutes: rgb(0x22, 0xC5, 0x5E),
        seconds: rgb(0x86, 0xEF, 0xAC),
    },
    PaletteSpec {
        id: "amber",
        aliases: &["amber-crt", "amber_crt", "crt"],
        description: "amber: warm amber CRT phosphor display",
        border: rgb(0x78, 0x3C, 0x00),
        separator: rgb(0x8C, 0x46, 0x00),
        date: rgb(0xFB, 0xBF, 0x24),
        hours_minutes: rgb(0xF5, 0x9E, 0x0B),
        seconds: rgb(0xFE, 0xD7, 0xAA),
    },
    PaletteSpec {
        id: "monochrome",
        aliases: &["mono", "gray", "grey", "white"],
        description: "monochrome: crisp high-contrast white and silver monochrome",
        border: rgb(0x4B, 0x55, 0x63),
        separator: rgb(0x6B, 0x72, 0x80),
        date: rgb(0xD1, 0xD5, 0xDB),
        hours_minutes: rgb(0xFF, 0xFF, 0xFF),
        seconds: rgb(0x9C, 0xA3, 0xAF),
    },
    PaletteSpec {
        id: "rainbow",
        aliases: &["spectral", "colors", "color"],
        description: "rainbow: vibrant multi-color spectral digits",
        border: rgb(0x63, 0x66, 0xF1),
        separator: rgb(0x81, 0x8C, 0xF8),
        date: rgb(0xFD, 0xE0, 0x47),
        hours_minutes: rgb(0xEF, 0x44, 0x44),
        seconds: rgb(0x8B, 0x5C, 0xF6),
    },
];

const RAINBOW_DIGITS: [Color; 8] = [
    rgb(0xEF, 0x44, 0x44),
    rgb(0xF9, 0x73, 0x16),
    rgb(0xEA, 0xB3, 0x08),
    rgb(0x22, 0xC5, 0x5E),
    rgb(0x06, 0xB6, 0xD4),
    rgb(0x3B, 0x82, 0xF6),
    rgb(0x8B, 0x5C, 0xF6),
    rgb(0xEC, 0x48, 0x99),
];

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Large,
    Medium,
    Small,
    Auto,
}

/// A [`Theme`] whose glyph geometry is already known.
///
/// `Theme::Auto` has no face of its own, so layout and font code only ever see
/// this type: `Theme::resolve` is the single way to produce one, which makes
/// "used the auto theme's geometry by mistake" unrepresentable.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ResolvedTheme(Theme);

impl ResolvedTheme {
    /// Wraps a theme that is already concrete. Used by `resolve` and by tests.
    pub fn concrete(theme: Theme) -> Self {
        debug_assert_ne!(
            theme,
            Theme::Auto,
            "Theme::Auto has no geometry; resolve it against the terminal first"
        );
        Self(theme)
    }

    pub fn style(self) -> Style {
        self.0.concrete_style()
    }

    /// Width of the digit face in terminal columns.
    pub fn clock_width(self, layout: FaceLayout) -> usize {
        line_width(layout.clock_chars(), self.style())
    }

    /// Height of the whole framed block in terminal rows.
    pub fn block_height(self, layout: FaceLayout) -> usize {
        layout.rows(self.style().face.glyph_h())
    }

    /// Whether the framed face fits without clipping or scrolling the viewport.
    pub fn fits(self, cols: usize, lines: usize, layout: FaceLayout) -> bool {
        cols >= self.clock_width(layout) + FRAME_MARGIN && lines >= self.block_height(layout)
    }
}

impl Theme {
    pub const ALL: [Theme; 4] = [Theme::Large, Theme::Medium, Theme::Small, Theme::Auto];

    pub fn all() -> &'static [Theme] {
        &Self::ALL
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
        self.name()
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
        Theme::ALL.into_iter().find(|t| t.hotkey() == lower)
    }

    /// Glyph geometry of a concrete theme. `Auto` must be resolved first.
    fn concrete_style(self) -> Style {
        match self {
            Theme::Large | Theme::Auto => Style {
                face: crate::font::Face::Block,
                scale_x: 2,
                gap_x: 2,
            },
            Theme::Medium => Style {
                face: crate::font::Face::Block,
                scale_x: 1,
                gap_x: 1,
            },
            Theme::Small => Style {
                face: crate::font::Face::Compact,
                scale_x: 1,
                gap_x: 1,
            },
        }
    }

    pub fn auto_fit(cols: usize, lines: usize, layout: FaceLayout) -> Theme {
        if Theme::Large.fits(cols, lines, layout) {
            Theme::Large
        } else if Theme::Medium.fits(cols, lines, layout) {
            Theme::Medium
        } else {
            Theme::Small
        }
    }

    /// Turns a possibly-automatic theme into one with concrete geometry.
    ///
    /// A concrete theme that does not fit degrades to the largest theme that
    /// does, exactly like `auto`. That way the digit face is never clipped and
    /// the framed layout is only abandoned when no face fits at all.
    pub fn resolve(self, cols: usize, lines: usize, layout: FaceLayout) -> ResolvedTheme {
        let concrete = match self {
            Theme::Auto => Theme::auto_fit(cols, lines, layout),
            requested if requested.fits(cols, lines, layout) => requested,
            _ => Theme::auto_fit(cols, lines, layout),
        };
        ResolvedTheme::concrete(concrete)
    }

    fn name(self) -> &'static str {
        match self {
            Theme::Large => "large",
            Theme::Medium => "medium",
            Theme::Small => "small",
            Theme::Auto => "auto",
        }
    }

    fn fits(self, cols: usize, lines: usize, layout: FaceLayout) -> bool {
        ResolvedTheme::concrete(self).fits(cols, lines, layout)
    }
}

impl FromStr for Theme {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "large" | "modern" | "big" => Ok(Theme::Large),
            "medium" | "classic" | "mid" => Ok(Theme::Medium),
            "small" | "compact" | "mini" => Ok(Theme::Small),
            "auto" | "adaptive" | "fit" => Ok(Theme::Auto),
            _ => Err(ParseError::new("theme", s, theme_ids())),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Palette {
    #[default]
    BlackRed = 0,
    Purple = 1,
    Matrix = 2,
    Amber = 3,
    Monochrome = 4,
    Rainbow = 5,
}

impl Palette {
    pub const ALL: [Palette; 6] = [
        Palette::BlackRed,
        Palette::Purple,
        Palette::Matrix,
        Palette::Amber,
        Palette::Monochrome,
        Palette::Rainbow,
    ];

    pub fn all() -> &'static [Palette] {
        &Self::ALL
    }

    fn spec(self) -> &'static PaletteSpec {
        &PALETTES[self as usize]
    }

    pub fn id(self) -> &'static str {
        self.spec().id
    }

    pub fn aliases(self) -> &'static [&'static str] {
        self.spec().aliases
    }

    pub fn description(self) -> &'static str {
        self.spec().description
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
        self.spec().border
    }

    pub fn separator(self) -> Color {
        self.spec().separator
    }

    pub fn date(self) -> Color {
        self.spec().date
    }

    pub fn hours_minutes(self) -> Color {
        self.spec().hours_minutes
    }

    pub fn seconds(self) -> Color {
        self.spec().seconds
    }

    /// Colour of the digit at `index` of the time string. `seconds_start` is
    /// the column where the seconds field begins, or `None` when the seconds
    /// are hidden; everything before it uses the hours/minutes colour.
    pub fn time_color(self, index: usize, seconds_start: Option<usize>) -> Color {
        if matches!(self, Palette::Rainbow) {
            return RAINBOW_DIGITS[index % RAINBOW_DIGITS.len()];
        }
        match seconds_start {
            Some(start) if index >= start => self.seconds(),
            _ => self.hours_minutes(),
        }
    }
}

impl FromStr for Palette {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_ascii_lowercase();
        PALETTES
            .iter()
            .position(|spec| spec.id == lower || spec.aliases.contains(&lower.as_str()))
            .map(palette_at)
            .ok_or_else(|| ParseError::new("palette", s, palette_ids()))
    }
}

const fn palette_at(index: usize) -> Palette {
    match index {
        0 => Palette::BlackRed,
        1 => Palette::Purple,
        2 => Palette::Matrix,
        3 => Palette::Amber,
        4 => Palette::Monochrome,
        _ => Palette::Rainbow,
    }
}

pub fn theme_ids() -> String {
    Theme::ALL
        .iter()
        .map(|t| t.id())
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn palette_ids() -> String {
    Palette::ALL
        .iter()
        .map(|p| p.id())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::Face;

    const FULL: FaceLayout = FaceLayout::new(true, true, true);

    fn style_of(theme: Theme) -> Style {
        ResolvedTheme::concrete(theme).style()
    }

    #[test]
    fn parses_ids_case_insensitively() {
        assert_eq!("large".parse::<Theme>(), Ok(Theme::Large));
        assert_eq!("Medium".parse::<Theme>(), Ok(Theme::Medium));
        assert_eq!("SMALL".parse::<Theme>(), Ok(Theme::Small));
        assert_eq!("auto".parse::<Theme>(), Ok(Theme::Auto));
        assert_eq!("adaptive".parse::<Theme>(), Ok(Theme::Auto));
        assert_eq!("fit".parse::<Theme>(), Ok(Theme::Auto));
        assert_eq!("modern".parse::<Theme>(), Ok(Theme::Large));
        assert_eq!("classic".parse::<Theme>(), Ok(Theme::Medium));
        assert_eq!("compact".parse::<Theme>(), Ok(Theme::Small));
        assert!("retro".parse::<Theme>().is_err());
        assert!("".parse::<Theme>().is_err());
    }

    #[test]
    fn parse_error_lists_theme_ids() {
        let e = "retro".parse::<Theme>().unwrap_err();
        assert_eq!(
            e.to_string(),
            "unknown theme `retro` (expected one of: large, medium, small, auto)"
        );
    }

    #[test]
    fn theme_all_and_next_cycling() {
        assert_eq!(Theme::all().len(), 4);
        let mut curr = Theme::Large;
        for next in [Theme::Medium, Theme::Small, Theme::Auto, Theme::Large] {
            curr = curr.next();
            assert_eq!(curr, next);
        }
    }

    #[test]
    fn hotkeys_cover_all_themes_uniquely() {
        let keys: Vec<char> = Theme::ALL.iter().map(|t| t.hotkey()).collect();
        let mut unique = keys.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), keys.len(), "theme hotkeys must be unique");
        for t in Theme::ALL {
            assert_eq!(Theme::from_hotkey(t.hotkey()), Some(t));
        }
        assert_eq!(Theme::from_hotkey('0'), Some(Theme::Auto));
        assert_eq!(Theme::from_hotkey('q'), None);
        for reserved in ['s', 'h', 'b', 'd', 'i', 'q', 'p', 't'] {
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
        assert_eq!(style_of(Theme::Large).face, Face::Block);
        assert_eq!(style_of(Theme::Large).face.glyph_h(), 7);
        assert_eq!(style_of(Theme::Medium).face.glyph_h(), 7);
        assert_eq!(style_of(Theme::Small).face.glyph_h(), 5);
        assert_eq!(line_width(8, style_of(Theme::Large)), 8 * 10 + 7 * 2);
        assert_eq!(line_width(8, style_of(Theme::Medium)), 8 * 5 + 7);
        assert_eq!(line_width(8, style_of(Theme::Small)), 8 * 3 + 7);
    }

    #[test]
    fn resolve_always_yields_a_concrete_face() {
        let large = ResolvedTheme::concrete(Theme::Large);
        let medium = ResolvedTheme::concrete(Theme::Medium);
        let small = ResolvedTheme::concrete(Theme::Small);
        assert_eq!(Theme::Auto.resolve(120, 30, FULL), large);
        assert_eq!(Theme::Auto.resolve(80, 24, FULL), medium);
        assert_eq!(Theme::Auto.resolve(45, 14, FULL), small);
        // A concrete theme is kept whenever it fits.
        assert_eq!(Theme::Large.resolve(120, 30, FULL), large);
        assert_eq!(Theme::Small.resolve(200, 60, FULL), small);
        assert_eq!(Theme::Medium.resolve(60, 20, FULL), medium);
    }

    #[test]
    fn resolve_degrades_instead_of_clipping() {
        // The regression that started all this: `large` on a standard 80x24
        // terminal does not fit, so it must degrade rather than clip digits.
        let resolved = Theme::Large.resolve(80, 24, FULL);
        assert_eq!(resolved, ResolvedTheme::concrete(Theme::Medium));
        assert!(resolved.fits(80, 24, FULL));
        assert!(resolved.clock_width(FULL) + FRAME_MARGIN <= 80);

        // Even a request for the smallest face degrades when nothing fits,
        // and the caller then sees `fits() == false` to pick the fallback.
        let tiny = ResolvedTheme::concrete(Theme::Small);
        assert!(!tiny.fits(10, 8, FULL));
        assert_eq!(
            Theme::Small.resolve(10, 8, FULL),
            ResolvedTheme::concrete(Theme::auto_fit(10, 8, FULL))
        );
    }

    #[test]
    fn auto_fit_follows_visible_content() {
        // Large needs 98 columns for 8 characters, so 80x24 gets Medium.
        assert_eq!(Theme::auto_fit(120, 30, FULL), Theme::Large);
        assert_eq!(Theme::auto_fit(80, 24, FULL), Theme::Medium);
        assert_eq!(Theme::auto_fit(45, 14, FULL), Theme::Small);
        // Without seconds `large` only needs 58 columns, so 80x24 gets Large.
        let no_seconds = FaceLayout::new(false, true, true);
        assert_eq!(Theme::auto_fit(80, 24, no_seconds), Theme::Large);
        // Wide but vertically constrained: height alone drops us to Small.
        assert_eq!(Theme::auto_fit(120, 12, FULL), Theme::Small);
    }

    #[test]
    fn fits_accounts_for_seconds_date_and_hint() {
        let large = ResolvedTheme::concrete(Theme::Large);
        let clock = large.clock_width(FULL);
        // Exactly at the margin the frame fits; one column less it does not.
        assert!(large.fits(clock + FRAME_MARGIN, 40, FULL));
        assert!(!large.fits(clock + FRAME_MARGIN - 1, 40, FULL));
        // Hiding seconds shrinks the required width by three columns.
        let no_seconds = FaceLayout::new(false, true, true);
        // Three fewer characters: three digits plus three inter-gaps.
        assert_eq!(large.clock_width(no_seconds), clock - 36);
        assert!(large.fits(clock + FRAME_MARGIN - 36, 40, no_seconds));
        // Hiding date and hint shrinks the required height by two rows.
        let full_h = large.block_height(FULL);
        let bare_h = large.block_height(FaceLayout::new(true, false, false));
        assert_eq!(full_h - bare_h, 2);
        assert!(large.fits(200, full_h, FULL));
        assert!(!large.fits(200, full_h - 1, FULL));
        assert!(large.fits(200, bare_h, FaceLayout::new(true, false, false)));
    }

    #[test]
    fn palette_all_and_next_cycling() {
        assert_eq!(Palette::all().len(), 6);
        let mut curr = Palette::BlackRed;
        for next in [
            Palette::Purple,
            Palette::Matrix,
            Palette::Amber,
            Palette::Monochrome,
            Palette::Rainbow,
            Palette::BlackRed,
        ] {
            curr = curr.next();
            assert_eq!(curr, next);
        }
    }

    #[test]
    fn palette_specs_are_indexed_by_discriminant() {
        for (i, palette) in Palette::ALL.iter().enumerate() {
            assert_eq!(*palette as usize, i, "PALETTES table is out of order");
            assert_eq!(palette_at(i), *palette);
            assert!(!palette.id().is_empty());
            assert!(!palette.description().is_empty());
        }
    }

    #[test]
    fn palette_parses_ids_and_aliases() {
        for id in [
            "black-red",
            "purple",
            "matrix",
            "amber",
            "monochrome",
            "rainbow",
        ] {
            let parsed: Palette = id.parse().unwrap_or_else(|_| panic!("{id} must parse"));
            assert_eq!(parsed.id(), id);
        }
        assert_eq!("black_red".parse::<Palette>(), Ok(Palette::BlackRed));
        assert_eq!("red".parse::<Palette>(), Ok(Palette::BlackRed));
        assert_eq!("crimson".parse::<Palette>(), Ok(Palette::BlackRed));
        assert_eq!("default".parse::<Palette>(), Ok(Palette::BlackRed));
        assert_eq!("Matrix".parse::<Palette>(), Ok(Palette::Matrix));
        assert_eq!("matrix-green".parse::<Palette>(), Ok(Palette::Matrix));
        assert_eq!("GREEN".parse::<Palette>(), Ok(Palette::Matrix));
        assert_eq!("amber-crt".parse::<Palette>(), Ok(Palette::Amber));
        assert_eq!("crt".parse::<Palette>(), Ok(Palette::Amber));
        assert_eq!("mono".parse::<Palette>(), Ok(Palette::Monochrome));
        assert_eq!("gray".parse::<Palette>(), Ok(Palette::Monochrome));
        assert_eq!("white".parse::<Palette>(), Ok(Palette::Monochrome));
        assert_eq!("spectral".parse::<Palette>(), Ok(Palette::Rainbow));
        assert_eq!("colors".parse::<Palette>(), Ok(Palette::Rainbow));
        assert!("unknown".parse::<Palette>().is_err());
        assert!("".parse::<Palette>().is_err());
    }

    #[test]
    fn palette_parse_error_lists_ids() {
        let e = "neon".parse::<Palette>().unwrap_err();
        assert_eq!(
            e.to_string(),
            "unknown palette `neon` (expected one of: black-red, purple, matrix, amber, monochrome, rainbow)"
        );
    }

    #[test]
    fn documented_palette_colours() {
        assert_eq!(Palette::BlackRed.hours_minutes(), rgb(0xEF, 0x44, 0x44));
        assert_eq!(Palette::BlackRed.seconds(), rgb(0xFC, 0xA5, 0xA5));
        assert_eq!(Palette::BlackRed.border(), rgb(0x7F, 0x1D, 0x1D));
        assert_eq!(Palette::BlackRed.date(), rgb(0xF8, 0x71, 0x71));
        assert_eq!(Palette::Purple.hours_minutes(), rgb(0xA8, 0x55, 0xF7));
        assert_eq!(Palette::Amber.seconds(), rgb(0xFE, 0xD7, 0xAA));
        assert_eq!(Palette::Monochrome.hours_minutes(), rgb(0xFF, 0xFF, 0xFF));
        assert_eq!(Palette::Matrix.date(), rgb(0x32, 0xCD, 0x32));
    }

    #[test]
    fn rainbow_palette_gives_distinct_colors_per_index() {
        let rainbow = Palette::Rainbow;
        let colors: Vec<Color> = (0..8).map(|i| rainbow.time_color(i, None)).collect();
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(colors[i], colors[j], "indices {i} and {j} collide");
            }
        }
        // The spectrum wraps instead of panicking on a long time string.
        assert_eq!(rainbow.time_color(8, None), rainbow.time_color(0, None));
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
            for i in 0..6 {
                assert_eq!(palette.time_color(i, Some(6)), palette.hours_minutes());
            }
            for i in 6..8 {
                assert_eq!(palette.time_color(i, Some(6)), palette.seconds());
            }
            // With hidden seconds everything is hours/minutes.
            for i in 0..5 {
                assert_eq!(palette.time_color(i, None), palette.hours_minutes());
            }
        }
    }

    #[test]
    fn time_color_follows_the_supplied_boundary() {
        let p = Palette::BlackRed;
        // A boundary other than the historical constant 5 is honoured.
        assert_eq!(p.time_color(4, Some(4)), p.seconds());
        assert_eq!(p.time_color(3, Some(4)), p.hours_minutes());
    }
}
