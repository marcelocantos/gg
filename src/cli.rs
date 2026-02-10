use std::path::PathBuf;

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
    /// Repo URL or path (with --get), or alias command name (with -i)
    #[clap(value_name = "TARGET")]
    pub target: Option<String>,

    /// Path prefix for alias command (with -i, e.g., github.com/org)
    #[clap(value_name = "ALIAS_PREFIX")]
    pub alias_prefix: Option<String>,

    /// Print actions to perform but do nothing
    #[clap(short = 'n', long)]
    pub dry_run: bool,

    /// Turn debugging information on
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    /// Emit shell integration code; optionally define an alias command
    /// with a prefix (e.g., -i zsh ghg github.com)
    #[clap(short, long, value_enum, value_name = "SHELL")]
    pub install: Option<Shell>,

    /// (Internal) get a repo
    #[clap(long, hide = true)]
    pub get: bool,

    /// (Internal) repo spec prefix for alias invocations
    #[clap(long, hide = true)]
    pub prefix: Option<PathBuf>,
}

#[derive(clap::ValueEnum, Clone)]
pub enum Shell {
    Zsh,
    Bash,
    Fish,
}
