use std::io;

use crossterm::{cursor, execute, terminal};

pub struct TerminalGuard;

// In raw mode Ctrl+C arrives as a key event (no SIGINT), handled in `app`.
impl TerminalGuard {
    pub fn enter() -> io::Result<Self> {
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
