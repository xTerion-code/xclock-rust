use std::fmt::Write as _;
use std::str::FromStr;

use crate::display::{DisplayOptions, HourFormat};
use crate::theme::{Palette, Theme, palette_ids, theme_ids};
use crate::ui::{HINT_FULL, HINT_SHORT};

#[derive(Debug)]
pub struct Config {
    pub theme: Theme,
    pub palette: Palette,
    pub display: DisplayOptions,
}

#[derive(Debug)]
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
    let mut show_hint = true;

    let mut it = args.iter().skip(1);
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
        } else if arg == "--show-hint" {
            show_hint = true;
        } else if arg == "--no-hint" || arg == "--hide-hint" || arg == "--minimal" {
            show_hint = false;
        } else if arg == "--12h" {
            hour_format = HourFormat::H12;
        } else if arg == "--24h" {
            hour_format = HourFormat::H24;
        } else if arg == "-t" || arg == "--theme" {
            let value = next_value(&mut it, arg, theme_ids)?;
            theme = Theme::from_str(&value).map_err(|e| e.to_string())?;
        } else if let Some(value) = arg.strip_prefix("--theme=") {
            theme = Theme::from_str(value).map_err(|e| e.to_string())?;
        } else if arg == "-p" || arg == "--palette" {
            let value = next_value(&mut it, arg, palette_ids)?;
            palette = Palette::from_str(&value).map_err(|e| e.to_string())?;
        } else if let Some(value) = arg.strip_prefix("--palette=") {
            palette = Palette::from_str(value).map_err(|e| e.to_string())?;
        } else if arg == "--hour-format" {
            let value = next_value(&mut it, arg, || "12, 12h, 24, 24h".to_string())?;
            hour_format = HourFormat::from_str(&value).map_err(|e| e.to_string())?;
        } else if let Some(value) = arg.strip_prefix("--hour-format=") {
            hour_format = HourFormat::from_str(value).map_err(|e| e.to_string())?;
        } else {
            return Err(format!("unexpected argument `{arg}`"));
        }
    }

    let mut display = DisplayOptions::with_hour_format(show_seconds, hour_format);
    display.blink_colon = blink_colon;
    display.show_date = show_date;
    display.show_hint = show_hint;
    Ok(Action::Run(Config {
        theme,
        palette,
        display,
    }))
}

/// Reads the value of a flag that was given as a separate argument.
fn next_value<'a>(
    it: &mut impl Iterator<Item = &'a String>,
    flag: &str,
    expected: impl Fn() -> String,
) -> Result<String, String> {
    it.next()
        .cloned()
        .ok_or_else(|| format!("missing value: `{flag}` expects one of: {}", expected()))
}

pub fn usage() -> String {
    format!(
        "Usage: xclock [--theme <name> | -t <name>] [--palette <name> | -p <name>] [--show-seconds | --no-seconds] [--12h | --24h] [--blink | --no-blink] [--show-date | --no-date] [--show-hint | --no-hint] [--list-themes] [--list-palettes] [-v | --version] [--help]\n\
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
           --show-hint           show the live-keys hint row (default)\n  \
           --no-hint             hide the hint row\n  \
           --list-themes         list available themes\n  \
           --list-palettes       list available color palettes\n  \
           -v, -V, --version     show version information\n  \
           -h, --help            show this help\n  \
         \n\
         Live keys: {}\n\
         The hint row shortens itself on narrow terminals and disappears entirely when it cannot fit. Use --no-hint to always remove it.",
        theme_ids(),
        palette_ids(),
        HINT_FULL,
    )
}

pub fn themes_list() -> String {
    let mut out = String::from("Available themes:\n");
    for t in Theme::all() {
        // Writing into a `String` cannot fail.
        let _ = writeln!(out, "  {:<8} {}", t.id(), t.description());
    }
    out.push('\n');
    out.push_str("Live keys: ");
    out.push_str(HINT_FULL);
    out.push('\n');
    out.push_str("On narrow terminals the frame shows a shorter reminder: ");
    out.push_str(HINT_SHORT);
    out.push('\n');
    out
}

