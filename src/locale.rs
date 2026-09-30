use chrono::{DateTime, Datelike, Local, Weekday};

pub fn weekday_ru(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "Понедельник",
        Weekday::Tue => "Вторник",
        Weekday::Wed => "Среда",
        Weekday::Thu => "Четверг",
        Weekday::Fri => "Пятница",
        Weekday::Sat => "Суббота",
        Weekday::Sun => "Воскресенье",
    }
}

pub fn month_ru(month: u32) -> &'static str {
    match month {
        1 => "января",
        2 => "февраля",
        3 => "марта",
        4 => "апреля",
        5 => "мая",
        6 => "июня",
        7 => "июля",
        8 => "августа",
        9 => "сентября",
        10 => "октября",
        11 => "ноября",
        12 => "декабря",
        _ => "",
    }
}

/// "30 сентября 2026 · Вторник"
pub fn format_date(now: &DateTime<Local>) -> String {
    format!(
        "{} {} {} · {}",
        now.day(),
        month_ru(now.month()),
        now.year(),
        weekday_ru(now.weekday())
    )
}
