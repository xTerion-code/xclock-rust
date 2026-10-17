use chrono::{DateTime, Datelike, Local, Timelike, Weekday};
use std::str::FromStr;

use crate::font::FaceLayout;
use crate::parse::ParseError;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum HourFormat {
    #[default]
    H24,
    H12,
}

impl HourFormat {
    pub fn is_12h(self) -> bool {
        self == HourFormat::H12
    }

    pub fn toggle(self) -> Self {
        match self {
            HourFormat::H24 => HourFormat::H12,
            HourFormat::H12 => HourFormat::H24,
        }
    }
}

impl FromStr for HourFormat {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "12" | "12h" => Ok(HourFormat::H12),
            "24" | "24h" => Ok(HourFormat::H24),
            _ => Err(ParseError::new("hour format", s, "12, 12h, 24, 24h")),
        }
    }
}

/// Everything the user can change while `xclock` is running.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "five independent user-visible switches"
)]
pub struct DisplayOptions {
    pub show_seconds: bool,
    pub hour_format: HourFormat,
    pub blink_colon: bool,
    pub show_date: bool,
    /// Live-keys hint row inside the frame. Off is the mode to use for
    /// wallpapers and tiny panes where the row is pure noise.
    pub show_hint: bool,
}

impl DisplayOptions {
    pub fn new(show_seconds: bool) -> Self {
        Self::with_hour_format(show_seconds, HourFormat::default())
    }

    pub fn with_hour_format(show_seconds: bool, hour_format: HourFormat) -> Self {
        Self {
            show_seconds,
            hour_format,
            blink_colon: true,
            show_date: true,
            show_hint: true,
        }
    }

    /// Geometry-relevant view of these options.
    pub fn face(self) -> FaceLayout {
        FaceLayout::new(self.show_seconds, self.show_date, self.show_hint)
    }

    pub fn toggle_seconds(&mut self) {
        self.show_seconds = !self.show_seconds;
    }

    pub fn toggle_hour_format(&mut self) {
        self.hour_format = self.hour_format.toggle();
    }

    pub fn toggle_blink(&mut self) {
        self.blink_colon = !self.blink_colon;
    }

    pub fn toggle_date(&mut self) {
        self.show_date = !self.show_date;
    }

    pub fn toggle_hint(&mut self) {
        self.show_hint = !self.show_hint;
    }
}

impl Default for DisplayOptions {
    fn default() -> Self {
        Self::new(true)
    }
}

/// The rendered time string plus the index at which the seconds field starts.
///
/// Palettes colour digits by position; deriving the boundary from the string
/// that was actually formatted keeps the colours correct even if the time
/// format changes length.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TimeFace {
    pub text: String,
    pub seconds_start: Option<usize>,
}

pub fn format_time(now: &DateTime<Local>, options: DisplayOptions) -> TimeFace {
    let pattern = match (options.hour_format, options.show_seconds) {
        (HourFormat::H12, true) => "%I:%M:%S",
        (HourFormat::H12, false) => "%I:%M",
        (HourFormat::H24, true) => "%H:%M:%S",
        (HourFormat::H24, false) => "%H:%M",
    };
    let text = now.format(pattern).to_string();
    // `HH:MM[:SS]`: after the first colon come two minute digits, the second
    // colon, and then the seconds. The boundary is derived from the string
    // itself, so a different field layout cannot silently shift the colours.
    let seconds_start = options
        .show_seconds
        .then(|| text.find(':').map_or(text.chars().count(), |i| i + 4));
    TimeFace {
        text,
        seconds_start,
    }
}

pub fn format_meridiem(now: &DateTime<Local>) -> &'static str {
    if now.hour() < 12 { "AM" } else { "PM" }
}

const WEEKDAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

pub fn weekday_name(weekday: Weekday) -> &'static str {
    WEEKDAYS[weekday.num_days_from_monday() as usize]
}

pub fn month_name(month: u32) -> &'static str {
    month
        .checked_sub(1)
        .and_then(|m| MONTHS.get(m as usize))
        .copied()
        .unwrap_or("")
}

