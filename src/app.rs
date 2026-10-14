use std::io::{self, BufWriter, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use chrono::Local;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    queue,
    style::{ResetColor, SetForegroundColor},
    terminal,
};

use crate::display::{DisplayOptions, format_date, format_meridiem, format_time};
use crate::font::{line_width, render_big};
use crate::terminal::TerminalGuard;
use crate::theme::{Palette, Theme};
use crate::ui;

pub fn run(
    initial_theme: Theme,
    initial_palette: Palette,
    initial_display: DisplayOptions,
) -> io::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let _ = ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    });

    let _guard = TerminalGuard::enter()?;
    let mut stdout = BufWriter::new(io::stdout());
    let mut theme = initial_theme;
    let mut palette = initial_palette;
    let mut display = initial_display;

    // Last drawn frame: skip redraws when nothing visible changed
    // (blink ticks at 2 Hz, time at 1 Hz, but we poll at ~10 Hz).
    let mut last_key: Option<(i64, bool, u16, u16, Theme, DisplayOptions, Palette)> = None;
    let mut last_layout_dims: Option<(u16, u16, usize, usize)> = None;

    while running.load(Ordering::SeqCst) {
        // 100 ms poll doubles as the frame timer.
        if event::poll(Duration::from_millis(100))? {
            while event::poll(Duration::from_millis(0))? {
                if let Event::Key(key) = event::read()?
                    && key.kind != KeyEventKind::Release
                {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                            running.store(false, Ordering::SeqCst);
                            break;
                        }
                        KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Char('d') | KeyCode::Char('D')
                            if key.modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            running.store(false, Ordering::SeqCst);
                            break;
                        }
                        KeyCode::Char(c) => {
                            if let Some(t) = Theme::from_hotkey(c)
                                && t != theme
                            {
                                theme = t;
                                last_key = None;
                            } else if c == 't' || c == 'T' {
                                theme = theme.next();
                                last_key = None;
                            } else if c == 'p' || c == 'P' {
                                palette = palette.next();
                                last_key = None;
                            } else if c == 's' || c == 'S' {
                                display.toggle_seconds();
                                last_key = None;
                            } else if c == 'h' || c == 'H' {
                                display.toggle_hour_format();
                                last_key = None;
                            } else if c == 'b' || c == 'B' {
                                display.toggle_blink();
                                last_key = None;
                            } else if c == 'd' || c == 'D' {
                                display.toggle_date();
                                last_key = None;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if !running.load(Ordering::SeqCst) {
            break;
        }

        let now = Local::now();
        let colon_visible = !display.blink_colon || now.timestamp_subsec_millis() < 500;
        let (cols, lines) = terminal::size().unwrap_or((80, 24));
        let effective_theme =
            theme.resolve(cols as usize, lines as usize, display.show_seconds, display.show_date);

        let time_sec = if display.show_seconds {
            now.timestamp()
        } else {
            now.timestamp() / 60
        };
        let key = (
            time_sec,
            colon_visible,
            cols,
            lines,
            effective_theme,
            display,
            palette,
        );
        if last_key == Some(key) {
            continue;
        }
        last_key = Some(key);

        let min_lines = Theme::Small.block_height(display.show_date);
        if cols < 23 || lines < min_lines as u16 {
            let time = format_time(&now, display);
            let time_str = if display.hour_format.is_12h() {
                format!("{} {}", time, format_meridiem(&now))
            } else {
                time
            };
            let current_dims = (cols, lines, 0, 0);
            let needs_clear = last_layout_dims != Some(current_dims);
            last_layout_dims = Some(current_dims);

            let visible: String = time_str.chars().take(cols as usize).collect();
            let pad_y = (lines as usize).saturating_sub(1) / 2;
            let pad_x = (cols as usize).saturating_sub(visible.chars().count()) / 2;

            if needs_clear {
                queue!(
                    stdout,
                    terminal::Clear(terminal::ClearType::All),
                    cursor::MoveTo(0, 0)
                )?;
            }
            queue!(
                stdout,
                cursor::MoveTo(pad_x as u16, pad_y as u16),
                terminal::Clear(terminal::ClearType::CurrentLine),
                SetForegroundColor(palette.hours_minutes())
            )?;
            write!(stdout, "{visible}")?;
            queue!(stdout, ResetColor)?;
            stdout.flush()?;
            continue;
        }

        let time = format_time(&now, display);
        let style = effective_theme.style();
        let time_chars: Vec<char> = time.chars().collect();
        let rows = render_big(&time, colon_visible, style);
        let date_line = if display.hour_format.is_12h() {
            format!("{} · {}", format_date(&now), format_meridiem(&now))
        } else {
            format_date(&now)
        };

        let layout = ui::compute_layout(
            cols as usize,
            lines as usize,
            line_width(time_chars.len(), style),
            date_line.chars().count(),
            effective_theme,
            display.show_date,
        );

        let current_dims = (
            cols,
            lines,
            layout.inner_w,
            effective_theme.block_height(display.show_date),
        );
        let needs_clear = last_layout_dims != Some(current_dims);
        last_layout_dims = Some(current_dims);

        // Queued (not executed one-by-one): a single flush per frame,
        // no per-cell syscalls, minimal flicker. Clear all only on dimension changes.
        if needs_clear {
            queue!(
                stdout,
                terminal::Clear(terminal::ClearType::All),
                cursor::MoveTo(0, 0)
            )?;
        } else {
            queue!(stdout, cursor::MoveTo(0, 0))?;
        }
        ui::render(
            &mut stdout,
            &layout,
            &rows,
            &time_chars,
            display.show_date.then_some(date_line.as_str()),
            effective_theme,
            palette,
        )?;
        stdout.flush()?;
    }

    stdout.flush()?;
    Ok(())
}
