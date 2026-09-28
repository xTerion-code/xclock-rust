use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use chrono::Local;
use crossterm::{cursor, execute, terminal};

const GLYPH_H: usize = 5;

fn glyph(ch: char) -> [&'static str; GLYPH_H] {
    match ch {
        '0' => [" ███ ", "█   █", "█   █", "█   █", " ███ "],
        '1' => ["  █  ", " ██  ", "  █  ", "  █  ", "█████"],
        '2' => ["████ ", "    █", " ███ ", "█    ", "█████"],
        '3' => ["████ ", "    █", " ███ ", "    █", "████ "],
        '4' => ["█   █", "█   █", "█████", "    █", "    █"],
        '5' => ["█████", "█    ", "████ ", "    █", "████ "],
        '6' => [" ███ ", "█    ", "████ ", "█   █", " ███ "],
        '7' => ["█████", "    █", "   █ ", "  █  ", " █   "],
        '8' => [" ███ ", "█   █", " ███ ", "█   █", " ███ "],
        '9' => [" ███ ", "█   █", " ████", "    █", " ███ "],
        ':' => ["     ", "  █  ", "     ", "  █  ", "     "],
        _ => ["     ", "     ", "     ", "     ", "     "],
    }
}

fn render_big(time: &str) -> Vec<String> {
    let chars: Vec<char> = time.chars().collect();
    let mut rows = vec![String::new(); GLYPH_H];
    for (i, ch) in chars.iter().enumerate() {
        let g = glyph(*ch);
        for r in 0..GLYPH_H {
            rows[r].push_str(g[r]);
            if i + 1 < chars.len() {
                rows[r].push(' ');
            }
        }
    }
    rows
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
        let time = Local::now().format("%H:%M:%S").to_string();
        let rows = render_big(&time);

        let (cols, lines) = terminal::size().unwrap_or((80, 24));
        let cols = cols as usize;
        let lines = lines as usize;

        let clock_w = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0);
        let pad_x = cols.saturating_sub(clock_w) / 2;
        let pad_y = lines.saturating_sub(GLYPH_H) / 2;

        execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
        execute!(stdout, cursor::MoveTo(0, 0))?;

        for _ in 0..pad_y {
            writeln!(stdout)?;
        }
        let pad = " ".repeat(pad_x);
        for row in rows {
            writeln!(stdout, "{pad}{row}")?;
        }
        stdout.flush()?;

        thread::sleep(Duration::from_millis(200));
    }

    execute!(stdout, cursor::Show)?;
    execute!(stdout, terminal::LeaveAlternateScreen)?;
    Ok(())
}
