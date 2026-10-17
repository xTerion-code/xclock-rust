use std::io::{self, Write};

use crossterm::{
    cursor, queue,
    style::{Attribute, ResetColor, SetAttribute, SetForegroundColor},
    terminal,
};

use crate::font::{FaceLayout, Style};
use crate::theme::{FRAME_COLUMNS, Palette, ResolvedTheme};

/// Full live-keys reminder, used when the frame is wide enough for it.
pub const HINT_FULL: &str =
    "1/2/3/a theme · p palette · s seconds · h 12/24h · b blink · d date · i hint · q quit";
/// Trimmed reminder for narrow panes, where the full line would be cut off.
pub const HINT_SHORT: &str = "1/2/3 theme · p palette · i hint · q quit";

const H_PAD_INSIDE: usize = 4;

pub struct Layout {
    pub pad_x: usize,
    pub pad_y: usize,
    pub inner_w: usize,
    pub clock_offset: usize,
    /// Reminder row for this frame, or `None` when it would not fit or the
    /// user turned it off. The row is omitted from the frame in that case, so
    /// height and content always agree.
    pub hint: Option<&'static str>,
    /// Total height of the block, borders included.
    pub rows: usize,
}

/// Screen rectangle covered by one drawn frame.
///
/// Used to erase exactly what the next frame will not overwrite, instead of
/// clearing the whole screen on every layout change.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FrameRect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

impl FrameRect {
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Self {
            x: u16::try_from(x).unwrap_or(u16::MAX),
            y: u16::try_from(y).unwrap_or(u16::MAX),
            w: u16::try_from(w).unwrap_or(u16::MAX),
            h: u16::try_from(h).unwrap_or(u16::MAX),
        }
    }

    /// Smallest rectangle containing both frames.
    pub fn union(self, other: Self) -> Self {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        Self {
            x,
            y,
            w: (self.x + self.w).max(other.x + other.w).saturating_sub(x),
            h: (self.y + self.h).max(other.y + other.h).saturating_sub(y),
        }
    }
}

/// The content of one clock face, decoupled from where it is placed.
pub struct Frame<'a> {
    pub rows: &'a [String],
    pub time_chars: &'a [char],
    pub seconds_start: Option<usize>,
    pub date: Option<&'a str>,
}

pub fn compute_layout(
    cols: usize,
    lines: usize,
    clock_w: usize,
    date_w: usize,
    theme: ResolvedTheme,
    face: FaceLayout,
) -> Layout {
    let content_w = clock_w.max(if face.date { date_w } else { 0 });
    let avail = cols.saturating_sub(FRAME_COLUMNS);
    let hint = pick_hint(content_w, avail, face.hint);
    let hint_w = hint.map_or(0, |h| h.chars().count());
    let inner_w = (content_w.max(hint_w) + H_PAD_INSIDE * 2).min(avail);
    let box_w = inner_w + FRAME_COLUMNS;
    let rows = theme.block_height(face);
    Layout {
        pad_x: cols.saturating_sub(box_w) / 2,
        pad_y: lines.saturating_sub(rows) / 2,
        inner_w,
        clock_offset: inner_w.saturating_sub(clock_w) / 2,
        hint,
        rows,
    }
}

/// Longest reminder that still leaves the whole frame inside the terminal.
/// Returning `None` removes the row entirely, so nothing is ever half-printed.
fn pick_hint(content_w: usize, avail: usize, show_hint: bool) -> Option<&'static str> {
    if !show_hint {
        return None;
    }
    [HINT_FULL, HINT_SHORT]
        .into_iter()
        .find(|hint| content_w.max(hint.chars().count()) + H_PAD_INSIDE * 2 <= avail)
}

/// Erases a previously drawn frame. Callers pass the union of the old and the
/// new rectangle so a shrinking or moving frame leaves no leftovers.
pub fn clear_frame(stdout: &mut impl Write, rect: FrameRect) -> io::Result<()> {
    for row in 0..rect.h {
        queue!(
            stdout,
            cursor::MoveTo(rect.x, rect.y.saturating_add(row)),
            terminal::Clear(terminal::ClearType::UntilNewLine)
        )?;
    }
    Ok(())
}

// Note: `\r\n` endings everywhere — the app runs in raw mode,
// where a bare `\n` does not return the carriage.

