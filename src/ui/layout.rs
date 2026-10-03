use crate::theme::Theme;

pub const HINT: &str = "1/2 theme · s seconds · h 12/24h · b blink · d date · q quit";

pub struct Layout {
    pub pad_x: usize,
    pub pad_y: usize,
    pub inner_w: usize,
    pub clock_offset: usize,
}

const H_PAD_INSIDE: usize = 4;

fn block_height(theme: Theme, show_date: bool) -> usize {
    // frame top + empty + clock + empty + [date] + hint + empty + frame bottom
    theme.style().face.glyph_h() + if show_date { 7 } else { 6 }
}

pub fn compute_layout(
    cols: usize,
    lines: usize,
    clock_w: usize,
    date_w: usize,
    theme: Theme,
    show_date: bool,
) -> Layout {
    let content_w = clock_w.max(if show_date { date_w } else { 0 });
    let inner_w = content_w.max(HINT.chars().count()) + H_PAD_INSIDE * 2;
    // Never wider than the screen: frame borders take 2 columns.
    let inner_w = inner_w.min(cols.saturating_sub(2));
    let box_w = inner_w + 2;
    Layout {
        pad_x: cols.saturating_sub(box_w) / 2,
        pad_y: lines.saturating_sub(block_height(theme, show_date)) / 2,
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
        let l = compute_layout(100, 40, 20, 10, modern(), true);
        assert_eq!(l.inner_w, expected_inner);
        assert_eq!(l.pad_x, (100 - (expected_inner + 2)) / 2);
        assert_eq!(l.pad_y, (40 - (modern().style().face.glyph_h() + 7)) / 2);
        assert_eq!(l.clock_offset, (expected_inner - 20) / 2);
    }

    #[test]
    fn clamps_to_narrow_screen() {
        let l = compute_layout(20, 24, 94, 27, modern(), true);
        assert_eq!(l.inner_w, 18);
        assert_eq!(l.pad_x, 0);
        assert_eq!(l.clock_offset, 0);
    }

    #[test]
    fn hidden_date_ignores_date_width_and_shrinks_height() {
        let shown = compute_layout(100, 41, 20, 80, modern(), true);
        let hidden = compute_layout(100, 41, 20, 80, modern(), false);
        assert!(hidden.inner_w < shown.inner_w);
        assert_eq!(hidden.pad_y, shown.pad_y + 1);
    }

    #[test]
    fn block_height_follows_face() {
        assert_eq!(
            block_height(Theme::Modern, true),
            Theme::Modern.style().face.glyph_h() + 7
        );
        assert_eq!(
            block_height(Theme::Classic, true),
            Theme::Classic.style().face.glyph_h() + 7
        );
        assert_eq!(
            block_height(Theme::Modern, true),
            block_height(Theme::Classic, true)
        );
        assert_eq!(
            block_height(Theme::Modern, false),
            Theme::Modern.style().face.glyph_h() + 6
        );
    }
}
