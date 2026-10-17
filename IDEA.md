# xClock — development notes

`xclock` is a terminal digital clock written in Rust on top of `crossterm`.
It draws a block digit clockface inside a rounded Unicode frame, adapts to the
live terminal size, and can be reconfigured entirely from the keyboard.

## Architecture

The crate is a single binary with one dependency direction. Nothing imports
upwards, and there are no cycles.

```text
font     glyph tables, Face/Style geometry, FaceLayout      (no deps)
parse    shared ParseError for FromStr                       (no deps)
display  DisplayOptions, TimeFace, time/date formatting       (font)
theme    Theme, ResolvedTheme, Palette colour table          (font, parse)
ui       Layout, Frame, FrameRect, frame rendering           (font, theme)
terminal TerminalGuard: raw mode, alt screen, panic hook     (crossterm)
cli      argument parsing, usage and catalog listings        (display, theme, ui)
app      event loop, frame cache, live controls              (everything)
main     maps the parsed action onto an exit code
```

Rules that keep this shape intact:

- **Geometry flows through `FaceLayout`.** Every optional element (`seconds`,
  `date`, `hint`) is described once by `font::FaceLayout`, derived from
  `DisplayOptions::face()`. Width budgets, height budgets and the render pass
  all read that one value, so they cannot disagree about the frame contents.
- **Only `ResolvedTheme` has geometry.** `Theme::Auto` has no face of its own.
  `Theme::resolve` converts it (and degrades a concrete theme that does not
  fit, so digits are never clipped); `ui` and `font` geometry only ever accept
  a `ResolvedTheme`. Do not reintroduce a path that takes a raw `Theme` where
  geometry is computed.
- **A theme fits or degrades, it never clips.** `ResolvedTheme::fits` is the
  single definition of "this face fits". `app` consults it before choosing the
  framed layout, and `ui::compute_layout` clamps to the same budget. The
  invariant is covered by `fitting_theme_never_clips_the_face`.
- **The hint row is chosen, never truncated.** `ui::pick_hint` returns the
  longest reminder that fits and `None` when none does; the row is then absent
  from the height budget as well. `HINT_FULL` / `HINT_SHORT` are the single
  source of truth and `cli` prints them verbatim, so `--help` cannot drift.
- **Frames are erased by rectangle.** `Canvas::prepare` erases the union of the
  previous and the current `FrameRect`. `Clear(All)` happens exactly once, on
  the first frame. Never reintroduce a full clear on a layout change.
- **Rendering is a pure function of a cache key.** `FrameKey` covers time,
  colon phase, terminal size, resolved theme, display options, palette and the
  render mode. If the key did not change, nothing is written.

## Conventions

- `#![forbid(unsafe_code)]` is set in `main.rs`; there is no `unsafe` block.
- No `unwrap`, `expect` or `panic!` outside `#[cfg(test)]` in production code.
  Every width and height computation is saturating or checked.
- Comments explain *why*, not *what*. Layout arithmetic and the raw-mode
  constraints are the things worth a comment; `let x = y` is not.
- Names are spelled out: `clock_w`, `inner_w`, `pad_x`, `seconds_start`.
  Abbreviations are reserved for loop indices.
- Every public behaviour change comes with a test in the same module. There are
  no integration tests; `app::Canvas` is generic over `Write` so the render
  paths are exercised against a buffer.
- `cargo fmt` and `cargo clippy --all-targets` must be clean, including with
  `-W clippy::pedantic`. Where a pedantic lint is wrong for this code, allow it
  locally with a `reason =` string.

## Terminal lifecycle

`TerminalGuard` owns everything process-wide: raw mode, the alternate screen,
the hidden cursor, the pushed keyboard-enhancement flags and the panic hook. It
unwinds through `Drop` on both the success and the error path, and through the
panic hook while unwinding, restoring the previously installed hook either way.

`ctrlc` is built with the `termination` feature, so SIGINT, SIGTERM and SIGHUP
all reach the event loop and exit cleanly. `SIGQUIT`, `SIGKILL` and `SIGSTOP`
are not trapped by `ctrlc`; for the first two `reset` is the documented
recovery.

## Keyboard

The app asks for `REPORT_EVENT_TYPES` so a held key is reported once as
`Press`, then as `Repeat`. `app::RepeatFilter` ignores `Repeat` and `Release`
outright, and latches a key until its release arrives. Terminals that decline
the request resend the raw byte instead, so the filter falls back to a
`LEGACY_REPEAT_WINDOW`. Either way a held key is applied at most twice a
second, and exactly once on a terminal that reports event types.

Live keys are declared in `ui::HINT_FULL` / `ui::HINT_SHORT` and implemented in
`app::handle_key`. `Theme::hotkeys` and `theme::tests` guard against a theme
hotkey colliding with an action key.

## Testing

`cargo test` runs the whole suite. Notable invariants:

- `theme::fits_accounts_for_seconds_date_and_hint` — fit budgets per element.
- `theme::resolve_degrades_instead_of_clipping` — the original clipped-clock bug.
- `ui::fitting_theme_never_clips_the_face` — exhaustive over every theme, every
  `FaceLayout` and every terminal from 1x1 to 160x40.
- `ui::hint_degrades_then_disappears` / `hint_never_drives_the_frame_wider_than_the_screen`.
- `app::held_key_toggles_exactly_once_on_modern_terminals` and the legacy variant.
- `app::framed_redraw_erases_only_the_old_frame` — no `Clear(All)` after the first frame.

## Before opening a pull request

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
```

CI runs those plus `cargo audit`. README changes are part of the change: the
flag table, the key table and the theme geometry table are generated from the
same constants the binary uses, so keep them in sync.