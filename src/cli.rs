use crate::display::{DisplayOptions, HourFormat};
use crate::theme::{Palette, Theme};

pub struct Config {
    pub theme: Theme,
    pub palette: Palette,
    pub display: DisplayOptions,
}

pub enum Action {
    Run(Config),
    Help,
    Version,
    ListThemes,
    ListPalettes,
}

pub fn parse(args: &[String]) -> Result<Action, String> {
    let mut theme = Theme::Large;
    let mut palette = Palette::default();
    let mut show_seconds = true;
    let mut hour_format = HourFormat::default();
    let mut blink_colon = true;
    let mut show_date = true;

    let mut it = args.iter().skip(1).peekable();
    while let Some(arg) = it.next() {
        if arg == "-h" || arg == "--help" {
            return Ok(Action::Help);
        }
        if arg == "-v" || arg == "-V" || arg == "--version" {
            return Ok(Action::Version);
        }
        if arg == "--list-themes" {
            return Ok(Action::ListThemes);
        }
        if arg == "--list-palettes" {
            return Ok(Action::ListPalettes);
        }
        if arg == "--show-seconds" {
            show_seconds = true;
        } else if arg == "--no-seconds" || arg == "--hide-seconds" {
            show_seconds = false;
        } else if arg == "--blink" {
            blink_colon = true;
        } else if arg == "--no-blink" {
            blink_colon = false;
        } else if arg == "--show-date" {
            show_date = true;
        } else if arg == "--no-date" || arg == "--hide-date" {
            show_date = false;
        } else if arg == "--12h" {
            hour_format = HourFormat::H12;
        } else if arg == "--24h" {
            hour_format = HourFormat::H24;
        } else if arg == "-t" || arg == "--theme" {
            let value = it
                .next()
                .ok_or_else(|| format!("missing value: `{arg}` expects one of: {}", theme_ids()))?;
            theme = Theme::from_str(value).ok_or_else(|| {
                format!("unknown theme `{value}` (expected one of: {})", theme_ids())
            })?;
        } else if let Some(value) = arg.strip_prefix("--theme=") {
            theme = Theme::from_str(value).ok_or_else(|| {
                format!("unknown theme `{value}` (expected one of: {})", theme_ids())
            })?;
        } else if arg == "-p" || arg == "--palette" {
            let value = it.next().ok_or_else(|| {
                format!("missing value: `{arg}` expects one of: {}", palette_ids())
            })?;
            palette = Palette::from_str(value).ok_or_else(|| {
                format!(
                    "unknown palette `{value}` (expected one of: {})",
                    palette_ids()
                )
            })?;
        } else if let Some(value) = arg.strip_prefix("--palette=") {
            palette = Palette::from_str(value).ok_or_else(|| {
                format!(
                    "unknown palette `{value}` (expected one of: {})",
                    palette_ids()
                )
            })?;
        } else if let Some(value) = arg.strip_prefix("--hour-format=") {
            hour_format = HourFormat::from_str(value).ok_or_else(|| {
                format!("unknown hour format `{value}` (expected one of: 12, 12h, 24, 24h)")
            })?;
        } else if arg == "--hour-format" {
            let value = it.next().ok_or_else(|| {
                "missing value: `--hour-format` expects one of: 12, 12h, 24, 24h".to_string()
            })?;
            hour_format = HourFormat::from_str(value).ok_or_else(|| {
                format!("unknown hour format `{value}` (expected one of: 12, 12h, 24, 24h)")
            })?;
        } else {
            return Err(format!("unexpected argument `{arg}`"));
        }
    }

    let mut display = DisplayOptions::with_hour_format(show_seconds, hour_format);
    display.blink_colon = blink_colon;
    display.show_date = show_date;
    Ok(Action::Run(Config {
        theme,
        palette,
        display,
    }))
}

fn theme_ids() -> String {
    Theme::all()
        .iter()
        .map(|t| t.id())
        .collect::<Vec<_>>()
        .join(", ")
}

