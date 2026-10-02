#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DisplayOptions {
    pub show_seconds: bool,
}

impl DisplayOptions {
    pub fn new(show_seconds: bool) -> Self {
        Self { show_seconds }
    }

    pub fn toggle(&mut self) {
        self.show_seconds = !self.show_seconds;
    }
}

impl Default for DisplayOptions {
    fn default() -> Self {
        Self { show_seconds: true }
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
        o.toggle();
        assert!(!o.show_seconds);
        o.toggle();
        assert!(o.show_seconds);
    }
}
