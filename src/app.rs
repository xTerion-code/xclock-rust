use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use chrono::Local;
use crossterm::{cursor, execute, terminal};

use crate::font::{line_width, render_big};
use crate::locale::format_date;
use crate::ui;

pub fn run() -> io::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let _ = ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    });

    let mut stdout = io::stdout();

    // alternate screen + clear + hide cursor (restored on exit)
    execute!(stdout, terminal::EnterAlternateScreen)?;
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
    execute!(stdout, cursor::Hide)?;

    while running.load(Ordering::SeqCst) {
        let now = Local::now();
        // Colon blink at 1 Hz
        let colon_visible = now.timestamp_subsec_millis() < 500;
        let time = now.format("%H:%M:%S").to_string();
        let time_chars: Vec<char> = time.chars().collect();
        let rows = render_big(&time, colon_visible);
        let date_line = format_date(&now);

        let (cols, lines) = terminal::size().unwrap_or((80, 24));
        let layout = ui::compute_layout(
            cols as usize,
            lines as usize,
            line_width(time_chars.len()),
            date_line.chars().count(),
        );

        execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
        execute!(stdout, cursor::MoveTo(0, 0))?;
        ui::render(
            &mut stdout,
            &layout,
            &rows,
            &time_chars,
            &date_line,
            date_line.chars().count(),
        )?;
        stdout.flush()?;

        thread::sleep(Duration::from_millis(100));
    }

    execute!(stdout, cursor::Show)?;
    execute!(stdout, terminal::LeaveAlternateScreen)?;
    Ok(())
}
