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

/// Which optional parts of the framed face are currently visible.
///
/// Geometry calculations take this single description instead of loose
/// booleans, so the width budget, the height budget and the render pass can
/// never disagree about what the frame is supposed to contain.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "three independent visibility flags"
)]
pub struct FaceLayout {
    pub seconds: bool,
    pub date: bool,
    pub hint: bool,
}

impl FaceLayout {
    pub const fn new(seconds: bool, date: bool, hint: bool) -> Self {
        Self {
            seconds,
            date,
            hint,
        }
    }

    /// Characters in the rendered time string: `HH:MM:SS` or `HH:MM`.
    pub const fn clock_chars(self) -> usize {
        if self.seconds { 8 } else { 5 }
    }

    /// Total frame height in terminal rows: top border, one blank row, the
    /// digit face, one blank row, the optional date row, the optional hint
    /// row, one blank row, bottom border.
    pub const fn rows(self, face_height: usize) -> usize {
        face_height + 5 + self.date as usize + self.hint as usize
    }
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
        let glyph_rows = glyph_rows(*ch, colon_visible, style);
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

/// Bitmap rows for one character, or a blank glyph for anything the face does
/// not define (spaces, and the colon while it is blinked off).
fn glyph_rows(ch: char, colon_visible: bool, style: Style) -> &'static [&'static str] {
    match style.face {
        Face::Block => block_glyph(ch, colon_visible),
        Face::Compact => compact_glyph(ch, colon_visible),
    }
}

fn block_glyph(ch: char, colon_visible: bool) -> &'static [&'static str] {
    match ch {
        '0' => &[
            " ███ ",
            "█   █",
            "█   █",
            "█   █",
            "█   █",
            "█   █",
            " ███ ",
        ],
        '1' => &[
            "  █  ",
            " ██  ",
            "  █  ",
            "  █  ",
            "  █  ",
            "  █  ",
            "█████",
        ],
        '2' => &[
            "█████",
            "    █",
            "    █",
            "█████",
            "█    ",
            "█    ",
            "█████",
        ],
        '3' => &[
            "█████",
            "    █",
            "    █",
            " ████",
            "    █",
            "    █",
            "█████",
        ],
        '4' => &[
            "█   █",
            "█   █",
            "█   █",
            "█████",
            "    █",
            "    █",
            "    █",
        ],
        '5' => &[
            "█████",
            "█    ",
            "█    ",
            "█████",
            "    █",
            "    █",
            "█████",
        ],
        '6' => &[
            "█████",
            "█    ",
            "█    ",
            "█████",
            "█   █",
            "█   █",
            "█████",
        ],
        '7' => &[
            "█████",
            "    █",
            "    █",
            "   █ ",
            "  █  ",
            "  █  ",
            "  █  ",
        ],
        '8' => &[
            " ███ ",
            "█   █",
            "█   █",
            " ███ ",
            "█   █",
            "█   █",
            " ███ ",
        ],
        '9' => &[
            "█████",
            "█   █",
            "█   █",
            "█████",
            "    █",
            "    █",
            "█████",
        ],
        ':' if colon_visible => &[
            "     ", "     ", "  ▮  ", "     ", "     ", "  ▮  ", "     ",
        ],
        _ => &[
            "     ", "     ", "     ", "     ", "     ", "     ", "     ",
        ],
    }
}

fn compact_glyph(ch: char, colon_visible: bool) -> &'static [&'static str] {
    match ch {
        '0' => &["███", "█ █", "█ █", "█ █", "███"],
        '1' => &[" █ ", "██ ", " █ ", " █ ", "███"],
        '2' => &["███", "  █", "███", "█  ", "███"],
        '3' => &["███", "  █", "███", "  █", "███"],
        '4' => &["█ █", "█ █", "███", "  █", "  █"],
        '5' => &["███", "█  ", "███", "  █", "███"],
        '6' => &["███", "█  ", "███", "█ █", "███"],
        '7' => &["███", "  █", " █ ", " █ ", " █ "],
        '8' => &["███", "█ █", "███", "█ █", "███"],
        '9' => &["███", "█ █", "███", "  █", "███"],
        ':' if colon_visible => &["   ", " ▮ ", "   ", " ▮ ", "   "],
        _ => &["   ", "   ", "   ", "   ", "   "],
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
    fn glyph_rows_dispatch_on_face() {
        for ch in all_glyph_inputs() {
            assert_eq!(
                glyph_rows(ch, true, BLOCK_STYLE).len(),
                Face::Block.glyph_h()
            );
            assert_eq!(
                glyph_rows(ch, true, COMPACT_STYLE).len(),
                Face::Compact.glyph_h()
            );
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

    #[test]
    fn face_layout_counts_clock_chars_and_rows() {
        let full = FaceLayout::new(true, true, true);
        assert_eq!(full.clock_chars(), 8);
        assert_eq!(full.rows(7), 14);
        let minimal = FaceLayout::new(false, false, false);
        assert_eq!(minimal.clock_chars(), 5);
        assert_eq!(minimal.rows(7), 12);
        // Every optional row contributes exactly one line.
        assert_eq!(FaceLayout::new(true, false, false).rows(7), 12);
        assert_eq!(FaceLayout::new(true, true, false).rows(7), 13);
        assert_eq!(FaceLayout::new(true, false, true).rows(7), 13);
    }
}
