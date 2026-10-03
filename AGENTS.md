# AGENTS.md

Single-binary Rust TUI clock (`xclock`, edition 2024, Rust 1.98+). No workspace, no CI, no codegen.

## Commands

- Run: `cargo run --release [-- --theme <modern|classic>] [--show-seconds | --no-seconds]`
- Dev check: `cargo build && cargo test && cargo clippy --all-targets` — clippy must stay warning-free.
- Tests are unit tests co-located in `src/**/`: `cargo test` (25 tests). No integration suite, fixtures, or services.

## Architecture (`src/`)

- `main.rs` — module wiring + exit codes only: `0` ok, `1` runtime error, `2` CLI parse error. Put no logic here.
- `cli.rs` — hand-rolled arg parser (no clap). Supports `--theme/-t <name>`, `--theme=<name>`, `--show-seconds` (default), `--no-seconds` / `--hide-seconds`, `--list-themes`, `-h/--help`. Last seconds flag wins.
- `app.rs` — event loop: 100 ms `event::poll`, redraw only when `(time, colon_visible, cols, lines, theme, show_seconds)` changes. Colon blink = `timestamp_subsec_millis() < 500`. Terminal size fallback `(80, 24)`.
- `terminal.rs` — `TerminalGuard::enter()` RAII: raw mode + alternate screen + hide cursor; `Drop` restores all. Never bypass it in new entrypoints.
- `theme.rs` — exactly 2 themes (`Modern`, `Classic`), both `Face::Block`, differing only in `scale_x`/`gap_x`. `SECONDS_START = 5`: chars `0..5` (`HH:MM`) white, `5..` cyan.
- `font/` — `Face::Block` is the only face (5x7 glyphs); size differences come from `Style{scale_x, gap_x}`. `display/` — `DisplayOptions{show_seconds}` + `format_time`.
- `ui/` — `layout::compute_layout` centers and clamps to narrow screens; `render` draws rounded frame + date + hint. `locale.rs` — English-only date strings.
- Live keys (in `app.rs`): `1`/`2` switch theme via `Theme::from_hotkey`, `s` toggles seconds, `q`/`Esc` quit.

## Conventions

- English only: code, comments, identifiers, commit messages, docs. No non-English strings except in throwaway local notes (never committed).
- Minimal comments: explain only non-obvious logic (e.g. raw-mode signal handling, redraw-skip key). No restating code, no doc-comments on self-evident items.
- One feature per file: new feature = new module file under the owning directory (`font/`, `ui/`, `display/`) + `mod` + re-export, wired via `main.rs`. Do not pile unrelated logic into existing files.

## Gotchas

- Raw mode swallows SIGINT: `Ctrl+C` arrives as a key event (handled in `app.rs`) *and* via the `ctrlc` `AtomicBool`. Keep both paths; removing the handler hangs exit under raw mode.
- Adding a theme requires touching `Theme::all/id/description/hotkey/style`, `cli` usage text, and live-key handling together — hotkeys must stay unique (covered by `hotkeys_cover_all_themes_uniquely` test).
- Do not run the TUI to "verify" rendering in automation — rely on `cargo test`, especially `ui::render` narrow-screen tests. TUI needs ANSI/UTF-8 terminal (`█ ▮ ─ │ ╭ ╮ ╰ ╯`).
- `README.md` test count is stale (says 21, actually 25). Trust `cargo test` output.