pub(crate) fn write_spaces(stdout: &mut impl Write, count: usize) -> io::Result<()> {
    const SPACES: [u8; 64] = [b' '; 64];
    let mut remaining = count;
    while remaining > 0 {
        let chunk = remaining.min(SPACES.len());
        stdout.write_all(&SPACES[..chunk])?;
        remaining -= chunk;
    }
    Ok(())
}

fn write_repeated(stdout: &mut impl Write, s: &str, count: usize) -> io::Result<()> {
    for _ in 0..count {
        stdout.write_all(s.as_bytes())?;
    }
    Ok(())
}

fn write_top(
    stdout: &mut impl Write,
    palette: Palette,
    pad_x: usize,
    inner_w: usize,
) -> io::Result<()> {
    write_spaces(stdout, pad_x)?;
    queue!(stdout, SetForegroundColor(palette.border()))?;
    write!(stdout, "╭")?;
    write_repeated(stdout, "─", inner_w)?;
    write!(stdout, "╮\r\n")?;
    queue!(stdout, ResetColor)?;
    Ok(())
}

fn write_bottom(
    stdout: &mut impl Write,
    palette: Palette,
    pad_x: usize,
    inner_w: usize,
) -> io::Result<()> {
    write_spaces(stdout, pad_x)?;
    queue!(stdout, SetForegroundColor(palette.border()))?;
    write!(stdout, "╰")?;
    write_repeated(stdout, "─", inner_w)?;
    write!(stdout, "╯")?;
    queue!(stdout, ResetColor)?;
    Ok(())
}

fn write_empty(
    stdout: &mut impl Write,
    palette: Palette,
    pad_x: usize,
    inner_w: usize,
) -> io::Result<()> {
    write_spaces(stdout, pad_x)?;
    queue!(stdout, SetForegroundColor(palette.border()))?;
    write!(stdout, "│")?;
    queue!(stdout, ResetColor)?;
    write_spaces(stdout, inner_w)?;
    queue!(stdout, SetForegroundColor(palette.border()))?;
    write!(stdout, "│\r\n")?;
    queue!(stdout, ResetColor)?;
    Ok(())
}

fn write_clock_rows(
    stdout: &mut impl Write,
    layout: &Layout,
    frame: &Frame<'_>,
    palette: Palette,
    style: Style,
) -> io::Result<()> {
    let rendered_w = style.face.glyph_w() * style.scale_x;

    queue!(stdout, SetAttribute(Attribute::Bold))?;
    for row in frame.rows {
        write_spaces(stdout, layout.pad_x)?;
        queue!(stdout, SetForegroundColor(palette.border()))?;
        write!(stdout, "│")?;
        write_spaces(stdout, layout.clock_offset)?;

        let mut row_chars = row.chars();
        let mut used = layout.clock_offset;

        for (i, _ch) in frame.time_chars.iter().enumerate() {
            if used >= layout.inner_w {
                break;
            }
            queue!(
                stdout,
                SetForegroundColor(palette.time_color(i, frame.seconds_start))
            )?;
            for _ in 0..rendered_w {
                if used >= layout.inner_w {
                    break;
                }
                let c = row_chars.next().unwrap_or(' ');
                write!(stdout, "{c}")?;
                used += 1;
            }
            if i + 1 < frame.time_chars.len() && used < layout.inner_w {
                for _ in 0..style.gap_x {
                    if used >= layout.inner_w {
                        break;
                    }
                    let c = row_chars.next().unwrap_or(' ');
                    write!(stdout, "{c}")?;
                    used += 1;
                }
            }
        }

        let right = layout.inner_w.saturating_sub(used);
        write_spaces(stdout, right)?;
        queue!(stdout, SetForegroundColor(palette.border()))?;
        write!(stdout, "│\r\n")?;
        queue!(stdout, ResetColor)?;
    }
    queue!(stdout, SetAttribute(Attribute::Reset))?;
    Ok(())
}

