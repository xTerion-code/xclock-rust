use std::io::{self, Write};

use crossterm::{
    queue,
    style::{Attribute, ResetColor, SetAttribute, SetForegroundColor},
};

use super::layout::{HINT, Layout};
use crate::font::Style;
use crate::theme::Theme;

// Note: `\r\n` endings everywhere — the app runs in raw mode,
// where a bare `\n` does not return the carriage.

fn write_top(
    stdout: &mut impl Write,
    theme: Theme,
    pad_x: usize,
    inner_w: usize,
) -> io::Result<()> {
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
    let row_chars: Vec<Vec<char>> = rows.iter().map(|r| r.chars().collect()).collect();
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

pub fn render(
    stdout: &mut impl Write,
    layout: &Layout,
    rows: &[String],
    time_chars: &[char],
    date_line: Option<&str>,
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
    if let Some(date) = date_line {
        write_centered(stdout, layout, date, theme, theme.date())?;
    }
    write_centered(stdout, layout, HINT, theme, theme.separator())?;
    write_empty(stdout, theme, layout.pad_x, layout.inner_w)?;
    write_bottom(stdout, theme, layout.pad_x, layout.inner_w)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    fn modern() -> Theme {
        Theme::Modern
    }

    #[test]
    fn render_smoke_frame_date_and_hint() {
        for theme in Theme::all() {
            let style = theme.style();
            let rows = crate::font::render_big("12:34:56", true, style);
            let time_chars: Vec<char> = "12:34:56".chars().collect();
            let date = "September 30, 2026 · Wednesday";
            let l = crate::ui::compute_layout(
                140,
                40,
                crate::font::line_width(time_chars.len(), style),
                date.chars().count(),
                theme,
                true,
            );
            let mut buf: Vec<u8> = Vec::new();
            render(&mut buf, &l, &rows, &time_chars, Some(date), theme).unwrap();
            let out = String::from_utf8(buf).unwrap();
            assert!(out.contains('╭') && out.contains('╯'), "frame missing");
            assert!(out.contains(date), "date missing");
            assert!(out.contains(HINT), "hint missing");
            assert!(out.contains(&"─".repeat(l.inner_w)), "top width wrong");
        }
    }

    #[test]
    fn render_without_date_omits_date_row() {
        let style = modern().style();
        let rows = crate::font::render_big("12:34:56", true, style);
        let time_chars: Vec<char> = "12:34:56".chars().collect();
        let date = "September 30, 2026 · Wednesday";
        let l = crate::ui::compute_layout(
            140,
            40,
            crate::font::line_width(time_chars.len(), style),
            date.chars().count(),
            modern(),
            false,
        );
        let mut buf: Vec<u8> = Vec::new();
        render(&mut buf, &l, &rows, &time_chars, None, modern()).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(!out.contains(date), "date should be hidden");
        assert!(out.contains(HINT), "hint missing");
    }

    #[test]
    fn render_narrow_never_panics_or_wraps_wide() {
        let style = modern().style();
        let rows = crate::font::render_big("12:34:56", true, style);
        let time_chars: Vec<char> = "12:34:56".chars().collect();
        let l = crate::ui::compute_layout(
            30,
            24,
            crate::font::line_width(8, style),
            27,
            modern(),
            true,
        );
        let mut buf: Vec<u8> = Vec::new();
        render(
            &mut buf,
            &l,
            &rows,
            &time_chars,
            Some("September 30, 2026 · Wednesday"),
            modern(),
        )
        .unwrap();
        assert!(String::from_utf8(buf).is_ok());
    }
}
