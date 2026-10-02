mod app;
mod cli;
mod font;
mod locale;
mod theme;
mod ui;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match cli::parse(&args) {
        Ok(cli::Action::Help) => println!("{}", cli::usage()),
        Ok(cli::Action::ListThemes) => println!("{}", cli::themes_list()),
        Ok(cli::Action::Run(theme)) => {
            if let Err(e) = app::run(theme) {
                eprintln!("xclock: {e}");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("xclock: {e}\n{}", cli::usage());
            std::process::exit(2);
        }
    }
}