fn write_centered(
    stdout: &mut impl Write,
    layout: &Layout,
    text: &str,
    palette: Palette,
    color: crossterm::style::Color,
) -> io::Result<()> {
    let text_w = text.chars().count();
    let offset = layout.inner_w.saturating_sub(text_w) / 2;
    write_spaces(stdout, layout.pad_x)?;
    queue!(stdout, SetForegroundColor(palette.border()))?;
    write!(stdout, "│")?;
    write_spaces(stdout, offset)?;
    queue!(stdout, SetForegroundColor(color))?;
    let avail = layout.inner_w.saturating_sub(offset);
    let mut shown_w = 0;
    for ch in text.chars().take(avail) {
        write!(stdout, "{ch}")?;
        shown_w += 1;
    }
    let right = layout.inner_w.saturating_sub(offset + shown_w);
    write_spaces(stdout, right)?;
    queue!(stdout, SetForegroundColor(palette.border()))?;
    write!(stdout, "│\r\n")?;
    queue!(stdout, ResetColor)?;
    Ok(())
}

pub fn render(
    stdout: &mut impl Write,
    layout: &Layout,
    frame: &Frame<'_>,
    theme: ResolvedTheme,
    palette: Palette,
) -> io::Result<()> {
    let style = theme.style();
    for _ in 0..layout.pad_y {
        write!(stdout, "\r\n")?;
    }

    write_top(stdout, palette, layout.pad_x, layout.inner_w)?;
    write_empty(stdout, palette, layout.pad_x, layout.inner_w)?;
    write_clock_rows(stdout, layout, frame, palette, style)?;
    write_empty(stdout, palette, layout.pad_x, layout.inner_w)?;
    if let Some(date) = frame.date {
        write_centered(stdout, layout, date, palette, palette.date())?;
    }
    if let Some(hint) = layout.hint {
        write_centered(stdout, layout, hint, palette, palette.separator())?;
    }
    write_empty(stdout, palette, layout.pad_x, layout.inner_w)?;
    write_bottom(stdout, palette, layout.pad_x, layout.inner_w)?;
    Ok(())
}

