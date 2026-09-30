use std::io::{self, Write};

use crossterm::{
    execute,
    style::{Attribute, ResetColor, SetAttribute, SetForegroundColor},
};

use crate::font::{GLYPH_H, GLYPH_W, line_width};
use crate::theme;

/// Геометрия центрированного блока (рамка + часы + дата).
pub struct Layout {
    pub pad_x: usize,
    pub pad_y: usize,
    pub inner_w: usize,
    pub clock_offset: usize,
    pub date_offset: usize,
}

const H_PAD_INSIDE: usize = 4;

fn block_height() -> usize {
    // верх рамки + пустая + часы + gap + дата + пустая + низ рамки
    GLYPH_H + 6
}

pub fn compute_layout(
    cols: usize,
    lines: usize,
    clock_w: usize,
    date_w: usize,
) -> Layout {
    let inner_w = clock_w.max(date_w) + H_PAD_INSIDE * 2;
    let box_w = inner_w + 2;
    Layout {
        pad_x: cols.saturating_sub(box_w) / 2,
        pad_y: lines.saturating_sub(block_height()) / 2,
        inner_w,
        clock_offset: inner_w.saturating_sub(clock_w) / 2,
        date_offset: inner_w.saturating_sub(date_w) / 2,
    }
}

fn write_top(stdout: &mut io::Stdout, pad_x: usize, inner_w: usize) -> io::Result<()> {
    let pad = " ".repeat(pad_x);
    execute!(stdout, SetForegroundColor(theme::BORDER))?;
    writeln!(stdout, "{pad}╭{}╮", "─".repeat(inner_w))?;
    execute!(stdout, ResetColor)?;
    Ok(())
}

fn write_bottom(stdout: &mut io::Stdout, pad_x: usize, inner_w: usize) -> io::Result<()> {
    let pad = " ".repeat(pad_x);
    execute!(stdout, SetForegroundColor(theme::BORDER))?;
    writeln!(stdout, "{pad}╰{}╯", "─".repeat(inner_w))?;
    execute!(stdout, ResetColor)?;
    Ok(())
}

fn write_empty(stdout: &mut io::Stdout, pad_x: usize, inner_w: usize) -> io::Result<()> {
    let pad = " ".repeat(pad_x);
    execute!(stdout, SetForegroundColor(theme::BORDER))?;
    write!(stdout, "{pad}│")?;
    execute!(stdout, ResetColor)?;
    write!(stdout, "{}", " ".repeat(inner_w))?;
    execute!(stdout, SetForegroundColor(theme::BORDER))?;
    writeln!(stdout, "│")?;
    execute!(stdout, ResetColor)?;
    Ok(())
}

fn write_clock_rows(
    stdout: &mut io::Stdout,
    layout: &Layout,
    rows: &[String],
    time_chars: &[char],
) -> io::Result<()> {
    let row_chars: Vec<Vec<char>> = rows
        .iter()
        .map(|r| r.chars().collect())
        .collect();

    execute!(stdout, SetAttribute(Attribute::Bold))?;
    for row in &row_chars {
        let pad = " ".repeat(layout.pad_x);
        execute!(stdout, SetForegroundColor(theme::BORDER))?;
        write!(stdout, "{pad}│{}", " ".repeat(layout.clock_offset))?;

        let mut col = 0usize;
        for (i, _ch) in time_chars.iter().enumerate() {
            execute!(
                stdout,
                SetForegroundColor(theme::time_char_color(i))
            )?;
            for k in 0..GLYPH_W {
                let c = row.get(col + k).copied().unwrap_or(' ');
                write!(stdout, "{c}")?;
            }
            col += GLYPH_W;
            if i + 1 < time_chars.len() {
                execute!(stdout, SetForegroundColor(theme::SEPARATOR))?;
                let c = row.get(col).copied().unwrap_or(' ');
                write!(stdout, "{c}")?;
                col += 1;
            }
        }

        let clock_w = line_width(time_chars.len());
        let right = layout.inner_w.saturating_sub(layout.clock_offset + clock_w);
        execute!(stdout, SetForegroundColor(theme::BORDER))?;
        writeln!(stdout, "{}│", " ".repeat(right))?;
        execute!(stdout, ResetColor)?;
    }
    execute!(stdout, SetAttribute(Attribute::Reset))?;
    Ok(())
}

fn write_date(
    stdout: &mut io::Stdout,
    layout: &Layout,
    date_line: &str,
    date_w: usize,
) -> io::Result<()> {
    let pad = " ".repeat(layout.pad_x);
    execute!(stdout, SetForegroundColor(theme::BORDER))?;
    write!(stdout, "{pad}│{}", " ".repeat(layout.date_offset))?;
    execute!(stdout, SetForegroundColor(theme::DATE))?;
    write!(stdout, "{date_line}")?;
    let right = layout.inner_w.saturating_sub(layout.date_offset + date_w);
    execute!(stdout, SetForegroundColor(theme::BORDER))?;
    writeln!(stdout, "{}│", " ".repeat(right))?;
    execute!(stdout, ResetColor)?;
    Ok(())
}

/// Полный кадр: вертикальный отступ + рамка с часами и датой.
pub fn render(
    stdout: &mut io::Stdout,
    layout: &Layout,
    rows: &[String],
    time_chars: &[char],
    date_line: &str,
    date_w: usize,
) -> io::Result<()> {
    for _ in 0..layout.pad_y {
        writeln!(stdout)?;
    }

    write_top(stdout, layout.pad_x, layout.inner_w)?;
    write_empty(stdout, layout.pad_x, layout.inner_w)?;
    write_clock_rows(stdout, layout, rows, time_chars)?;
    write_empty(stdout, layout.pad_x, layout.inner_w)?;
    write_date(stdout, layout, date_line, date_w)?;
    write_empty(stdout, layout.pad_x, layout.inner_w)?;
    write_bottom(stdout, layout.pad_x, layout.inner_w)?;
    Ok(())
}
