use std::io;
use std::panic;

use crossterm::{cursor, execute, terminal};

pub struct TerminalGuard;

// In raw mode Ctrl+C arrives as a key event (no SIGINT), handled in `app`.
impl TerminalGuard {
    pub fn enter() -> io::Result<Self> {
        let prev_hook = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            let _ = terminal::disable_raw_mode();
            let mut out = io::stdout();
            let _ = execute!(out, cursor::Show, terminal::LeaveAlternateScreen);
            prev_hook(info);
        }));

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
        let _ = execute!(out, cursor::Show, terminal::LeaveAlternateScreen);
    }
}
