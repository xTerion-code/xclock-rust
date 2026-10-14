# xclock-rust

Terminal clock in Rust: large block digit face, date with weekday, frame and blinking colon on crossterm.

## Features

- Three themes: `large` (large double-width block digits for wide terminals),
  `medium` (single-width block digits for standard terminals),
  `small` (compact 3x5 digits for small terminals)
- `HH:MM:SS` time format, colon blinking at 1 Hz
- 12-hour format with `AM`/`PM` (`--12h`, live key `h`)
- Steady colon (`--no-blink`, live key `b`), hidden date (`--no-date`, live key `d`)
- Date under the clock: `September 30, 2026 · Wednesday`
- Colors: `HH:MM` in vivid purple, seconds in light lavender, frame in dark purple on black
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
cargo run --release -- --theme medium
cargo run --release -- -t small
cargo run --release -- -t large
```

Available themes (`--list-themes`): `large`, `medium`, `small`.

Exit: `Ctrl+C` or `q`/`Esc`. Live keys: `1`/`2`/`3` theme, `s` seconds, `h` 12/24h, `b` blink, `d` date.

## Examples

`large` (default):

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
│                     1/2/3 theme · s seconds · h 12/24h · b blink · d date · q quit                     │
│                                                                                                      │
╰──────────────────────────────────────────────────────────────────────────────────────────────────────╯
```

Rendering depends on the terminal font; glyphs are `█` and `▮`.

## Project structure

```text
src/
  main.rs       — module wiring, CLI dispatch, calls app::run(theme, display)
  cli.rs        — argument parsing: --theme/-t, --show-seconds/--no-seconds,
                  --12h/--24h/--hour-format, --blink/--no-blink,
                  --show-date/--no-date, --list-themes, --help
  app.rs        — application loop, raw-mode keys, Ctrl+C, redraw on change
  terminal.rs   — terminal lifecycle: raw mode, alternate screen, cursor
  display/      — options (DisplayOptions), hour format (HourFormat),
                  format_time, format_meridiem
  font/         — typefaces: mod (Face/Style), block (block_glyph),
                  render (render_big, line_width)
  locale.rs     — date: weekday_name(), month_name(), format_date()
  theme.rs      — themes: Theme (large/medium/small), palettes, style(), time_color()
  ui/           — geometry and drawing: mod, layout (Layout, HINT, compute_layout),
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

Verified: `cargo build`, `cargo test` (38 tests) and `cargo clippy --all-targets` with no warnings.

## Dependencies

- `chrono 0.4` — local time and date
- `crossterm 0.29` — alternate screen, cursor, colors
- `ctrlc 3` — graceful `Ctrl+C` exit
