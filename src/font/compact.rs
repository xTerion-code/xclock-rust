/// One character in the compact plain-ASCII font.
/// Only `_`, `|` and `.` are used, so it renders in any font.
/// `colon_visible` controls colon blinking.
pub fn compact_glyph(ch: char, colon_visible: bool) -> [&'static str; 3] {
    match ch {
        '0' => [" _ ", "| |", "|_|"],
        '1' => ["   ", "  |", "  |"],
        '2' => [" _ ", " _|", "|_ "],
        '3' => [" _ ", " _|", " _|"],
        '4' => ["   ", "|_|", "  |"],
        '5' => [" _ ", "|_ ", " _|"],
        '6' => [" _ ", "|_ ", "|_|"],
        '7' => [" _ ", "  |", "  |"],
        '8' => [" _ ", "|_|", "|_|"],
        '9' => [" _ ", "|_|", " _|"],
        ':' if colon_visible => ["   ", " . ", " . "],
        _ => ["   ", "   ", "   "],
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
    fn every_compact_glyph_row_is_glyph_w_wide() {
        for ch in all_glyph_inputs() {
            for visible in [true, false] {
                for row in compact_glyph(ch, visible) {
                    assert_eq!(
                        row.chars().count(),
                        Face::Compact.glyph_w(),
                        "glyph {ch:?} visible={visible} row {row:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn compact_glyphs_are_plain_ascii() {
        for ch in all_glyph_inputs() {
            for visible in [true, false] {
                for row in compact_glyph(ch, visible) {
                    assert!(
                        row.is_ascii(),
                        "glyph {ch:?} row {row:?} is not ASCII"
                    );
                }
            }
        }
    }

    #[test]
    fn compact_colon_dots_stay_off_the_top_row() {
        // Dots share the top row with `_` segments there, which reads as
        // noise; mid+bottom rows sit in the whitespace between glyphs.
        assert_eq!(compact_glyph(':', true), ["   ", " . ", " . "]);
        assert_eq!(compact_glyph(':', false), ["   ", "   ", "   "]);
    }
}
