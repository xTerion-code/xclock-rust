use chrono::{DateTime, Datelike, Local, Weekday};

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
