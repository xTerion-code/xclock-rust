/// Available typefaces.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Face {
    /// Large 5×7 block digits (`█`, `▮`).
    Block,
    /// Compact 3×3 plain-ASCII digits (`_`, `|`, `.`).
    Compact,
}

impl Face {
    pub fn glyph_h(self) -> usize {
        match self {
            Face::Block => 7,
            Face::Compact => 3,
        }
    }

    pub fn glyph_w(self) -> usize {
        match self {
            Face::Block => 5,
            Face::Compact => 3,
        }
    }
}

/// How a face is rasterized: which typeface, horizontal pixel doubling
/// (terminal cells are ~2x taller than wide) and inter-glyph gap.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Style {
    pub face: Face,
    pub scale_x: usize,
    pub gap_x: usize,
}

/// One character in the large block font.
/// `colon_visible` controls colon blinking.
pub fn block_glyph(ch: char, colon_visible: bool) -> [&'static str; 7] {
    match ch {
        '0' => [
            " ███ ",
            "█   █",
            "█   █",
            "█   █",
            "█   █",
            "█   █",
            " ███ ",
        ],
        '1' => [
            "  █  ",
            " ██  ",
            "  █  ",
            "  █  ",
            "  █  ",
            "  █  ",
            "█████",
        ],
        '2' => [
            "█████",
            "    █",
            "    █",
            "█████",
            "█    ",
            "█    ",
            "█████",
        ],
        '3' => [
            "█████",
            "    █",
            "    █",
            " ████",
            "    █",
            "    █",
            "█████",
        ],
        '4' => [
            "█   █",
            "█   █",
            "█   █",
            "█████",
            "    █",
            "    █",
            "    █",
        ],
        '5' => [
            "█████",
            "█    ",
            "█    ",
            "█████",
            "    █",
            "    █",
            "█████",
        ],
        '6' => [
            "█████",
            "█    ",
            "█    ",
            "█████",
            "█   █",
            "█   █",
            "█████",
        ],
        '7' => [
            "█████",
            "    █",
            "    █",
            "   █ ",
            "  █  ",
            "  █  ",
            "  █  ",
        ],
        '8' => [
            " ███ ",
            "█   █",
            "█   █",
            " ███ ",
            "█   █",
            "█   █",
            " ███ ",
        ],
        '9' => [
            "█████",
            "█   █",
            "█   █",
            "█████",
            "    █",
            "    █",
            "█████",
        ],
        ':' if colon_visible => [
            "     ",
            "     ",
            "  ▮  ",
            "     ",
            "     ",
            "  ▮  ",
            "     ",
        ],
        _ => [
            "     ",
            "     ",
            "     ",
            "     ",
            "     ",
            "     ",
            "     ",
        ],
    }
}

/// One character in the compact plain-ASCII font.
/// Only `_`, `|` and `.` are used, so it renders in any font.
/// `colon_visible` controls colon blinking.
pub fn compact_glyph(ch: char, colon_visible: bool) -> [&'static str; 3] {
    match ch {
        '0' => [" _ ", "| |", "|_|"],
        '1' => ["   ", "  |", "  |"],
        '2' => [" _ ", " _|", "|_ "],
        '3' => [" _ ", " _|", " _|"],
        '4' => ["   ", "|_|", "  |"],
        '5' => [" _ ", "|_ ", " _|"],
        '6' => [" _ ", "|_ ", "|_|"],
        '7' => [" _ ", "  |", "  |"],
        '8' => [" _ ", "|_|", "|_|"],
        '9' => [" _ ", "|_|", " _|"],
        ':' if colon_visible => [" . ", "   ", " . "],
        _ => ["   ", "   ", "   "],
    }
}

fn glyph_rows(face: Face, ch: char, colon_visible: bool) -> Vec<&'static str> {
    match face {
        Face::Block => block_glyph(ch, colon_visible).to_vec(),
        Face::Compact => compact_glyph(ch, colon_visible).to_vec(),
    }
}

