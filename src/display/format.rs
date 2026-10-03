use chrono::{DateTime, Local, Timelike};

use super::hour_format::HourFormat;
use super::options::DisplayOptions;

pub fn format_time(now: &DateTime<Local>, options: DisplayOptions) -> String {
    let use_12h = options.hour_format == HourFormat::H12;
    if options.show_seconds {
        if use_12h {
            now.format("%I:%M:%S").to_string()
        } else {
            now.format("%H:%M:%S").to_string()
        }
    } else if use_12h {
        now.format("%I:%M").to_string()
    } else {
        now.format("%H:%M").to_string()
    }
}

pub fn format_meridiem(now: &DateTime<Local>) -> &'static str {
    if now.hour() < 12 { "AM" } else { "PM" }
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

    #[test]
    fn formats_12h_with_leading_zero() {
        let h12 = DisplayOptions::with_hour_format(true, HourFormat::H12);
        let h12_no_sec = DisplayOptions::with_hour_format(false, HourFormat::H12);
        let midnight = Local.with_ymd_and_hms(2026, 9, 30, 0, 5, 6).unwrap();
        let noon = Local.with_ymd_and_hms(2026, 9, 30, 12, 34, 56).unwrap();
        let evening = Local.with_ymd_and_hms(2026, 9, 30, 23, 7, 8).unwrap();
        assert_eq!(format_time(&midnight, h12), "12:05:06");
        assert_eq!(format_time(&noon, h12), "12:34:56");
        assert_eq!(format_time(&evening, h12), "11:07:08");
        assert_eq!(format_time(&midnight, h12_no_sec), "12:05");
        assert_eq!(format_time(&evening, h12_no_sec), "11:07");
    }

    #[test]
    fn meridiem_splits_at_noon_and_midnight() {
        let am = Local.with_ymd_and_hms(2026, 9, 30, 0, 0, 0).unwrap();
        let before_noon = Local.with_ymd_and_hms(2026, 9, 30, 11, 59, 59).unwrap();
        let noon = Local.with_ymd_and_hms(2026, 9, 30, 12, 0, 0).unwrap();
        let pm = Local.with_ymd_and_hms(2026, 9, 30, 23, 59, 59).unwrap();
        assert_eq!(format_meridiem(&am), "AM");
        assert_eq!(format_meridiem(&before_noon), "AM");
        assert_eq!(format_meridiem(&noon), "PM");
        assert_eq!(format_meridiem(&pm), "PM");
    }
}
