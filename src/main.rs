// gg will clone or fetch a repo into a standardised location (e.g.,
// ~/work/github.com/org/repo). It will also cd into it and open it in an IDE.

mod bash;
mod cli;
mod env;
mod fish;
mod getgit;
mod help;
mod shell;
mod zsh;

use std::path::Path;

use clap::Parser;
use help::help;

use bash::bash;
use fish::fish;
use getgit::getgit;
use zsh::zsh;

use anyhow::Result;

fn main() {
    if let Err(e) = run() {
        eprintln!("gg: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = cli::Cli::parse();

    let home = env::home()?;
    let exepath = env::exepath()?;
    let squiggle = env::squiggler(home.as_path());

    let ggroot = match env::var("GGROOT").as_str() {
        "" => home.join("work"),
        var => Path::new(var).to_path_buf(),
    };

    if cli.get {
        return match cli.target {
            Some(ref path) => getgit(
                Path::new(path.as_str()),
                cli.prefix.as_deref(),
                cli.dry_run,
                ggroot.as_path(),
            ),
            None => {
                eprintln!("Usage: gg <path>");
                Ok(())
            }
        };
    };

    match cli.install {
        None => {
            eprint!("{}", help(squiggle(exepath.as_path()).as_path()));
            Ok(())
        }
        Some(shell) => {
            let command = cli.target.as_deref();
            let prefix = cli.alias_prefix.as_deref();
            match shell {
                cli::Shell::Zsh => zsh(command, prefix, &exepath, &ggroot),
                cli::Shell::Bash => bash(command, prefix, &exepath, &ggroot),
                cli::Shell::Fish => fish(command, prefix, &exepath, &ggroot),
            }
        }
    }
}
