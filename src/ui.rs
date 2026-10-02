use std::io::{self, Write};

use crossterm::{
    queue,
    style::{Attribute, ResetColor, SetAttribute, SetForegroundColor},
};

use crate::font::Style;
use crate::theme::Theme;

/// Live-key hint shown inside the frame (fits even the compact width).
pub const HINT: &str = "1/2/3 theme · q quit";

/// Geometry of the centered block (frame + clock + date + hint).
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

// Note: `\r\n` endings everywhere — the app runs in raw mode,
// where a bare `\n` does not return the carriage.

fn write_top(stdout: &mut impl Write, theme: Theme, pad_x: usize, inner_w: usize) -> io::Result<()> {
    let pad = " ".repeat(pad_x);
    queue!(stdout, SetForegroundColor(theme.border()))?;
    write!(stdout, "{pad}╭{}╮\r\n", "─".repeat(inner_w))?;
    queue!(stdout, ResetColor)?;
    Ok(())
}

fn write_bottom(
    stdout: &mut impl Write,
    theme: Theme,
    pad_x: usize,
    inner_w: usize,
) -> io::Result<()> {
    let pad = " ".repeat(pad_x);
    queue!(stdout, SetForegroundColor(theme.border()))?;
    write!(stdout, "{pad}╰{}╯\r\n", "─".repeat(inner_w))?;
    queue!(stdout, ResetColor)?;
    Ok(())
}

fn write_empty(
    stdout: &mut impl Write,
    theme: Theme,
    pad_x: usize,
    inner_w: usize,
) -> io::Result<()> {
    let pad = " ".repeat(pad_x);
    queue!(stdout, SetForegroundColor(theme.border()))?;
    write!(stdout, "{pad}│")?;
    queue!(stdout, ResetColor)?;
    write!(stdout, "{}", " ".repeat(inner_w))?;
    queue!(stdout, SetForegroundColor(theme.border()))?;
    write!(stdout, "│\r\n")?;
    queue!(stdout, ResetColor)?;
    Ok(())
}

fn write_clock_rows(
    stdout: &mut impl Write,
    layout: &Layout,
    rows: &[String],
    time_chars: &[char],
    theme: Theme,
    style: Style,
) -> io::Result<()> {
    let row_chars: Vec<Vec<char>> = rows
        .iter()
        .map(|r| r.chars().collect())
        .collect();
    let rendered_w = style.face.glyph_w() * style.scale_x;

    queue!(stdout, SetAttribute(Attribute::Bold))?;
    for row in &row_chars {
        let pad = " ".repeat(layout.pad_x);
        queue!(stdout, SetForegroundColor(theme.border()))?;
        write!(stdout, "{pad}│{}", " ".repeat(layout.clock_offset))?;

        // Cells used inside the frame so far; stop at `inner_w`
        // instead of wrapping on narrow terminals.
        let mut used = layout.clock_offset;
        let mut col = 0usize;
        for (i, _ch) in time_chars.iter().enumerate() {
            if used >= layout.inner_w {
                break;
            }
            queue!(stdout, SetForegroundColor(theme.time_color(i)))?;
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
                queue!(stdout, SetForegroundColor(theme.separator()))?;
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
        queue!(stdout, SetForegroundColor(theme.border()))?;
        write!(stdout, "{}│\r\n", " ".repeat(right))?;
        queue!(stdout, ResetColor)?;
    }
    queue!(stdout, SetAttribute(Attribute::Reset))?;
    Ok(())
}

/// Centered single line in `color`, truncated (never wrapped)
/// on narrow terminals.
fn write_centered(
    stdout: &mut impl Write,
    layout: &Layout,
    text: &str,
    theme: Theme,
    color: crossterm::style::Color,
) -> io::Result<()> {
    let pad = " ".repeat(layout.pad_x);
    let text_w = text.chars().count();
    let offset = layout.inner_w.saturating_sub(text_w) / 2;
    queue!(stdout, SetForegroundColor(theme.border()))?;
    write!(stdout, "{pad}│{}", " ".repeat(offset))?;
    queue!(stdout, SetForegroundColor(color))?;
    let avail = layout.inner_w.saturating_sub(offset);
    let shown: String = text.chars().take(avail).collect();
    let shown_w = shown.chars().count();
    write!(stdout, "{shown}")?;
    let right = layout.inner_w.saturating_sub(offset + shown_w);
    queue!(stdout, SetForegroundColor(theme.border()))?;
    write!(stdout, "{}│\r\n", " ".repeat(right))?;
    queue!(stdout, ResetColor)?;
    Ok(())
}

/// Full frame: vertical offset + frame with clock, date and key hint.
/// Callers flush once after `render` for a tear-free frame.
pub fn render(
    stdout: &mut impl Write,
    layout: &Layout,
    rows: &[String],
    time_chars: &[char],
    date_line: &str,
    theme: Theme,
) -> io::Result<()> {
    let style = theme.style();
    for _ in 0..layout.pad_y {
        write!(stdout, "\r\n")?;
    }

    write_top(stdout, theme, layout.pad_x, layout.inner_w)?;
    write_empty(stdout, theme, layout.pad_x, layout.inner_w)?;
    write_clock_rows(stdout, layout, rows, time_chars, theme, style)?;
    write_empty(stdout, theme, layout.pad_x, layout.inner_w)?;
    write_centered(stdout, layout, date_line, theme, theme.date())?;
    write_centered(stdout, layout, HINT, theme, theme.separator())?;
    write_empty(stdout, theme, layout.pad_x, layout.inner_w)?;
    write_bottom(stdout, theme, layout.pad_x, layout.inner_w)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn modern() -> Theme {
        Theme::Modern
    }

    #[test]
    fn centers_block_on_roomy_screen() {
        // clock 20 + date 10 -> inner 28, box 30, 100x40 screen.
        let l = compute_layout(100, 40, 20, 10, modern());
        assert_eq!(l.inner_w, 28);
        assert_eq!(l.pad_x, (100 - 30) / 2);
        assert_eq!(
            l.pad_y,
            (40 - (modern().style().face.glyph_h() + 7)) / 2
        );
        assert_eq!(l.clock_offset, (28 - 20) / 2);
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
            block_height(Theme::Compact),
            Theme::Compact.style().face.glyph_h() + 7
        );
        assert!(block_height(Theme::Compact) < block_height(Theme::Modern));
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
            );
            let mut buf: Vec<u8> = Vec::new();
            render(&mut buf, &l, &rows, &time_chars, date, theme).unwrap();
            let out = String::from_utf8(buf).unwrap();
            assert!(out.contains('╭') && out.contains('╯'), "frame missing");
            assert!(out.contains(date), "date missing");
            assert!(out.contains(HINT), "hint missing");
            assert!(out.contains(&"─".repeat(l.inner_w)), "top width wrong");
        }
    }

    #[test]
    fn render_narrow_never_panics_or_wraps_wide() {
        let style = modern().style();
        let rows = crate::font::render_big("12:34:56", true, style);
        let time_chars: Vec<char> = "12:34:56".chars().collect();
        let l = compute_layout(
            30,
            24,
            crate::font::line_width(8, style),
            27,
            modern(),
        );
        let mut buf: Vec<u8> = Vec::new();
        render(
            &mut buf,
            &l,
            &rows,
            &time_chars,
            "September 30, 2026 · Wednesday",
            modern(),
        )
        .unwrap();
        assert!(String::from_utf8(buf).is_ok());
    }
}
