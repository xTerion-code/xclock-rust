use std::io::{self, BufWriter, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

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
use crate::theme::{Palette, ResolvedTheme, Theme};
use crate::ui;

/// Longest the loop ever sleeps in one go. Keeps the terminal responsive to
/// resize events and to anything the event source reports on its own, even when
/// nothing visible can change.
const MAX_WAIT_MS: u64 = 500;
const MAX_WAIT: Duration = Duration::from_millis(MAX_WAIT_MS);

/// Colon blink half-period: the colon is on for this long, then off.
const BLINK_PERIOD_MS: u64 = 500;

/// Fallback repeat window for terminals that do not report event types.
///
/// Such terminals resend the very same byte while a key is held, and nothing in
/// the event kind distinguishes that from a fresh press. `xclock` asks the
/// terminal for `REPORT_EVENT_TYPES` first; only when the terminal declines does
/// this window take over, so a held key is applied twice per second at worst
/// instead of thirty times.
const LEGACY_REPEAT_WINDOW: Duration = Duration::from_millis(500);

/// Everything a rendered frame depends on. When none of it changed since the
/// last frame, drawing is skipped entirely.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct FrameKey {
    /// Second-resolution time, or minute-resolution when seconds are hidden.
    time: i64,
    colon_visible: bool,
    cols: u16,
    lines: u16,
    theme: ResolvedTheme,
    display: DisplayOptions,
    palette: Palette,
    /// `true` for the one-line fallback, which shares every other key field
    /// with the framed face but occupies a different rectangle.
    plain: bool,
}

/// The current look of the clock, resolved against the live terminal.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Look {
    theme: ResolvedTheme,
    display: DisplayOptions,
    palette: Palette,
}

impl Look {
    fn new(theme: Theme, palette: Palette, display: DisplayOptions, cols: u16, lines: u16) -> Self {
        let theme = theme.resolve(cols as usize, lines as usize, display.face());
        Self {
            theme,
            display,
            palette,
        }
    }

    /// Whether the framed face fits without clipping digits or scrolling rows.
    fn fits_framed(self, cols: u16, lines: u16) -> bool {
        self.theme
            .fits(cols as usize, lines as usize, self.display.face())
    }

    fn key(self, now: &DateTime<Local>, cols: u16, lines: u16) -> FrameKey {
        let sub = now.timestamp_subsec_millis();
        FrameKey {
            time: if self.display.show_seconds {
                now.timestamp()
            } else {
                now.timestamp() / 60
            },
            colon_visible: !self.display.blink_colon || u64::from(sub) < BLINK_PERIOD_MS,
            cols,
            lines,
            theme: self.theme,
            display: self.display,
            palette: self.palette,
            plain: !self.fits_framed(cols, lines),
        }
    }
}

/// Buffered output plus the state needed to skip and to erase previous frames.
///
/// Generic over the sink so the render paths can be exercised against a `Vec`
/// instead of the real terminal.
struct Canvas<'a, W: Write> {
    out: &'a mut W,
    last_key: Option<FrameKey>,
    last_rect: Option<ui::FrameRect>,
}

impl<'a, W: Write> Canvas<'a, W> {
    fn new(out: &'a mut W) -> Self {
        Self {
            out,
            last_key: None,
            last_rect: None,
        }
    }

    /// Erases what the previous frame occupied when the layout moved, and
    /// reports whether this frame has to be drawn at all.
    fn prepare(&mut self, key: FrameKey, rect: ui::FrameRect) -> io::Result<bool> {
        if self.last_key == Some(key) {
            return Ok(false);
        }
        match self.last_rect {
            // Only the rows the two frames share need erasing. Clearing the
            // whole screen here is what made every toggle flash.
            Some(prev) => ui::clear_frame(self.out, prev.union(rect))?,
            // First frame: the alternate screen may still hold old content.
            None => queue!(self.out, terminal::Clear(terminal::ClearType::All))?,
        }
        self.last_key = Some(key);
        self.last_rect = Some(rect);
        queue!(self.out, cursor::MoveTo(0, 0))?;
        Ok(true)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }
}

