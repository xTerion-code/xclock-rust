/// One character in the large block font.
/// `colon_visible` controls colon blinking.
pub fn block_glyph(ch: char, colon_visible: bool) -> [&'static str; 7] {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::Face;

    fn all_glyph_inputs() -> Vec<char> {
        ('0'..='9').chain([':', ' ', 'X']).collect()
    }

    #[test]
    fn every_block_glyph_row_is_glyph_w_wide() {
        for ch in all_glyph_inputs() {
            for visible in [true, false] {
                for row in block_glyph(ch, visible) {
                    assert_eq!(
                        row.chars().count(),
                        Face::Block.glyph_w(),
                        "glyph {ch:?} visible={visible} row {row:?}"
                    );
                }
            }
        }
    }
}
