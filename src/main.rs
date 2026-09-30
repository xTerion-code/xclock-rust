use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use chrono::{Datelike, Local, Weekday};
use crossterm::{
    cursor, execute,
    style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor},
    terminal,
};

const GLYPH_H: usize = 7;
const GLYPH_W: usize = 5;

fn glyph(ch: char, colon_visible: bool) -> [&'static str; GLYPH_H] {
    match ch {
        '0' => [
            " ███ ",
            "█   █",
            "█   █",
            "█   █",
            "█   █",
            "█   █",
            " ███ ",
        ],
        '1' => [
            "  █  ",
            " ██  ",
            "  █  ",
            "  █  ",
            "  █  ",
            "  █  ",
            "█████",
        ],
        '2' => [
            "█████",
            "    █",
            "    █",
            "█████",
            "█    ",
            "█    ",
            "█████",
        ],
        '3' => [
            "█████",
            "    █",
            "    █",
            " ████",
            "    █",
            "    █",
            "█████",
        ],
        '4' => [
            "█   █",
            "█   █",
            "█   █",
            "█████",
            "    █",
            "    █",
            "    █",
        ],
        '5' => [
            "█████",
            "█    ",
            "█    ",
            "█████",
            "    █",
            "    █",
            "█████",
        ],
        '6' => [
            "█████",
            "█    ",
            "█    ",
            "█████",
            "█   █",
            "█   █",
            "█████",
        ],
        '7' => [
            "█████",
            "    █",
            "    █",
            "   █ ",
            "  █  ",
            "  █  ",
            "  █  ",
        ],
        '8' => [
            " ███ ",
            "█   █",
            "█   █",
            " ███ ",
            "█   █",
            "█   █",
            " ███ ",
        ],
        '9' => [
            "█████",
            "█   █",
            "█   █",
            "█████",
            "    █",
            "    █",
            "█████",
        ],
        ':' if colon_visible => [
            "     ",
            "     ",
            "  ▮  ",
            "     ",
            "     ",
            "  ▮  ",
            "     ",
        ],
        _ => [
            "     ",
            "     ",
            "     ",
            "     ",
            "     ",
            "     ",
            "     ",
        ],
    }
}

fn weekday_ru(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "Понедельник",
        Weekday::Tue => "Вторник",
        Weekday::Wed => "Среда",
        Weekday::Thu => "Четверг",
        Weekday::Fri => "Пятница",
        Weekday::Sat => "Суббота",
        Weekday::Sun => "Воскресенье",
    }
}

fn month_ru(month: u32) -> &'static str {
    match month {
        1 => "января",
        2 => "февраля",
        3 => "марта",
        4 => "апреля",
        5 => "мая",
        6 => "июня",
        7 => "июля",
        8 => "августа",
        9 => "сентября",
        10 => "октября",
        11 => "ноября",
        12 => "декабря",
        _ => "",
    }
}

fn render_big(time: &str, colon_visible: bool) -> Vec<String> {
    let chars: Vec<char> = time.chars().collect();
    let mut rows = vec![String::new(); GLYPH_H];
    for (i, ch) in chars.iter().enumerate() {
        let g = glyph(*ch, colon_visible);
        for r in 0..GLYPH_H {
            rows[r].push_str(g[r]);
            if i + 1 < chars.len() {
                rows[r].push(' ');
            }
        }
    }
    rows
}

fn char_color(index: usize) -> Color {
    // "HH:MM:SS" -> 0..4 часы+минуты, 5 второе двоеточие, 6..7 секунды
    if index <= 4 {
        Color::White
    } else {
        Color::Cyan
    }
}

fn write_border_line(stdout: &mut io::Stdout, pad_x: usize, inner_w: usize) -> io::Result<()> {
    let pad = " ".repeat(pad_x);
    execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
    writeln!(stdout, "{pad}╭{}╮", "─".repeat(inner_w))?;
    execute!(stdout, ResetColor)?;
    Ok(())
}

fn write_border_bottom(stdout: &mut io::Stdout, pad_x: usize, inner_w: usize) -> io::Result<()> {
    let pad = " ".repeat(pad_x);
    execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
    writeln!(stdout, "{pad}╰{}╯", "─".repeat(inner_w))?;
    execute!(stdout, ResetColor)?;
    Ok(())
}

