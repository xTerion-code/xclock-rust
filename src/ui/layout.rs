use crate::theme::Theme;

pub const HINT: &str = "1/2/3 theme · s seconds · h 12/24h · b blink · d date · q quit";

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

    fn large() -> Theme {
        Theme::Large
    }

    #[test]
    fn centers_block_on_roomy_screen() {
        let hint_w = HINT.chars().count();
        let expected_inner = 20.max(hint_w) + 8;
        let l = compute_layout(100, 40, 20, 10, large(), true);
        assert_eq!(l.inner_w, expected_inner);
        assert_eq!(l.pad_x, (100 - (expected_inner + 2)) / 2);
        assert_eq!(l.pad_y, (40 - (large().style().face.glyph_h() + 7)) / 2);
        assert_eq!(l.clock_offset, (expected_inner - 20) / 2);
    }

    #[test]
    fn clamps_to_narrow_screen() {
        let l = compute_layout(20, 24, 94, 27, large(), true);
        assert_eq!(l.inner_w, 18);
        assert_eq!(l.pad_x, 0);
        assert_eq!(l.clock_offset, 0);
    }

    #[test]
    fn hidden_date_ignores_date_width_and_shrinks_height() {
        let shown = compute_layout(100, 41, 20, 80, large(), true);
        let hidden = compute_layout(100, 41, 20, 80, large(), false);
        assert!(hidden.inner_w < shown.inner_w);
        assert_eq!(hidden.pad_y, shown.pad_y + 1);
    }

    #[test]
    fn block_height_follows_face() {
        assert_eq!(
            block_height(Theme::Large, true),
            Theme::Large.style().face.glyph_h() + 7
        );
        assert_eq!(
            block_height(Theme::Medium, true),
            Theme::Medium.style().face.glyph_h() + 7
        );
        assert_eq!(
            block_height(Theme::Small, true),
            Theme::Small.style().face.glyph_h() + 7
        );
        assert_eq!(
            block_height(Theme::Large, true),
            block_height(Theme::Medium, true)
        );
        assert!(
            block_height(Theme::Small, true) < block_height(Theme::Medium, true)
        );
        assert_eq!(
            block_height(Theme::Large, false),
            Theme::Large.style().face.glyph_h() + 6
        );
        assert_eq!(
            block_height(Theme::Small, false),
            Theme::Small.style().face.glyph_h() + 6
        );
    }

    #[test]
    fn layout_fits_respective_target_terminals() {
        let small_w = crate::font::line_width(8, Theme::Small.style());
        let med_w = crate::font::line_width(8, Theme::Medium.style());
        let large_w = crate::font::line_width(8, Theme::Large.style());

        // Small theme fits on a compact terminal (45x14)
        let l_small = compute_layout(45, 14, small_w, 25, Theme::Small, true);
        assert!(small_w <= l_small.inner_w);
        assert!(block_height(Theme::Small, true) <= 14);

        // Medium theme fits comfortably on a standard 80x24 terminal
        let l_med = compute_layout(80, 24, med_w, 31, Theme::Medium, true);
        assert!(med_w <= l_med.inner_w);
        assert!(l_med.inner_w + 2 <= 80);
        assert!(block_height(Theme::Medium, true) <= 24);

        // Large theme fits on a wide terminal (120x30)
        let l_large = compute_layout(120, 30, large_w, 31, Theme::Large, true);
        assert!(large_w <= l_large.inner_w);
        assert!(l_large.inner_w + 2 <= 120);
        assert!(block_height(Theme::Large, true) <= 30);
    }
}