pub fn run(
    initial_theme: Theme,
    initial_palette: Palette,
    initial_display: DisplayOptions,
) -> io::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = Arc::clone(&running);
    // The `termination` feature also covers SIGTERM and SIGHUP, so a supervisor
    // or a closing window cannot leave the terminal in raw mode.
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .map_err(|e| io::Error::other(e.to_string()))?;

    let _guard = TerminalGuard::enter()?;
    let mut stdout = BufWriter::new(io::stdout());
    let mut canvas = Canvas::new(&mut stdout);
    let mut theme = initial_theme;
    let mut palette = initial_palette;
    let mut display = initial_display;

    let mut repeat = RepeatFilter::default();

    while running.load(Ordering::SeqCst) {
        // Sleep exactly until something visible could change; input wakes us
        // early regardless.
        if event::poll(wait_time(&Local::now(), display))? {
            // Drain every queued event first so a burst of key presses is
            // never partially applied, then render once.
            while event::poll(Duration::ZERO)? {
                let Event::Key(key) = event::read()? else {
                    continue;
                };
                if !repeat.accepts(&key, Instant::now()) {
                    continue;
                }
                if handle_key(key, &mut theme, &mut palette, &mut display) {
                    running.store(false, Ordering::SeqCst);
                    break;
                }
            }
        }

        if !running.load(Ordering::SeqCst) {
            break;
        }

        let (cols, lines) = terminal::size().unwrap_or((80, 24));
        let look = Look::new(theme, palette, display, cols, lines);
        let key = look.key(&Local::now(), cols, lines);
        if key.plain {
            draw_plain(&mut canvas, &Local::now(), cols, lines, look, key)?;
        } else {
            draw_framed(&mut canvas, &Local::now(), cols, lines, look, key)?;
        }
    }

    canvas.flush()?;
    Ok(())
}

/// How long to block before the next frame *could* differ: the next blink
/// edge, or the next second when blinking is off. Sleeping until the boundary
/// keeps digit dwell times even instead of drifting by up to a poll interval.
fn wait_time(now: &DateTime<Local>, display: DisplayOptions) -> Duration {
    wait_from(u64::from(now.timestamp_subsec_millis()), display)
}

/// Same as [`wait_time`], but from a raw millisecond-within-second stamp.
fn wait_from(sub: u64, display: DisplayOptions) -> Duration {
    let until_change_ms = if display.blink_colon {
        // Blink edges are a superset of the second boundaries.
        BLINK_PERIOD_MS - sub % BLINK_PERIOD_MS
    } else if display.show_seconds {
        1000 - sub
    } else {
        MAX_WAIT_MS
    };
    MAX_WAIT.min(Duration::from_millis(until_change_ms.max(1)))
}

/// Collapses terminal auto-repeat into a single applied action.
#[derive(Default)]
struct RepeatFilter {
    /// Key currently held, tracked once the terminal reports event types.
    held: Option<KeyCode>,
    /// Set once a release has been seen, i.e. the terminal distinguishes a held
    /// key from a fresh press and latching becomes exact.
    tracks_releases: bool,
    /// Last applied press, used only by the legacy fallback.
    last: Option<(KeyCode, Instant)>,
}

impl RepeatFilter {
    /// Whether `key` should be applied now.
    fn accepts(&mut self, key: &KeyEvent, now: Instant) -> bool {
        match key.kind {
            KeyEventKind::Release => {
                if self.held == Some(key.code) {
                    self.held = None;
                }
                self.tracks_releases = true;
                return false;
            }
            KeyEventKind::Repeat => return false,
            KeyEventKind::Press => {}
        }

        if self.tracks_releases {
            // Exact: one application per physical press.
            if self.held == Some(key.code) {
                return false;
            }
            self.held = Some(key.code);
            return true;
        }

        if let Some((code, at)) = self.last
            && code == key.code
            && now.duration_since(at) < LEGACY_REPEAT_WINDOW
        {
            return false;
        }
        self.last = Some((key.code, now));
        true
    }
}

/// Applies one key press to the live options in place.
/// Returns `true` when the app should quit.
fn handle_key(
    key: KeyEvent,
    theme: &mut Theme,
    palette: &mut Palette,
    display: &mut DisplayOptions,
) -> bool {
    // Auto-repeat and key release must not re-apply an action: holding `s`
    // would otherwise toggle the seconds dozens of times per second.
    if key.kind != KeyEventKind::Press {
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
            } else if c.eq_ignore_ascii_case(&'i') {
                display.toggle_hint();
            }
            false
        }
        _ => false,
    }
}