/// Time string → glyph rows. Each glyph column is repeated `scale_x`
/// times; glyphs are separated by `gap_x` spaces.
pub fn render_big(time: &str, colon_visible: bool, style: Style) -> Vec<String> {
    let chars: Vec<char> = time.chars().collect();
    let width = line_width(chars.len(), style);
    let mut rows = vec![String::new(); style.face.glyph_h()];
    for r in &mut rows {
        r.reserve(width);
    }
    for (i, ch) in chars.iter().enumerate() {
        let g = glyph_rows(style.face, *ch, colon_visible);
        for (r, glyph_row) in g.iter().enumerate() {
            for c in glyph_row.chars() {
                for _ in 0..style.scale_x {
                    rows[r].push(c);
                }
            }
            if i + 1 < chars.len() {
                for _ in 0..style.gap_x {
                    rows[r].push(' ');
                }
            }
        }
    }
    rows
}

/// Width of a rendered line of `n_chars` characters in `style`.
pub fn line_width(n_chars: usize, style: Style) -> usize {
    let rendered_w = style.face.glyph_w() * style.scale_x;
    n_chars * rendered_w + n_chars.saturating_sub(1) * style.gap_x
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLOCK_STYLE: Style = Style {
        face: Face::Block,
        scale_x: 2,
        gap_x: 2,
    };
    const COMPACT_STYLE: Style = Style {
        face: Face::Compact,
        scale_x: 1,
        gap_x: 1,
    };

    fn all_glyph_inputs() -> Vec<char> {
        ('0'..='9').chain([':', ' ', 'X']).collect()
    }

    #[test]
    fn every_block_glyph_row_is_glyph_w_wide() {
        for ch in all_glyph_inputs() {
            for visible in [true, false] {
                for row in block_glyph(ch, visible) {
                    assert_eq!(
                        row.chars().count(),
                        Face::Block.glyph_w(),
                        "glyph {ch:?} visible={visible} row {row:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn every_compact_glyph_row_is_glyph_w_wide() {
        for ch in all_glyph_inputs() {
            for visible in [true, false] {
                for row in compact_glyph(ch, visible) {
                    assert_eq!(
                        row.chars().count(),
                        Face::Compact.glyph_w(),
                        "glyph {ch:?} visible={visible} row {row:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn compact_glyphs_are_plain_ascii() {
        for ch in all_glyph_inputs() {
            for visible in [true, false] {
                for row in compact_glyph(ch, visible) {
                    assert!(
                        row.is_ascii(),
                        "glyph {ch:?} row {row:?} is not ASCII"
                    );
                }
            }
        }
    }

    #[test]
    fn rendered_width_matches_line_width() {
        for style in [BLOCK_STYLE, COMPACT_STYLE] {
            for time in ["", "1", "12:34:56"] {
                for visible in [true, false] {
                    let rows = render_big(time, visible, style);
                    assert_eq!(rows.len(), style.face.glyph_h());
                    for r in &rows {
                        assert_eq!(
                            r.chars().count(),
                            line_width(time.chars().count(), style),
                            "time {time:?} style {style:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn hidden_colon_keeps_layout_stable() {
        for style in [BLOCK_STYLE, COMPACT_STYLE] {
            let shown = render_big("12:34:56", true, style);
            let hidden = render_big("12:34:56", false, style);
            assert_eq!(shown.len(), hidden.len());
            for (a, b) in shown.iter().zip(&hidden) {
                assert_eq!(a.chars().count(), b.chars().count());
            }
        }
        // Hidden colon rows are blank where the dots were.
        assert!(render_big(":", false, BLOCK_STYLE)[2].trim().is_empty());
    }

    #[test]
    fn line_width_edge_cases() {
        assert_eq!(line_width(0, BLOCK_STYLE), 0);
        assert_eq!(line_width(1, BLOCK_STYLE), 2 * Face::Block.glyph_w());
        assert_eq!(
            line_width(8, BLOCK_STYLE),
            8 * Face::Block.glyph_w() * 2 + 7 * 2
        );
        assert_eq!(line_width(8, COMPACT_STYLE), 8 * 3 + 7);
    }
}
