use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;

use anyhow::{bail, Result};

use crate::env;

pub fn setup(exepath: &Path) -> Result<()> {
    let home = env::home()?;
    let zshrc = home.join(".zshrc");

    // Check if already installed.
    if zshrc.is_file() {
        let content = fs::read_to_string(&zshrc)?;
        if content.contains("-i zsh)\"") {
            eprintln!("gg is already installed in {}", zshrc.display());
            return Ok(());
        }
    }

    let exepath_str = exepath.display().to_string();
    let mut lines: Vec<String> = Vec::new();
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    // GGROOT
    let ggroot = prompt(&mut reader, "Repo root directory", "~/work")?;
    if ggroot != "~/work" {
        lines.push(format!("export GGROOT={ggroot}"));
    }

    // GGHTTP
    let proto = prompt(&mut reader, "Default git protocol [ssh/https]", "ssh")?;
    if proto == "https" {
        lines.push("export GGHTTP=1".to_string());
    }

    // GGDIRVIEWER
    if vscode_installed() {
        let answer = prompt(
            &mut reader,
            "Open repos in VSCode after clone? [Y/n/-]",
            "y",
        )?;
        match answer.as_str() {
            "n" | "N" => {
                let viewer = prompt(
                    &mut reader,
                    "Command to open repos after clone (or - for none)",
                    "-",
                )?;
                if viewer == "-" {
                    lines.push("export GGDIRVIEWER=-".to_string());
                } else if !viewer.is_empty() {
                    lines.push(format!("export GGDIRVIEWER={viewer}"));
                }
            }
            "-" => {
                lines.push("export GGDIRVIEWER=-".to_string());
            }
            _ => {} // default VSCode — no export needed
        }
    } else {
        let viewer = prompt(
            &mut reader,
            "Command to open repos after clone (or - for none)",
            "-",
        )?;
        if viewer == "-" {
            lines.push("export GGDIRVIEWER=-".to_string());
        } else if !viewer.is_empty() {
            lines.push(format!("export GGDIRVIEWER={viewer}"));
        }
    }

    // Core eval line
    lines.push(format!("eval \"$({exepath_str} -i zsh)\""));

    // Aliases
    eprintln!();
    eprintln!("You can add shorthand aliases, e.g., \x1b[32mghg github.com\x1b[0m creates a");
    eprintln!(
        "command \x1b[32mghg\x1b[0m that prefixes its argument with \x1b[32mgithub.com\x1b[0m."
    );
    loop {
        let alias = prompt(&mut reader, "Add alias (CMD PREFIX, or Enter to skip)", "")?;
        if alias.is_empty() {
            break;
        }
        let parts: Vec<&str> = alias.splitn(2, ' ').collect();
        if parts.len() != 2 || parts[1].is_empty() {
            eprintln!("  Expected: CMD PREFIX (e.g., ghg github.com)");
            continue;
        }
        lines.push(format!(
            "eval \"$({exepath_str} -i zsh {} {})\"",
            parts[0], parts[1]
        ));
    }

    // Build the block
    let block = format!("# --- gg ---\n{}\n# --- end gg ---\n", lines.join("\n"));

    // Show and confirm
    eprintln!();
    eprintln!("Generated configuration:");
    eprintln!();
    for line in block.lines() {
        eprintln!("  {line}");
    }
    eprintln!();

    let action = prompt(
        &mut reader,
        &format!("Append to {}? [Y/n/print]", squiggle_path(&home, &zshrc)),
        "y",
    )?;

    match action.to_lowercase().as_str() {
        "y" | "yes" | "" => {
            let mut file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&zshrc)?;
            writeln!(file)?; // blank line before block
            write!(file, "{block}")?;
            eprintln!("\x1b[32mInstalled!\x1b[0m Open a new shell to activate gg.");
        }
        "print" | "p" => {
            // Print to stdout so user can copy/paste
            print!("{block}");
        }
        _ => {
            eprintln!("No changes made.");
        }
    }

    Ok(())
}

fn prompt(reader: &mut impl BufRead, question: &str, default: &str) -> Result<String> {
    let mut out = io::stderr();
    if default.is_empty() {
        write!(out, "{question}: ")?;
    } else {
        write!(out, "{question} [{default}]: ")?;
    }
    out.flush()?;

    let mut input = String::new();
    if reader.read_line(&mut input)? == 0 {
        bail!("unexpected end of input");
    }
    let input = input.trim().to_string();
    if input.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(input)
    }
}

fn vscode_installed() -> bool {
    Path::new("/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code").is_file()
}

fn squiggle_path(home: &Path, path: &Path) -> String {
    match path.strip_prefix(home) {
        Ok(tail) => format!("~/{}", tail.display()),
        Err(_) => path.display().to_string(),
    }
}
