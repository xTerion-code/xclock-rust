# xclock — Rust Terminal Digital Clock

[![Rust](https://img.shields.io/badge/Rust-1.98%2B-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Edition](https://img.shields.io/badge/Edition-2024-orange?style=for-the-badge&logo=rust&logoColor=white)](https://doc.rust-lang.org/edition-guide/rust-2024/)
[![Terminal](https://img.shields.io/badge/Terminal-TUI-41CD52?style=for-the-badge&logo=gnome-terminals&logoColor=white)](https://gnome-terminal.org/)
[![License](https://img.shields.io/badge/License-MIT-b91c1c?style=for-the-badge&logo=open-source-initiative&logoColor=white)](https://github.com/xTerionsilker/xClock-Rust)
[![Tests](https://img.shields.io/badge/tests-53%20passed-4CAF50?style=for-the-badge&logo=rust&logoColor=white)](https://github.com/xTerionsilker/xClock-Rust)

**xclock** (*x* clock) is a terminal digital clock written in Rust on top of [`crossterm`](https://crates.io/crates/crossterm), drawing bold typographic block-digit clockfaces inside a rounded Unicode frame. It combines real-time responsive layout adaptation, six truecolor CRT/phosphor palettes, flexible time and date formatting, and live keyboard controls — all rendered flicker-free with single-flush buffered output.

Current version is **0.1.0**. The whole UI follows a typographic pixel-craft aesthetic: hand-authored 5x7 and 3x5 glyph tables, box-drawing frames, and 24-bit truecolor palettes that render identically regardless of the terminal's own color scheme.

---

## Table of Contents

- [Features](#features)
- [Preview](#preview)
- [Architecture & Codebase Tour](#architecture--codebase-tour)
- [Requirements](#requirements)
- [Installation & Build](#installation--build)
  - [1. Build from Source](#1-build-from-source)
  - [2. Install to Cargo Bin Path](#2-install-to-cargo-bin-path)
- [Configuration](#configuration)
  - [Command-Line Options](#command-line-options)
  - [Options Reference](#options-reference)
  - [Exit Codes](#exit-codes)
- [Interactive Live Controls](#interactive-live-controls)
- [Themes and Palettes Catalog](#themes-and-palettes-catalog)
  - [Themes Reference](#themes-reference)
  - [Palettes Reference](#palettes-reference)
- [Verification & Diagnostics](#verification--diagnostics)
- [Troubleshooting](#troubleshooting)
- [Development](#development)
- [Uninstallation](#uninstallation)
- [License](#license)

---

## Features

↑ [Table of Contents](#table-of-contents)

- **Typographic Clockfaces**: High-contrast, pixel-crafted glyph tables for digits `0-9` plus colon separators, stored as raw bitmap rows and rasterized at runtime.
  - **Block (5x7)**: Heavy-weight glyphs optimized for readability.
  - **Compact (3x5)**: Condensed glyphs for constrained terminals and small window splits.
- **4 Sizing Themes**:
  - `large`: Double-width 5x7 block digits with expanded spacing for wide screens.
  - `medium`: Single-width 5x7 block digits for standard terminals.
  - `small`: 3x5 compact digits for small displays and status panes.
  - `auto`: Layout engine that resolves the largest theme fitting the current terminal dimensions on every frame.
- **6 Truecolor Palettes**:
  - `black-red` *(default)*: High-contrast crimson and red digits on dark framing with light coral seconds.
  - `purple`: Neon purple digits with lavender seconds on dark purple framing.
  - `matrix`: Green phosphor CRT aesthetic.
  - `amber`: Warm amber CRT phosphor glow.
  - `monochrome`: White and silver for minimalist configurations.
  - `rainbow`: Spectral coloring across each digit position.
- **Interactive Live Controls**: Switch themes (`1`/`2`/`3`/`a`/`0`), cycle themes (`t`), cycle palettes (`p`), toggle seconds (`s`), switch 12h/24h format (`h`), toggle colon blinking (`b`), and toggle date display (`d`) — all without a restart.
- **Time and Date**:
  - 24-hour or 12-hour format with `AM`/`PM` indicator.
  - Optional seconds display (`HH:MM:SS` or `HH:MM`).
  - 1 Hz colon pulse (500 ms on / 500 ms off) or steady display.
  - Date line with full month name, day, year, and weekday.
- **Performance and Terminal Lifecycle**:
  - **Flicker-Free Rendering**: Single-flush buffered output (`BufWriter`) with state-diff change detection (~10 Hz polling, redundant redraws skipped entirely).
  - **RAII Cleanup**: Raw mode and alternate screen management via `TerminalGuard`, ensuring cursor and screen restoration including on abort, `Ctrl+C`, or panic unwind.
  - **Viewport Safety**: The framed block is clamped to the live terminal size, preventing wrapping, scrolling, or panics on short panes.

---

## Preview

↑ [Table of Contents](#table-of-contents)

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
│                                    October 14, 2026 · Wednesday                                      │
│               1/2/3/a theme · p palette · s seconds · h 12/24h · b blink · d date · q quit           │
│                                                                                                      │
╰──────────────────────────────────────────────────────────────────────────────────────────────────────╯
```

---

## Architecture & Codebase Tour

↑ [Table of Contents](#table-of-contents)

`xclock` follows a strict separation of concerns, avoids heap allocation during steady-state redraws, and keeps external dependencies to a minimum:

```text
xClock/
├── Cargo.toml                 # Crate manifest: edition 2024, three dependencies, no build scripts
└── src/
    ├── main.rs       # Entrypoint: maps CLI parse results onto process exit codes (0, 1, 2)
    ├── cli.rs        # Hand-rolled zero-dependency CLI parser, flag deduplication, usage output
    ├── app.rs        # 100 ms event loop, live key processing, redraw cache key, render dispatch
    ├── terminal.rs   # TerminalGuard RAII guard: raw mode, alternate screen, cursor hiding, signal cleanup
    ├── theme.rs      # Theme and Palette definitions, RGB color mappings, auto_fit geometry resolution
    ├── font.rs       # Face (Block 5x7 / Compact 3x5), Style, glyph tables (0-9, colon), rasterization
    ├── display.rs    # DisplayOptions state, 12h/24h time formatting, meridiem helper, date formatter
    └── ui.rs         # Dynamic Layout computation, clamping to screen boundaries, frame buffer rendering
```

### Rendering Pipeline

↑ [Table of Contents](#table-of-contents)

1. **Poll and Wait**: `crossterm::event::poll` blocks for up to 100 ms, serving as an energy-efficient clock tick (~10 FPS ceiling).
2. **Key Drain**: Any burst of pending events is drained with a zero-timeout `poll` before rendering, so fast keypresses are never dropped.
3. **State Key Diffing**: The current render factors `(time, colon_visible, cols, lines, effective_theme, display, palette)` are compared against `last_key`. If no visual change is detected, layout computation and terminal I/O are skipped entirely.
4. **Layout Solver**: `ui::compute_layout` calculates horizontal and vertical padding to center the clock block. The inner width is clamped to `cols - 2` so the frame never wraps.
5. **Buffered Flush**: All ANSI escape codes and Unicode box characters are written into a `BufWriter<Stdout>` and flushed in a single write per visible frame, eliminating screen tearing.

---

## Requirements

↑ [Table of Contents](#table-of-contents)

### Build Dependencies

↑ [Table of Contents](#table-of-contents)

- **Rust Toolchain**: Rust `1.98+` with Cargo (edition `2024`).
- A POSIX-like environment (Linux, macOS, BSD, WSL) providing a TTY and a C toolchain for the `crossterm` dependency.

### Runtime Dependencies

↑ [Table of Contents](#table-of-contents)

- **Terminal Emulator**: UTF-8 encoding and 24-bit Truecolor support (`COLORTERM=truecolor` or `24bit`) — e.g. Alacritty, Kitty, WezTerm, iTerm2, GNOME Terminal, Foot.
- **No runtime binaries**: `xclock` spawns no external processes and reads no configuration files — everything is driven by CLI flags and live hotkeys.

---

## Installation & Build

↑ [Table of Contents](#table-of-contents)

### 1. Build from Source

↑ [Table of Contents](#table-of-contents)

Clone the repository and build the optimized binary:

```bash
git clone https://github.com/xTerionsilker/xClock-Rust.git xclock
cd xclock
cargo build --release
```

The compiled binary is located at `./target/release/xclock`.

### 2. Install to Cargo Bin Path

↑ [Table of Contents](#table-of-contents)

```bash
cargo install --path .
```

This installs `xclock` into `~/.cargo/bin`. Ensure that directory is present in your `$PATH`:

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
```

---

## Configuration

↑ [Table of Contents](#table-of-contents)

`xclock` is configured entirely through command-line arguments. There is no config file — every option can be changed at runtime through the [live controls](#interactive-live-controls).

### Command-Line Options

↑ [Table of Contents](#table-of-contents)

| Option | Short | Arguments | Default | Description |
|---|---|---|---|---|
| `--theme` | `-t` | `<name>` | `large` | Clock theme: `large`, `medium`, `small`, or `auto`. |
| `--palette` | `-p` | `<name>` | `black-red` | Color palette: `black-red`, `purple`, `matrix`, `amber`, `monochrome`, or `rainbow`. |
| `--show-seconds` | | *none* | *enabled* | Display seconds (`HH:MM:SS`). |
| `--no-seconds`, `--hide-seconds` | | *none* | | Hide seconds (`HH:MM`). |
| `--24h` | | *none* | *enabled* | Display time in 24-hour format (`00`–`23`). |
| `--12h` | | *none* | | Display time in 12-hour format (`01`–`12`) with `AM`/`PM` on the date line. |
| `--hour-format` | | `<12\|24>` | `24` | Explicit hour format (`12`, `12h`, `24`, `24h`). |
| `--blink` | | *none* | *enabled* | Blink colon separators at 1 Hz. |
| `--no-blink` | | *none* | | Display colon separators steadily. |
| `--show-date` | | *none* | *enabled* | Show the date line below the clock face. |
| `--no-date`, `--hide-date` | | *none* | | Hide the date line, reducing frame height. |
| `--list-themes` | | *none* | | List available themes with descriptions and exit. |
| `--list-palettes` | | *none* | | List available color palettes with descriptions and exit. |
| `--version` | `-v`, `-V` | *none* | | Display version information and exit. |
| `--help` | `-h` | *none* | | Display usage information and exit. |

> 💡 **Note on CLI Syntax**:
> - Space-separated and equals syntax are supported (e.g. `-t medium`, `--theme medium`, `--theme=medium`).
> - Theme, palette, and hour-format values are parsed case-insensitively and accept documented aliases (see [Themes Reference](#themes-reference) and [Palettes Reference](#palettes-reference)).
> - Flags within the same functional group follow "last flag wins" precedence (e.g. `--no-seconds --show-seconds` results in seconds being shown).

### Usage Examples

↑ [Table of Contents](#table-of-contents)

```bash
# Launch with defaults (large theme, black-red palette, 24h, seconds, blink, date)
xclock

# Launch with a specific theme and palette
xclock --theme medium --palette matrix

# Compact 12-hour mode without seconds or date
xclock -t small -p amber --12h --no-seconds --no-date

# Automatic sizing that adapts to live terminal resizing
xclock -t auto -p rainbow

# Inspect the built-in catalogs without starting the TUI
xclock --list-themes
xclock --list-palettes
```

### Exit Codes

↑ [Table of Contents](#table-of-contents)

| Code | Meaning | Description |
| :---: | :--- | :--- |
| `0` | **Success** | Clean exit by the user (`q`, `Esc`, `Ctrl+C`, or `Ctrl+D`), or a standard help/version/listing flag. |
| `1` | **Runtime Error** | Terminal initialization failure, raw mode error, or unexpected I/O failure. |
| `2` | **CLI Error** | Invalid argument, missing parameter value, non-UTF-8 arguments, or invalid option format. |

---

## Interactive Live Controls

↑ [Table of Contents](#table-of-contents)

While `xclock` is running in raw mode, settings can be modified without a restart:

| Key | Action | Details |
| :---: | :--- | :--- |
| `1` | **Large Theme** | Double-width 5x7 block face; requires a wide terminal. |
| `2` | **Medium Theme** | Single-width 5x7 block face; fits standard 80x24 terminals. |
| `3` | **Small Theme** | Compact 3x5 face; fits small terminals and window splits down to 45x14. |
| `a` / `0` | **Auto Theme** | Adapts theme selection to live terminal width and height. |
| `t` / `T` | **Cycle Theme** | Cycles sequentially: `large` → `medium` → `small` → `auto` → `large`. |
| `p` / `P` | **Cycle Palette** | Cycles: `black-red` → `purple` → `matrix` → `amber` → `monochrome` → `rainbow` → `black-red`. |
| `s` / `S` | **Toggle Seconds** | Toggles between `HH:MM:SS` and `HH:MM`. |
| `h` / `H` | **Toggle 12h / 24h** | Switches between 24-hour mode and 12-hour mode with `AM`/`PM` label. |
| `b` / `B` | **Toggle Blink** | Toggles colon blinking at 1 Hz versus static display. |
| `d` / `D` | **Toggle Date** | Toggles date display, dynamically adjusting frame height. |
| `q` / `Q` / `Esc` | **Quit** | Restores the terminal cursor, leaves the alternate screen, and exits. |
| `Ctrl+C` | **Quit** | Handled both as a raw key event and through a `ctrlc` signal handler for safe exit. |
| `Ctrl+D` | **Quit** | Standard EOF exit in raw mode. |

The active keys are always visible in the hint line rendered at the bottom of the frame, so no external cheat sheet is required.

---

## Themes and Palettes Catalog

↑ [Table of Contents](#table-of-contents)

### Themes Reference

↑ [Table of Contents](#table-of-contents)

| Theme | Aliases | Face Grid | Scale X | Character Gap | Min Cols (8 chars) | Target Terminal |
| :--- | :--- | :---: | :---: | :---: | :---: | :--- |
| `large` | `modern`, `big` | 5x7 | 2x | 2 | ~98 | Wide displays / full-screen panes (≥ 100x20) |
| `medium` | `classic`, `mid` | 5x7 | 1x | 1 | ~49 | Standard terminals (80x24) |
| `small` | `compact`, `mini` | 3x5 | 1x | 1 | ~33 | Splits, sidebar panes, compact terminals (≥ 45x14) |
| `auto` | `adaptive`, `fit` | *dynamic* | *dynamic* | *dynamic* | *dynamic* | Evaluates `large` → `medium` → `small` per frame |

`auto` resolves against the *effective* width, which changes when seconds are hidden: an 80x24 terminal with `--no-seconds` is wide enough for `large`.

### Palettes Reference

↑ [Table of Contents](#table-of-contents)

All palettes use direct 24-bit Truecolor RGB values for consistent appearance regardless of the terminal base theme:

| Palette | Aliases | Hours and Minutes | Seconds | Frame and Separators | Date Line |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`black-red`** | `red`, `black_red`, `crimson`, `default` | Vivid Red (`#EF4444`) | Soft Coral (`#FCA5A5`) | Deep Crimson (`#7F1D1D`) | Rose Red (`#F87171`) |
| **`purple`** | — | Vivid Purple (`#A855F7`) | Light Lavender (`#D8B4FE`) | Deep Purple (`#581C87`) | Soft Lilac (`#C084FC`) |
| **`matrix`** | `matrix-green`, `green` | Terminal Green (`#22C55E`) | Pale Mint (`#86EFAC`) | Dark Phosphor (`#145014`) | Lime Green (`#32CD32`) |
| **`amber`** | `amber-crt`, `crt` | Warm Amber (`#F59E0B`) | Soft Peach (`#FED7AA`) | Vintage Bronze (`#783C00`) | Golden Amber (`#FBBF24`) |
| **`monochrome`** | `mono`, `white`, `gray`, `grey` | Crisp White (`#FFFFFF`) | Silver Gray (`#9CA3AF`) | Slate Gray (`#4B5563`) | Light Gray (`#D1D5DB`) |
| **`rainbow`** | `spectral`, `colors`, `color` | Multi-color spectrum | Violet (`#8B5CF6`) | Indigo (`#6366F1`) | Sunburst Yellow (`#FDE047`) |

> In the `rainbow` palette, each character position receives a distinct hue from Red (`#EF4444`), Orange (`#F97316`), Yellow (`#EAB308`), Green (`#22C55E`), Cyan (`#06B6D4`), Blue (`#3B82F6`), Violet (`#8B5CF6`), to Pink (`#EC4899`).

---

## Verification & Diagnostics

↑ [Table of Contents](#table-of-contents)

Verify that your build and environment are behaving as expected:

```bash
# 1. Confirm the installed version
xclock --version

# 2. Inspect the built-in theme and palette catalogs
xclock --list-themes
xclock --list-palettes

# 3. Review the full flag surface
xclock --help

# 4. Confirm truecolor support (a blank theme value is intentionally rejected)
echo "${COLORTERM:-unset}"   # expect: truecolor | 24bit

# 5. Smoke-test rendering in a throwaway pane, then quit with q
xclock -t medium -p matrix
```

---

## Troubleshooting

↑ [Table of Contents](#table-of-contents)

| Symptom | Cause | Solution |
|---|---|---|
| `unknown theme ...` / `unknown palette ...` | The value is not a documented id or alias. | Run `xclock --list-themes` / `xclock --list-palettes` and use one of the printed ids. |
| `missing value: --theme expects one of ...` | A flag that requires a parameter was passed without one. | Provide the value in either `--theme=medium` or `-t medium` form. |
| Frame wraps, text scrolls, or the clock is clipped | The terminal is narrower or shorter than the active theme requires. | Switch to a smaller theme (`2` or `3`) or enable the `auto` theme (`a`). |
| Colors look wrong or washed out | The terminal does not advertise 24-bit Truecolor. | Check `COLORTERM=truecolor`; otherwise use the `monochrome` palette. |
| Garbled digits or frame characters | The terminal does not support UTF-8 / box-drawing glyphs. | Switch to a UTF-8 capable terminal such as Alacritty, Kitty, or WezTerm. |
| Terminal left in raw mode after a crash | The process was killed with `SIGKILL`, which cannot be trapped. | Run `reset` or `stty sane` in the affected shell. |
| Seconds or colon look static | Blinking is disabled, or the display updates too fast to notice. | Press `b` to toggle blinking and `s` to toggle seconds. |

---

## Development

↑ [Table of Contents](#table-of-contents)

### Building and Checking

↑ [Table of Contents](#table-of-contents)

```bash
# Build the debug binary
cargo build

# Run the test suite (53 tests across all modules)
cargo test

# Run clippy linter (warning-free)
cargo clippy --all-targets

# Build the optimized release binary
cargo build --release
```

### Testing Strategy

↑ [Table of Contents](#table-of-contents)

The test suite validates:
- **CLI Parsing**: Long, short, and equals-syntax flags, aliases, default values, priority ordering, and error handling.
- **Display Formatting**: 12-hour / 24-hour conversions, zero padding, noon/midnight meridiem handling, month names, and weekday names.
- **Font and Glyph Geometry**: Verification that every row of every glyph matches the expected width and height, and that the rendered line width equals the computed layout metrics.
- **Theme and Palette Lifecycle**: Hotkey collision checks, color index assignment, auto-fit resolution across terminal dimensions, and cyclic palette transitions.
- **UI and Layout Bounding**: Narrow terminal clamping, padding calculations, box-drawing integrity, and smoke tests across all themes and palettes.

### Dependencies

↑ [Table of Contents](#table-of-contents)

- [`crossterm 0.29`](https://crates.io/crates/crossterm) — Cross-platform terminal manipulation, raw mode, alternate screen, event polling, and Truecolor styling.
- [`chrono 0.4`](https://crates.io/crates/chrono) — Local system date, time zone resolution, and calendar calculation.
- [`ctrlc 3`](https://crates.io/crates/ctrlc) — Cross-platform signal handling to ensure clean terminal state teardown on `SIGINT`.

---

## Uninstallation

↑ [Table of Contents](#table-of-contents)

If `xclock` was installed through Cargo, remove it with:

```bash
cargo uninstall xclock
```

If it was built from source, simply remove the working copy:

```bash
rm -rf /path/to/xclock
```

`xclock` writes no configuration files, caches, or state outside its own binary, so no further cleanup is required.

---

## License

↑ [Table of Contents](#table-of-contents)

Distributed under the MIT License. See `LICENSE` for more information.