fn write_empty_line(stdout: &mut io::Stdout, pad_x: usize, inner_w: usize) -> io::Result<()> {
    let pad = " ".repeat(pad_x);
    execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
    write!(stdout, "{pad}│")?;
    execute!(stdout, ResetColor)?;
    write!(stdout, "{}", " ".repeat(inner_w))?;
    execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
    writeln!(stdout, "│")?;
    execute!(stdout, ResetColor)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let _ = ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    });

    let mut stdout = io::stdout();

    // alternate screen + clear + hide cursor
    execute!(stdout, terminal::EnterAlternateScreen)?;
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
    execute!(stdout, cursor::Hide)?;

    while running.load(Ordering::SeqCst) {
        let now = Local::now();
        // Мигание двоеточия 1 Гц
        let colon_visible = now.timestamp_subsec_millis() < 500;
        let time = now.format("%H:%M:%S").to_string();
        let rows = render_big(&time, colon_visible);
        let date_line = format!(
            "{} {} {} · {}",
            now.day(),
            month_ru(now.month()),
            now.year(),
            weekday_ru(now.weekday())
        );

        let (cols, lines) = terminal::size().unwrap_or((80, 24));
        let cols = cols as usize;
        let lines = lines as usize;

        let n_chars = time.chars().count();
        let clock_w = n_chars * GLYPH_W + (n_chars.saturating_sub(1));
        let date_w = date_line.chars().count();

        // Рамка с внутренними отступами
        let h_pad_inside: usize = 4;
        let inner_w = clock_w.max(date_w).saturating_add(h_pad_inside * 2);
        let box_w = inner_w + 2;
        let block_h = GLYPH_H + 6; // рамка + отступы + часы + gap + дата
        let pad_x = cols.saturating_sub(box_w) / 2;
        let pad_y = lines.saturating_sub(block_h) / 2;

        let clock_offset = (inner_w.saturating_sub(clock_w)) / 2;
        let date_offset = (inner_w.saturating_sub(date_w)) / 2;

        execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
        execute!(stdout, cursor::MoveTo(0, 0))?;

        for _ in 0..pad_y {
            writeln!(stdout)?;
        }

        write_border_line(&mut stdout, pad_x, inner_w)?;
        write_empty_line(&mut stdout, pad_x, inner_w)?;

        // Часы: HH:MM белым, SS бирюзовым, всё жирным
        let time_chars: Vec<char> = time.chars().collect();
        execute!(stdout, SetAttribute(Attribute::Bold))?;
        for row in &rows {
            let pad = " ".repeat(pad_x);
            execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
            write!(stdout, "{pad}│{}",
                " ".repeat(clock_offset)
            )?;
            // Печатаем глиф за глифом, переключая цвет
            // Каждый глиф 5 символов + 1 пробел-разделитель
            let mut col = 0usize;
            for (i, _ch) in time_chars.iter().enumerate() {
                let color = char_color(i);
                execute!(stdout, SetForegroundColor(color))?;
                for k in 0..GLYPH_W {
                    let c = row.chars().nth(col + k).unwrap_or(' ');
                    write!(stdout, "{c}")?;
                }
                col += GLYPH_W;
                if i + 1 < time_chars.len() {
                    // разделитель — нейтральный тёмный
                    execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
                    let c = row.chars().nth(col).unwrap_or(' ');
                    write!(stdout, "{c}")?;
                    col += 1;
                }
            }
            // правый внутренний отступ
            let used = clock_offset + clock_w;
            let right = inner_w.saturating_sub(used);
            execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
            writeln!(stdout, "{}│", " ".repeat(right))?;
            execute!(stdout, ResetColor)?;
        }
        execute!(stdout, SetAttribute(Attribute::Reset))?;

        write_empty_line(&mut stdout, pad_x, inner_w)?;

        // Дата приглушённым серым
        {
            let pad = " ".repeat(pad_x);
            execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
            write!(stdout, "{pad}│{}",
                " ".repeat(date_offset)
            )?;
            execute!(stdout, SetForegroundColor(Color::Grey))?;
            write!(stdout, "{date_line}")?;
            let used = date_offset + date_w;
            let right = inner_w.saturating_sub(used);
            execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
            writeln!(stdout, "{}│", " ".repeat(right))?;
            execute!(stdout, ResetColor)?;
        }

        write_empty_line(&mut stdout, pad_x, inner_w)?;
        write_border_bottom(&mut stdout, pad_x, inner_w)?;

        stdout.flush()?;

        thread::sleep(Duration::from_millis(100));
    }

    execute!(stdout, cursor::Show)?;
    execute!(stdout, terminal::LeaveAlternateScreen)?;
    Ok(())
}
