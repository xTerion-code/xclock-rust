# xclock — Your Terminal Just Learned How to Tell Time in Style

[![Rust](https://img.shields.io/badge/rust-1.98%2B-blue.svg)](https://www.rust-lang.org)
[![Edition](https://img.shields.io/badge/edition-2024-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](#license)
[![Tests](https://img.shields.io/badge/tests-53%20passed-brightgreen.svg)](#development)

Forget that sad little `HH:MM` in your status bar. `xclock` is a loud, proud, flicker-free digital clock for your terminal — built in Rust with [`crossterm`](https://crates.io/crates/crossterm), wrapped in a slick rounded frame, and glowing like it just escaped from a 1983 sci-fi movie.

Big blocky digits. Six shamelessly saturated color palettes. Live keyboard controls. Zero flicker. It does exactly one job — showing the time — and it overdoes it gloriously.

---

## Why xclock?

- **It is huge.** Double-width 5x7 block digits you can read from across the room.
- **It is dramatic.** CRT amber glow, Matrix green rain vibes, full rainbow chaos — pick your personality.
- **It is alive.** Smash keys while it runs and it morphs instantly. No restart, no config file archaeology.
- **It is polite.** Cleans up raw mode, restores your cursor, leaves the alternate screen spotless — even when you `Ctrl+C` out of panic.
- **It is fast.** Buffered single-flush rendering at ~10 Hz with state-diffing, so it redraws only when something actually changed.

In short: `watch -n1 date` wishes it looked this good.

---

## Features

- **Two Hand-Crafted Typefaces**
  - **Block (5x7):** Thick, confident, highly readable. The main character.
  - **Compact (3x5):** The same attitude, squeezed into tiny splits and side panes.
- **4 Sizing Themes**
  - `large`: Double-width 5x7. For wide screens and showing off.
  - `medium`: Single-width 5x7. Fits a classic 80x24 like a glove.
  - `small`: 3x5 compact. Survives tmux splits down to 45x14.
  - `auto`: The smart one. Picks the biggest theme that fits your window, live, every frame.
- **6 Truecolor Palettes**
  - `black-red` *(default)*: Crimson on black. Business in the front, alarm clock in the back.
  - `purple`: Neon purple with lavender seconds. Retrowave approved.
  - `matrix`: Green phosphor. Wake up, Neo — standup is in 10 minutes.
  - `amber`: Warm vintage workstation glow. Smells faintly of old capacitors.
  - `monochrome`: Crisp white and silver. For minimalists and people with strong opinions about fonts.
  - `rainbow`: Every digit gets its own color. Subtlety left the chat.
- **Live Remote Control for Time Itself:** Themes (`1`/`2`/`3`/`a`), cycle themes (`t`), cycle palettes (`p`), seconds (`s`), 12h/24h (`h`), colon blink (`b`), date (`d`) — all without restarting.
- **Proper Timekeeping:**
  - 24h military time or 12h with `AM`/`PM`.
  - Optional seconds (`HH:MM:SS` vs. chill `HH:MM`).
  - 1 Hz colon pulse (500ms on / 500ms off) or rock-steady colons.
  - Full date line: month name, day, year, weekday. No cryptic `10/14/26` nonsense.
- **Engineered, Not Hacked Together:**
  - **Flicker-free:** `BufWriter` + single flush + render-key diffing. No tearing, no CPU spinning.
  - **RAII cleanup:** `TerminalGuard` restores raw mode, cursor, and screen even on abort or `Ctrl+C`.

---

## Preview

Behold, 22:19 in all its blocky glory:

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

Yes, the footer is a cheat sheet. You will memorize it in about 30 seconds and then feel like a clock wizard.

---

## Installation

### What You Need

- **Rust 1.98+ + Cargo (2024 edition).** Time travel requires a modern compiler.
- **A decent terminal** with UTF-8 and 24-bit Truecolor (`COLORTERM=truecolor` or `24bit`) — Alacritty, Kitty, WezTerm, iTerm2, GNOME Terminal, Foot, etc.

### Build From Source

```bash
git clone https://github.com/xTerionsilker/xClock-Rust.git xclock
cd xclock
cargo build --release
```

Binary lands at `./target/release/xclock`. Big, fast, ready to glow.

### Install to Your PATH

```bash
cargo install --path .
```

Make sure `~/.cargo/bin` is in your `$PATH`, or your shell will pretend it never happened.

---

## Usage

```bash
# Just vibes. Defaults: large theme, black-red palette, 24h, seconds on, blink on.
xclock

# Hacker terminal look
xclock --theme medium --palette matrix

# Tiny, polite, 12-hour clock for a corner pane
xclock -t small -p amber --12h --no-seconds --no-date

# Maximum adaptability + maximum color chaos, follows window resizes live
xclock -t auto -p rainbow
```

### Command-Line Options

| Option | Short | Args | Default | What It Does |
| :--- | :--- | :--- | :--- | :--- |
| `--theme` | `-t` | `<name>` | `large` | `large`, `medium`, `small`, or `auto`. Go big or go `auto`. |
| `--palette` | `-p` | `<name>` | `black-red` | `black-red`, `purple`, `matrix`, `amber`, `monochrome`, or `rainbow`. |
| `--show-seconds` | | *none* | *on* | Full `HH:MM:SS` experience. |
| `--no-seconds`, `--hide-seconds` | | *none* | | Zen mode: just `HH:MM`. |
| `--24h` | | *none* | *on* | Military time (`00`–`23`). No AM/PM drama. |
| `--12h` | | *none* | | Civilian time (`01`–`12`) with `AM`/`PM` tag. |
| `--hour-format` | | `<12\|24>` | `24` | Explicit version of the above (`12`, `12h`, `24`, `24h`). |
| `--blink` | | *none* | *on* | Colons pulse at 1 Hz. Hypnotic. |
| `--no-blink` | | *none* | | Colons stay solid. For the easily distracted. |
| `--show-date` | | *none* | *on* | Date line under the clock. |
| `--no-date`, `--hide-date` | | *none* | | Date? Never heard of her. Frame gets shorter. |
| `--list-themes` | | *none* | | Show themes and bail out. |
| `--list-palettes` | | *none* | | Show palettes and bail out. |
| `--version` | `-v`, `-V` | *none* | | Print version, exit. |
| `--help` | `-h` | *none* | | Print help, exit. |

> **CLI house rules:**
> - Both `-t medium` and `--theme=medium` work. We are not monsters.
> - Same group? Last flag wins. `--no-seconds --show-seconds` means seconds win.

### Exit Codes

| Code | Meaning | Translation |
| :---: | :--- | :--- |
| `0` | **Success** | You quit gracefully (`q`, `Esc`, `Ctrl+C`, `Ctrl+D`) or just asked for `--help`. Well done. |
| `1` | **Runtime Error** | Terminal refused to cooperate — raw mode or I/O blew up. |
| `2` | **CLI Error** | Typo, unknown flag, or missing value. The parser judges silently. |

---

## Live Interactive Controls

This is the fun part. While `xclock` is running, just press stuff:

| Key | Action | Effect |
| :---: | :--- | :--- |
| `1` | **Large Theme** | Maximum chonk. Needs a wide window. |
| `2` | **Medium Theme** | The reliable 80x24 citizen. |
| `3` | **Small Theme** | Fits splits and sidebars down to 45x14. |
| `a` / `0` | **Auto Theme** | Watches your window size and adapts live. |
| `t` / `T` | **Cycle Theme** | `large` → `medium` → `small` → `auto` → repeat. Theme roulette. |
| `p` / `P` | **Cycle Palette** | `black-red` → `purple` → `matrix` → `amber` → `monochrome` → `rainbow` → repeat. |
| `s` / `S` | **Toggle Seconds** | `HH:MM:SS` vs. `HH:MM`. Instant gratification. |
| `h` / `H` | **Toggle 12h / 24h** | Time-travel between army time and normal human time. |
| `b` / `B` | **Toggle Blink** | Pulsing colons vs. stoic colons. |
| `d` / `D` | **Toggle Date** | Date line appears / vanishes, frame resizes itself. |
| `q` / `Esc` | **Quit** | Clean exit. Cursor restored, screen restored, dignity intact. |
| `Ctrl+C` | **Quit** | Caught twice (key handler + signal handler) because we really do not want to trash your terminal. |
| `Ctrl+D` | **Quit** | Classic EOF goodbye. |

Pro tip: mash `p` repeatedly in a dark room. You are welcome.

---

## Themes & Palettes Catalog

### Themes Reference

| Theme | Aliases | Grid | Scale X | Gap | Min Cols (8 chars) | Best For |
| :--- | :--- | :---: | :---: | :---: | :---: | :--- |
| `large` | `modern`, `big` | 5x7 | 2x | 2 | ~98 | Wide screens, full panes (≥ 100x20). Statement piece. |
| `medium` | `classic`, `mid` | 5x7 | 1x | 1 | ~49 | Standard 80x24 terminals. Daily driver. |
| `small` | `compact`, `mini` | 3x5 | 1x | 1 | ~33 | Splits and sidebars (≥ 45x14). Pocket clock. |
| `auto` | `adaptive`, `fit` | *dynamic* | *dynamic* | *dynamic* | *dynamic* | Indecisive windows. Tries `large` → `medium` → `small` every frame. |

### Palettes Reference

All palettes are hardcoded 24-bit Truecolor RGB, so they look identical no matter what your terminal theme is doing:

| Palette | Aliases | Hours & Minutes | Seconds | Frame | Date |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`black-red`** | `red`, `black_red`, `crimson`, `default` | Vivid Red (`#EF4444`) | Soft Coral (`#FCA5A5`) | Deep Crimson (`#7F1D1D`) | Rose Red (`#F87171`) |
| **`purple`** | | Vivid Purple (`#A855F7`) | Light Lavender (`#D8B4FE`) | Deep Purple (`#581C87`) | Soft Lilac (`#C084FC`) |
| **`matrix`** | `matrix-green`, `green` | Terminal Green (`#22C55E`) | Pale Mint (`#86EFAC`) | Dark Phosphor (`#145014`) | Lime Green (`#32CD32`) |
| **`amber`** | `amber-crt`, `crt` | Warm Amber (`#F59E0B`) | Soft Peach (`#FED7AA`) | Vintage Bronze (`#783C00`) | Golden Amber (`#FBBF24`) |
| **`monochrome`** | `mono`, `white`, `gray` | Crisp White (`#FFFFFF`) | Silver Gray (`#9CA3AF`) | Slate Gray (`#4B5563`) | Light Gray (`#D1D5DB`) |
| **`rainbow`** | `spectral`, `colors` | Full spectrum | Violet (`#8B5CF6`) | Indigo (`#6366F1`) | Sunburst Yellow (`#FDE047`) |

> `rainbow` assigns a different hue to every digit position: Red (`#EF4444`), Orange (`#F97316`), Yellow (`#EAB308`), Green (`#22C55E`), Cyan (`#06B6D4`), Blue (`#3B82F6`), Violet (`#8B5CF6`), Pink (`#EC4899`). It is scientifically impossible to be sad while looking at it.

---

## Architecture & Codebase Tour

Clean modules, no heap churn in the hot loop, almost no dependencies. Just how Rust likes it:

```text
src/
├── main.rs       # Entrypoint: routes CLI results to exit codes (0, 1, 2)
├── cli.rs        # Hand-rolled, zero-dependency CLI parser with flag dedup and help text
├── app.rs        # 100 ms event loop, live key handling, redraw cache key, frame dispatch
├── terminal.rs   # TerminalGuard RAII: raw mode, alt screen, hidden cursor, signal-safe cleanup
├── theme.rs      # Theme + Palette definitions, RGB maps, auto_fit geometry solver
├── font.rs       # 5x7 / 3x5 glyph tables (0-9, colon) and horizontal rasterizer
├── display.rs    # DisplayOptions, 12h/24h formatting, meridiem logic, date formatter
└── ui.rs         # Layout math, screen-edge clamping, frame buffer renderer
```

### How a Frame Gets Born

1. **Poll & nap:** `crossterm::event::poll` waits up to 100 ms. That wait *is* the clock tick (~10 FPS ceiling, near-zero idle CPU).
2. **Diff or ditch:** Build a key from `(time, colon_visible, cols, lines, effective_theme, display, palette)`. Same as last frame? Skip everything. No work is the fastest work.
3. **Solve layout:** `ui::compute_layout` centers the clock block. Window too narrow? Content gets clamped instead of wrapping and wrecking your scrollback.
4. **One big flush:** Everything — ANSI codes, box-drawing characters, colors — goes into a `BufWriter<Stdout>` and hits the terminal in a single write. No tearing, no flicker.

---

## Development

### Build, Test, Lint

```bash
# Debug build for tinkering
cargo build

# Full test suite — 53 tests, all modules
cargo test

# Clippy, zero-tolerance mode
cargo clippy --all-targets

# Release build for showing off
cargo build --release
```

### What the Tests Actually Check

- **CLI Parsing:** Long/short/equals syntax, aliases, defaults, last-flag-wins, error paths.
- **Display Formatting:** 12h/24h conversion, zero-padding, noon/midnight edge cases, month + weekday names.
- **Font Geometry:** Every glyph row is the right width/height, rendered width matches layout math.
- **Theme & Palette Lifecycle:** No hotkey collisions, correct color indexing, auto-fit across window sizes, palette cycling order.
- **UI & Layout:** Narrow-terminal clamping, centering padding, box integrity, smoke tests over every theme x palette combo.

---

## Dependencies

Only three, because restraint is a feature:

- [`crossterm 0.29`](https://crates.io/crates/crossterm) — Raw mode, alternate screen, event polling, Truecolor styling. The terminal whisperer.
- [`chrono 0.4`](https://crates.io/crates/chrono) — Local time, time zones, calendar math. Knows what day it is so you do not have to.
- [`ctrlc 3`](https://crates.io/crates/ctrlc) — Catches `SIGINT` so your terminal survives your `Ctrl+C` reflex.

---

## License

MIT — see [LICENSE](LICENSE) for details. Go forth and put a giant glowing clock on everything.
