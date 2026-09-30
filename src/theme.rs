use crossterm::style::Color;

pub const BORDER: Color = Color::DarkGrey;
pub const SEPARATOR: Color = Color::DarkGrey;
pub const DATE: Color = Color::Grey;
pub const HOURS_MINUTES: Color = Color::White;
pub const SECONDS: Color = Color::Cyan;

/// "HH:MM:SS" -> 0..=4 hours+minutes in white, the rest (second colon
/// and seconds) in cyan.
pub fn time_char_color(index: usize) -> Color {
    if index <= 4 {
        HOURS_MINUTES
    } else {
        SECONDS
    }
}
