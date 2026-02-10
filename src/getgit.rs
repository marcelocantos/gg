use std::fs::create_dir_all;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use regex::{Captures, Regex};

use crate::cli;
use crate::env;
use crate::shell;

pub fn getgit(
    path: &Path,
    cli: &cli::Cli,
    ggroot: &Path,
    squiggle: &dyn for<'r> Fn(&'r Path) -> PathBuf,
    mut out: &mut Box<dyn Write>,
) -> Result<()> {
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
        Some(m) => write_commands(ggroot, &mut out, m, squiggle),
        None => bail!("invalid path: {}", path.display()),
    }
}

fn write_commands(
    ggroot: &Path,
    out: &mut Box<dyn Write>,
    m: Captures,
    squiggle: &dyn for<'r> Fn(&'r Path) -> PathBuf,
) -> Result<()> {
    // println!("{:?}", m);
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
    let repo = m.name("repo").unwrap();
    let tail = match m.name("tail") {
        Some(cap) => cap.as_str().to_string(),
        None => "".to_string(),
    };
    eprintln!(
        "👉 \x1b[1;30m{}/\x1b[1;31m{}\x1b[0m/\x1b[1;32m{}\x1b[0m/\x1b[1;34m{}\x1b[0m{}",
        squiggle(ggroot).display(),
        host.as_str(),
        org,
        repo.as_str(),
        tail.as_str()
    );
    let hostorg = prefix.to_string() + org;
    let giturl = Path::new(hostorg.as_str())
        .join(repo.as_str())
        .with_extension("git");

    let orgroot = hostroot.join(org);
    if create_dir_all(orgroot.as_path()).is_err() {
        bail!("failed to create {}", orgroot.display());
    }
    if env::var("TERM_PROGRAM") != "vscode" && !env::ggnoautocd() {
        let host = host.as_str();
        let repo = repo.as_str();
        let tail = tail.as_str();
        let reporoot = orgroot.join(repo);
        let git_cmd = if reporoot.is_dir() {
            let reporoot = shell::escape(&reporoot.display().to_string());
            format!("git -C '{reporoot}' fetch --all --prune --jobs=10 --recurse-submodules=yes")
        } else {
            let orgroot = shell::escape(&orgroot.display().to_string());
            let giturl = shell::escape(&giturl.display().to_string());
            format!("git -C '{orgroot}' clone --recurse-submodules '{giturl}'")
        };
        let reporoot = shell::escape(&reporoot.display().to_string());
        let tail = shell::escape(tail);
        let cd_cmd = format!(" && cd '{reporoot}{tail}'");
        let viewer = env::ggdirviewer();
        let viewer_cmd = match viewer {
            Some(viewer) => {
                let viewer = shell::escape(&viewer);
                let target = shell::escape(&format!(
                    "{}/{}/{}/{}{}", ggroot.display(), host, org, repo, tail
                ));
                format!(" && '{viewer}' '{target}'\n")
            }
            None => "".to_string(),
        };
        write!(out, "{git_cmd}{cd_cmd}{viewer_cmd}")?;
    }
    Ok(())
}
