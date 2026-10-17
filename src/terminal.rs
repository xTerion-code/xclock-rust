use std::io;
use std::panic::{self, PanicHookInfo};
use std::sync::{Arc, Mutex};

use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::{cursor, execute, terminal};

/// Asks the terminal to label key presses, repeats and releases separately.
///
/// Without this, a terminal resends the same byte while a key is held and the
/// only way to suppress auto-repeat is a timeout. Terminals that do not
/// understand the sequence ignore it, and `app` falls back to its window.
const KEYBOARD_FLAGS: KeyboardEnhancementFlags = KeyboardEnhancementFlags::REPORT_EVENT_TYPES;

/// The signature `panic::take_hook` hands back and `set_hook` accepts.
type PanicHook = Box<dyn Fn(&PanicHookInfo<'_>) + Sync + Send + 'static>;

/// The hook that was installed before the guard, shared between the guard and
/// the closure it installs so either side can consume it exactly once.
type SharedHook = Arc<Mutex<Option<PanicHook>>>;

/// Restores the terminal when dropped, and while panicking.
///
/// Raw mode and the alternate screen are process-wide side effects, so they are
/// unwound together: `Drop` covers normal returns and errors, the panic hook
/// covers unwinding, and the previously installed hook is restored so the guard
/// can be entered and left more than once per process.
pub struct TerminalGuard {
    prev_hook: Option<SharedHook>,
}

impl TerminalGuard {
    pub fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut out = io::stdout();
        if let Err(e) = execute!(out, terminal::EnterAlternateScreen, cursor::Hide) {
            let _ = terminal::disable_raw_mode();
            return Err(e);
        }
        // Best effort: failing to ask for key event types only costs us exact
        // auto-repeat handling, so it must never abort the start-up.
        let _ = execute!(out, PushKeyboardEnhancementFlags(KEYBOARD_FLAGS));

        // Installed only after the terminal is actually in raw mode, so a
        // failed `enter` never leaves a cleanup hook behind.
        let shared: SharedHook = Arc::new(Mutex::new(Some(panic::take_hook())));
        let hook_hook = Arc::clone(&shared);
        panic::set_hook(Box::new(move |info| {
            restore_terminal();
            if let Some(prev) = take_hook(&hook_hook) {
                prev(info);
            }
        }));
        Ok(Self {
            prev_hook: Some(shared),
        })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();
        if let Some(shared) = self.prev_hook.take()
            && let Some(prev) = take_hook(&shared)
        {
            // Remove our teardown wrapper before reinstalling the original.
            let _ = panic::take_hook();
            panic::set_hook(prev);
        }
        // If the slot was empty the teardown hook already fired while
        // panicking and consumed the original; it stays installed, and it is
        // idempotent, so repeated panics keep restoring the terminal.
    }
}

/// Consumes the shared hook exactly once, tolerating a poisoned lock.
fn take_hook(shared: &SharedHook) -> Option<PanicHook> {
    shared.lock().ok().and_then(|mut slot| slot.take())
}

/// Best-effort teardown; every step is ignored because there is nothing useful
/// to do with a failure while the terminal is being abandoned.
fn restore_terminal() {
    let _ = terminal::disable_raw_mode();
    let mut out = io::stdout();
    let _ = execute!(out, PopKeyboardEnhancementFlags);
    let _ = execute!(out, cursor::Show, terminal::LeaveAlternateScreen);
}
