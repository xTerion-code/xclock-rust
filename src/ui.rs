use std::io::{self, Write};

use crossterm::{
    queue,
    style::{Attribute, ResetColor, SetAttribute, SetForegroundColor},
};

use crate::font::Style;
use crate::theme::{Palette, Theme};

pub const HINT: &str = "1/2/3/a theme · p palette · s seconds · h 12/24h · b blink · d date · q quit";

pub struct Layout {
    pub pad_x: usize,
    pub pad_y: usize,
    pub inner_w: usize,
    pub clock_offset: usize,
}

const H_PAD_INSIDE: usize = 4;

fn block_height(theme: Theme, show_date: bool) -> usize {
    theme.block_height(show_date)
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

// Note: `\r\n` endings everywhere — the app runs in raw mode,
// where a bare `\n` does not return the carriage.

fn write_top(
    stdout: &mut impl Write,
    palette: Palette,
    pad_x: usize,
    inner_w: usize,
) -> io::Result<()> {
    let pad = " ".repeat(pad_x);
    queue!(stdout, SetForegroundColor(palette.border()))?;
    write!(stdout, "{pad}╭{}╮\r\n", "─".repeat(inner_w))?;
    queue!(stdout, ResetColor)?;
    Ok(())
}

fn write_bottom(
    stdout: &mut impl Write,
    palette: Palette,
    pad_x: usize,
    inner_w: usize,
) -> io::Result<()> {
    let pad = " ".repeat(pad_x);
    queue!(stdout, SetForegroundColor(palette.border()))?;
    write!(stdout, "{pad}╰{}╯\r\n", "─".repeat(inner_w))?;
    queue!(stdout, ResetColor)?;
    Ok(())
}

fn write_empty(
    stdout: &mut impl Write,
    palette: Palette,
    pad_x: usize,
    inner_w: usize,
) -> io::Result<()> {
    let pad = " ".repeat(pad_x);
    queue!(stdout, SetForegroundColor(palette.border()))?;
    write!(stdout, "{pad}│")?;
    queue!(stdout, ResetColor)?;
    write!(stdout, "{}", " ".repeat(inner_w))?;
    queue!(stdout, SetForegroundColor(palette.border()))?;
    write!(stdout, "│\r\n")?;
    queue!(stdout, ResetColor)?;
    Ok(())
}

fn write_clock_rows(
    stdout: &mut impl Write,
    layout: &Layout,
    rows: &[String],
    time_chars: &[char],
    palette: Palette,
    style: Style,
) -> io::Result<()> {
    let row_chars: Vec<Vec<char>> = rows.iter().map(|r| r.chars().collect()).collect();
    let rendered_w = style.face.glyph_w() * style.scale_x;

    queue!(stdout, SetAttribute(Attribute::Bold))?;
    for row in &row_chars {
        let pad = " ".repeat(layout.pad_x);
        queue!(stdout, SetForegroundColor(palette.border()))?;
        write!(stdout, "{pad}│{}", " ".repeat(layout.clock_offset))?;

        // Cells used inside the frame so far; stop at `inner_w`
        // instead of wrapping on narrow terminals.
        let mut used = layout.clock_offset;
        let mut col = 0usize;
        for (i, _ch) in time_chars.iter().enumerate() {
            if used >= layout.inner_w {
                break;
            }
            queue!(stdout, SetForegroundColor(palette.time_color(i)))?;
            for k in 0..rendered_w {
                if used >= layout.inner_w {
                    break;
                }
                let c = row.get(col + k).copied().unwrap_or(' ');
                write!(stdout, "{c}")?;
                used += 1;
            }
            col += rendered_w;
            if i + 1 < time_chars.len() && used < layout.inner_w {
                queue!(stdout, SetForegroundColor(palette.separator()))?;
                for _ in 0..style.gap_x {
                    if used >= layout.inner_w {
                        break;
                    }
                    let c = row.get(col).copied().unwrap_or(' ');
                    write!(stdout, "{c}")?;
                    col += 1;
                    used += 1;
                }
            }
        }

        let right = layout.inner_w.saturating_sub(used);
        queue!(stdout, SetForegroundColor(palette.border()))?;
        write!(stdout, "{}│\r\n", " ".repeat(right))?;
        queue!(stdout, ResetColor)?;
    }
    queue!(stdout, SetAttribute(Attribute::Reset))?;
    Ok(())
}

fn write_centered(
    stdout: &mut impl Write,
    layout: &Layout,
    text: &str,
    palette: Palette,
    color: crossterm::style::Color,
) -> io::Result<()> {
    let pad = " ".repeat(layout.pad_x);
    let text_w = text.chars().count();
    let offset = layout.inner_w.saturating_sub(text_w) / 2;
    queue!(stdout, SetForegroundColor(palette.border()))?;
    write!(stdout, "{pad}│{}", " ".repeat(offset))?;
    queue!(stdout, SetForegroundColor(color))?;
    let avail = layout.inner_w.saturating_sub(offset);
    let shown: String = text.chars().take(avail).collect();
    let shown_w = shown.chars().count();
    write!(stdout, "{shown}")?;
    let right = layout.inner_w.saturating_sub(offset + shown_w);
    queue!(stdout, SetForegroundColor(palette.border()))?;
    write!(stdout, "{}│\r\n", " ".repeat(right))?;
    queue!(stdout, ResetColor)?;
    Ok(())
}

pub fn render(
    stdout: &mut impl Write,
    layout: &Layout,
    rows: &[String],
    time_chars: &[char],
    date_line: Option<&str>,
    theme: Theme,
    palette: Palette,
) -> io::Result<()> {
    let style = theme.style();
    for _ in 0..layout.pad_y {
        write!(stdout, "\r\n")?;
    }

    write_top(stdout, palette, layout.pad_x, layout.inner_w)?;
    write_empty(stdout, palette, layout.pad_x, layout.inner_w)?;
    write_clock_rows(stdout, layout, rows, time_chars, palette, style)?;
    write_empty(stdout, palette, layout.pad_x, layout.inner_w)?;
    if let Some(date) = date_line {
        write_centered(stdout, layout, date, palette, palette.date())?;
    }
    write_centered(stdout, layout, HINT, palette, palette.separator())?;
    write_empty(stdout, palette, layout.pad_x, layout.inner_w)?;
    write_bottom(stdout, palette, layout.pad_x, layout.inner_w)?;
    Ok(())
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

    #[test]
    fn render_smoke_frame_date_and_hint() {
        for theme in Theme::all() {
            let style = theme.style();
            let rows = crate::font::render_big("12:34:56", true, style);
            let time_chars: Vec<char> = "12:34:56".chars().collect();
            let date = "September 30, 2026 · Wednesday";
            let l = compute_layout(
                140,
                40,
                crate::font::line_width(time_chars.len(), style),
                date.chars().count(),
                theme,
                true,
            );
            let mut buf: Vec<u8> = Vec::new();
            render(&mut buf, &l, &rows, &time_chars, Some(date), theme, Palette::Purple).unwrap();
            let out = String::from_utf8(buf).unwrap();
            assert!(out.contains('╭') && out.contains('╯'), "frame missing");
            assert!(out.contains(date), "date missing");
            assert!(out.contains(HINT), "hint missing");
            assert!(out.contains(&"─".repeat(l.inner_w)), "top width wrong");
        }
    }

    #[test]
    fn render_without_date_omits_date_row() {
        let style = large().style();
        let rows = crate::font::render_big("12:34:56", true, style);
        let time_chars: Vec<char> = "12:34:56".chars().collect();
        let date = "September 30, 2026 · Wednesday";
        let l = compute_layout(
            140,
            40,
            crate::font::line_width(time_chars.len(), style),
            date.chars().count(),
            large(),
            false,
        );
        let mut buf: Vec<u8> = Vec::new();
        render(&mut buf, &l, &rows, &time_chars, None, large(), Palette::Purple).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(!out.contains(date), "date should be hidden");
        assert!(out.contains(HINT), "hint missing");
    }

    #[test]
    fn render_narrow_never_panics_or_wraps_wide() {
        for theme in Theme::all() {
            let style = theme.style();
            let rows = crate::font::render_big("12:34:56", true, style);
            let time_chars: Vec<char> = "12:34:56".chars().collect();
            let l = compute_layout(
                30,
                24,
                crate::font::line_width(8, style),
                27,
                theme,
                true,
            );
            let mut buf: Vec<u8> = Vec::new();
            render(
                &mut buf,
                &l,
                &rows,
                &time_chars,
                Some("September 30, 2026 · Wednesday"),
                theme,
                Palette::Purple,
            )
            .unwrap();
            assert!(String::from_utf8(buf).is_ok());
        }
    }

    #[test]
    fn render_smoke_all_palettes() {
        let theme = Theme::Large;
        let style = theme.style();
        let rows = crate::font::render_big("12:34:56", true, style);
        let time_chars: Vec<char> = "12:34:56".chars().collect();
        let date = "September 30, 2026 · Wednesday";
        let l = compute_layout(
            140,
            40,
            crate::font::line_width(time_chars.len(), style),
            date.chars().count(),
            theme,
            true,
        );
        for palette in Palette::all() {
            let mut buf: Vec<u8> = Vec::new();
            render(&mut buf, &l, &rows, &time_chars, Some(date), theme, palette).unwrap();
            let out = String::from_utf8(buf).unwrap();
            assert!(out.contains('╭') && out.contains('╯'));
            assert!(out.contains(date));
            assert!(out.contains(HINT));
        }
    }
}
