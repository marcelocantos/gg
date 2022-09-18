use std::io::Write;
use std::path::Path;

use anyhow::Result;

pub fn zsh(
    arg1: Option<&str>,
    arg2: Option<&str>,
    exepath: &Path,
    ggroot: &Path,
    out: &mut Box<dyn Write>,
) -> Result<()> {
    match arg1 {
        Some(command) => {
            let prefix = arg2.unwrap();
            let prefix_path = Path::new(prefix);
            let prefix_root = ggroot.join(prefix_path);

            let prefix_path = prefix_path.display();
            let prefix_root = prefix_root.display();
            write!(
                out,
                "\
                    {command}() {{ gg --prefix '{prefix_path}' \"$@\"); }};\n\
                    _{command}() {{ _path_files -/ -W '{prefix_root}' }};\n\
                    compdef _{command} {command};\n\
                ",
            )?;
        }
        None => {
            let exepath = exepath.display();
            let ggroot = ggroot.display();
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