pub fn format_date(now: &DateTime<Local>) -> String {
    format!(
        "{} {}, {} · {}",
        month_name(now.month()),
        now.day(),
        now.year(),
        weekday_name(now.weekday())
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn default_is_24h() {
        assert_eq!(HourFormat::default(), HourFormat::H24);
        assert!(!HourFormat::default().is_12h());
        assert!(HourFormat::H12.is_12h());
    }

    #[test]
    fn toggle_flips_format() {
        assert_eq!(HourFormat::H24.toggle(), HourFormat::H12);
        assert_eq!(HourFormat::H12.toggle(), HourFormat::H24);
    }

    #[test]
    fn parses_known_ids() {
        assert_eq!("12".parse::<HourFormat>(), Ok(HourFormat::H12));
        assert_eq!("12h".parse::<HourFormat>(), Ok(HourFormat::H12));
        assert_eq!("24".parse::<HourFormat>(), Ok(HourFormat::H24));
        assert_eq!("24H".parse::<HourFormat>(), Ok(HourFormat::H24));
        assert!("13".parse::<HourFormat>().is_err());
        assert!("".parse::<HourFormat>().is_err());
    }

    #[test]
    fn parse_error_names_the_option() {
        let e = "13".parse::<HourFormat>().unwrap_err();
        assert_eq!(
            e.to_string(),
            "unknown hour format `13` (expected one of: 12, 12h, 24, 24h)"
        );
    }

    #[test]
    fn default_shows_seconds() {
        assert!(DisplayOptions::default().show_seconds);
        assert!(DisplayOptions::default().show_hint);
    }

    #[test]
    fn toggles_flip_every_option() {
        let mut o = DisplayOptions::default();
        o.toggle_seconds();
        assert!(!o.show_seconds);
        o.toggle_seconds();
        assert!(o.show_seconds);
        o.toggle_hour_format();
        assert_eq!(o.hour_format, HourFormat::H12);
        o.toggle_hour_format();
        assert_eq!(o.hour_format, HourFormat::H24);
        assert!(o.blink_colon);
        o.toggle_blink();
        assert!(!o.blink_colon);
        assert!(o.show_date);
        o.toggle_date();
        assert!(!o.show_date);
        o.toggle_hint();
        assert!(!o.show_hint);
        o.toggle_hint();
        assert!(o.show_hint);
    }

    #[test]
    fn face_mirrors_the_geometry_flags() {
        let mut o = DisplayOptions::default();
        assert_eq!(o.face(), FaceLayout::new(true, true, true));
        o.toggle_date();
        o.toggle_hint();
        assert_eq!(o.face(), FaceLayout::new(true, false, false));
    }

    #[test]
    fn formats_with_and_without_seconds() {
        let now = Local.with_ymd_and_hms(2026, 9, 30, 12, 34, 56).unwrap();
        assert_eq!(
            format_time(&now, DisplayOptions::new(true)).text,
            "12:34:56"
        );
        assert_eq!(format_time(&now, DisplayOptions::new(false)).text, "12:34");
    }

    #[test]
    fn formats_12h_with_leading_zero() {
        let h12 = DisplayOptions::with_hour_format(true, HourFormat::H12);
        let h12_no_sec = DisplayOptions::with_hour_format(false, HourFormat::H12);
        let midnight = Local.with_ymd_and_hms(2026, 9, 30, 0, 5, 6).unwrap();
        let noon = Local.with_ymd_and_hms(2026, 9, 30, 12, 34, 56).unwrap();
        let evening = Local.with_ymd_and_hms(2026, 9, 30, 23, 7, 8).unwrap();
        assert_eq!(format_time(&midnight, h12).text, "12:05:06");
        assert_eq!(format_time(&noon, h12).text, "12:34:56");
        assert_eq!(format_time(&evening, h12).text, "11:07:08");
        assert_eq!(format_time(&midnight, h12_no_sec).text, "12:05");
        assert_eq!(format_time(&evening, h12_no_sec).text, "11:07");
    }

    #[test]
    fn seconds_start_points_at_the_second_field() {
        let now = Local.with_ymd_and_hms(2026, 9, 30, 12, 34, 56).unwrap();
        // `12:34:56` — seconds start one column past the minutes colon.
        let with = format_time(&now, DisplayOptions::new(true));
        assert_eq!(with.seconds_start, Some(6));
        assert_eq!(with.text.chars().nth(6), Some('5'));
        let without = format_time(&now, DisplayOptions::new(false));
        assert_eq!(without.seconds_start, None);
    }

    #[test]
    fn seconds_start_follows_the_actual_string_not_a_constant() {
        // Every 12h value is two digits, so the boundary never moves.
        for hour in 0..24 {
            let now = Local.with_ymd_and_hms(2026, 9, 30, hour, 59, 59).unwrap();
            for format in [HourFormat::H12, HourFormat::H24] {
                let face = format_time(&now, DisplayOptions::with_hour_format(true, format));
                let start = face.seconds_start.expect("seconds shown");
                assert_eq!(face.text.chars().count(), 8);
                assert_eq!(face.text.as_bytes()[start], b'5', "hour {hour}");
            }
        }
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

    #[test]
    fn weekday_table_covers_all_days() {
        let names = [
            (Weekday::Mon, "Monday"),
            (Weekday::Tue, "Tuesday"),
            (Weekday::Wed, "Wednesday"),
            (Weekday::Thu, "Thursday"),
            (Weekday::Fri, "Friday"),
            (Weekday::Sat, "Saturday"),
            (Weekday::Sun, "Sunday"),
        ];
        for (day, name) in names {
            assert_eq!(weekday_name(day), name);
        }
    }

    #[test]
    fn month_table_and_invalid_fallback() {
        assert_eq!(month_name(1), "January");
        assert_eq!(month_name(9), "September");
        assert_eq!(month_name(12), "December");
        assert_eq!(month_name(0), "");
        assert_eq!(month_name(13), "");
    }

    #[test]
    fn formats_known_date() {
        let dt = Local.with_ymd_and_hms(2026, 9, 30, 12, 0, 0).unwrap();
        assert_eq!(format_date(&dt), "September 30, 2026 · Wednesday");
    }
}