/// Every row the block occupies on screen, borders included.
pub fn frame_rect(layout: &Layout) -> FrameRect {
    FrameRect::new(
        layout.pad_x,
        layout.pad_y,
        layout.inner_w + FRAME_COLUMNS,
        layout.rows,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Face, line_width};
    use crate::theme::Theme;

    const FULL: FaceLayout = FaceLayout::new(true, true, true);

    fn large() -> ResolvedTheme {
        ResolvedTheme::concrete(Theme::Large)
    }

    fn small() -> ResolvedTheme {
        ResolvedTheme::concrete(Theme::Small)
    }

    fn frame_for(theme: ResolvedTheme) -> (Vec<String>, Vec<char>, usize) {
        let style = theme.style();
        let time = "12:34:56";
        let rows = crate::font::render_big(time, true, style);
        let time_chars: Vec<char> = time.chars().collect();
        let width = line_width(time_chars.len(), style);
        (rows, time_chars, width)
    }

    #[test]
    fn centers_block_on_roomy_screen() {
        let hint_w = HINT_FULL.chars().count();
        let expected_inner = 20.max(hint_w) + 8;
        let l = compute_layout(140, 40, 20, 10, large(), FULL);
        assert_eq!(l.inner_w, expected_inner);
        assert_eq!(l.pad_x, (140 - (expected_inner + 2)) / 2);
        assert_eq!(l.pad_y, (40 - l.rows) / 2);
        assert_eq!(l.clock_offset, (expected_inner - 20) / 2);
        assert_eq!(l.hint, Some(HINT_FULL));
    }

    #[test]
    fn clamps_to_narrow_screen() {
        let l = compute_layout(20, 24, 94, 27, large(), FULL);
        assert_eq!(l.inner_w, 18);
        assert_eq!(l.pad_x, 0);
        assert_eq!(l.clock_offset, 0);
        assert_eq!(l.hint, None);
    }

    #[test]
    fn hint_degrades_then_disappears() {
        let clock = 20;
        // Wide enough for the full reminder.
        let wide = compute_layout(120, 40, clock, 10, large(), FULL);
        assert_eq!(wide.hint, Some(HINT_FULL));
        // Too narrow for the full line, but the short one fits.
        let mid = compute_layout(HINT_FULL.chars().count() + 8, 40, clock, 10, large(), FULL);
        assert_eq!(mid.hint, Some(HINT_SHORT));
        assert!(mid.inner_w >= HINT_SHORT.chars().count());
        // Too narrow for any reminder: the row disappears entirely.
        let narrow = compute_layout(HINT_SHORT.chars().count() + 8, 40, clock, 10, large(), FULL);
        assert_eq!(narrow.hint, None);
    }

    #[test]
    fn hint_disabled_drops_the_row_and_the_height() {
        let no_hint = FaceLayout::new(true, true, false);
        let with = compute_layout(120, 40, 20, 10, large(), FULL);
        let without = compute_layout(120, 40, 20, 10, large(), no_hint);
        assert_eq!(without.hint, None);
        assert_eq!(with.rows - without.rows, 1);
        assert!(without.inner_w <= with.inner_w);
    }

    #[test]
    fn hint_never_drives_the_frame_wider_than_the_screen() {
        for cols in 24..=140 {
            let l = compute_layout(cols, 40, 31, 27, small(), FULL);
            assert!(l.inner_w + FRAME_COLUMNS <= cols, "cols={cols}");
            assert!(l.pad_x + l.inner_w + FRAME_COLUMNS <= cols, "cols={cols}");
            if let Some(hint) = l.hint {
                assert!(
                    hint.chars().count() <= l.inner_w,
                    "hint is cut off at cols={cols}"
                );
            }
        }
    }

    #[test]
    fn hidden_date_ignores_date_width_and_shrinks_height() {
        let face = FaceLayout::new(true, true, true);
        let no_date = FaceLayout::new(true, false, true);
        let shown = compute_layout(140, 41, 20, 80, large(), face);
        let hidden = compute_layout(140, 41, 20, 80, large(), no_date);
        assert_eq!(shown.rows - hidden.rows, 1, "one row less without the date");
        // A hidden date must not influence the width: a 40-character date and
        // an 80-character one produce the exact same frame.
        let wide_date = compute_layout(140, 41, 20, 80, large(), no_date);
        let narrow_date = compute_layout(140, 41, 20, 40, large(), no_date);
        assert_eq!(hidden.inner_w, wide_date.inner_w);
        assert_eq!(hidden.inner_w, narrow_date.inner_w);
        // When the date is the widest element it does drive the frame.
        let date_driven = compute_layout(200, 41, 20, 160, large(), face);
        let date_hidden = compute_layout(200, 41, 20, 160, large(), no_date);
        assert!(date_hidden.inner_w < date_driven.inner_w);
    }

    #[test]
    fn block_height_follows_face() {
        assert_eq!(large().block_height(FULL), Face::Block.glyph_h() + 7);
        assert_eq!(
            ResolvedTheme::concrete(Theme::Medium).block_height(FULL),
            Face::Block.glyph_h() + 7
        );
        assert_eq!(small().block_height(FULL), Face::Compact.glyph_h() + 7);
        assert_eq!(
            large().block_height(FaceLayout::new(true, true, false)),
            Face::Block.glyph_h() + 6
        );
        assert_eq!(
            small().block_height(FaceLayout::new(true, false, false)),
            Face::Compact.glyph_h() + 5
        );
    }

    #[test]
    fn layout_fits_respective_target_terminals() {
        // Small theme fits on a compact terminal.
        let small_w = line_width(8, small().style());
        let l_small = compute_layout(45, 14, small_w, 27, small(), FULL);
        assert!(small().fits(45, 14, FULL));
        assert!(small_w <= l_small.inner_w);
        assert!(l_small.rows <= 14);

        // Medium theme fits comfortably on a standard 80x24 terminal.
        let medium = ResolvedTheme::concrete(Theme::Medium);
        let med_w = line_width(8, medium.style());
        let l_med = compute_layout(80, 24, med_w, 31, medium, FULL);
        assert!(med_w <= l_med.inner_w);
        assert!(l_med.inner_w + FRAME_COLUMNS <= 80);
        assert!(l_med.rows <= 24);

        // Large theme fits on a wide terminal.
        let large_w = line_width(8, large().style());
        let l_large = compute_layout(120, 30, large_w, 31, large(), FULL);
        assert!(large_w <= l_large.inner_w);
        assert!(l_large.inner_w + FRAME_COLUMNS <= 120);
        assert!(l_large.rows <= 30);
    }

    /// Regression guard for the clipped-clock bug: whenever a theme claims it
    /// fits, the layout it produces must actually contain the whole face.
    #[test]
    fn fitting_theme_never_clips_the_face() {
        let faces = [
            FaceLayout::new(true, true, true),
            FaceLayout::new(false, true, true),
            FaceLayout::new(true, false, true),
            FaceLayout::new(true, false, false),
            FaceLayout::new(false, false, false),
        ];
        for theme in [Theme::Large, Theme::Medium, Theme::Small] {
            let resolved = ResolvedTheme::concrete(theme);
            for face in faces {
                for cols in 1..=160usize {
                    for lines in 1..=40usize {
                        if !resolved.fits(cols, lines, face) {
                            continue;
                        }
                        let time = if face.seconds { "12:34:56" } else { "12:34" };
                        let time_chars: Vec<char> = time.chars().collect();
                        let clock_w = resolved.clock_width(face);
                        let rows = crate::font::render_big(time, true, resolved.style());
                        let date = "September 30, 2026 · Wednesday · AM";
                        let l = compute_layout(
                            cols,
                            lines,
                            clock_w,
                            date.chars().count(),
                            resolved,
                            face,
                        );
                        assert!(
                            l.inner_w >= clock_w,
                            "{theme:?} clipped: inner_w={} clock_w={clock_w} cols={cols}",
                            l.inner_w
                        );
                        assert!(
                            l.clock_offset + clock_w <= l.inner_w,
                            "{theme:?} clock runs past the frame at cols={cols}"
                        );
                        assert!(
                            l.pad_x + l.inner_w + FRAME_COLUMNS <= cols,
                            "{theme:?} frame wider than the screen at cols={cols}"
                        );
                        assert!(l.rows <= lines, "{theme:?} frame taller than the screen");
                        let mut buf: Vec<u8> = Vec::new();
                        render(
                            &mut buf,
                            &l,
                            &Frame {
                                rows: &rows,
                                time_chars: &time_chars,
                                seconds_start: Some(6),
                                date: face.date.then_some(date),
                            },
                            resolved,
                            Palette::Purple,
                        )
                        .unwrap();
                        let out = String::from_utf8(buf).unwrap();
                        assert!(out.contains(&"─".repeat(l.inner_w)), "top width wrong");
                    }
                }
            }
        }
    }

    #[test]
    fn render_smoke_frame_date_and_hint() {
        for theme in Theme::all() {
            let resolved = theme.resolve(140, 40, FULL);
            let (rows, time_chars, clock_w) = frame_for(resolved);
            let date = "September 30, 2026 · Wednesday";
            let l = compute_layout(140, 40, clock_w, date.chars().count(), resolved, FULL);
            let mut buf: Vec<u8> = Vec::new();
            render(
                &mut buf,
                &l,
                &Frame {
                    rows: &rows,
                    time_chars: &time_chars,
                    seconds_start: Some(6),
                    date: Some(date),
                },
                resolved,
                Palette::Purple,
            )
            .unwrap();
            let out = String::from_utf8(buf).unwrap();
            assert!(out.contains('╭') && out.contains('╯'), "frame missing");
            assert!(out.contains(date), "date missing");
            assert!(out.contains(HINT_FULL), "hint missing");
            assert!(out.contains(&"─".repeat(l.inner_w)), "top width wrong");
        }
    }

    #[test]
    fn render_without_date_omits_date_row() {
        let (rows, time_chars, clock_w) = frame_for(large());
        let date = "September 30, 2026 · Wednesday";
        let face = FaceLayout::new(true, false, true);
        let l = compute_layout(140, 40, clock_w, date.chars().count(), large(), face);
        let mut buf: Vec<u8> = Vec::new();
        render(
            &mut buf,
            &l,
            &Frame {
                rows: &rows,
                time_chars: &time_chars,
                seconds_start: Some(6),
                date: None,
            },
            large(),
            Palette::Purple,
        )
        .unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(!out.contains(date), "date should be hidden");
        assert!(out.contains(HINT_FULL), "hint missing");
    }

    #[test]
    fn render_without_hint_omits_the_row_entirely() {
        let face = FaceLayout::new(true, true, false);
        let (rows, time_chars, clock_w) = frame_for(large());
        let date = "September 30, 2026 · Wednesday";
        let l = compute_layout(140, 40, clock_w, date.chars().count(), large(), face);
        let mut buf: Vec<u8> = Vec::new();
        render(
            &mut buf,
            &l,
            &Frame {
                rows: &rows,
                time_chars: &time_chars,
                seconds_start: Some(6),
                date: Some(date),
            },
            large(),
            Palette::Purple,
        )
        .unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains(date), "date should stay");
        assert!(!out.contains("q quit"), "hint should be gone");
    }

    #[test]
    fn render_narrow_never_panics_or_wraps_wide() {
        for theme in Theme::all() {
            let resolved = theme.resolve(30, 24, FULL);
            let (rows, time_chars, _) = frame_for(resolved);
            let l = compute_layout(30, 24, 94, 27, resolved, FULL);
            let mut buf: Vec<u8> = Vec::new();
            render(
                &mut buf,
                &l,
                &Frame {
                    rows: &rows,
                    time_chars: &time_chars,
                    seconds_start: Some(6),
                    date: Some("September 30, 2026 · Wednesday"),
                },
                resolved,
                Palette::Purple,
            )
            .unwrap();
            assert!(String::from_utf8(buf).is_ok());
        }
    }

    #[test]
    fn render_smoke_all_palettes() {
        let (rows, time_chars, clock_w) = frame_for(large());
        let date = "September 30, 2026 · Wednesday";
        let l = compute_layout(140, 40, clock_w, date.chars().count(), large(), FULL);
        for &palette in Palette::all() {
            let mut buf: Vec<u8> = Vec::new();
            render(
                &mut buf,
                &l,
                &Frame {
                    rows: &rows,
                    time_chars: &time_chars,
                    seconds_start: Some(6),
                    date: Some(date),
                },
                large(),
                palette,
            )
            .unwrap();
            let out = String::from_utf8(buf).unwrap();
            assert!(out.contains('╭') && out.contains('╯'));
            assert!(out.contains(date));
            assert!(out.contains(HINT_FULL));
        }
    }

    #[test]
    fn frame_rect_covers_the_whole_block() {
        let (rows, time_chars, clock_w) = frame_for(large());
        let l = compute_layout(140, 40, clock_w, 27, large(), FULL);
        let rect = frame_rect(&l);
        assert_eq!(usize::from(rect.x), l.pad_x);
        assert_eq!(usize::from(rect.y), l.pad_y);
        assert_eq!(usize::from(rect.w), l.inner_w + FRAME_COLUMNS);
        assert_eq!(usize::from(rect.h), l.rows);
        let rendered = render(
            &mut Vec::new(),
            &l,
            &Frame {
                rows: &rows,
                time_chars: &time_chars,
                seconds_start: Some(6),
                date: None,
            },
            large(),
            Palette::Purple,
        );
        assert!(rendered.is_ok());
    }

    #[test]
    fn clear_frame_erases_every_row_of_the_rectangle() {
        let mut buf: Vec<u8> = Vec::new();
        clear_frame(&mut buf, FrameRect::new(4, 2, 10, 3)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert_eq!(out.matches("\u{1b}[K").count(), 3);
        for y in 2..5 {
            let row = y + 1;
            assert!(out.contains(&format!("\u{1b}[{row};5H")), "row {y}");
        }
    }

    #[test]
    fn rect_union_covers_both_frames() {
        let a = FrameRect::new(10, 10, 40, 14);
        let b = FrameRect::new(4, 6, 20, 12);
        let u = a.union(b);
        assert_eq!((u.x, u.y, u.w, u.h), (4, 6, 46, 18));
        assert_eq!(a.union(a), a);
        // A rectangle fully inside the other one does not change the union.
        let inside = FrameRect::new(12, 12, 2, 2);
        assert_eq!(a.union(inside), a);
        // A rectangle to the left and above does grow it.
        let corner = FrameRect::new(0, 0, 1, 1);
        assert_eq!(a.union(corner), FrameRect::new(0, 0, 50, 24));
    }

    #[test]
    fn writing_spaces_handles_any_count() {
        for count in [0usize, 1, 63, 64, 65, 200] {
            let mut buf: Vec<u8> = Vec::new();
            write_spaces(&mut buf, count).unwrap();
            assert_eq!(buf.len(), count);
            assert!(buf.iter().all(|b| *b == b' '));
        }
    }
}
