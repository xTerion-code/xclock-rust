use super::hour_format::HourFormat;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DisplayOptions {
    pub show_seconds: bool,
    pub hour_format: HourFormat,
    pub blink_colon: bool,
}

impl DisplayOptions {
    pub fn new(show_seconds: bool) -> Self {
        Self {
            show_seconds,
            hour_format: HourFormat::default(),
            blink_colon: true,
        }
    }

    pub fn with_hour_format(show_seconds: bool, hour_format: HourFormat) -> Self {
        Self {
            show_seconds,
            hour_format,
            blink_colon: true,
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
}

impl Default for DisplayOptions {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(o.hour_format, crate::display::HourFormat::H24);
        o.toggle_hour_format();
        assert_eq!(o.hour_format, crate::display::HourFormat::H12);
        o.toggle_hour_format();
        assert_eq!(o.hour_format, crate::display::HourFormat::H24);
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
}