/// Renders the plain one-line time for terminals too small for the framed face.
fn draw_plain<W: Write>(
    canvas: &mut Canvas<'_, W>,
    now: &DateTime<Local>,
    cols: u16,
    lines: u16,
    look: Look,
    key: FrameKey,
) -> io::Result<()> {
    let time = format_time(now, look.display).text;
    let text = if look.display.hour_format.is_12h() {
        format!("{time} {}", format_meridiem(now))
    } else {
        time
    };
    let visible: String = text.chars().take(usize::from(cols)).collect();
    let visible_w = visible.chars().count();
    let y = usize::from(lines).saturating_sub(1) / 2;
    let x = usize::from(cols).saturating_sub(visible_w) / 2;

    if !canvas.prepare(key, ui::FrameRect::new(x, y, visible_w, 1))? {
        return Ok(());
    }
    queue!(
        canvas.out,
        cursor::MoveTo(
            u16::try_from(x).unwrap_or(u16::MAX),
            u16::try_from(y).unwrap_or(u16::MAX)
        ),
        terminal::Clear(terminal::ClearType::CurrentLine),
        SetForegroundColor(look.palette.hours_minutes())
    )?;
    write!(canvas.out, "{visible}")?;
    queue!(canvas.out, ResetColor)?;
    canvas.flush()
}

