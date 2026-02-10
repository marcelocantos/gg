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

use std::io::{self, stdout};
use std::path::Path;

use clap::Parser;
use help::help;

use bash::bash;
use fish::fish;
use getgit::getgit;
use zsh::zsh;

use anyhow::Result;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    let home = env::home()?;
    let exepath = env::exepath()?;
    let squiggle = env::squiggler(home.as_path());

    let ggroot = match env::var("GGROOT").as_str() {
        "" => home.join("work"),
        var => Path::new(var).to_path_buf(),
    };

    let mut out: Box<dyn io::Write> = Box::new(stdout());

    if cli.get {
        return match cli.arg1 {
            Some(ref path) => getgit(
                Path::new(path.as_str()),
                &cli,
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
            let arg1 = cli.arg1.as_deref();
            let arg2 = cli.arg2.as_deref();
            match shell {
                cli::Shell::Zsh => zsh(arg1, arg2, &exepath, &ggroot, &mut out),
                cli::Shell::Bash => bash(arg1, arg2, &exepath, &ggroot, &mut out),
                cli::Shell::Fish => fish(arg1, arg2, &exepath, &ggroot, &mut out),
            }
        }
    }?;

    Ok(())
}
