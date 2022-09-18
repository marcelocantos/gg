use std::path::PathBuf;

use clap;
use clap::Parser;

#[derive(Parser)]
#[clap(
    version,
    about = "\
        gg makes it easy to find, fetch and work with your git repos. Run \
        gg without arguments for installation instructions.",
    long_about = None,
)]
pub struct Cli {
    #[clap()]
    pub arg1: Option<String>,

    #[clap()]
    pub arg2: Option<String>,

    /// Print actions to perform but do nothing
    #[clap(short = 'n', long)]
    pub dry_run: bool,

    /// Turn debugging information on
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    /// Print $(eval ...) setup for shell function and auto-complete (only zsh
    /// is implemented)
    #[clap(short, long, value_enum, value_name = "SHELL")]
    pub install: Option<Shell>,

    /// (Internal use only) repo spec prefix
    #[clap(long)]
    pub prefix: Option<PathBuf>,
}

#[derive(clap::ValueEnum, Clone)]
pub enum Shell {
    Zsh,
    Bash,
    Fish,
}
