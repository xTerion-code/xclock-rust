use chrono::{DateTime, Local};

use super::options::DisplayOptions;

pub fn format_time(now: &DateTime<Local>, options: DisplayOptions) -> String {
    if options.show_seconds {
        now.format("%H:%M:%S").to_string()
    } else {
        now.format("%H:%M").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn formats_with_and_without_seconds() {
        let now = Local.with_ymd_and_hms(2026, 9, 30, 12, 34, 56).unwrap();
        assert_eq!(format_time(&now, DisplayOptions::new(true)), "12:34:56");
        assert_eq!(format_time(&now, DisplayOptions::new(false)), "12:34");
    }
}
