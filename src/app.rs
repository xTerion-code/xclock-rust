use std::io::{self, BufWriter, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use chrono::Local;
use crossterm::{cursor, execute, queue, terminal};

use crate::font::{line_width, render_big};
use crate::locale::format_date;
use crate::ui;

/// Enters the alternate screen and hides the cursor on creation,
/// restores both on drop — even on early `?` return or panic unwind.
struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        let mut out = io::stdout();
        execute!(out, terminal::EnterAlternateScreen, cursor::Hide)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut out = io::stdout();
        let _ = execute!(out, cursor::Show);
        let _ = execute!(out, terminal::LeaveAlternateScreen);
    }
}

pub fn run() -> io::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let _ = ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    });

    let _guard = TerminalGuard::enter()?;
    let mut stdout = BufWriter::new(io::stdout());

    // Last drawn frame: skip redraws when nothing visible changed
    // (blink ticks at 2 Hz, time at 1 Hz, but we poll at 10 Hz).
    let mut last_key: Option<(String, bool, u16, u16)> = None;

    while running.load(Ordering::SeqCst) {
        let now = Local::now();
        // Colon blink at 1 Hz
        let colon_visible = now.timestamp_subsec_millis() < 500;
        let time = now.format("%H:%M:%S").to_string();
        let (cols, lines) = terminal::size().unwrap_or((80, 24));

        let key = (time.clone(), colon_visible, cols, lines);
        if last_key.as_ref() == Some(&key) {
            thread::sleep(Duration::from_millis(100));
            continue;
        }
        last_key = Some(key);

        let time_chars: Vec<char> = time.chars().collect();
        let rows = render_big(&time, colon_visible);
        let date_line = format_date(&now);

        let layout = ui::compute_layout(
            cols as usize,
            lines as usize,
            line_width(time_chars.len()),
            date_line.chars().count(),
        );

        // Queued (not executed one-by-one): a single flush per frame,
        // no per-cell syscalls, minimal flicker.
        queue!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        ui::render(&mut stdout, &layout, &rows, &time_chars, &date_line)?;
        stdout.flush()?;

        thread::sleep(Duration::from_millis(100));
    }

    stdout.flush()?;
    Ok(())
}
