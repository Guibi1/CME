use clap::Parser;
use std::str::FromStr;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The file to open
    pub file: String,

    /// Text area height
    #[arg(
        long,
        short = 's',
        default_value_t = 5,
        value_parser = clap::value_parser!(u16).range(1..)
    )]
    pub height: u16,

    /// Enables fullscreen mode (ignores height)
    ///
    /// Modes: enabled | disabled | N (fullscreen when the file exceeds N lines)
    #[arg(
        long,
        short,
        default_value = "disabled",
        default_missing_value = "enabled",
        num_args = 0..=1,
        value_name = "MODE"
    )]
    pub fullscreen: ArgsFullscreenMode,

    /// Show a title bar
    #[arg(long, short)]
    pub title: Option<String>,

    /// Hides the keybinds bar
    #[arg(long, short = 'k')]
    pub no_keybinds: bool,
}

/// Fullscreen mode
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ArgsFullscreenMode {
    Disabled,
    Enabled,
    Adaptive(u16),
}

impl FromStr for ArgsFullscreenMode {
    type Err = <u16 as FromStr>::Err;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "disabled" | "false" => Ok(Self::Disabled),
            "enabled" | "true" => Ok(Self::Enabled),
            _ => Ok(Self::Adaptive(value.parse::<u16>()?)),
        }
    }
}
