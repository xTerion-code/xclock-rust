#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Face {
    Block,
    Compact,
}

impl Face {
    pub fn glyph_h(self) -> usize {
        match self {
            Face::Block => 7,
            Face::Compact => 5,
        }
    }

    pub fn glyph_w(self) -> usize {
        match self {
            Face::Block => 5,
            Face::Compact => 3,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Style {
    pub face: Face,
    pub scale_x: usize,
    pub gap_x: usize,
}

pub fn line_width(n_chars: usize, style: Style) -> usize {
    let rendered_w = style.face.glyph_w() * style.scale_x;
    n_chars * rendered_w + n_chars.saturating_sub(1) * style.gap_x
}

pub fn render_big(time: &str, colon_visible: bool, style: Style) -> Vec<String> {
    let chars: Vec<char> = time.chars().collect();
    let width = line_width(chars.len(), style);
    let mut rows = vec![String::new(); style.face.glyph_h()];
    for r in &mut rows {
        r.reserve(width * 4);
    }
    for (i, ch) in chars.iter().enumerate() {
        let (b_g, c_g);
        let glyph_rows: &[&str] = match style.face {
            Face::Block => {
                b_g = block_glyph(*ch, colon_visible);
                &b_g
            }
            Face::Compact => {
                c_g = compact_glyph(*ch, colon_visible);
                &c_g
            }
        };
        for (r, glyph_row) in glyph_rows.iter().enumerate() {
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

fn block_glyph(ch: char, colon_visible: bool) -> [&'static str; 7] {
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
            "     ", "     ", "  ▮  ", "     ", "     ", "  ▮  ", "     ",
        ],
        _ => [
            "     ", "     ", "     ", "     ", "     ", "     ", "     ",
        ],
    }
}

fn compact_glyph(ch: char, colon_visible: bool) -> [&'static str; 5] {
    match ch {
        '0' => [
            "███",
            "█ █",
            "█ █",
            "█ █",
            "███",
        ],
        '1' => [
            " █ ",
            "██ ",
            " █ ",
            " █ ",
            "███",
        ],
        '2' => [
            "███",
            "  █",
            "███",
            "█  ",
            "███",
        ],
        '3' => [
            "███",
            "  █",
            "███",
            "  █",
            "███",
        ],
        '4' => [
            "█ █",
            "█ █",
            "███",
            "  █",
            "  █",
        ],
        '5' => [
            "███",
            "█  ",
            "███",
            "  █",
            "███",
        ],
        '6' => [
            "███",
            "█  ",
            "███",
            "█ █",
            "███",
        ],
        '7' => [
            "███",
            "  █",
            " █ ",
            " █ ",
            " █ ",
        ],
        '8' => [
            "███",
            "█ █",
            "███",
            "█ █",
            "███",
        ],
        '9' => [
            "███",
            "█ █",
            "███",
            "  █",
            "███",
        ],
        ':' if colon_visible => [
            "   ",
            " ▮ ",
            "   ",
            " ▮ ",
            "   ",
        ],
        _ => [
            "   ",
            "   ",
            "   ",
            "   ",
            "   ",
        ],
    }
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
            assert!(render_big(":", false, style)[1].trim().is_empty());
        }
    }

    #[test]
    fn line_width_edge_cases() {
        assert_eq!(line_width(0, BLOCK_STYLE), 0);
        assert_eq!(line_width(1, BLOCK_STYLE), 2 * Face::Block.glyph_w());
        assert_eq!(
            line_width(8, BLOCK_STYLE),
            8 * Face::Block.glyph_w() * 2 + 7 * 2
        );
    }
}
