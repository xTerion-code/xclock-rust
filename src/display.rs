use chrono::{DateTime, Datelike, Local, Timelike, Weekday};

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

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "12" | "12h" => Some(HourFormat::H12),
            "24" | "24h" => Some(HourFormat::H24),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DisplayOptions {
    pub show_seconds: bool,
    pub hour_format: HourFormat,
    pub blink_colon: bool,
    pub show_date: bool,
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
        }
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
}

impl Default for DisplayOptions {
    fn default() -> Self {
        Self::new(true)
    }
}

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
        assert_eq!(HourFormat::from_str("12"), Some(HourFormat::H12));
        assert_eq!(HourFormat::from_str("12h"), Some(HourFormat::H12));
        assert_eq!(HourFormat::from_str("24"), Some(HourFormat::H24));
        assert_eq!(HourFormat::from_str("24H"), Some(HourFormat::H24));
        assert_eq!(HourFormat::from_str("13"), None);
        assert_eq!(HourFormat::from_str(""), None);
    }

    #[test]
    fn default_shows_seconds() {
        assert!(DisplayOptions::default().show_seconds);
    }

    #[test]
    fn toggle_flips_show_seconds() {
        let mut o = DisplayOptions::default();
        o.toggle_seconds();
        assert!(!o.show_seconds);
        o.toggle_seconds();
        assert!(o.show_seconds);
    }

    #[test]
    fn toggle_hour_format_flips_format() {
        let mut o = DisplayOptions::default();
        assert_eq!(o.hour_format, HourFormat::H24);
        o.toggle_hour_format();
        assert_eq!(o.hour_format, HourFormat::H12);
        o.toggle_hour_format();
        assert_eq!(o.hour_format, HourFormat::H24);
    }

    #[test]
    fn blink_defaults_on_and_toggles() {
        let mut o = DisplayOptions::default();
        assert!(o.blink_colon);
        o.toggle_blink();
        assert!(!o.blink_colon);
        o.toggle_blink();
        assert!(o.blink_colon);
    }

    #[test]
    fn date_defaults_shown_and_toggles() {
        let mut o = DisplayOptions::default();
        assert!(o.show_date);
        o.toggle_date();
        assert!(!o.show_date);
        o.toggle_date();
        assert!(o.show_date);
    }

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