fn palette_ids() -> String {
    Palette::all()
        .iter()
        .map(|p| p.id())
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn usage() -> String {
    format!(
        "Usage: xclock [--theme <name> | -t <name>] [--palette <name> | -p <name>] [--show-seconds | --no-seconds] [--12h | --24h] [--blink | --no-blink] [--show-date | --no-date] [--list-themes] [--list-palettes] [-v | --version] [--help]\n\
         \n\
         Options:\n  \
           -t, --theme <name>    clock theme (default: large): {}\n  \
           -p, --palette <name>  color palette (default: black-red): {}\n  \
           --show-seconds        show seconds (default)\n  \
           --no-seconds          hide seconds\n  \
           --12h                 use 12-hour format\n  \
           --24h                 use 24-hour format (default)\n  \
           --hour-format <12|24> same as --12h/--24h (last flag wins)\n  \
           --blink               blink colon (default)\n  \
           --no-blink            steady colon\n  \
           --show-date           show date (default)\n  \
           --no-date             hide date\n  \
           --list-themes         list available themes\n  \
           --list-palettes       list available color palettes\n  \
           -v, -V, --version     show version information\n  \
           -h, --help            show this help\n\
         \n\
         Live keys: 1/2/3/a switch theme, t cycles theme, p cycles palette, s toggles seconds, h toggles 12/24h, b toggles blink, d toggles date, q or Esc quits.",
        theme_ids(),
        palette_ids()
    )
}

pub fn themes_list() -> String {
    let mut out = String::from("Available themes:\n");
    for t in Theme::all() {
        out.push_str(&format!("  {:<8} {}\n", t.id(), t.description()));
    }
    out.push_str("\nLive keys: press 1/2/3/a to switch, t to cycle themes, p to cycle palette, s to toggle seconds, h to toggle 12/24h, b to toggle blink, d to toggle date, q or Esc to quit.");
    out
}

pub fn palettes_list() -> String {
    let mut out = String::from("Available palettes:\n");
    for p in Palette::all() {
        out.push_str(&format!("  {:<12} {}\n", p.id(), p.description()));
    }
    out.push_str("\nLive key: press p to cycle palettes.");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    fn run_config(argv: &[&str]) -> Config {
        match parse(&args(argv)).unwrap() {
            Action::Run(c) => c,
            _ => panic!("expected Run for {argv:?}"),
        }
    }

    fn run_theme(argv: &[&str]) -> Theme {
        run_config(argv).theme
    }

    #[test]
    fn defaults_to_large() {
        let c = run_config(&["xclock"]);
        assert_eq!(c.theme, Theme::Large);
        assert_eq!(c.palette, Palette::BlackRed);
        assert!(c.display.show_seconds);
        assert_eq!(c.display.hour_format, HourFormat::H24);
    }

    #[test]
    fn accepts_long_short_and_eq_forms() {
        assert_eq!(run_theme(&["xclock", "--theme", "medium"]), Theme::Medium);
        assert_eq!(run_theme(&["xclock", "-t", "medium"]), Theme::Medium);
        assert_eq!(run_theme(&["xclock", "--theme=medium"]), Theme::Medium);
        assert_eq!(run_theme(&["xclock", "--theme", "small"]), Theme::Small);
        assert_eq!(run_theme(&["xclock", "-t", "small"]), Theme::Small);
        assert_eq!(run_theme(&["xclock", "--theme=small"]), Theme::Small);
        assert_eq!(run_theme(&["xclock", "--theme", "large"]), Theme::Large);
        assert_eq!(run_theme(&["xclock", "--theme", "auto"]), Theme::Auto);
        assert_eq!(run_theme(&["xclock", "-t", "auto"]), Theme::Auto);
        assert_eq!(run_theme(&["xclock", "--theme=auto"]), Theme::Auto);
        assert_eq!(run_theme(&["xclock", "--theme", "classic"]), Theme::Medium);
        assert_eq!(run_theme(&["xclock", "--theme", "modern"]), Theme::Large);
        assert_eq!(run_theme(&["xclock", "--theme", "adaptive"]), Theme::Auto);
    }

    #[test]
    fn toggles_seconds_from_cli() {
        assert!(run_config(&["xclock"]).display.show_seconds);
        assert!(!run_config(&["xclock", "--no-seconds"]).display.show_seconds);
        assert!(
            !run_config(&["xclock", "--hide-seconds"])
                .display
                .show_seconds
        );
        assert!(
            run_config(&["xclock", "--no-seconds", "--show-seconds"])
                .display
                .show_seconds
        );
    }

    #[test]
    fn toggles_hour_format_from_cli_last_flag_wins() {
        assert_eq!(run_config(&["xclock"]).display.hour_format, HourFormat::H24);
        assert_eq!(
            run_config(&["xclock", "--12h"]).display.hour_format,
            HourFormat::H12
        );
        assert_eq!(
            run_config(&["xclock", "--12h", "--24h"])
                .display
                .hour_format,
            HourFormat::H24
        );
        assert_eq!(
            run_config(&["xclock", "--hour-format", "12"])
                .display
                .hour_format,
            HourFormat::H12
        );
        assert_eq!(
            run_config(&["xclock", "--hour-format=24h"])
                .display
                .hour_format,
            HourFormat::H24
        );
        assert!(parse(&args(&["xclock", "--hour-format", "13"])).is_err());
        assert!(parse(&args(&["xclock", "--hour-format"])).is_err());
    }

    #[test]
    fn toggles_blink_from_cli_last_flag_wins() {
        assert!(run_config(&["xclock"]).display.blink_colon);
        assert!(!run_config(&["xclock", "--no-blink"]).display.blink_colon);
        assert!(
            run_config(&["xclock", "--no-blink", "--blink"])
                .display
                .blink_colon
        );
    }

    #[test]
    fn toggles_date_from_cli_last_flag_wins() {
        assert!(run_config(&["xclock"]).display.show_date);
        assert!(!run_config(&["xclock", "--no-date"]).display.show_date);
        assert!(!run_config(&["xclock", "--hide-date"]).display.show_date);
        assert!(
            run_config(&["xclock", "--no-date", "--show-date"])
                .display
                .show_date
        );
    }

    #[test]
    fn accepts_palette_flags_and_aliases() {
        assert_eq!(
            run_config(&["xclock", "--palette", "black-red"]).palette,
            Palette::BlackRed
        );
        assert_eq!(
            run_config(&["xclock", "-p", "red"]).palette,
            Palette::BlackRed
        );
        assert_eq!(
            run_config(&["xclock", "--palette=purple"]).palette,
            Palette::Purple
        );
        assert_eq!(
            run_config(&["xclock", "--palette", "matrix"]).palette,
            Palette::Matrix
        );
        assert_eq!(
            run_config(&["xclock", "-p", "amber"]).palette,
            Palette::Amber
        );
        assert_eq!(
            run_config(&["xclock", "--palette=monochrome"]).palette,
            Palette::Monochrome
        );
        assert_eq!(
            run_config(&["xclock", "-p", "rainbow"]).palette,
            Palette::Rainbow
        );
        assert_eq!(
            run_config(&["xclock", "--palette", "matrix-green"]).palette,
            Palette::Matrix
        );
        assert_eq!(
            run_config(&["xclock", "--palette=amber-crt"]).palette,
            Palette::Amber
        );
        assert_eq!(
            run_config(&["xclock", "-p", "mono"]).palette,
            Palette::Monochrome
        );
    }

    #[test]
    fn rejects_unknown_theme_and_flag() {
        assert!(parse(&args(&["xclock", "--theme", "retro"])).is_err());
        assert!(parse(&args(&["xclock", "--palette", "neon"])).is_err());
        assert!(parse(&args(&["xclock", "-p"])).is_err());
        assert!(parse(&args(&["xclock", "--bogus"])).is_err());
        assert!(parse(&args(&["xclock", "-t"])).is_err());
    }

    #[test]
    fn help_and_list_actions() {
        assert!(matches!(
            parse(&args(&["xclock", "--help"])),
            Ok(Action::Help)
        ));
        assert!(matches!(parse(&args(&["xclock", "-h"])), Ok(Action::Help)));
        assert!(matches!(
            parse(&args(&["xclock", "--version"])),
            Ok(Action::Version)
        ));
        assert!(matches!(
            parse(&args(&["xclock", "-v"])),
            Ok(Action::Version)
        ));
        assert!(matches!(
            parse(&args(&["xclock", "-V"])),
            Ok(Action::Version)
        ));
        assert!(matches!(
            parse(&args(&["xclock", "--list-themes"])),
            Ok(Action::ListThemes)
        ));
        assert!(matches!(
            parse(&args(&["xclock", "--list-palettes"])),
            Ok(Action::ListPalettes)
        ));
    }

    #[test]
    fn usage_lists_all_theme_ids() {
        let u = usage();
        for t in Theme::all() {
            assert!(u.contains(t.id()), "usage missing {}", t.id());
        }
    }

    #[test]
    fn usage_lists_all_palette_ids() {
        let u = usage();
        for p in Palette::all() {
            assert!(u.contains(p.id()), "usage missing {}", p.id());
        }
    }
}
