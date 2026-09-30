# xclock-rust

Terminal clock in Rust: large block digit face, date with weekday, frame and blinking colon on crossterm.

## Features

- Large ASCII digits 7×5, centered to the terminal size
- `HH:MM:SS` time format, colon blinking at 1 Hz
- Date under the clock: `September 30, 2026 · Tuesday`
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

Exit: `Ctrl+C`.

## Example

```text
╭───────────────────────────────────────────────────────╮
│                                                       │
│    ███  █   █ █████ █   █ █████       ████ █████       │
│   █   █ █   █     █ █   █ █               █ █   █      │
│   █   █ █   █     █ █   █ █        ▮      █ █   █      │
│   █   █ █████  ████ █████ ████            █ █████      │
│   █   █     █     █     █     █                   │
│   █   █     █     █     █     █        ▮      █     █  │
│    ███      █ █████     █ █████           █ █████      │
│                                                       │
│              September 30, 2026 · Tuesday             │
│                                                       │
╰───────────────────────────────────────────────────────╯
```

Rendering depends on the terminal font; glyphs are `█` and `▮`.

## Project structure

```text
src/
  main.rs    — module wiring, calls app::run()
  app.rs     — application loop, terminal, Ctrl+C, 100 ms frame
  font.rs    — typeface: GLYPH_H/W, glyph(), render_big(), line_width()
  locale.rs  — date: weekday_name(), month_name(), format_date()
  theme.rs   — colors: BORDER/SEPARATOR/DATE, time_char_color()
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
cargo clippy --all-targets
```

Verified: `cargo build` and `cargo clippy --all-targets` with no warnings.

## Dependencies

- `chrono 0.4` — local time and date
- `crossterm 0.29` — alternate screen, cursor, colors
- `ctrlc 3` — graceful `Ctrl+C` exit
