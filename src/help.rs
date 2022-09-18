use std::path::Path;

pub fn help(exepath: &Path) -> String {
    return format!("
\x1b[1mgg needs a bit of setup help from you.\x1b[0m

  • Add \x1b[32meval \"$({0} -i zsh)\"\x1b[0m to your \x1b[1m~/.zshrc\x1b[0m.
    - If you get \"zsh: command not found: compdef\" after starting a new shell,
      add \x1b[32mautoload -U compinit; compinit\x1b[0m above \x1b[32meval ...\x1b[0m and try again.
    - If you get the error \x1b[1;31mdefining function based on alias `gg'\x1b[0m, you need to
      \x1b[32munalias gg\x1b[0m before the \x1b[32meval ...\x1b[0m line.  If you're a die-hard git gui fan,
      take some time to reflect on your life choices.
    - You can kick the tyres by trying the above commands in your command line
      before altering \x1b[1m~/.zshrc\x1b[0m.
    - Caveat: bash is not supported yet.
  • Optional:
    - \x1b[32mexport GGROOT=\x1b[1mroot\x1b[0m sets the common root directory for all repos (default:
      \x1b[32m~/working\x1b[0m).
      \x1b[1;30m▸ ~ is not recommended as it makes tab-completion much harder.\x1b[0m
    - \x1b[32mexport GGDIRVIEWER=\x1b[1mviewer\x1b[0m provides a command to open the target directory
      (default: \x1b[32mcode\x1b[0m if VSCode is installed, else nothing).  If you don't want
      to open a viewer, \x1b[32mexport GGDIRVIEWER=-\x1b[0m.
    - \x1b[32mexport GGNOAUTOCD=1\x1b[0m to not auto-cd into the target directory.
      \x1b[1;30m▸ auto-cd is automatically suppressed inside VSCode.\x1b[0m
    - \x1b[32mexport GGHTTP=1\x1b[0m to default to https:// git URLs (default: ssh URLs).
  • Run \x1b[32mgg \x1b[1mrepo-url\x1b[0m in a new shell.  Try tab-completion on cloned repos.
  • You can add more helpers to \x1b[1m~/.zshrc\x1b[0m as desired, for example:
\x1b[32m
      eval \"$({0} -i zsh ghg github.com)\"
      eval \"$({0} -i zsh gmg github.com/marcelocantos)\"
\x1b[0m
    The above lines will add commands ghg and gmg, which will prefix their
    parameter with the given argument.  For example, \x1b[32mgmg gg\x1b[0m will fetch the
    \x1b[1manz.com/marcelocantos/gg\x1b[0m repo.  Auto-completion will also work for any
    locally cloned repos.  For example, \x1b[32mgg \x1b[1m[tab]\x1b[0m will tab-complete github.com
    and any other git hosting services from which you have cloned any repos
    locally.

Happy \x1b[1;33mgg\x1b[0metting!

", exepath.display());
}
