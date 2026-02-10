use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};

use crate::shell;

pub fn zsh(
    command: Option<&str>,
    prefix: Option<&str>,
    exepath: &Path,
    ggroot: &Path,
    out: &mut Box<dyn Write>,
) -> Result<()> {
    match command {
        Some(command) => {
            let prefix = prefix.context("missing prefix argument")?;
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
gg() {{\n\
    local output\n\
    output=$('{exepath}' --get \"$@\") || return $?\n\
    [ -z \"$output\" ] && return\n\
    local action git_dir git_url cd_dir\n\
    while IFS= read -r _gg_line; do\n\
        case \"${{_gg_line%%=*}}\" in\n\
            action) action=\"${{_gg_line#*=}}\" ;;\n\
            git_dir) git_dir=\"${{_gg_line#*=}}\" ;;\n\
            git_url) git_url=\"${{_gg_line#*=}}\" ;;\n\
            cd_dir) cd_dir=\"${{_gg_line#*=}}\" ;;\n\
        esac\n\
    done <<< \"$output\"\n\
    if [[ \"${{TERM_PROGRAM:-}}\" == \"vscode\" || -n \"${{GGNOAUTOCD:-}}\" ]]; then return; fi\n\
    case \"$action\" in\n\
        clone) git -C \"$git_dir\" clone --recurse-submodules \"$git_url\" || return ;;\n\
        fetch) git -C \"$git_dir\" fetch --all --prune --jobs=10 --recurse-submodules=yes || return ;;\n\
    esac\n\
    cd \"$cd_dir\" || return\n\
    local viewer=\"${{GGDIRVIEWER:-}}\"\n\
    if [ -z \"$viewer\" ]; then\n\
        local vscode='/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code'\n\
        [ -x \"$vscode\" ] && viewer=\"$vscode\"\n\
    elif [ \"$viewer\" = \"-\" ]; then\n\
        viewer=''\n\
    fi\n\
    if [ -n \"$viewer\" ]; then \"$viewer\" \"$cd_dir\"; fi\n\
}};\n\
_gg() {{ _path_files -/ -W '{ggroot}'; }};\n\
compdef _gg gg;\n\
",
            )?;
        }
    }
    Ok(())
}
