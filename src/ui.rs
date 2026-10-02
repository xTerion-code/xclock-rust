use std::io::{self, Write};

use crossterm::{
    queue,
    style::{Attribute, ResetColor, SetAttribute, SetForegroundColor},
};

use crate::font::{GAP_X, GLYPH_H, RENDERED_W};
use crate::theme;

/// Geometry of the centered block (frame + clock + date).
pub struct Layout {
    pub pad_x: usize,
    pub pad_y: usize,
    pub inner_w: usize,
    pub clock_offset: usize,
    pub date_offset: usize,
}

const H_PAD_INSIDE: usize = 4;

fn block_height() -> usize {
    // frame top + empty + clock + gap + date + empty + frame bottom
    GLYPH_H + 6
}

pub fn compute_layout(
    cols: usize,
    lines: usize,
    clock_w: usize,
    date_w: usize,
) -> Layout {
    let inner_w = clock_w.max(date_w) + H_PAD_INSIDE * 2;
    // Never wider than the screen: frame borders take 2 columns.
    let inner_w = inner_w.min(cols.saturating_sub(2));
    let box_w = inner_w + 2;
    Layout {
        pad_x: cols.saturating_sub(box_w) / 2,
        pad_y: lines.saturating_sub(block_height()) / 2,
        inner_w,
        clock_offset: inner_w.saturating_sub(clock_w) / 2,
        date_offset: inner_w.saturating_sub(date_w) / 2,
    }
}

fn write_top(stdout: &mut impl Write, pad_x: usize, inner_w: usize) -> io::Result<()> {
    let pad = " ".repeat(pad_x);
    queue!(stdout, SetForegroundColor(theme::BORDER))?;
    writeln!(stdout, "{pad}╭{}╮", "─".repeat(inner_w))?;
    queue!(stdout, ResetColor)?;
    Ok(())
}

fn write_bottom(stdout: &mut impl Write, pad_x: usize, inner_w: usize) -> io::Result<()> {
    let pad = " ".repeat(pad_x);
    queue!(stdout, SetForegroundColor(theme::BORDER))?;
    writeln!(stdout, "{pad}╰{}╯", "─".repeat(inner_w))?;
    queue!(stdout, ResetColor)?;
    Ok(())
}

fn write_empty(stdout: &mut impl Write, pad_x: usize, inner_w: usize) -> io::Result<()> {
    let pad = " ".repeat(pad_x);
    queue!(stdout, SetForegroundColor(theme::BORDER))?;
    write!(stdout, "{pad}│")?;
    queue!(stdout, ResetColor)?;
    write!(stdout, "{}", " ".repeat(inner_w))?;
    queue!(stdout, SetForegroundColor(theme::BORDER))?;
    writeln!(stdout, "│")?;
    queue!(stdout, ResetColor)?;
    Ok(())
}

