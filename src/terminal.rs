use std::io;

use crossterm::{cursor, execute, terminal};

/// Enters raw mode + alternate screen and hides the cursor on creation,
/// restores everything on drop — even on early `?` return or panic unwind.
///
/// Raw mode is required for instant key handling (`1`/`2`/`3`, `q`);
/// note that in raw mode `Ctrl+C` arrives as a key event (no SIGINT),
/// so it is handled in the event loop in `app`.
pub struct TerminalGuard;

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
