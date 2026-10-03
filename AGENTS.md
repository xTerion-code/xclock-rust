# AGENTS.md

Single-binary Rust TUI clock (`xclock`, edition 2024, Rust 1.98+). No workspace, no CI, no codegen.

## Commands

- Run: `cargo run --release [-- --theme <modern|classic>] [--show-seconds | --no-seconds]`
- Dev check: `cargo build && cargo test && cargo clippy --all-targets` — clippy must stay warning-free.
- Tests are unit tests co-located in `src/**/`: `cargo test` (38 tests). No integration suite, fixtures, or services.

## Architecture (`src/`)

- `main.rs` — module wiring + exit codes only: `0` ok, `1` runtime error, `2` CLI parse error. Put no logic here.
- `cli.rs` — hand-rolled arg parser (no clap). Supports `--theme/-t <name>`, `--theme=<name>`, `--show-seconds` (default) / `--no-seconds` / `--hide-seconds`, `--12h` / `--24h` (default) / `--hour-format <12|24>`, `--blink` (default) / `--no-blink`, `--show-date` (default) / `--no-date` / `--hide-date`, `--list-themes`, `-h/--help`. Last flag wins per group.
- `app.rs` — event loop: 100 ms `event::poll`, redraw only when `(time, colon_visible, cols, lines, theme, display)` changes (`display` is the full `DisplayOptions`). Colon blink = `timestamp_subsec_millis() < 500` unless `--no-blink`. Terminal size fallback `(80, 24)`. In 12h mode the date line gains `· AM/PM` via `format_meridiem`.
- `terminal.rs` — `TerminalGuard::enter()` RAII: raw mode + alternate screen + hide cursor; `Drop` restores all. Never bypass it in new entrypoints.
- `theme.rs` — exactly 2 themes (`Modern`, `Classic`), both `Face::Block`, differing only in `scale_x`/`gap_x`. `SECONDS_START = 5`: chars `0..5` (`HH:MM`) white, `5..` cyan.
- `font/` — `Face::Block` is the only face (5x7 glyphs); size differences come from `Style{scale_x, gap_x}`. `display/` — `DisplayOptions{show_seconds, hour_format, blink_colon, show_date}` + toggles, `HourFormat{H24, H12}`, `format_time` + `format_meridiem`.
- `ui/` — `layout::compute_layout` centers and clamps to narrow screens; inner width follows `max(clock, date, HINT)` + padding, hidden date shrinks block height by one row; `render` draws rounded frame + optional date + hint. `locale.rs` — English-only date strings.
- Live keys (in `app.rs`): `1`/`2` switch theme via `Theme::from_hotkey`, `s` toggles seconds, `h` toggles 12/24h, `b` toggles blink, `d` toggles date, `q`/`Esc` quit.

## Conventions

- English only: code, comments, identifiers, commit messages, docs. No non-English strings except in throwaway local notes (never committed).
- Minimal comments: explain only non-obvious logic (e.g. raw-mode signal handling, redraw-skip key). No restating code, no doc-comments on self-evident items.
- One feature per file: new feature = new module file under the owning directory (`font/`, `ui/`, `display/`) + `mod` + re-export, wired via `main.rs`. Do not pile unrelated logic into existing files.

## Gotchas

- Raw mode swallows SIGINT: `Ctrl+C` arrives as a key event (handled in `app.rs`) *and* via the `ctrlc` `AtomicBool`. Keep both paths; removing the handler hangs exit under raw mode.
- Adding a theme requires touching `Theme::all/id/description/hotkey/style`, `cli` usage text, and live-key handling together — hotkeys must stay unique and not collide with action keys `s/h/b/d/q` (covered by `hotkeys_cover_all_themes_uniquely` test).
- Do not run the TUI to "verify" rendering in automation — rely on `cargo test`, especially `ui::render` narrow-screen tests. TUI needs ANSI/UTF-8 terminal (`█ ▮ ─ │ ╭ ╮ ╰ ╯`). Trust `cargo test` output for the test count.
