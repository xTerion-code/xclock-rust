use crate::theme::Theme;

/// What the program should do after parsing CLI arguments.
pub enum Action {
    Run(Theme),
    Help,
    ListThemes,
}

/// `xclock [--theme <name> | -t <name>] [--list-themes] [--help]`.
///
/// Theme defaults to [`Theme::Modern`]. Errors are human-readable
/// strings; `main` prints them with usage and exits with code 2.
pub fn parse(args: &[String]) -> Result<Action, String> {
    let mut theme = Theme::Modern;

    let mut it = args.iter().skip(1).peekable();
    while let Some(arg) = it.next() {
        if arg == "-h" || arg == "--help" {
            return Ok(Action::Help);
        }
        if arg == "--list-themes" {
            return Ok(Action::ListThemes);
        }
        if arg == "-t" || arg == "--theme" {
            let value = it.next().ok_or_else(|| {
                format!("missing value: `{arg}` expects one of: {}", theme_ids())
            })?;
            theme = Theme::from_str(value).ok_or_else(|| {
                format!("unknown theme `{value}` (expected one of: {})", theme_ids())
            })?;
        } else if let Some(value) = arg.strip_prefix("--theme=") {
            theme = Theme::from_str(value).ok_or_else(|| {
                format!("unknown theme `{value}` (expected one of: {})", theme_ids())
            })?;
        } else {
            return Err(format!("unexpected argument `{arg}`"));
        }
    }

    Ok(Action::Run(theme))
}

fn theme_ids() -> String {
    Theme::all()
        .iter()
        .map(|t| t.id())
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn usage() -> String {
    format!(
        "Usage: xclock [--theme <name> | -t <name>] [--list-themes] [--help]\n\
         \n\
         Options:\n  \
           -t, --theme <name>  clock theme (default: modern): {}\n  \
           --list-themes       list available themes\n  \
           -h, --help          show this help\n\
         \n\
         Live keys: 1/2/3 switch theme, q or Esc quits.",
        theme_ids()
    )
}

pub fn themes_list() -> String {
    let mut out = String::from("Available themes:\n");
    for t in Theme::all() {
        out.push_str(&format!("  {:<8} {}\n", t.id(), t.description()));
    }
    out.push_str("\nLive keys: press 1/2/3 to switch, q or Esc to quit.");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    fn run_theme(argv: &[&str]) -> Theme {
        match parse(&args(argv)).unwrap() {
            Action::Run(t) => t,
            _ => panic!("expected Run for {argv:?}"),
        }
    }

    #[test]
    fn defaults_to_modern() {
        assert_eq!(run_theme(&["xclock"]), Theme::Modern);
    }

    #[test]
    fn accepts_long_short_and_eq_forms() {
        assert_eq!(run_theme(&["xclock", "--theme", "compact"]), Theme::Compact);
        assert_eq!(run_theme(&["xclock", "-t", "classic"]), Theme::Classic);
        assert_eq!(run_theme(&["xclock", "--theme=compact"]), Theme::Compact);
    }

    #[test]
    fn rejects_unknown_theme_and_flag() {
        assert!(parse(&args(&["xclock", "--theme", "retro"])).is_err());
        assert!(parse(&args(&["xclock", "--bogus"])).is_err());
        assert!(parse(&args(&["xclock", "-t"])).is_err());
    }

    #[test]
    fn help_and_list_actions() {
        assert!(matches!(parse(&args(&["xclock", "--help"])), Ok(Action::Help)));
        assert!(matches!(parse(&args(&["xclock", "-h"])), Ok(Action::Help)));
        assert!(matches!(
            parse(&args(&["xclock", "--list-themes"])),
            Ok(Action::ListThemes)
        ));
    }

    #[test]
    fn usage_lists_all_theme_ids() {
        let u = usage();
        for t in Theme::all() {
            assert!(u.contains(t.id()), "usage missing {}", t.id());
        }
    }
}
