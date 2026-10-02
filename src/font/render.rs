use super::{Style, block_glyph};

pub fn render_big(time: &str, colon_visible: bool, style: Style) -> Vec<String> {
    let chars: Vec<char> = time.chars().collect();
    let width = line_width(chars.len(), style);
    let mut rows = vec![String::new(); style.face.glyph_h()];
    for r in &mut rows {
        r.reserve(width);
    }
    for (i, ch) in chars.iter().enumerate() {
        let g = block_glyph(*ch, colon_visible);
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

pub fn line_width(n_chars: usize, style: Style) -> usize {
    let rendered_w = style.face.glyph_w() * style.scale_x;
    n_chars * rendered_w + n_chars.saturating_sub(1) * style.gap_x
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::Face;

    const BLOCK_STYLE: Style = Style {
        face: Face::Block,
        scale_x: 2,
        gap_x: 2,
    };

    #[test]
    fn rendered_width_matches_line_width() {
        let style = BLOCK_STYLE;
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

    #[test]
    fn hidden_colon_keeps_layout_stable() {
        let style = BLOCK_STYLE;
        {
            let shown = render_big("12:34:56", true, style);
            let hidden = render_big("12:34:56", false, style);
            assert_eq!(shown.len(), hidden.len());
            for (a, b) in shown.iter().zip(&hidden) {
                assert_eq!(a.chars().count(), b.chars().count());
            }
        }
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
    }
}
