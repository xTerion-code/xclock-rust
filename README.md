# xclock-rust

Terminal clock in Rust: large block digit face, date with weekday, frame and blinking colon on crossterm.

## Features

- Three themes: `modern` (current large double-width block digits),
  `classic` (old single-width block digits), `compact` (tiny 3×3 plain-ASCII digits)
- `HH:MM:SS` time format, colon blinking at 1 Hz
- Date under the clock: `September 30, 2026 · Wednesday`
- Colors: `HH:MM` in bold white, seconds in cyan, frame and separators dimmed
  (`compact` uses a green phosphor palette instead)
- Rounded frame `╭─╮│╰─╯` around the clock and date block
- Alternate screen: the terminal is restored on exit
- `Ctrl+C` exit with cursor and screen restored

## Requirements

- Rust 1.98+ / Cargo (edition 2024)
- Unix terminal with ANSI and UTF-8 support (frame and `█ ▮ ─ │ ╭ ╮ ╰ ╯`)

## Install and run

```bash
git clone git@github.com:xTerion-code/xclock-rust.git
cd xclock-rust
cargo run --release
```

With a theme:

```bash
cargo run --release -- --theme compact
cargo run --release -- -t classic
```

Available themes (`--list-themes`): `modern`, `classic`, `compact`.

Exit: `Ctrl+C` or `q`/`Esc`. Switch theme live with `1` / `2` / `3`.

## Examples

`modern` (default):

```text
╭──────────────────────────────────────────────────────────────────────────────────────────────────────╮
│                                                                                                      │
│        ██      ██████████              ██████████  ██      ██              ██████████  ██████████    │
│      ████              ██                      ██  ██      ██              ██          ██            │
│        ██              ██      ▮▮              ██  ██      ██      ▮▮      ██          ██            │
│        ██      ██████████                ████████  ██████████              ██████████  ██████████    │
│        ██      ██                              ██          ██                      ██  ██      ██    │
│        ██      ██              ▮▮              ██          ██      ▮▮              ██  ██      ██    │
│    ██████████  ██████████              ██████████          ██              ██████████  ██████████    │
│                                                                                                      │
│                                    September 30, 2026 · Wednesday                                    │
│                                         1/2/3 theme · q quit                                         │
│                                                                                                      │
╰──────────────────────────────────────────────────────────────────────────────────────────────────────╯
```

`compact` (`--theme compact`):

```text
╭───────────────────────────────────────╮
│                                       │
│         _       _           _   _     │
│      |  _|  .   _| |_|  .  |_  |_     │
│      | |_   .   _|   |  .   _| |_|    │
│                                       │
│    September 30, 2026 · Wednesday     │
│         1/2/3 theme · q quit          │
│                                       │
╰───────────────────────────────────────╯
```

Rendering depends on the terminal font; glyphs are `█` and `▮`.

## Project structure

```text
src/
  main.rs    — module wiring, CLI dispatch, calls app::run(theme)
  cli.rs     — argument parsing: --theme/-t, --list-themes, --help
  app.rs     — application loop, terminal guard, raw-mode keys, Ctrl+C, redraw on change
  font.rs    — typefaces: Face/Style, block_glyph(), compact_glyph(), render_big(), line_width()
  locale.rs  — date: weekday_name(), month_name(), format_date()
  theme.rs   — themes: Theme (modern/classic/compact), palettes, style(), time_color()
  ui.rs      — geometry and drawing: Layout, compute_layout(), render()
```

Extension points:

- new typeface — only `src/font.rs`
- different date localization — only `src/locale.rs`
- different palette — only `src/theme.rs`
- different layout/frame — only `src/ui.rs`

## Development

```bash
cargo build
cargo test
cargo clippy --all-targets
```

Verified: `cargo build`, `cargo test` (25 tests) and `cargo clippy --all-targets` with no warnings.

## Dependencies

- `chrono 0.4` — local time and date
- `crossterm 0.29` — alternate screen, cursor, colors
- `ctrlc 3` — graceful `Ctrl+C` exit
