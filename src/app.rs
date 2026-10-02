use std::io::{self, BufWriter, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::Local;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    queue, terminal,
};

use crate::display::{DisplayOptions, format_time};
use crate::font::{line_width, render_big};
use crate::locale::format_date;
use crate::terminal::TerminalGuard;
use crate::theme::Theme;
use crate::ui;

pub fn run(initial_theme: Theme, initial_display: DisplayOptions) -> io::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let _ = ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    });

    let _guard = TerminalGuard::enter()?;
    let mut stdout = BufWriter::new(io::stdout());
    let mut theme = initial_theme;
    let mut display = initial_display;

    // Last drawn frame: skip redraws when nothing visible changed
    // (blink ticks at 2 Hz, time at 1 Hz, but we poll at ~10 Hz).
    let mut last_key: Option<(String, bool, u16, u16, Theme, bool)> = None;

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
                        last_key = None;
                    } else if c == 's' || c == 'S' {
                        display.toggle();
                        last_key = None;
                    }
                }
                _ => {}
            }
        }

        let now = Local::now();
        let colon_visible = now.timestamp_subsec_millis() < 500;
        let time = format_time(&now, display);
        let (cols, lines) = terminal::size().unwrap_or((80, 24));

        let key = (
            time.clone(),
            colon_visible,
            cols,
            lines,
            theme,
            display.show_seconds,
        );
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
