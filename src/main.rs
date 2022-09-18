// gg will clone or fetch a repo into a standardised location (e.g.,
// ~/working/github.com/org/repo). It will also cd into it and open it in an
// IDE.

mod bash;
mod cli;
mod env;
mod fish;
mod getgit;
mod help;
mod zsh;

use std;
use std::io::{self, stderr, stdout};
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

    let home = env::home();
    let exepath = env::exepath();
    let squiggle = env::squiggler(home.as_path());

    let ggroot = match env::var("GGROOT").as_str() {
        "" => home.join("work"),
        var => Path::new(var).to_path_buf(),
    };

    let mut out: Box<dyn io::Write> = if cli.dry_run {
        Box::new(stderr())
    } else {
        Box::new(stdout())
    };

    if cli.get {
        return match cli.arg1 {
            Some(ref path) => getgit(
                Path::new(path.as_str()),
                &cli,
                ggroot.as_path(),
                squiggle.as_ref(),
                &mut out,
            ),
            None => Ok(eprintln!("Usage: gg <path>")),
        };
    };

    match cli.install {
        None => Ok(eprint!("{}", help(squiggle(exepath.as_path()).as_path()))),
        Some(shell) => {
            let arg1 = cli.arg1.as_ref().map(|s| s.as_str());
            let arg2 = cli.arg2.as_ref().map(|s| s.as_str());
            match shell {
                cli::Shell::Zsh => zsh(arg1, arg2, &exepath, &ggroot, &mut out),
                cli::Shell::Bash => bash(arg1, arg2, &exepath, &ggroot, &mut out),
                cli::Shell::Fish => fish(arg1, arg2, &exepath, &ggroot, &mut out),
            }
        }
    }?;

    Ok(())
}
