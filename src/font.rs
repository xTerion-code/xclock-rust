pub const GLYPH_H: usize = 7;
pub const GLYPH_W: usize = 5;

/// Horizontal scale: terminal cells are ~2x taller than wide,
/// so each glyph column is doubled to keep digits proportionate.
pub const SCALE_X: usize = 2;
/// Gap between glyphs after scaling.
pub const GAP_X: usize = 2;

/// Rendered (scaled) glyph width in terminal columns.
pub const RENDERED_W: usize = GLYPH_W * SCALE_X;

/// One character in the large block font.
/// `colon_visible` controls colon blinking.
pub fn glyph(ch: char, colon_visible: bool) -> [&'static str; GLYPH_H] {
    match ch {
        '0' => [
            " ███ ",
            "█   █",
            "█   █",
            "█   █",
            "█   █",
            "█   █",
            " ███ ",
        ],
        '1' => [
            "  █  ",
            " ██  ",
            "  █  ",
            "  █  ",
            "  █  ",
            "  █  ",
            "█████",
        ],
        '2' => [
            "█████",
            "    █",
            "    █",
            "█████",
            "█    ",
            "█    ",
            "█████",
        ],
        '3' => [
            "█████",
            "    █",
            "    █",
            " ████",
            "    █",
            "    █",
            "█████",
        ],
        '4' => [
            "█   █",
            "█   █",
            "█   █",
            "█████",
            "    █",
            "    █",
            "    █",
        ],
        '5' => [
            "█████",
            "█    ",
            "█    ",
            "█████",
            "    █",
            "    █",
            "█████",
        ],
        '6' => [
            "█████",
            "█    ",
            "█    ",
            "█████",
            "█   █",
            "█   █",
            "█████",
        ],
        '7' => [
            "█████",
            "    █",
            "    █",
            "   █ ",
            "  █  ",
            "  █  ",
            "  █  ",
        ],
        '8' => [
            " ███ ",
            "█   █",
            "█   █",
            " ███ ",
            "█   █",
            "█   █",
            " ███ ",
        ],
        '9' => [
            "█████",
            "█   █",
            "█   █",
            "█████",
            "    █",
            "    █",
            "█████",
        ],
        ':' if colon_visible => [
            "     ",
            "     ",
            "  ▮  ",
            "     ",
            "     ",
            "  ▮  ",
            "     ",
        ],
        _ => [
            "     ",
            "     ",
            "     ",
            "     ",
            "     ",
            "     ",
            "     ",
        ],
    }
}

/// Time string → glyph rows. Each glyph column is repeated `SCALE_X`
/// times; glyphs are separated by `GAP_X` spaces.
pub fn render_big(time: &str, colon_visible: bool) -> Vec<String> {
    let chars: Vec<char> = time.chars().collect();
    let mut rows = vec![String::new(); GLYPH_H];
    for (i, ch) in chars.iter().enumerate() {
        let g = glyph(*ch, colon_visible);
        for r in 0..GLYPH_H {
            for c in g[r].chars() {
                for _ in 0..SCALE_X {
                    rows[r].push(c);
                }
            }
            if i + 1 < chars.len() {
                for _ in 0..GAP_X {
                    rows[r].push(' ');
                }
            }
        }
    }
    rows
}

/// Width of a rendered line of `n_chars` characters.
pub fn line_width(n_chars: usize) -> usize {
    n_chars * RENDERED_W + n_chars.saturating_sub(1) * GAP_X
}
