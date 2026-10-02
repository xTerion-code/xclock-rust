use crossterm::style::Color;

pub const BORDER: Color = Color::DarkGrey;
pub const SEPARATOR: Color = Color::DarkGrey;
pub const DATE: Color = Color::Grey;
pub const HOURS_MINUTES: Color = Color::White;
pub const SECONDS: Color = Color::Cyan;

/// First char index of the seconds part in "HH:MM:SS".
pub const SECONDS_START: usize = 5;

/// "HH:MM:SS" -> chars before `SECONDS_START` (hours, minutes and the
/// first colon) in white, the second colon and seconds in cyan.
pub fn time_char_color(index: usize) -> Color {
    if index < SECONDS_START {
        HOURS_MINUTES
    } else {
        SECONDS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_hours_minutes_from_seconds() {
        for i in 0..SECONDS_START {
            assert_eq!(time_char_color(i), HOURS_MINUTES, "index {i}");
        }
        for i in SECONDS_START..8 {
            assert_eq!(time_char_color(i), SECONDS, "index {i}");
        }
    }
}
