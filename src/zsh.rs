use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};

use crate::shell;

pub fn zsh(
    arg1: Option<&str>,
    arg2: Option<&str>,
    exepath: &Path,
    ggroot: &Path,
    out: &mut Box<dyn Write>,
) -> Result<()> {
    match arg1 {
        Some(command) => {
            let prefix = arg2.context("missing prefix argument")?;
            let prefix_path = Path::new(prefix);
            let prefix_root = ggroot.join(prefix_path);

            let command = shell::escape(command);
            let prefix_path = shell::escape(&prefix_path.display().to_string());
            let prefix_root = shell::escape(&prefix_root.display().to_string());

            write!(
                out,
                "\
                    {command}() {{ gg --prefix '{prefix_path}' \"$@\"; }};\n\
                    _{command}() {{ _path_files -/ -W '{prefix_root}'; }};\n\
                    compdef _{command} {command};\n\
                ",
            )?;
        }
        None => {
            let exepath = shell::escape(&exepath.display().to_string());
            let ggroot = shell::escape(&ggroot.display().to_string());

            write!(
                out,
                "\
                    gg() {{ eval $('{exepath}' --get \"$@\"); }};\n\
                    _gg() {{ _path_files -/ -W '{ggroot}'; }};\n\
                    compdef _gg gg;\n\
                ",
            )?;
        }
    }
    Ok(())
}
