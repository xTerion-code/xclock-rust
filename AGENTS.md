# AGENTS.md

Single-binary Rust TUI clock (`xclock`, edition 2024, Rust 1.98+). No workspace, no CI, no codegen.

## Commands

- Run: `cargo run --release [-- --theme <large|medium|small|auto>] [--palette <purple|matrix|amber|monochrome|rainbow>] [--show-seconds | --no-seconds]`
- Dev check: `cargo build && cargo test && cargo clippy --all-targets` — clippy must stay warning-free.
- Tests are unit tests co-located in `src/`: `cargo test` (47 tests). No integration suite, fixtures, or services.

## Architecture (`src/`)

- `main.rs` — module wiring + exit codes only: `0` ok, `1` runtime error, `2` CLI parse error. Put no logic here.
- `cli.rs` — hand-rolled arg parser (no clap). Supports `--theme/-t <name>`, `--theme=<name>`, `--palette/-p <name>`, `--palette=<name>`, `--show-seconds` (default) / `--no-seconds` / `--hide-seconds`, `--12h` / `--24h` (default) / `--hour-format <12|24>`, `--blink` (default) / `--no-blink`, `--show-date` (default) / `--no-date` / `--hide-date`, `--list-themes`, `--list-palettes`, `-h/--help`. Last flag wins per group.
- `app.rs` — event loop: 100 ms `event::poll`, redraw only when `(time, colon_visible, cols, lines, effective_theme, display, palette)` changes (`display` is the full `DisplayOptions`). Colon blink = `timestamp_subsec_millis() < 500` unless `--no-blink`. Terminal size fallback `(80, 24)`. In 12h mode the date line gains `· AM/PM` via `format_meridiem`.
- `terminal.rs` — `TerminalGuard::enter()` RAII: raw mode + alternate screen + hide cursor; `Drop` restores all. Never bypass it in new entrypoints.
- `theme.rs` — 4 themes (`Large`, `Medium`, `Small`, `Auto`): `Large` (5x7 `Face::Block`, `scale_x: 2`), `Medium` (5x7 `Face::Block`, `scale_x: 1`), `Small` (3x5 `Face::Compact`, `scale_x: 1`), `Auto` (adapts theme to terminal size via `auto_fit`). 5 palettes (`Purple`, `Matrix`, `Amber`, `Monochrome`, `Rainbow`). `SECONDS_START = 5`.
- `font.rs` — `Face::Block` (5x7) and `Face::Compact` (3x5), `Style{face, scale_x, gap_x}`, glyph tables, and text rasterization (`render_big`, `line_width`).
- `display.rs` — `DisplayOptions{show_seconds, hour_format, blink_colon, show_date}` + toggles, `HourFormat{H24, H12}`, time formatting (`format_time`, `format_meridiem`), and date formatting (`format_date`).
- `ui.rs` — `compute_layout` centers and clamps to narrow screens; inner width follows `max(clock, date, HINT)` + padding, hidden date shrinks block height by one row; `render` draws rounded frame + optional date + hint with active palette.
- Live keys (in `app.rs`): `1`/`2`/`3`/`a` switch theme via `Theme::from_hotkey`, `p` cycles palette, `s` toggles seconds, `h` toggles 12/24h, `b` toggles blink, `d` toggles date, `q`/`Esc` quit.

## Conventions

- English only: code, comments, identifiers, commit messages, docs. No non-English strings except in throwaway local notes (never committed).
- Minimal comments: explain only non-obvious logic (e.g. raw-mode signal handling, redraw-skip key). No restating code, no doc-comments on self-evident items.
- Cohesive domain modules: Keep modules organized around clear domain boundaries (`cli`, `terminal`, `theme`, `font`, `display`, `ui`, `app`) rather than splintering tightly coupled types across micro-files.

## Gotchas

- Raw mode swallows SIGINT: `Ctrl+C` arrives as a key event (handled in `app.rs`) *and* via the `ctrlc` `AtomicBool`. Keep both paths; removing the handler hangs exit under raw mode.
- Adding a theme requires touching `Theme::all/id/description/hotkey/style`, `cli` usage text, and live-key handling together — hotkeys must stay unique and not collide with action keys `s/h/b/d/q` (covered by `hotkeys_cover_all_themes_uniquely` test).
- Do not run the TUI to "verify" rendering in automation — rely on `cargo test`, especially `ui::render` narrow-screen tests. TUI needs ANSI/UTF-8 terminal (`█ ▮ ─ │ ╭ ╮ ╰ ╯`). Trust `cargo test` output for the test count.
