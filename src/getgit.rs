use std::fs::create_dir_all;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::sync::LazyLock;

use anyhow::{bail, Result};
use regex::Regex;

use crate::env;

static URL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
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
});

pub fn getgit(path: &Path, prefix: Option<&Path>, dry_run: bool, ggroot: &Path) -> Result<()> {
    let home = env::home()?;
    let squiggle = env::squiggler(home.as_path());
    let url = match prefix {
        Some(prefix) => prefix.join(path).to_path_buf(),
        None => path.to_path_buf(),
    }
    .display()
    .to_string();

    let m = URL_RE.captures(url.as_str());

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

            let org = m.name("org").unwrap().as_str();
            let repo = m.name("repo").unwrap().as_str();
            let tail = match m.name("tail") {
                Some(cap) => cap.as_str().to_string(),
                None => "".to_string(),
            };

            // Construct git URL based on which regex group matched.
            let giturl = if m.name("git_host").is_some() {
                // Explicit SSH: git@host:org/repo.git — preserve as-is
                format!("git@{}:{}/{}.git", host.as_str(), org, repo)
            } else if m.name("http_host").is_some() {
                // Explicit HTTPS: https://host/org/repo.git — preserve
                let prefix = m.name("prefix").unwrap().as_str();
                format!("{}{}/{}.git", prefix, org, repo)
            } else {
                // Shorthand: host/org/repo — use SSH by default, HTTPS if GGHTTP is set
                let host = host.as_str();
                if env::var("GGHTTP").is_empty() {
                    format!("git@{host}:{org}/{repo}.git")
                } else {
                    format!("https://{host}/{org}/{repo}.git")
                }
            };

            let hostroot = ggroot.join(host.as_str());
            if !hostroot.is_dir() {
                // Host dir doesn't exist — verify the remote repo before creating it.
                eprintln!(
                    "host dir \x1b[1m{}\x1b[0m is new, verifying remote...",
                    squiggle(hostroot.as_path()).display()
                );
                let status = Command::new("git")
                    .args(["ls-remote", &giturl])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status();
                match status {
                    Ok(s) if s.success() => {
                        // Repo exists — create the host dir and proceed.
                    }
                    _ => {
                        bail!(
                            "remote not found at {}\nIf the host is correct, create {} manually",
                            giturl,
                            squiggle(hostroot.as_path()).display()
                        );
                    }
                }
            }

            eprintln!(
                "👉 \x1b[1;30m{}/\x1b[1;31m{}\x1b[0m/\x1b[1;32m{}\x1b[0m/\x1b[1;34m{}\x1b[0m{}",
                squiggle(ggroot).display(),
                host.as_str(),
                org,
                repo,
                tail.as_str()
            );

            let orgroot = hostroot.join(org);
            create_dir_all(orgroot.as_path())?;

            if !dry_run {
                let reporoot = orgroot.join(repo);
                let mut out = io::stdout();
                if reporoot.is_dir() {
                    writeln!(out, "action=fetch")?;
                    writeln!(out, "git_dir={}", reporoot.display())?;
                } else {
                    writeln!(out, "action=clone")?;
                    writeln!(out, "git_dir={}", orgroot.display())?;
                    writeln!(out, "git_url={giturl}")?;
                }
                writeln!(out, "cd_dir={}{}", reporoot.display(), tail)?;
            }

            Ok(())
        }
        None => bail!("invalid path: {}", path.display()),
    }
}