pub fn palettes_list() -> String {
    let mut out = String::from("Available palettes:\n");
    for p in Palette::all() {
        // Writing into a `String` cannot fail.
        let _ = writeln!(out, "  {:<12} {}", p.id(), p.description());
        if !p.aliases().is_empty() {
            let _ = writeln!(out, "  {:<12} aliases: {}", "", p.aliases().join(", "));
        }
    }
    out.push_str("\nLive key: press p to cycle palettes.");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(ToString::to_string).collect()
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
        assert!(c.display.show_hint);
        assert!(c.display.show_date);
        assert!(c.display.blink_colon);
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
    fn toggles_hint_from_cli_last_flag_wins() {
        assert!(run_config(&["xclock"]).display.show_hint);
        assert!(!run_config(&["xclock", "--no-hint"]).display.show_hint);
        assert!(!run_config(&["xclock", "--hide-hint"]).display.show_hint);
        assert!(!run_config(&["xclock", "--minimal"]).display.show_hint);
        assert!(
            run_config(&["xclock", "--no-hint", "--show-hint"])
                .display
                .show_hint
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
        let e = parse(&args(&["xclock", "--theme", "retro"])).unwrap_err();
        assert!(e.starts_with("unknown theme `retro`"), "{e}");
        let e = parse(&args(&["xclock", "--palette", "neon"])).unwrap_err();
        assert!(e.starts_with("unknown palette `neon`"), "{e}");
        let e = parse(&args(&["xclock", "-p"])).unwrap_err();
        assert!(e.starts_with("missing value: `-p`"), "{e}");
        assert!(parse(&args(&["xclock", "--bogus"])).is_err());
        assert!(parse(&args(&["xclock", "-t"])).is_err());
        assert!(parse(&args(&["xclock", "--theme="])).is_err());
    }

    #[test]
    fn help_and_list_actions() {
        for argv in [
            vec!["xclock", "--help"],
            vec!["xclock", "-h"],
            vec!["xclock", "--version"],
            vec!["xclock", "-v"],
            vec!["xclock", "-V"],
            vec!["xclock", "--list-themes"],
            vec!["xclock", "--list-palettes"],
        ] {
            let action = parse(&args(&argv)).unwrap();
            let ok = matches!(
                action,
                Action::Help | Action::Version | Action::ListThemes | Action::ListPalettes
            );
            assert!(ok, "{argv:?} should not start the TUI");
        }
    }

    #[test]
    fn help_is_returned_even_after_other_flags() {
        assert!(matches!(
            parse(&args(&["xclock", "--no-date", "--help"])),
            Ok(Action::Help)
        ));
    }

    #[test]
    fn usage_lists_every_catalog_id() {
        let u = usage();
        for t in Theme::all() {
            assert!(u.contains(t.id()), "usage missing {}", t.id());
        }
        for p in Palette::all() {
            assert!(u.contains(p.id()), "usage missing {}", p.id());
        }
        // The live keys are documented from the very string that is rendered.
        assert!(u.contains(HINT_FULL), "usage missing the live keys");
        assert!(u.contains("--no-hint"), "usage missing --no-hint");
    }

    #[test]
    fn listings_describe_every_entry() {
        let themes = themes_list();
        for t in Theme::all() {
            assert!(themes.contains(t.id()), "listing missing {}", t.id());
            assert!(
                themes.contains(t.description()),
                "listing missing description of {}",
                t.id()
            );
        }
        assert!(themes.contains(HINT_FULL));

        let palettes = palettes_list();
        for p in Palette::all() {
            assert!(palettes.contains(p.id()), "listing missing {}", p.id());
            assert!(
                palettes.contains(p.description()),
                "listing missing description of {}",
                p.id()
            );
            for alias in p.aliases() {
                assert!(palettes.contains(alias), "listing missing alias {alias}");
            }
        }
    }
}
