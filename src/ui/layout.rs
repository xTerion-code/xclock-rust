use crate::theme::Theme;

pub const HINT: &str = "1/2 theme · s seconds · q quit";

pub struct Layout {
    pub pad_x: usize,
    pub pad_y: usize,
    pub inner_w: usize,
    pub clock_offset: usize,
}

const H_PAD_INSIDE: usize = 4;

fn block_height(theme: Theme) -> usize {
    // frame top + empty + clock + empty + date + hint + empty + frame bottom
    theme.style().face.glyph_h() + 7
}

pub fn compute_layout(
    cols: usize,
    lines: usize,
    clock_w: usize,
    date_w: usize,
    theme: Theme,
) -> Layout {
    let inner_w = clock_w.max(date_w).max(HINT.chars().count()) + H_PAD_INSIDE * 2;
    // Never wider than the screen: frame borders take 2 columns.
    let inner_w = inner_w.min(cols.saturating_sub(2));
    let box_w = inner_w + 2;
    Layout {
        pad_x: cols.saturating_sub(box_w) / 2,
        pad_y: lines.saturating_sub(block_height(theme)) / 2,
        inner_w,
        clock_offset: inner_w.saturating_sub(clock_w) / 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn modern() -> Theme {
        Theme::Modern
    }

    #[test]
    fn centers_block_on_roomy_screen() {
        let hint_w = HINT.chars().count();
        let expected_inner = 20.max(hint_w) + 8;
        let l = compute_layout(100, 40, 20, 10, modern());
        assert_eq!(l.inner_w, expected_inner);
        assert_eq!(l.pad_x, (100 - (expected_inner + 2)) / 2);
        assert_eq!(
            l.pad_y,
            (40 - (modern().style().face.glyph_h() + 7)) / 2
        );
        assert_eq!(l.clock_offset, (expected_inner - 20) / 2);
    }

    #[test]
    fn clamps_to_narrow_screen() {
        let l = compute_layout(20, 24, 94, 27, modern());
        assert_eq!(l.inner_w, 18);
        assert_eq!(l.pad_x, 0);
        assert_eq!(l.clock_offset, 0);
    }

    #[test]
    fn block_height_follows_face() {
        assert_eq!(
            block_height(Theme::Modern),
            Theme::Modern.style().face.glyph_h() + 7
        );
        assert_eq!(
            block_height(Theme::Classic),
            Theme::Classic.style().face.glyph_h() + 7
        );
        assert_eq!(block_height(Theme::Modern), block_height(Theme::Classic));
    }
}
