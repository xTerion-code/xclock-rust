pub const GLYPH_H: usize = 7;
pub const GLYPH_W: usize = 5;

/// Один символ крупным блочным шрифтом.
/// `colon_visible` управляет миганием двоеточия.
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

/// Строка времени → строки глифов. Разделитель между глифами — 1 пробел.
pub fn render_big(time: &str, colon_visible: bool) -> Vec<String> {
    let chars: Vec<char> = time.chars().collect();
    let mut rows = vec![String::new(); GLYPH_H];
    for (i, ch) in chars.iter().enumerate() {
        let g = glyph(*ch, colon_visible);
        for r in 0..GLYPH_H {
            rows[r].push_str(g[r]);
            if i + 1 < chars.len() {
                rows[r].push(' ');
            }
        }
    }
    rows
}

/// Ширина отрендеренной строки из `n_chars` символов.
pub fn line_width(n_chars: usize) -> usize {
    n_chars * GLYPH_W + n_chars.saturating_sub(1)
}