/// Renders the framed digit face.
fn draw_framed<W: Write>(
    canvas: &mut Canvas<'_, W>,
    now: &DateTime<Local>,
    cols: u16,
    lines: u16,
    look: Look,
    key: FrameKey,
) -> io::Result<()> {
    let time = format_time(now, look.display);
    let date_line = if look.display.hour_format.is_12h() {
        format!("{} · {}", format_date(now), format_meridiem(now))
    } else {
        format_date(now)
    };
    let layout = ui::compute_layout(
        cols as usize,
        lines as usize,
        line_width(time.text.chars().count(), look.theme.style()),
        date_line.chars().count(),
        look.theme,
        look.display.face(),
    );
    let time_chars: Vec<char> = time.text.chars().collect();
    let frame = ui::Frame {
        rows: &render_big(&time.text, key.colon_visible, look.theme.style()),
        time_chars: &time_chars,
        seconds_start: time.seconds_start,
        date: look.display.show_date.then_some(date_line.as_str()),
    };

    if !canvas.prepare(key, ui::frame_rect(&layout))? {
        return Ok(());
    }
    ui::render(canvas.out, &layout, &frame, look.theme, look.palette)?;
    canvas.flush()
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

    /// Collects everything written so a single frame stays inspectable after
    /// the canvas flushed it.
    #[derive(Clone, Default)]
    struct Sink(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

    impl Sink {
        fn bytes(&self) -> Vec<u8> {
            self.0.lock().unwrap().clone()
        }

        fn text(&self) -> String {
            strip_ansi(&String::from_utf8(self.bytes()).unwrap())
        }
    }

    impl Write for Sink {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn look_for(cols: u16, lines: u16, display: DisplayOptions) -> Look {
        Look::new(Theme::Large, Palette::BlackRed, display, cols, lines)
    }

    fn test_key(cols: u16, lines: u16, display: DisplayOptions, now: &DateTime<Local>) -> FrameKey {
        look_for(cols, lines, display).key(now, cols, lines)
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
    fn release_and_repeat_events_are_ignored() {
        let (mut theme, mut palette, mut display) = default_state();
        for kind in [KeyEventKind::Release, KeyEventKind::Repeat] {
            for code in [KeyCode::Char('q'), KeyCode::Char('s')] {
                let event = KeyEvent::new_with_kind(code, KeyModifiers::NONE, kind);
                assert!(!handle_key(event, &mut theme, &mut palette, &mut display));
            }
        }
        assert_eq!(display, DisplayOptions::default());
    }

    #[test]
    fn holding_a_key_does_not_repeat_the_action() {
        let (mut theme, mut palette, mut display) = default_state();
        for key in ['s', 'p', 't', 'd', 'i'] {
            for _ in 0..40 {
                let event = KeyEvent::new_with_kind(
                    KeyCode::Char(key),
                    KeyModifiers::NONE,
                    KeyEventKind::Repeat,
                );
                assert!(!handle_key(event, &mut theme, &mut palette, &mut display));
            }
        }
        assert_eq!(theme, Theme::Large);
        assert_eq!(palette, Palette::BlackRed);
        assert_eq!(display, DisplayOptions::default());
    }

    fn key_event(code: KeyCode, kind: KeyEventKind) -> KeyEvent {
        KeyEvent::new_with_kind(code, KeyModifiers::NONE, kind)
    }

    #[test]
    fn repeat_filter_latches_when_the_terminal_reports_releases() {
        let start = Instant::now();
        let mut filter = RepeatFilter::default();
        let s = KeyCode::Char('s');

        assert!(filter.accepts(&key_event(s, KeyEventKind::Press), start));
        // A release tells us the key was let go, which arms exact tracking.
        assert!(!filter.accepts(&key_event(s, KeyEventKind::Release), start));
        assert!(filter.tracks_releases);

        // A fresh press after the release applies again...
        assert!(filter.accepts(&key_event(s, KeyEventKind::Press), start));
        // ...but every further press while it stays down does not.
        for step in 1..50 {
            let now = start + Duration::from_millis(step * 33);
            assert!(
                !filter.accepts(&key_event(s, KeyEventKind::Press), now),
                "held key applied again after {} ms",
                step * 33
            );
            assert!(!filter.accepts(&key_event(s, KeyEventKind::Repeat), now));
        }
        // Releasing and pressing again works, and another key is never blocked.
        assert!(!filter.accepts(&key_event(s, KeyEventKind::Release), start));
        assert!(filter.accepts(&key_event(s, KeyEventKind::Press), start));
        assert!(filter.accepts(&key_event(KeyCode::Char('p'), KeyEventKind::Press), start));
    }

    #[test]
    fn repeat_filter_falls_back_to_a_window_for_legacy_terminals() {
        let start = Instant::now();
        let mut filter = RepeatFilter::default();
        let s = KeyCode::Char('s');
        let press = |filter: &mut RepeatFilter, at: Duration| {
            filter.accepts(&key_event(s, KeyEventKind::Press), start + at)
        };

        assert!(press(&mut filter, Duration::ZERO));
        assert!(!press(&mut filter, Duration::from_millis(33)));
        let just_under = LEGACY_REPEAT_WINDOW
            .checked_sub(Duration::from_millis(1))
            .unwrap();
        assert!(!press(&mut filter, just_under));
        assert!(press(&mut filter, LEGACY_REPEAT_WINDOW));
        assert!(!filter.tracks_releases);
    }

    #[test]
    fn held_key_toggles_at_most_twice_per_second_on_legacy_terminals() {
        // A terminal without event types resends the same byte ~30 times per
        // second while a key is held. A one second hold must not storm.
        let mut theme = Theme::Large;
        let mut palette = Palette::BlackRed;
        let mut display = DisplayOptions::default();
        let mut filter = RepeatFilter::default();
        let mut applied = 0;
        let start = Instant::now();
        for step in 0..30 {
            let key = key_event(KeyCode::Char('s'), KeyEventKind::Press);
            if filter.accepts(&key, start + Duration::from_millis(step * 33))
                && !handle_key(key, &mut theme, &mut palette, &mut display)
            {
                applied += 1;
            }
        }
        assert!(
            applied <= 2,
            "a one second hold applied the action {applied} times"
        );
    }

    #[test]
    fn held_key_toggles_exactly_once_on_modern_terminals() {
        let mut theme = Theme::Large;
        let mut palette = Palette::BlackRed;
        let mut display = DisplayOptions::default();
        let mut filter = RepeatFilter::default();
        let mut applied = 0;
        let start = Instant::now();
        let s = KeyCode::Char('s');

        // A real hold on a modern terminal: one press, then repeat events,
        // then a single release when the key comes back up.
        let press = key_event(s, KeyEventKind::Press);
        if filter.accepts(&press, start)
            && !handle_key(press, &mut theme, &mut palette, &mut display)
        {
            applied += 1;
        }
        for step in 1..30 {
            let now = start + Duration::from_millis(step * 33);
            if filter.accepts(&key_event(s, KeyEventKind::Repeat), now) {
                applied += 1;
            }
        }
        filter.accepts(
            &key_event(s, KeyEventKind::Release),
            start + Duration::from_secs(1),
        );
        assert_eq!(applied, 1, "a held key must be applied exactly once");
        assert!(!display.show_seconds);
        // Tapping again after the release works.
        let again = key_event(s, KeyEventKind::Press);
        assert!(filter.accepts(&again, start + Duration::from_secs(2)));
        assert!(!handle_key(again, &mut theme, &mut palette, &mut display));
        assert!(display.show_seconds);
    }

    #[test]
    fn repeat_filter_rejects_release_and_repeat_events() {
        let start = Instant::now();
        let mut filter = RepeatFilter::default();
        for kind in [KeyEventKind::Release, KeyEventKind::Repeat] {
            assert!(!filter.accepts(&key_event(KeyCode::Char('s'), kind), start));
        }
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
        press('i', &mut theme, &mut palette, &mut display);
        assert!(!display.show_hint);
        press('I', &mut theme, &mut palette, &mut display);
        assert!(display.show_hint);
        // Unknown keys leave the state untouched.
        press('z', &mut theme, &mut palette, &mut display);
        assert_eq!(theme, Theme::Medium);
        assert_eq!(palette, Palette::Purple);
        assert!(!display.show_seconds);
    }

    #[test]
    fn wait_lands_on_the_next_visible_change() {
        let now = Local::now();
        let blink = DisplayOptions::default();
        let w = wait_time(&now, blink);
        assert!(w > Duration::ZERO);
        assert!(w <= MAX_WAIT);
        // With seconds hidden and no blinking nothing visible can change.
        let steady = DisplayOptions {
            blink_colon: false,
            show_seconds: false,
            ..DisplayOptions::default()
        };
        assert_eq!(wait_time(&now, steady), MAX_WAIT);
    }

    #[test]
    fn wait_never_overshoots_the_next_change() {
        // Every millisecond of a second, for every combination of blinking and
        // seconds: the wait must always land on a real boundary.
        for millis in 0..1000u64 {
            for blink in [true, false] {
                for seconds in [true, false] {
                    let display = DisplayOptions {
                        blink_colon: blink,
                        show_seconds: seconds,
                        ..DisplayOptions::default()
                    };
                    let w = wait_from(millis, display);
                    assert!(w > Duration::ZERO, "millis={millis}");
                    assert!(w <= MAX_WAIT, "millis={millis} wait={w:?}");
                    if blink {
                        assert!(w <= Duration::from_millis(BLINK_PERIOD_MS));
                    }
                    if seconds && !blink {
                        assert!(w <= Duration::from_millis(1000));
                    }
                }
            }
        }
    }

    #[test]
    fn framed_layout_is_used_wherever_a_face_fits() {
        let display = DisplayOptions::default();
        // The original bug: `large` on 80x24 used to be drawn clipped. It now
        // degrades to a face that fits and stays framed.
        assert!(look_for(80, 24, display).fits_framed(80, 24));
        assert_eq!(
            look_for(80, 24, display).theme.style().scale_x,
            1,
            "must degrade to single-width digits on 80 columns"
        );
        assert!(look_for(40, 24, display).fits_framed(40, 24));
        // Only when no face at all fits does the plain line take over.
        for (cols, lines) in [(10u16, 20u16), (20, 6), (23, 8), (35, 8)] {
            let look = look_for(cols, lines, display);
            assert!(!look.fits_framed(cols, lines), "{cols}x{lines}");
            assert!(look.key(&Local::now(), cols, lines).plain, "{cols}x{lines}");
        }
    }

    #[test]
    fn framed_key_differs_from_plain_key() {
        let display = DisplayOptions::default();
        let now = Local::now();
        let framed = test_key(120, 30, display, &now);
        let plain = test_key(10, 20, display, &now);
        assert!(!framed.plain);
        assert!(plain.plain);
        // The two modes share every other field, so the `plain` flag is the
        // only thing forcing a redraw when the terminal crosses the boundary.
        assert_eq!(framed.time, plain.time);
        assert_ne!(framed.plain, plain.plain);
    }

    #[test]
    fn draw_plain_writes_a_single_centre_line() {
        let now = Local::now();
        let display = DisplayOptions::default();
        let look = look_for(20, 5, display);
        let key = look.key(&now, 20, 5);
        let sink = Sink::default();
        {
            let mut handle = sink.clone();
            let mut canvas = Canvas::new(&mut handle);
            draw_plain(&mut canvas, &now, 20, 5, look, key).unwrap();
        }
        let plain = sink.text();
        assert_eq!(
            plain.lines().filter(|l| !l.trim().is_empty()).count(),
            1,
            "fallback draws exactly one line: {plain:?}"
        );
        assert!(!plain.contains('│'), "fallback must not draw a frame");
        assert!(plain.chars().count() <= 20);
    }

    #[test]
    fn draw_plain_skips_identical_frames() {
        let now = Local::now();
        let display = DisplayOptions {
            show_seconds: false,
            blink_colon: false,
            ..DisplayOptions::default()
        };
        let look = look_for(20, 5, display);
        let key = look.key(&now, 20, 5);
        let sink = Sink::default();
        let mut handle = sink.clone();
        let mut canvas = Canvas::new(&mut handle);
        draw_plain(&mut canvas, &now, 20, 5, look, key).unwrap();
        let after_first = sink.bytes().len();
        assert!(after_first > 0, "the first frame must draw something");
        draw_plain(&mut canvas, &now, 20, 5, look, key).unwrap();
        assert_eq!(
            sink.bytes().len(),
            after_first,
            "identical frame must not write anything"
        );
    }

    #[test]
    fn draw_plain_clamps_narrow_terminals() {
        let now = Local::now();
        for cols in [1u16, 2, 4, 8, 16, 40] {
            let display = DisplayOptions::default();
            let look = look_for(cols, 3, display);
            let key = look.key(&now, cols, 3);
            let sink = Sink::default();
            {
                let mut handle = sink.clone();
                let mut canvas = Canvas::new(&mut handle);
                draw_plain(&mut canvas, &now, cols, 3, look, key).unwrap();
            }
            for line in sink.text().lines() {
                assert!(line.chars().count() <= cols as usize, "cols={cols}");
            }
        }
    }

    #[test]
    fn framed_redraw_erases_only_the_old_frame() {
        let now = Local::now();
        let display = DisplayOptions::default();
        let look = look_for(120, 30, display);
        let sink = Sink::default();
        let mut handle = sink.clone();
        let mut canvas = Canvas::new(&mut handle);
        let key = look.key(&now, 120, 30);
        assert!(!key.plain);
        draw_framed(&mut canvas, &now, 120, 30, look, key).unwrap();
        assert!(
            String::from_utf8(sink.bytes())
                .unwrap()
                .contains("\u{1b}[2J"),
            "the very first frame clears the alternate screen"
        );

        // Same instant, seconds toggled: the layout shrinks, so a repaint is
        // due, but only the affected rows may be erased.
        let hidden = DisplayOptions {
            show_seconds: false,
            ..display
        };
        let look2 = Look::new(Theme::Large, Palette::BlackRed, hidden, 120, 30);
        let key2 = look2.key(&now, 120, 30);
        let before = sink.bytes().len();
        draw_framed(&mut canvas, &now, 120, 30, look2, key2).unwrap();
        let text = String::from_utf8(sink.bytes()[before..].to_vec()).unwrap();
        assert!(
            !text.contains("\u{1b}[2J"),
            "a layout change must not clear the whole screen"
        );
        assert!(text.contains("\u{1b}[K"), "old rows must be erased");
        assert!(text.contains('╭'), "new frame must be drawn");
    }

    #[test]
    fn first_frame_clears_the_alternate_screen() {
        let now = Local::now();
        let display = DisplayOptions::default();
        let look = look_for(120, 30, display);
        let key = look.key(&now, 120, 30);
        let sink = Sink::default();
        {
            let mut handle = sink.clone();
            let mut canvas = Canvas::new(&mut handle);
            draw_framed(&mut canvas, &now, 120, 30, look, key).unwrap();
        }
        let text = String::from_utf8(sink.bytes()).unwrap();
        assert_eq!(
            text.matches("\u{1b}[2J").count(),
            1,
            "the screen is cleared once, not per frame"
        );
    }

    fn strip_ansi(text: &str) -> String {
        let mut out = String::new();
        let mut chars = text.chars();
        while let Some(c) = chars.next() {
            if c == '\u{1b}' {
                for c in chars.by_ref() {
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            } else {
                out.push(c);
            }
        }
        out
    }
}
