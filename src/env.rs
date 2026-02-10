use std::env;
use std::path::Path;
use std::path::PathBuf;

use anyhow::{bail, Result};
use home::home_dir;

pub fn home() -> Result<PathBuf> {
    match home_dir() {
        Some(path) => Ok(path),
        None => bail!("home directory not found"),
    }
}

pub fn exepath() -> Result<PathBuf> {
    env::current_exe().map_err(|e| anyhow::anyhow!("gg executable not found: {e}"))
}

pub fn ggnoautocd() -> bool {
    var("GGNOAUTOCD") != ""
}

pub fn ggdirviewer() -> Option<String> {
    match std::env::var("GGDIRVIEWER") {
        Ok(var) => match var.as_str() {
            "-" | "" => None,
            var => Some(var.to_string()),
        },
        Err(_) => {
            let vscodecli =
                Path::new("/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code");
            if vscodecli.exists() {
                Some(vscodecli.display().to_string())
            } else {
                None
            }
        }
    }
}

pub fn squiggler<'a>(home: &'a Path) -> Box<dyn Fn(&Path) -> PathBuf + 'a> {
    Box::new(move |path: &Path| match path.strip_prefix(home) {
        Ok(tail) => Path::new("~").join(tail),
        Err(_) => path.to_path_buf(),
    })
}

pub fn var(key: &str) -> String {
    std::env::var(key).unwrap_or_default()
}
