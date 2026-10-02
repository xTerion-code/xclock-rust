pub const GLYPH_H: usize = 7;
pub const GLYPH_W: usize = 5;

/// Horizontal scale: terminal cells are ~2x taller than wide,
/// so each glyph column is doubled to keep digits proportionate.
pub const SCALE_X: usize = 2;
/// Gap between glyphs after scaling.
pub const GAP_X: usize = 2;

/// Rendered (scaled) glyph width in terminal columns.
pub const RENDERED_W: usize = GLYPH_W * SCALE_X;

/// One character in the large block font.
/// `colon_visible` controls colon blinking.
pub fn glyph(ch: char, colon_visible: bool) -> [&'static str; GLYPH_H] {
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

/// Time string → glyph rows. Each glyph column is repeated `SCALE_X`
/// times; glyphs are separated by `GAP_X` spaces.
pub fn render_big(time: &str, colon_visible: bool) -> Vec<String> {
    let chars: Vec<char> = time.chars().collect();
    let width = line_width(chars.len());
    let mut rows = vec![String::new(); GLYPH_H];
    for r in &mut rows {
        r.reserve(width);
    }
    for (i, ch) in chars.iter().enumerate() {
        let g = glyph(*ch, colon_visible);
        for r in 0..GLYPH_H {
            for c in g[r].chars() {
                for _ in 0..SCALE_X {
                    rows[r].push(c);
                }
            }
            if i + 1 < chars.len() {
                for _ in 0..GAP_X {
                    rows[r].push(' ');
                }
            }
        }
    }
    rows
}

/// Width of a rendered line of `n_chars` characters.
pub fn line_width(n_chars: usize) -> usize {
    n_chars * RENDERED_W + n_chars.saturating_sub(1) * GAP_X
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_glyph_inputs() -> Vec<char> {
        ('0'..='9').chain([':', ' ', 'X']).collect()
    }

    #[test]
    fn every_glyph_row_is_glyph_w_wide() {
        for ch in all_glyph_inputs() {
            for visible in [true, false] {
                for row in glyph(ch, visible) {
                    assert_eq!(
                        row.chars().count(),
                        GLYPH_W,
                        "glyph {ch:?} visible={visible} row {row:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn rendered_width_matches_line_width() {
        for time in ["", "1", "12:34:56"] {
            for visible in [true, false] {
                let rows = render_big(time, visible);
                assert_eq!(rows.len(), GLYPH_H);
                for r in &rows {
                    assert_eq!(
                        r.chars().count(),
                        line_width(time.chars().count()),
                        "time {time:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn hidden_colon_keeps_layout_stable() {
        let shown = render_big("12:34:56", true);
        let hidden = render_big("12:34:56", false);
        assert_eq!(shown.len(), hidden.len());
        for (a, b) in shown.iter().zip(&hidden) {
            assert_eq!(a.chars().count(), b.chars().count());
        }
        // Hidden colon rows are blank where the dots were.
        assert!(hidden[2].contains("  "));
    }

    #[test]
    fn line_width_edge_cases() {
        assert_eq!(line_width(0), 0);
        assert_eq!(line_width(1), RENDERED_W);
        assert_eq!(line_width(8), 8 * RENDERED_W + 7 * GAP_X);
    }
}
