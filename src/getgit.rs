use std::fs::create_dir_all;
use std::io::{self, Write};
use std::path::Path;

use anyhow::{bail, Result};
use regex::Regex;

use crate::cli;
use crate::env;

pub fn getgit(path: &Path, cli: &cli::Cli, ggroot: &Path) -> Result<()> {
    let home = env::home()?;
    let squiggle = env::squiggler(home.as_path());
    let url = match &cli.prefix {
        Some(prefix) => prefix.join(path).to_path_buf(),
        None => path.to_path_buf(),
    }
    .display()
    .to_string();

    let m = Regex::new(
        r"(?x)
        ^
        (?P<prefix>
            git@(?P<git_host>[^:]+):
          | https?://(?P<http_host>[^/]+)/
          | (?P<host>[^/:]+) [/:] )
        (?P<org>[^/]+)
        /
        (?P<repo>[^/\.]+)
        (?: (?P<tail>/.*) | \.git )?
        $",
    )
    .unwrap()
    .captures(url.as_str());

    match m {
        Some(m) => {
            let host = match m.name("git_host") {
                Some(host) => host,
                None => match m.name("http_host") {
                    Some(host) => host,
                    None => match m.name("host") {
                        Some(host) => host,
                        None => bail!("no host in URL"),
                    },
                },
            };
            let hostroot = ggroot.join(host.as_str());
            if !hostroot.is_dir() {
                eprintln!(
                    "host dir \x1b[1m{}\x1b[0m not found; create manually if correct",
                    squiggle(hostroot.as_path()).display()
                );
                return Ok(());
            }

            let prefix = m.name("prefix").unwrap().as_str();
            let prefix = if !Regex::new(r"https?://|git@").unwrap().is_match(prefix) {
                format!("https://{prefix}")
            } else {
                prefix.to_string()
            };

            let org = m.name("org").unwrap().as_str();
            let repo = m.name("repo").unwrap().as_str();
            let tail = match m.name("tail") {
                Some(cap) => cap.as_str().to_string(),
                None => "".to_string(),
            };

            eprintln!(
                "👉 \x1b[1;30m{}/\x1b[1;31m{}\x1b[0m/\x1b[1;32m{}\x1b[0m/\x1b[1;34m{}\x1b[0m{}",
                squiggle(ggroot).display(),
                host.as_str(),
                org,
                repo,
                tail.as_str()
            );

            let hostorg = prefix.to_string() + org;
            let giturl = Path::new(hostorg.as_str())
                .join(repo)
                .with_extension("git");

            let orgroot = hostroot.join(org);
            if create_dir_all(orgroot.as_path()).is_err() {
                bail!("failed to create {}", orgroot.display());
            }

            if !cli.dry_run {
                let reporoot = orgroot.join(repo);
                let mut out = io::stdout();
                if reporoot.is_dir() {
                    writeln!(out, "action=fetch")?;
                    writeln!(out, "git_dir={}", reporoot.display())?;
                } else {
                    writeln!(out, "action=clone")?;
                    writeln!(out, "git_dir={}", orgroot.display())?;
                    writeln!(out, "git_url={}", giturl.display())?;
                }
                writeln!(out, "cd_dir={}{}", reporoot.display(), tail)?;
            }

            Ok(())
        }
        None => bail!("invalid path: {}", path.display()),
    }
}
