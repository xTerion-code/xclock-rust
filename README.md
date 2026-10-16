# xclock

[![Rust](https://img.shields.io/badge/rust-1.98%2B-blue.svg)](https://www.rust-lang.org)
[![Edition](https://img.shields.io/badge/edition-2024-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](#license)
[![Tests](https://img.shields.io/badge/tests-53%20passed-brightgreen.svg)](#testing)

A modern, responsive, and lightweight terminal digital clock for Unix environments, built in Rust using [`crossterm`](https://crates.io/crates/crossterm).

`xclock` pairs bold typographic block-digit faces with real-time responsive layout adaptation, truecolor CRT/phosphor color palettes, flexible time/date formatting, and instant live keyboard controls—all rendered flicker-free within an elegant rounded Unicode frame.

---

## Features

- **Typographic Clockfaces**: High-contrast, pixel-crafted vector glyphs for digits `0-9` and colon separators.
  - **Block (5x7)**: Solid heavy-weight glyphs designed for high readability.
  - **Compact (3x5)**: Space-efficient condensed glyphs for constrained terminals and small window splits.
- **4 Sizing Themes**:
  - `large`: Double-width 5x7 block digits with expanded spacing for wide screens.
  - `medium`: Single-width 5x7 block digits for standard terminals.
  - `small`: 3x5 compact digits for small displays and status panes.
  - `auto`: Dynamic layout engine that automatically selects the largest theme that comfortably fits your current terminal dimensions.
- **6 Truecolor Palettes**:
  - `black-red` *(default)*: High-contrast crimson and bold red digits on dark framing with light coral seconds.
  - `purple`: Vivid neon purple digits with soft lavender seconds on dark purple framing.
  - `matrix`: Classic green phosphor CRT aesthetic inspired by digital rain terminals.
  - `amber`: Warm amber CRT phosphor glow reminiscent of vintage workstations.
  - `monochrome`: Crisp, high-contrast white and silver for minimalist setups.
  - `rainbow`: Vibrant spectral rainbow across each digit position.
- **Interactive Live Controls**: Switch themes (`1`/`2`/`3`/`a`), cycle themes (`t`), cycle palettes (`p`), toggle seconds (`s`), flip 12h/24h time (`h`), toggle colon blinking (`b`), and hide/show date (`d`) on the fly without restarting.
- **Precision Time & Date**:
  - 24-hour or 12-hour format with trailing `AM`/`PM` indicator.
  - Optional seconds display (`HH:MM:SS` or `HH:MM`).
  - 1 Hz colon pulse (500 ms on / 500 ms off) or steady colon.
  - Localized date header displaying full month name, day, year, and weekday.
- **Performance & Terminal Lifecycle**:
  - **Flicker-Free Rendering**: Single-flush buffered output (`BufWriter`) with state-diff change detection (~10 Hz poll skipping redundant redraws).
  - **Graceful RAII Cleanup**: Safe raw mode and alternate screen management via `TerminalGuard`, ensuring cursor and screen restoration even on unexpected aborts or `Ctrl+C`.

---

## Preview

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

## Installation

### Prerequisites

- **Rust Toolchain**: Rust 1.98+ and Cargo (2024 edition).
- **Terminal Emulator**: Any modern terminal with UTF-8 encoding and 24-bit Truecolor (`COLORTERM=truecolor` or `24bit`) support (e.g., Alacritty, Kitty, WezTerm, iTerm2, GNOME Terminal, Foot).

### Build from Source

```bash
git clone https://github.com/xTerionsilker/xClock-Rust.git xclock
cd xclock
cargo build --release
```

The compiled binary will be placed at `./target/release/xclock`.

### Install to Cargo Bin Path

```bash
cargo install --path .
```

Ensure `~/.cargo/bin` is in your `$PATH`.

---

## Usage

```bash
# Launch with default settings (Large theme, Purple palette, 24h, seconds on, colon blink on)
xclock

# Launch with a specific theme and palette
xclock --theme medium --palette matrix

# Launch in compact 12-hour mode without seconds or date
xclock -t small -p amber --12h --no-seconds --no-date

# Launch with automatic sizing that adapts to terminal window resizing
xclock -t auto -p rainbow
```

### Command-Line Options

| Option | Short | Arguments | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `--theme` | `-t` | `<name>` | `large` | Clock theme: `large`, `medium`, `small`, or `auto`. |
| `--palette` | `-p` | `<name>` | `black-red` | Color palette: `black-red`, `purple`, `matrix`, `amber`, `monochrome`, or `rainbow`. |
| `--show-seconds` | | *none* | *active* | Display seconds (`HH:MM:SS`). |
| `--no-seconds`, `--hide-seconds` | | *none* | | Hide seconds (`HH:MM`). |
| `--24h` | | *none* | *active* | Display time in 24-hour military format (`00`–`23`). |
| `--12h` | | *none* | | Display time in 12-hour format (`01`–`12`) with `AM`/`PM` appended to date line. |
| `--hour-format` | | `<12\|24>` | `24` | Explicit hour format (`12`, `12h`, `24`, `24h`). |
| `--blink` | | *none* | *active* | Blink colon separators at 1 Hz. |
| `--no-blink` | | *none* | | Keep colon separators steadily visible. |
| `--show-date` | | *none* | *active* | Show the date line under the clock face. |
| `--no-date`, `--hide-date` | | *none* | | Hide the date line, reducing frame height. |
| `--list-themes` | | *none* | | Print available themes with descriptions and exit. |
| `--list-palettes` | | *none* | | Print available color palettes with descriptions and exit. |
| `--version` | `-v`, `-V` | *none* | | Display version information and exit. |
| `--help` | `-h` | *none* | | Display usage help text and exit. |

> **Note on CLI Syntax**:
> - Both space-separated and equals syntax are supported (e.g., `-t medium`, `--theme medium`, `--theme=medium`).
> - Flags within the same functional group follow "last flag wins" precedence (e.g., `--no-seconds --show-seconds` resolves to showing seconds).

### Exit Codes

| Code | Meaning | Description |
| :---: | :--- | :--- |
| `0` | **Success** | Clean exit triggered by user (`q`, `Esc`, `Ctrl+C`, or `Ctrl+D`), or standard help/version/listing flag. |
| `1` | **Runtime Error** | Terminal initialization failure, raw mode error, or unexpected I/O break. |
| `2` | **CLI Error** | Invalid or unrecognized argument, missing parameter value, or invalid option format. |

---

## Live Interactive Controls

While `xclock` is running in raw mode, you can change settings dynamically without restarting:

| Key | Action | Details |
| :---: | :--- | :--- |
| `1` | **Large Theme** | Double-width 5x7 block face; requires wide terminal. |
| `2` | **Medium Theme** | Single-width 5x7 block face; fits standard 80x24 terminals. |
| `3` | **Small Theme** | Compact 3x5 face; fits small terminals and window splits down to 45x14. |
| `a` / `0` | **Auto Theme** | Dynamically adapts theme choice to live terminal width and height. |
| `t` / `T` | **Cycle Theme** | Cycles sequentially: `large` → `medium` → `small` → `auto` → `large`. |
| `p` / `P` | **Cycle Palette** | Cycles: `black-red` → `purple` → `matrix` → `amber` → `monochrome` → `rainbow` → `black-red`. |
| `s` / `S` | **Toggle Seconds** | Toggles between `HH:MM:SS` and `HH:MM`. |
| `h` / `H` | **Toggle 12h / 24h** | Switches between 24-hour mode and 12-hour mode with `AM`/`PM` label. |
| `b` / `B` | **Toggle Blink** | Toggles colon blinking at 1 Hz vs. static display. |
| `d` / `D` | **Toggle Date** | Toggles date display on and off, dynamically resizing frame height. |
| `q` / `Esc` | **Quit** | Restores terminal cursor, leaves alternate screen, and exits cleanly. |
| `Ctrl+C` | **Quit** | Intercepted in both raw key handler and POSIX signal handler for safe exit. |
| `Ctrl+D` | **Quit** | Standard EOF exit in raw mode. |

---

## Themes & Palettes Catalog

### Themes Reference

| Theme | Aliases | Face Grid | Scale X | Character Gap | Min Cols (8 chars) | Target Terminal |
| :--- | :--- | :---: | :---: | :---: | :---: | :--- |
| `large` | `modern`, `big` | 5x7 | 2x | 2 | ~98 | Wide displays / full-screen panes (≥ 100x20) |
| `medium` | `classic`, `mid` | 5x7 | 1x | 1 | ~49 | Standard terminals (80x24) |
| `small` | `compact`, `mini` | 3x5 | 1x | 1 | ~33 | Splits, sidebar panes, compact terminals (≥ 45x14) |
| `auto` | `adaptive`, `fit` | *dynamic* | *dynamic* | *dynamic* | *dynamic* | Evaluates `large` → `medium` → `small` per frame |

### Palettes Reference

All palettes use direct 24-bit Truecolor RGB values for uniform appearance regardless of terminal base theme:

| Palette | Aliases | Hours & Minutes | Seconds | Frame & Separators | Date Line |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`black-red`** | `red`, `black_red`, `crimson`, `default` | Vivid Red (`#EF4444`) | Soft Coral (`#FCA5A5`) | Deep Crimson (`#7F1D1D`) | Rose Red (`#F87171`) |
| **`purple`** | | Vivid Purple (`#A855F7`) | Light Lavender (`#D8B4FE`) | Deep Purple (`#581C87`) | Soft Lilac (`#C084FC`) |
| **`matrix`** | `matrix-green`, `green` | Terminal Green (`#22C55E`) | Pale Mint (`#86EFAC`) | Dark Phosphor (`#145014`) | Lime Green (`#32CD32`) |
| **`amber`** | `amber-crt`, `crt` | Warm Amber (`#F59E0B`) | Soft Peach (`#FED7AA`) | Vintage Bronze (`#783C00`) | Golden Amber (`#FBBF24`) |
| **`monochrome`** | `mono`, `white`, `gray` | Crisp White (`#FFFFFF`) | Silver Gray (`#9CA3AF`) | Slate Gray (`#4B5563`) | Light Gray (`#D1D5DB`) |
| **`rainbow`** | `spectral`, `colors` | Multi-color spectrum | Violet (`#8B5CF6`) | Indigo (`#6366F1`) | Sunburst Yellow (`#FDE047`) |

> In the `rainbow` palette, every character position receives a distinct chromatic hue from Red (`#EF4444`), Orange (`#F97316`), Yellow (`#EAB308`), Green (`#22C55E`), Cyan (`#06B6D4`), Blue (`#3B82F6`), Violet (`#8B5CF6`), to Pink (`#EC4899`).

---

## Architecture & Codebase Tour

`xclock` is designed around strict separation of concerns, zero heap churn during steady-state redraws, and minimal external dependencies:

```text
src/
├── main.rs       # Entrypoint: dispatches CLI parse results and maps process exit codes (0, 1, 2)
├── cli.rs        # Hand-rolled zero-dependency CLI parser, flag deduplication, and usage output
├── app.rs        # 100 ms event loop, live key processing, redraw caching key, and frame render dispatch
├── terminal.rs   # TerminalGuard RAII guard: raw mode, alternate screen, cursor hiding, and signal cleanup
├── theme.rs      # Theme & Palette definitions, RGB color mappings, auto_fit geometry resolution
├── font.rs       # Face (5x7 / 3x5), Style, glyph tables (0-9, colon), and horizontal rasterization
├── display.rs    # DisplayOptions state, 12h/24h time formatting, meridiem helper, and date formatter
└── ui.rs         # Dynamic Layout computation, clamping to screen boundaries, and frame buffer rendering
```

### Rendering Pipeline

1. **Poll & Sleep**: `crossterm::event::poll` blocks for up to 100 ms, serving as an energy-efficient clock tick (~10 FPS max).
2. **State Key Diffing**: Current render factors `(time, colon_visible, cols, lines, effective_theme, display, palette)` are checked against `last_key`. If the terminal state has not changed visually, frame computation and terminal I/O are completely skipped.
3. **Layout Solver**: `ui::compute_layout` calculates horizontal and vertical padding to center the clock block within the terminal. If the window is narrower than the requested content, layout clamping prevents text wrapping or terminal scroll distortion.
4. **Buffered Flush**: All ANSI escape codes and Unicode box characters are written into a `BufWriter<Stdout>`, flushing to the kernel in a single write operation per visible frame to eliminate screen tearing.

---

## Development

### Building & Checking

```bash
# Build debug binary
cargo build

# Run comprehensive test suite (53 tests across all modules)
cargo test

# Run clippy linter (strictly warning-free)
cargo clippy --all-targets

# Run release build
cargo build --release
```

### Testing Strategy

The test suite validates:
- **CLI Parsing**: Long, short, and equal-syntax flags, aliases, default values, priority ordering, and error handling.
- **Display Formatting**: 12-hour / 24-hour conversions, zero padding, noon/midnight meridiem splits, month names, and weekday names.
- **Font & Glyph Geometry**: Invariant check ensuring every row of every glyph matches expected width and height, and that rendered line width equals computed layout metrics.
- **Theme & Palette Lifecycle**: Hotkey collision freedom, color index assignment, auto-fit resolution across varying terminal dimensions, and cyclic palette transitions.
- **UI & Layout Bounding**: Narrow terminal clamping, padding calculations, box-drawing integrity, and smoke tests across all themes and palettes.

---

## Dependencies

- [`crossterm 0.29`](https://crates.io/crates/crossterm) — Cross-platform terminal manipulation, raw mode, alternate screen, event polling, and Truecolor styling.
- [`chrono 0.4`](https://crates.io/crates/chrono) — Local system date, time zone resolution, and calendar calculation.
- [`ctrlc 3`](https://crates.io/crates/ctrlc) — Cross-platform signal handling to guarantee clean terminal state teardown on `SIGINT`.

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
