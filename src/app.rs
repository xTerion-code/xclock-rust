use std::io::{self, BufWriter, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::Local;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute, queue, terminal,
};

use crate::font::{line_width, render_big};
use crate::locale::format_date;
use crate::theme::Theme;
use crate::ui;

/// Enters raw mode + alternate screen and hides the cursor on creation,
/// restores everything on drop — even on early `?` return or panic unwind.
///
/// Raw mode is required for instant key handling (`1`/`2`/`3`, `q`);
/// note that in raw mode `Ctrl+C` arrives as a key event (no SIGINT),
/// so it is handled in the event loop below.
struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut out = io::stdout();
        if let Err(e) = execute!(out, terminal::EnterAlternateScreen, cursor::Hide) {
            let _ = terminal::disable_raw_mode();
            return Err(e);
        }
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let mut out = io::stdout();
        let _ = execute!(out, cursor::Show);
        let _ = execute!(out, terminal::LeaveAlternateScreen);
    }
}

pub fn run(initial_theme: Theme) -> io::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let _ = ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    });

    let _guard = TerminalGuard::enter()?;
    let mut stdout = BufWriter::new(io::stdout());
    let mut theme = initial_theme;

    // Last drawn frame: skip redraws when nothing visible changed
    // (blink ticks at 2 Hz, time at 1 Hz, but we poll at ~10 Hz).
    let mut last_key: Option<(String, bool, u16, u16, Theme)> = None;

    while running.load(Ordering::SeqCst) {
        // 100 ms poll doubles as the frame timer.
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('c')
                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    break;
                }
                KeyCode::Char(c) => {
                    if let Some(t) = Theme::from_hotkey(c)
                        && t != theme
                    {
                        theme = t;
                        last_key = None; // force immediate redraw
                    }
                }
                _ => {}
            }
        }

        let now = Local::now();
        // Colon blink at 1 Hz
        let colon_visible = now.timestamp_subsec_millis() < 500;
        let time = now.format("%H:%M:%S").to_string();
        let (cols, lines) = terminal::size().unwrap_or((80, 24));

        let key = (time.clone(), colon_visible, cols, lines, theme);
        if last_key.as_ref() == Some(&key) {
            continue;
        }
        last_key = Some(key);

        let style = theme.style();
        let time_chars: Vec<char> = time.chars().collect();
        let rows = render_big(&time, colon_visible, style);
        let date_line = format_date(&now);

        let layout = ui::compute_layout(
            cols as usize,
            lines as usize,
            line_width(time_chars.len(), style),
            date_line.chars().count(),
            theme,
        );

        // Queued (not executed one-by-one): a single flush per frame,
        // no per-cell syscalls, minimal flicker.
        queue!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        ui::render(&mut stdout, &layout, &rows, &time_chars, &date_line, theme)?;
        stdout.flush()?;
    }

    stdout.flush()?;
    Ok(())
}
