# xclock — Rust Terminal Digital Clock

[![Rust](https://img.shields.io/badge/Rust-1.85%2B-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Edition](https://img.shields.io/badge/Edition-2024-orange?style=for-the-badge&logo=rust&logoColor=white)](https://doc.rust-lang.org/edition-guide/rust-2024/)
[![Terminal](https://img.shields.io/badge/Terminal-TUI-41CD52?style=for-the-badge&logo=gnome-terminals&logoColor=white)](https://gnome-terminal.org/)
[![License](https://img.shields.io/badge/License-MIT-b91c1c?style=for-the-badge&logo=open-source-initiative&logoColor=white)](https://github.com/xTerionsilker/xClock-Rust)
[![CI](https://img.shields.io/badge/CI-fmt%20%2B%20clippy%20%2B%20tests-4CAF50?style=for-the-badge&logo=githubactions&logoColor=white)](https://github.com/xTerionsilker/xClock-Rust/actions/workflows/ci.yml)

**xclock** (*x* clock) is a terminal digital clock written in Rust on top of [`crossterm`](https://crates.io/crates/crossterm), drawing bold typographic block-digit clockfaces inside a rounded Unicode frame. It combines real-time responsive layout adaptation, six truecolor CRT/phosphor palettes, flexible time and date formatting, and live keyboard controls — all rendered flicker-free with single-flush buffered output.

Current version is **1.0.0**. The whole UI follows a typographic pixel-craft aesthetic: hand-authored 5x7 and 3x5 glyph tables, box-drawing frames, and 24-bit truecolor palettes that render identically regardless of the terminal's own color scheme.

---

## Table of Contents

- [Features](#features)
- [Preview](#preview)
- [Architecture & Codebase Tour](#architecture--codebase-tour)
- [Graceful Degradation](#graceful-degradation)
- [Requirements](#requirements)
- [Installation & Build](#installation--build)
  - [1. Build from Source](#1-build-from-source)
  - [2. Install to Cargo Bin Path](#2-install-to-cargo-bin-path)
- [Configuration](#configuration)
  - [Command-Line Options](#command-line-options)
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
  - Any theme that does not fit the current window degrades to the largest one that does, so digits are never clipped (see [Graceful Degradation](#graceful-degradation)).
- **6 Truecolor Palettes**:
  - `black-red` *(default)*: High-contrast crimson and red digits on dark framing with light coral seconds.
  - `purple`: Neon purple digits with lavender seconds on dark purple framing.
  - `matrix`: Green phosphor CRT aesthetic.
  - `amber`: Warm amber CRT phosphor glow.
  - `monochrome`: White and silver for minimalist configurations.
  - `rainbow`: Spectral coloring across each digit position.
- **Interactive Live Controls**: Switch themes (`1`/`2`/`3`/`a`/`0`), cycle themes (`t`), cycle palettes (`p`), toggle seconds (`s`), switch 12h/24h format (`h`), toggle colon blinking (`b`), toggle date display (`d`), and toggle the hint row (`i`) — all without a restart.
- **Time and Date**:
  - 24-hour or 12-hour format with `AM`/`PM` indicator.
  - Optional seconds display (`HH:MM:SS` or `HH:MM`).
  - Colon pulse at 2 Hz (500 ms on / 500 ms off, one full cycle per second) or steady display.
  - Date line with full month name, day, year, and weekday.
- **Adaptive Hint Row**: The live-keys reminder is printed in full when it fits, shortened when it does not, and removed from the frame entirely (height included) when even the short form would be cut. `--no-hint` removes it unconditionally.
- **Performance and Terminal Lifecycle**:
  - **Flicker-Free Rendering**: Single-flush buffered output (`BufWriter`) with state-diff change detection. The loop sleeps until the exact instant the next frame *can* differ, and a frame that would be identical is not drawn at all. Idle CPU is 0.0%.
  - **Rectangle Erasing**: Only the rows that the previous and the next frame share are erased. The screen is cleared exactly once, on the first frame, so toggling an option does not flash.
  - **RAII Cleanup**: Raw mode, alternate screen, cursor visibility, keyboard-enhancement flags and the panic hook are all owned by `TerminalGuard`, which restores the terminal on normal exit, on error and during panic unwind.
  - **Signal Safety**: SIGINT, SIGTERM and SIGHUP all exit cleanly, so a supervisor or a closing window cannot leave the shell in raw mode.
  - **Repeat-Safe Input**: A held key is applied once, not thirty times: `xclock` requests `REPORT_EVENT_TYPES` and falls back to a short window on terminals that decline.
  - **Viewport Safety**: The framed block is clamped to the live terminal size and a face that does not fit is replaced, never clipped. No wrapping, scrolling or panics on short panes.

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
│    1/2/3/a theme · p palette · s seconds · h 12/24h · b blink · d date · i hint · q quit           │
│                                                                                                      │
╰──────────────────────────────────────────────────────────────────────────────────────────────────────╯
```

---

## Architecture & Codebase Tour

↑ [Table of Contents](#table-of-contents)

`xclock` follows a strict separation of concerns with a single dependency direction and only three external dependencies:

```text
xClock/
├── Cargo.toml             # Crate manifest: edition 2024, rust-version 1.85, no build scripts
├── rust-toolchain.toml    # Pinned toolchain with rustfmt and clippy
└── src/
    ├── main.rs       # Entrypoint: maps CLI parse results onto process exit codes (0, 1, 2)
    ├── cli.rs        # Hand-rolled zero-dependency CLI parser, last-flag-wins, usage and catalog output
    ├── app.rs        # Event loop, live key processing, frame cache, rectangle erasing, render dispatch
    ├── terminal.rs   # TerminalGuard RAII guard: raw mode, alternate screen, cursor, key flags, panic hook
    ├── theme.rs      # Theme/ResolvedTheme, const Palette colour table, fit budgets, auto_fit resolution
    ├── font.rs       # Face (Block 5x7 / Compact 3x5), Style, FaceLayout, glyph tables (0-9, colon), rasterization
    ├── display.rs    # DisplayOptions state, TimeFace, 12h/24h time formatting, meridiem and date formatters
    ├── parse.rs      # Shared ParseError used by every FromStr implementation
    └── ui.rs         # Layout computation, clamping to screen boundaries, hint selection, frame rendering
```

Three invariants carry most of the weight:

- **`FaceLayout` describes the frame once.** Every optional element — seconds, date, hint — is described in a single value derived from `DisplayOptions::face()`, so the width budget, the height budget and the render pass cannot disagree about what the frame contains.
- **Only `ResolvedTheme` carries geometry.** `Theme::Auto` has no face of its own; `Theme::resolve` produces a `ResolvedTheme` that layout and font code accept, which makes "used the auto theme's geometry by mistake" unrepresentable.
- **Frames are erased by rectangle.** The renderer remembers the rectangle the last frame occupied and erases only the union with the new one. `Clear(All)` runs exactly once, on the first frame.

`IDEA.md` documents these rules and the reasoning behind them for future contributors.

### Rendering Pipeline

↑ [Table of Contents](#table-of-contents)

1. **Wait for the Next Visible Change**: `crossterm::event::poll` blocks until either input arrives or the next instant at which the picture could change — the next blink edge, or the next second when blinking is off. Sleeping to the boundary keeps digit dwell times even instead of drifting by up to a poll interval. The wait is capped at 500 ms so resize events are always noticed.
2. **Key Drain and Repeat Filter**: Any burst of pending events is drained with a zero-timeout `poll` before rendering, so a fast sequence of presses is applied in full. `RepeatFilter` discards `Repeat` and `Release` events and latches a key until its release arrives; terminals that do not report event types resend the raw byte, so a short window catches those.
3. **State Key Diffing**: The current render factors `(time, colon_visible, cols, lines, resolved_theme, display, palette, mode)` are compared against the last drawn frame. If nothing visible changed, not a single byte is written.
4. **Layout Solver**: `ui::compute_layout` centers the block and clamps the inner width to `cols - 2`. The hint row is selected here: the longest reminder that fits, or none at all, in which case it is also dropped from the height budget.
5. **Rectangle Erasing**: The rows shared with the previously drawn frame are erased with `Clear(UntilNewLine)`, then the new frame is written from the top-left corner.
6. **Buffered Flush**: All ANSI escape codes and Unicode box characters go into a `BufWriter<Stdout>` and are flushed in a single write per visible frame.

Time formatting allocates a handful of small `String`s per frame — the digits, the date and the reminder are built at render time rather than cached, which keeps the code free of a stale-state class of bugs. That is a few kilobytes of garbage per second at most, and the idle process still measures 0.0% CPU because unchanged frames skip the rendering entirely.

---

## Graceful Degradation

`xclock` never draws a broken frame. Whenever the live terminal is too small
for the requested clockface, it steps down in defined stages:

| Stage | Condition | Result |
| :--- | :--- | :--- |
| 1. Theme degradation | The requested theme needs more columns or rows than the window has. | The largest theme that fits is drawn instead (`large` → `medium` → `small`). |
| 2. Hint degradation | The full live-keys reminder does not fit inside the frame. | A shorter reminder is printed in full. |
| 3. Hint removal | Not even the short reminder fits, or `--no-hint` was passed. | The row disappears and the frame loses one row. |
| 4. Plain fallback | No clockface fits at all. | A single centred `HH:MM:SS` line is drawn without a frame. |

This is why `xclock` on a stock 80x24 terminal with the default `large` theme
renders the `medium` face instead of clipping digits: the digits are never cut
off, and the framed layout is only abandoned when no face fits.

The fit budgets are computed, not guessed:

| Theme | Clock width (8 chars) | Minimum columns | Minimum rows (with date and hint) |
| :--- | ---: | ---: | ---: |
| `large` | 94 | 98 | 14 |
| `medium` | 47 | 51 | 14 |
| `small` | 31 | 35 | 12 |

Hiding seconds frees 36 columns on every theme; hiding the date and the hint
frees two rows. `--minimal` (`--no-hint --no-date`) gives the smallest frame.

---

## Requirements

↑ [Table of Contents](#table-of-contents)

### Build Dependencies

↑ [Table of Contents](#table-of-contents)

- **Rust Toolchain**: Rust `1.85+` with Cargo (edition `2024`). The pinned toolchain in `rust-toolchain.toml` supplies `rustfmt` and `clippy`; `rustup` installs it automatically on first build.
- A POSIX-like environment (Linux, macOS, BSD, WSL) providing a TTY and a C toolchain for the `crossterm` dependency.

### Runtime Dependencies

↑ [Table of Contents](#table-of-contents)

- **Terminal Emulator**: UTF-8 encoding and 24-bit Truecolor support (`COLORTERM=truecolor` or `24bit`) — e.g. Alacritty, Kitty, WezTerm, iTerm2, GNOME Terminal, Foot. Terminals that implement the [kitty keyboard protocol](https://sw.kovidgoyal.net/kitty/keyboard-protocol/) get exact key-repeat handling; the others still work.
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
| `--blink` | | *none* | *enabled* | Blink colon separators: 500 ms on, 500 ms off. |
| `--no-blink` | | *none* | | Display colon separators steadily. |
| `--show-date` | | *none* | *enabled* | Show the date line below the clock face. |
| `--no-date`, `--hide-date` | | *none* | | Hide the date line, reducing frame height. |
| `--show-hint` | | *none* | *enabled* | Show the live-keys reminder row at the bottom of the frame. |
| `--no-hint`, `--hide-hint`, `--minimal` | | *none* | | Remove the reminder row entirely, shrinking the frame. |
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
| `b` / `B` | **Toggle Blink** | Toggles the colon pulse (500 ms on / 500 ms off) versus a static display. |
| `d` / `D` | **Toggle Date** | Toggles date display, dynamically adjusting frame height. |
| `i` / `I` | **Toggle Hint** | Shows or removes the live-keys row. Recommended for wallpapers and tiny panes. |
| `q` / `Q` / `Esc` | **Quit** | Restores the terminal cursor, leaves the alternate screen, and exits. |
| `Ctrl+C` | **Quit** | Handled both as a raw key event and through a `ctrlc` signal handler for safe exit. |

Holding a key applies it once. `xclock` requests `REPORT_EVENT_TYPES` from the
terminal so auto-repeat arrives as explicit `Repeat` events, and falls back to
a short suppression window on terminals that decline the request.
| `Ctrl+D` | **Quit** | Standard EOF exit in raw mode. |

The active keys are visible in the hint row at the bottom of the frame, so no external cheat sheet is required. The row prints in full when it fits, shortens itself when it does not, and is dropped from the frame entirely when even the short form would be cut off:

```text
wide:    1/2/3/a theme · p palette · s seconds · h 12/24h · b blink · d date · i hint · q quit
narrow:  1/2/3 theme · p palette · i hint · q quit
```

---

## Themes and Palettes Catalog

↑ [Table of Contents](#table-of-contents)

### Themes Reference

↑ [Table of Contents](#table-of-contents)

| Theme | Aliases | Face Grid | Scale X | Character Gap | Min Cols (8 chars) | Target Terminal |
| :--- | :--- | :---: | :---: | :---: | :---: | :--- |
| `large` | `modern`, `big` | 5x7 | 2x | 2 | 98 | Wide displays / full-screen panes (≥ 100x20) |
| `medium` | `classic`, `mid` | 5x7 | 1x | 1 | 51 | Standard terminals (80x24) |
| `small` | `compact`, `mini` | 3x5 | 1x | 1 | 35 | Splits, sidebar panes, compact terminals (≥ 45x14) |
| `auto` | `adaptive`, `fit` | *dynamic* | *dynamic* | *dynamic* | *dynamic* | Evaluates `large` → `medium` → `small` per frame |

`auto` resolves against the *effective* width, which changes when seconds are hidden: an 80x24 terminal with `--no-seconds` is wide enough for `large`. The same applies to an explicitly requested theme — see [Graceful Degradation](#graceful-degradation).

### Palettes Reference

↑ [Table of Contents](#table-of-contents)

All palettes use direct 24-bit Truecolor RGB values for consistent appearance regardless of the terminal base theme:

| Palette | Aliases | Hours and Minutes | Seconds | Frame and Separators | Date Line |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`black-red`** | `red`, `black_red`, `crimson`, `default` | Vivid Red (`#EF4444`) | Soft Coral (`#FCA5A5`) | Deep Crimson (`#7F1D1D`) | Rose Red (`#F87171`) |
| **`purple`** | — | Vivid Purple (`#A855F7`) | Light Lavender (`#D8B4FE`) | Deep Purple (`#581C87`) | Soft Lilac (`#C084FC`) |
| **`matrix`** | `matrix-green`, `matrix_green`, `green` | Terminal Green (`#22C55E`) | Pale Mint (`#86EFAC`) | Dark Phosphor (`#145014`) | Lime Green (`#32CD32`) |
| **`amber`** | `amber-crt`, `amber_crt`, `crt` | Warm Amber (`#F59E0B`) | Soft Peach (`#FED7AA`) | Vintage Bronze (`#783C00`) | Golden Amber (`#FBBF24`) |
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

# 6. Check graceful degradation: the same binary on a narrow window
xclock -t large -p black-red --no-hint --no-date
```

`xclock` reacts to live resizing, so the narrowest case is easiest to verify by
dragging the window smaller and watching the theme step down.

---

## Troubleshooting

↑ [Table of Contents](#table-of-contents)

| Symptom | Cause | Solution |
|---|---|---|
| `unknown theme ...` / `unknown palette ...` | The value is not a documented id or alias. | Run `xclock --list-themes` / `xclock --list-palettes` and use one of the printed ids. |
| `missing value: --theme expects one of ...` | A flag that requires a parameter was passed without one. | Provide the value in either `--theme=medium` or `-t medium` form. |
| Frame wraps, text scrolls, or the clock is clipped | Should not happen: a theme that does not fit is replaced automatically. If you see it, please [open an issue](https://github.com/xTerionsilker/xClock-Rust/issues). | Press `a` for the `auto` theme, or `2`/`3` to pick a smaller face deliberately. |
| The theme is not the one that was requested | The requested theme does not fit the current window, so `xclock` degraded to the largest face that does. | Widen the window, hide seconds with `s`, or run `xclock --minimal` for the smallest frame. |
| The live-keys row is missing | Not even the short reminder fits, or `--no-hint` is in effect. | Press `i` to toggle it, or widen the terminal. |
| A held key toggles twice in a row | The terminal does not report key event types, so repeat arrives as a repeated byte and only the fallback suppression window applies. | Update the terminal emulator; on modern terminals the action is applied exactly once. |
| Colors look wrong or washed out | The terminal does not advertise 24-bit Truecolor. | Check `COLORTERM=truecolor`; otherwise use the `monochrome` palette. |
| Garbled digits or frame characters | The terminal does not support UTF-8 / box-drawing glyphs. | Switch to a UTF-8 capable terminal such as Alacritty, Kitty, or WezTerm. |
| Terminal left in raw mode after a crash | The process was killed with `SIGKILL`, `SIGSTOP` or `SIGQUIT`, which are not trapped. Every other exit path — `q`, `Esc`, `Ctrl+C`, `Ctrl+D`, `SIGINT`, `SIGTERM`, `SIGHUP` and a panic — restores the terminal. | Run `reset` or `stty sane` in the affected shell. |
| Seconds or colon look static | Blinking is disabled, or the display updates too fast to notice. | Press `b` to toggle blinking and `s` to toggle seconds. |
| Digits are clipped at the frame edge | A regression; `xclock` degrades the theme instead of clipping. | Please [open an issue](https://github.com/xTerionsilker/xClock-Rust/issues) with your terminal size. |

---

## Development

↑ [Table of Contents](#table-of-contents)

### Building and Checking

↑ [Table of Contents](#table-of-contents)

```bash
# Build the debug binary
cargo build

# Check formatting (rustfmt, from the pinned toolchain)
cargo fmt --all -- --check

# Run the test suite
cargo test

# Run the clippy linter (warning-free, also clean under -W clippy::pedantic)
cargo clippy --all-targets

# Build the optimized release binary
cargo build --release
```

Continuous integration runs the same commands on every push and pull request,
plus a `cargo audit` dependency check. Locally, the full gate is:

```bash
cargo fmt --all -- --check \
  && cargo clippy --all-targets --locked -- -D warnings \
  && cargo test --all-targets --locked
```

Development conventions, the architecture invariants and the list of tests that
guard them live in [`IDEA.md`](IDEA.md).

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
- [`ctrlc 3`](https://crates.io/crates/ctrlc) — Signal handling with the `termination` feature, so `SIGINT`, `SIGTERM` and `SIGHUP` all exit with a clean terminal. `SIGQUIT`, `SIGKILL` and `SIGSTOP` are not trapped.

`Cargo.lock` is committed, the binary depends on exactly three crates, and CI
runs `cargo audit` against the locked graph.

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