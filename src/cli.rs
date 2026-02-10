use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(
    version,
    about = "gg makes it easy to find, fetch and work with your git repos.",
    long_about = None,
)]
pub struct Cli {
    /// Repo URL or path (with --get), or alias command name (with -i)
    #[arg(value_name = "TARGET")]
    pub target: Option<String>,

    /// Path prefix for alias command (with -i, e.g., github.com/org)
    #[arg(value_name = "ALIAS_PREFIX")]
    pub alias_prefix: Option<String>,

    /// Print actions to perform but do nothing
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Emit shell integration code; optionally define an alias command
    /// with a prefix (e.g., -i zsh ghg github.com)
    #[arg(short, long, value_enum, value_name = "SHELL")]
    pub install: Option<Shell>,

    /// (Internal) get a repo
    #[arg(long, hide = true)]
    pub get: bool,

    /// (Internal) repo spec prefix for alias invocations
    #[arg(long, hide = true)]
    pub prefix: Option<PathBuf>,
}

#[derive(clap::ValueEnum, Clone)]
pub enum Shell {
    Zsh,
    Bash,
    Fish,
}
