use home::home_dir;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process;

pub fn home() -> PathBuf {
    return match home_dir() {
        Some(path) => path,
        None => {
            eprintln!("home directory not found");
            process::exit(1);
        }
    };
}

pub fn exepath() -> PathBuf {
    return match env::current_exe() {
        Ok(exe_path) => exe_path,
        Err(e) => {
            eprintln!("gg executable not found: {e}");
            process::exit(1);
        }
    };
}

pub fn ggnoautocd() -> bool {
    return var("GGNOAUTOCD") != "";
}

pub fn ggdirviewer() -> Option<String> {
    return match std::env::var("GGDIRVIEWER") {
        Ok(var) => match var.as_str() {
            "-" | "" => Some("".to_string()),
            var => Some(var.to_string()),
        },
        Err(_) => {
            let vscodecli =
                Path::new("/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code");
            if vscodecli.exists() {
                Some(vscodecli.display().to_string())
            } else {
                // print!("export GGDIRVIEWER to specify your directory viewer");
                None
            }
        }
    };
}

pub fn squiggler<'a>(home: &'a Path) -> Box<dyn Fn(&Path) -> PathBuf + 'a> {
    return Box::new(move |path: &Path| match path.strip_prefix(home) {
        Ok(tail) => Path::new("~").join(tail),
        Err(_) => path.to_path_buf(),
    });
}

pub fn var(key: &str) -> String {
    return std::env::var(key).unwrap_or_default();
}