fn write_clock_rows(
    stdout: &mut impl Write,
    layout: &Layout,
    rows: &[String],
    time_chars: &[char],
) -> io::Result<()> {
    let row_chars: Vec<Vec<char>> = rows
        .iter()
        .map(|r| r.chars().collect())
        .collect();

    queue!(stdout, SetAttribute(Attribute::Bold))?;
    for row in &row_chars {
        let pad = " ".repeat(layout.pad_x);
        queue!(stdout, SetForegroundColor(theme::BORDER))?;
        write!(stdout, "{pad}│{}", " ".repeat(layout.clock_offset))?;

        // Cells used inside the frame so far; stop at `inner_w`
        // instead of wrapping on narrow terminals.
        let mut used = layout.clock_offset;
        let mut col = 0usize;
        for (i, _ch) in time_chars.iter().enumerate() {
            if used >= layout.inner_w {
                break;
            }
            queue!(
                stdout,
                SetForegroundColor(theme::time_char_color(i))
            )?;
            for k in 0..RENDERED_W {
                if used >= layout.inner_w {
                    break;
                }
                let c = row.get(col + k).copied().unwrap_or(' ');
                write!(stdout, "{c}")?;
                used += 1;
            }
            col += RENDERED_W;
            if i + 1 < time_chars.len() && used < layout.inner_w {
                queue!(stdout, SetForegroundColor(theme::SEPARATOR))?;
                for _ in 0..GAP_X {
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
        queue!(stdout, SetForegroundColor(theme::BORDER))?;
        writeln!(stdout, "{}│", " ".repeat(right))?;
        queue!(stdout, ResetColor)?;
    }
    queue!(stdout, SetAttribute(Attribute::Reset))?;
    Ok(())
}

fn write_date(
    stdout: &mut impl Write,
    layout: &Layout,
    date_line: &str,
) -> io::Result<()> {
    let pad = " ".repeat(layout.pad_x);
    queue!(stdout, SetForegroundColor(theme::BORDER))?;
    write!(stdout, "{pad}│{}", " ".repeat(layout.date_offset))?;
    queue!(stdout, SetForegroundColor(theme::DATE))?;
    // Truncate rather than wrap on narrow terminals.
    let avail = layout.inner_w.saturating_sub(layout.date_offset);
    let shown: String = date_line.chars().take(avail).collect();
    let shown_w = shown.chars().count();
    write!(stdout, "{shown}")?;
    let right = layout
        .inner_w
        .saturating_sub(layout.date_offset + shown_w);
    queue!(stdout, SetForegroundColor(theme::BORDER))?;
    writeln!(stdout, "{}│", " ".repeat(right))?;
    queue!(stdout, ResetColor)?;
    Ok(())
}

/// Full frame: vertical offset + frame with clock and date.
/// Callers flush once after `render` for a tear-free frame.
pub fn render(
    stdout: &mut impl Write,
    layout: &Layout,
    rows: &[String],
    time_chars: &[char],
    date_line: &str,
) -> io::Result<()> {
    for _ in 0..layout.pad_y {
        writeln!(stdout)?;
    }

    write_top(stdout, layout.pad_x, layout.inner_w)?;
    write_empty(stdout, layout.pad_x, layout.inner_w)?;
    write_clock_rows(stdout, layout, rows, time_chars)?;
    write_empty(stdout, layout.pad_x, layout.inner_w)?;
    write_date(stdout, layout, date_line)?;
    write_empty(stdout, layout.pad_x, layout.inner_w)?;
    write_bottom(stdout, layout.pad_x, layout.inner_w)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centers_block_on_roomy_screen() {
        // clock 20 + date 10 -> inner 28, box 30, 100x40 screen.
        let l = compute_layout(100, 40, 20, 10);
        assert_eq!(l.inner_w, 28);
        assert_eq!(l.pad_x, (100 - 30) / 2);
        assert_eq!(l.pad_y, (40 - (GLYPH_H + 6)) / 2);
        assert_eq!(l.clock_offset, (28 - 20) / 2);
        assert_eq!(l.date_offset, (28 - 10) / 2);
    }

    #[test]
    fn clamps_to_narrow_screen() {
        let l = compute_layout(20, 24, 94, 27);
        assert_eq!(l.inner_w, 18);
        assert_eq!(l.pad_x, 0);
        assert_eq!(l.clock_offset, 0);
        assert_eq!(l.date_offset, 0);
    }

    #[test]
    fn render_smoke_frame_and_date() {
        let rows = crate::font::render_big("12:34:56", true);
        let time_chars: Vec<char> = "12:34:56".chars().collect();
        let date = "September 30, 2026 · Wednesday";
        let l = compute_layout(
            120,
            40,
            crate::font::line_width(time_chars.len()),
            date.chars().count(),
        );
        let mut buf: Vec<u8> = Vec::new();
        render(&mut buf, &l, &rows, &time_chars, date).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains('╭') && out.contains('╯'), "frame missing");
        assert!(out.contains(date), "date missing");
        assert!(out.contains(&"─".repeat(l.inner_w)), "top width wrong");
    }

    #[test]
    fn render_narrow_never_panics_or_wraps_wide() {
        let rows = crate::font::render_big("12:34:56", true);
        let time_chars: Vec<char> = "12:34:56".chars().collect();
        let l = compute_layout(30, 24, crate::font::line_width(8), 27);
        let mut buf: Vec<u8> = Vec::new();
        render(&mut buf, &l, &rows, &time_chars, "September 30, 2026 · Wednesday").unwrap();
        assert!(String::from_utf8(buf).is_ok());
    }
}
