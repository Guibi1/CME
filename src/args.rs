use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The file to open
    pub file: String,

    /// Text area height
    #[arg(long, default_value_t = 5, value_parser = clap::value_parser!(u16).range(1..))]
    pub height: u16,

    /// Show a title bar
    #[arg(long)]
    pub title: Option<String>,

    /// Hides the keybinds bar
    #[arg(long, default_value_t = false)]
    pub no_keybinds: bool,
}
