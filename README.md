# xclock-rust

Terminal clock in Rust: large block digit face, date with weekday, frame and blinking colon on crossterm.

## Features

- Two themes: `modern` (current large double-width block digits),
  `classic` (old single-width block digits)
- `HH:MM:SS` time format, colon blinking at 1 Hz
- Date under the clock: `September 30, 2026 · Wednesday`
- Colors: `HH:MM` in bold white, seconds in cyan, frame and separators dimmed
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
cargo run --release -- --theme classic
cargo run --release -- -t modern
```

Available themes (`--list-themes`): `modern`, `classic`.

Exit: `Ctrl+C` or `q`/`Esc`. Switch theme live with `1` / `2`.

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
│                                         1/2 theme · q quit                                           │
│                                                                                                      │
╰──────────────────────────────────────────────────────────────────────────────────────────────────────╯
```

Rendering depends on the terminal font; glyphs are `█` and `▮`.

## Project structure

```text
src/
  main.rs       — module wiring, CLI dispatch, calls app::run(theme)
  cli.rs        — argument parsing: --theme/-t, --list-themes, --help
  app.rs        — application loop, raw-mode keys, Ctrl+C, redraw on change
  terminal.rs   — terminal lifecycle: raw mode, alternate screen, cursor
  font/         — typefaces: mod (Face/Style), block (block_glyph),
                  render (render_big, line_width)
  locale.rs     — date: weekday_name(), month_name(), format_date()
  theme.rs      — themes: Theme (modern/classic), palettes, style(), time_color()
  ui/           — geometry and drawing: mod, layout (Layout, compute_layout),
                  render (render)
```

Extension points:

- new typeface — only `src/font/`
- different date localization — only `src/locale.rs`
- different palette — only `src/theme.rs`
- different layout/frame — only `src/ui/`

## Development

```bash
cargo build
cargo test
cargo clippy --all-targets
```

Verified: `cargo build`, `cargo test` (21 tests) and `cargo clippy --all-targets` with no warnings.

## Dependencies

- `chrono 0.4` — local time and date
- `crossterm 0.29` — alternate screen, cursor, colors
- `ctrlc 3` — graceful `Ctrl+C` exit
