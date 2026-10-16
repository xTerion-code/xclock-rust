use std::io::{self, BufWriter, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use chrono::{DateTime, Local};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    queue,
    style::{ResetColor, SetForegroundColor},
    terminal,
};

use crate::display::{DisplayOptions, format_date, format_meridiem, format_time};
use crate::font::{line_width, render_big};
use crate::terminal::TerminalGuard;
use crate::theme::{Palette, Theme};
use crate::ui;

/// Narrowest terminal that still shows the compact `HH:MM` face
/// (`line_width(5, Theme::Small.style()) + 4`); below that the clock
/// degrades to a plain one-line time.
const MIN_RENDERABLE_COLS: u16 = 23;

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
                let Event::Key(key) = event::read()? else {
                    continue;
                };
                if handle_key(key, &mut theme, &mut palette, &mut display) {
                    running.store(false, Ordering::SeqCst);
                    break;
                }
            }
        }

        if !running.load(Ordering::SeqCst) {
            break;
        }

        let now = Local::now();
        let colon_visible = !display.blink_colon || now.timestamp_subsec_millis() < 500;
        let (cols, lines) = terminal::size().unwrap_or((80, 24));
        let effective_theme = theme.resolve(
            cols as usize,
            lines as usize,
            display.show_seconds,
            display.show_date,
        );

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

        if !fits_framed(cols, lines, effective_theme, display) {
            draw_compact(
                &mut stdout,
                &now,
                cols,
                lines,
                display,
                palette,
                &mut last_layout_dims,
            )?;
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

/// Applies one key event to the live options in place.
/// Returns `true` when the app should quit.
fn handle_key(
    key: KeyEvent,
    theme: &mut Theme,
    palette: &mut Palette,
    display: &mut DisplayOptions,
) -> bool {
    if key.kind == KeyEventKind::Release {
        return false;
    }
    match key.code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc => true,
        KeyCode::Char('c' | 'C' | 'd' | 'D') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            true
        }
        KeyCode::Char(c) => {
            if let Some(t) = Theme::from_hotkey(c) {
                *theme = t;
            } else if c.eq_ignore_ascii_case(&'t') {
                *theme = theme.next();
            } else if c.eq_ignore_ascii_case(&'p') {
                *palette = palette.next();
            } else if c.eq_ignore_ascii_case(&'s') {
                display.toggle_seconds();
            } else if c.eq_ignore_ascii_case(&'h') {
                display.toggle_hour_format();
            } else if c.eq_ignore_ascii_case(&'b') {
                display.toggle_blink();
            } else if c.eq_ignore_ascii_case(&'d') {
                display.toggle_date();
            }
            false
        }
        _ => false,
    }
}

/// Whether the framed digit face fits without overflowing the viewport.
/// Over-wide content is clipped horizontally by `ui::compute_layout`, but a
/// frame taller than the screen would scroll the viewport on every redraw.
fn fits_framed(cols: u16, lines: u16, theme: Theme, display: DisplayOptions) -> bool {
    cols >= MIN_RENDERABLE_COLS && usize::from(lines) >= theme.block_height(display.show_date)
}

/// Renders the plain one-line time for terminals too small for the framed face.
fn draw_compact(
    stdout: &mut impl Write,
    now: &DateTime<Local>,
    cols: u16,
    lines: u16,
    display: DisplayOptions,
    palette: Palette,
    last_layout_dims: &mut Option<(u16, u16, usize, usize)>,
) -> io::Result<()> {
    let time = format_time(now, display);
    let time_str = if display.hour_format.is_12h() {
        format!("{} {}", time, format_meridiem(now))
    } else {
        time
    };
    let current_dims = (cols, lines, 0, 0);
    let needs_clear = *last_layout_dims != Some(current_dims);
    *last_layout_dims = Some(current_dims);

    let visible: String = time_str.chars().take(cols as usize).collect();
    let pad_y = usize::from(lines).saturating_sub(1) / 2;
    let pad_x = usize::from(cols).saturating_sub(visible.chars().count()) / 2;

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
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::HourFormat;

    fn default_state() -> (Theme, Palette, DisplayOptions) {
        (Theme::Large, Palette::BlackRed, DisplayOptions::default())
    }

    fn press(
        c: char,
        theme: &mut Theme,
        palette: &mut Palette,
        display: &mut DisplayOptions,
    ) -> bool {
        handle_key(
            KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE),
            theme,
            palette,
            display,
        )
    }

    #[test]
    fn quit_keys_and_ctrl_chords_stop_the_app() {
        let (mut theme, mut palette, mut display) = default_state();
        for code in [KeyCode::Char('q'), KeyCode::Char('Q'), KeyCode::Esc] {
            assert!(handle_key(
                KeyEvent::new(code, KeyModifiers::NONE),
                &mut theme,
                &mut palette,
                &mut display
            ));
        }
        for code in [
            KeyCode::Char('c'),
            KeyCode::Char('C'),
            KeyCode::Char('d'),
            KeyCode::Char('D'),
        ] {
            assert!(handle_key(
                KeyEvent::new(code, KeyModifiers::CONTROL),
                &mut theme,
                &mut palette,
                &mut display
            ));
        }
        // Bare `c` does nothing; bare `d` toggles the date instead of quitting.
        assert!(!press('c', &mut theme, &mut palette, &mut display));
        let before = display.show_date;
        assert!(!press('d', &mut theme, &mut palette, &mut display));
        assert_eq!(display.show_date, !before);
    }

    #[test]
    fn release_events_are_ignored() {
        let (mut theme, mut palette, mut display) = default_state();
        let release = KeyEvent::new_with_kind(
            KeyCode::Char('q'),
            KeyModifiers::NONE,
            KeyEventKind::Release,
        );
        assert!(!handle_key(release, &mut theme, &mut palette, &mut display));
        assert_eq!(display, DisplayOptions::default());
    }

    #[test]
    fn theme_hotkeys_switch_immediately() {
        let (mut theme, mut palette, mut display) = default_state();
        let cases = [
            ('2', Theme::Medium),
            ('3', Theme::Small),
            ('a', Theme::Auto),
            ('A', Theme::Auto),
            ('0', Theme::Auto),
            ('1', Theme::Large),
        ];
        for (c, expected) in cases {
            assert!(!press(c, &mut theme, &mut palette, &mut display));
            assert_eq!(theme, expected, "hotkey {c:?}");
        }
    }

    #[test]
    fn cycle_and_toggle_keys_update_state() {
        let (mut theme, mut palette, mut display) = default_state();
        press('t', &mut theme, &mut palette, &mut display);
        assert_eq!(theme, Theme::Medium);
        press('p', &mut theme, &mut palette, &mut display);
        assert_eq!(palette, Palette::Purple);
        press('s', &mut theme, &mut palette, &mut display);
        assert!(!display.show_seconds);
        press('h', &mut theme, &mut palette, &mut display);
        assert_eq!(display.hour_format, HourFormat::H12);
        press('b', &mut theme, &mut palette, &mut display);
        assert!(!display.blink_colon);
        press('d', &mut theme, &mut palette, &mut display);
        assert!(!display.show_date);
        // Unknown keys leave the state untouched.
        press('z', &mut theme, &mut palette, &mut display);
        assert_eq!(theme, Theme::Medium);
        assert_eq!(palette, Palette::Purple);
        assert!(!display.show_seconds);
    }

    #[test]
    fn framed_layout_requires_enough_room() {
        let display = DisplayOptions::default();
        // Large needs 14 lines with a date.
        assert!(fits_framed(120, 30, Theme::Large, display));
        assert!(!fits_framed(120, 13, Theme::Large, display));
        // Without a date it needs one line less.
        let no_date = DisplayOptions {
            show_date: false,
            ..display
        };
        assert!(fits_framed(120, 13, Theme::Large, no_date));
        // Medium fits a standard-height terminal, the compact face needs less.
        assert!(fits_framed(80, 14, Theme::Medium, display));
        assert!(fits_framed(MIN_RENDERABLE_COLS, 12, Theme::Small, display));
        // Below the minimum width the plain fallback takes over.
        assert!(!fits_framed(
            MIN_RENDERABLE_COLS - 1,
            24,
            Theme::Small,
            display
        ));
    }
}
