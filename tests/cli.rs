use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use tempfile::TempDir;

fn binary_path() -> std::path::PathBuf {
    // cargo test builds to target/debug/
    let mut path = std::env::current_exe().unwrap();
    // exe is in target/debug/deps/cli-<hash>, go up to target/debug/
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push("gg");
    path
}

struct GgResult {
    stdout: String,
    stderr: String,
    success: bool,
}

impl GgResult {
    /// Parse stdout key=value lines into a map.
    fn parsed(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for line in self.stdout.lines() {
            if let Some((k, v)) = line.split_once('=') {
                map.insert(k.to_string(), v.to_string());
            }
        }
        map
    }
}

/// Run `gg --get <args>` with GGROOT set to the given dir.
fn run_gg(ggroot: &std::path::Path, args: &[&str]) -> GgResult {
    run_gg_env(ggroot, args, &[])
}

/// Run `gg --get <args>` with GGROOT and additional env vars.
fn run_gg_env(ggroot: &std::path::Path, args: &[&str], env: &[(&str, &str)]) -> GgResult {
    let mut cmd = Command::new(binary_path());
    cmd.arg("--get").args(args).env("GGROOT", ggroot);
    for (k, v) in env {
        cmd.env(k, v);
    }
    let output = cmd.output().expect("failed to run gg");

    GgResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        success: output.status.success(),
    }
}

/// Create host/org directories under ggroot, return the ggroot TempDir.
fn setup_ggroot(host: &str, org: &str) -> TempDir {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(host).join(org)).unwrap();
    tmp
}

// --- URL format tests ---

#[test]
fn shorthand_url_ssh() {
    let tmp = setup_ggroot("github.com", "org");
    let r = run_gg(tmp.path(), &["github.com/org/repo"]);
    assert!(r.success);
    let p = r.parsed();
    assert_eq!(p["action"], "clone");
    assert_eq!(p["git_url"], "git@github.com:org/repo.git");
    assert!(p["cd_dir"].ends_with("github.com/org/repo"));
}

#[test]
fn shorthand_url_https_with_gghttp() {
    let tmp = setup_ggroot("github.com", "org");
    let r = run_gg_env(tmp.path(), &["github.com/org/repo"], &[("GGHTTP", "1")]);
    assert!(r.success);
    let p = r.parsed();
    assert_eq!(p["action"], "clone");
    assert_eq!(p["git_url"], "https://github.com/org/repo.git");
    assert!(p["cd_dir"].ends_with("github.com/org/repo"));
}

#[test]
fn https_url() {
    let tmp = setup_ggroot("github.com", "org");
    let r = run_gg(tmp.path(), &["https://github.com/org/repo"]);
    assert!(r.success);
    let p = r.parsed();
    assert_eq!(p["action"], "clone");
    assert!(p["git_url"].starts_with("https://"));
    assert!(p["git_url"].ends_with("repo.git"));
}

#[test]
fn ssh_url() {
    let tmp = setup_ggroot("github.com", "org");
    let r = run_gg(tmp.path(), &["git@github.com:org/repo"]);
    assert!(r.success);
    let p = r.parsed();
    assert_eq!(p["action"], "clone");
    assert!(p["git_url"].starts_with("git@github.com:"));
    assert!(p["git_url"].ends_with("repo.git"));
}

#[test]
fn dotgit_suffix_stripped() {
    let tmp = setup_ggroot("github.com", "org");
    let r = run_gg(tmp.path(), &["https://github.com/org/repo.git"]);
    assert!(r.success);
    let p = r.parsed();
    // repo name should be "repo", not "repo.git"
    assert!(p["cd_dir"].ends_with("/repo"));
    assert!(!p["cd_dir"].ends_with(".git"));
}

#[test]
fn tail_path() {
    let tmp = setup_ggroot("github.com", "org");
    let r = run_gg(tmp.path(), &["github.com/org/repo/sub/path"]);
    assert!(r.success);
    let p = r.parsed();
    assert!(p["cd_dir"].ends_with("github.com/org/repo/sub/path"));
}

// --- Clone vs Fetch ---

#[test]
fn clone_when_repo_missing() {
    let tmp = setup_ggroot("github.com", "org");
    // org dir exists, repo dir does not
    let r = run_gg(tmp.path(), &["github.com/org/repo"]);
    assert!(r.success);
    let p = r.parsed();
    assert_eq!(p["action"], "clone");
    assert!(p.contains_key("git_url"));
    // git_dir should be the org dir (where clone runs)
    assert!(p["git_dir"].ends_with("github.com/org"));
}

#[test]
fn fetch_when_repo_exists() {
    let tmp = setup_ggroot("github.com", "org");
    // create the repo dir so getgit sees it as existing
    fs::create_dir_all(tmp.path().join("github.com/org/repo")).unwrap();
    let r = run_gg(tmp.path(), &["github.com/org/repo"]);
    assert!(r.success);
    let p = r.parsed();
    assert_eq!(p["action"], "fetch");
    assert!(!p.contains_key("git_url"));
    // git_dir should be the repo dir (where fetch runs)
    assert!(p["git_dir"].ends_with("github.com/org/repo"));
}

// --- Prefix ---

#[test]
fn prefix_flag() {
    let tmp = setup_ggroot("github.com", "org");
    let r = run_gg(tmp.path(), &["--prefix", "github.com/org", "repo"]);
    assert!(r.success);
    let p = r.parsed();
    assert_eq!(p["action"], "clone");
    assert!(p["cd_dir"].ends_with("github.com/org/repo"));
}

// --- Dry-run ---

#[test]
fn dry_run_no_stdout() {
    let tmp = setup_ggroot("github.com", "org");
    let output = Command::new(binary_path())
        .args(["--get", "-n", "github.com/org/repo"])
        .env("GGROOT", tmp.path())
        .output()
        .expect("failed to run gg");

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    // stderr should still have the info line
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("github.com"));
}

// --- Error cases ---

#[test]
fn invalid_url() {
    let tmp = setup_ggroot("github.com", "org");
    let r = run_gg(tmp.path(), &["not-a-valid-url"]);
    assert!(!r.success);
}

#[test]
fn missing_host_dir() {
    let tmp = TempDir::new().unwrap();
    // no host dir created under ggroot — git ls-remote will fail for the
    // bogus URL, so we expect non-zero exit with an error message.
    let r = run_gg(tmp.path(), &["github.com/org/repo"]);
    assert!(!r.success);
    assert!(r.stderr.contains("remote not found"));
}

// --- Setup (interactive installer) ---

/// Whether VSCode is installed on this machine (affects prompt flow).
fn has_vscode() -> bool {
    Path::new("/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code").is_file()
}

/// Build stdin input for the setup wizard.
/// Takes the answers in order: ggroot, protocol, viewer answer, aliases (vec), final action.
/// Handles the VSCode/no-VSCode prompt difference automatically.
fn setup_input(
    ggroot: &str,
    protocol: &str,
    viewer: SetupViewer,
    aliases: &[&str],
    action: &str,
) -> String {
    let mut lines = vec![ggroot.to_string(), protocol.to_string()];

    match viewer {
        SetupViewer::AcceptDefault => {
            // Send newline to accept default (VSCode "y" or no-VSCode "-")
            lines.push(String::new());
        }
        SetupViewer::No => {
            if has_vscode() {
                // "n" to decline VSCode, then "-" for "no viewer"
                lines.push("n".to_string());
                lines.push("-".to_string());
            } else {
                // "-" directly
                lines.push("-".to_string());
            }
        }
        SetupViewer::Custom(cmd) => {
            if has_vscode() {
                // "n" to decline VSCode, then the custom command
                lines.push("n".to_string());
                lines.push(cmd.to_string());
            } else {
                lines.push(cmd.to_string());
            }
        }
    }

    // Aliases, then empty line to stop
    for alias in aliases {
        lines.push(alias.to_string());
    }
    lines.push(String::new()); // end aliases

    lines.push(action.to_string());

    // Each line needs a newline
    lines.join("\n") + "\n"
}

enum SetupViewer {
    AcceptDefault,
    No,
    Custom(&'static str),
}

/// Run `gg` (no args) with HOME set to the given dir, piping stdin.
fn run_setup(home: &Path, stdin_input: &str) -> GgResult {
    let mut child = Command::new(binary_path())
        .env("HOME", home)
        .env_remove("GGROOT")
        .env_remove("GGHTTP")
        .env_remove("GGDIRVIEWER")
        .env_remove("GGNOAUTOCD")
        .env_remove("TERM_PROGRAM")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn gg");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin_input.as_bytes())
        .unwrap();

    let output = child.wait_with_output().expect("failed to wait on gg");

    GgResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        success: output.status.success(),
    }
}

/// Read .zshrc from the given home dir.
fn read_zshrc(home: &Path) -> String {
    let zshrc = home.join(".zshrc");
    if zshrc.is_file() {
        fs::read_to_string(zshrc).unwrap()
    } else {
        String::new()
    }
}

#[test]
fn setup_already_installed() {
    let tmp = TempDir::new().unwrap();
    let zshrc = tmp.path().join(".zshrc");
    fs::write(&zshrc, "eval \"$(gg -i zsh)\"\n").unwrap();

    let r = run_setup(tmp.path(), "");
    assert!(r.success);
    assert!(r.stderr.contains("already installed"));
    // .zshrc should be unchanged
    let content = fs::read_to_string(&zshrc).unwrap();
    assert_eq!(content, "eval \"$(gg -i zsh)\"\n");
}

#[test]
fn setup_defaults_append() {
    let tmp = TempDir::new().unwrap();
    let input = setup_input("", "", SetupViewer::AcceptDefault, &[], "y");
    let r = run_setup(tmp.path(), &input);
    assert!(r.success, "stderr: {}", r.stderr);

    let content = read_zshrc(tmp.path());
    assert!(content.contains("# --- gg ---"));
    assert!(content.contains("# --- end gg ---"));
    assert!(content.contains("-i zsh)\""));
    // With all defaults, no GGROOT or GGHTTP exports
    assert!(!content.contains("export GGROOT"));
    assert!(!content.contains("export GGHTTP"));
    assert!(r.stderr.contains("Installed!"));
}

#[test]
fn setup_custom_ggroot() {
    let tmp = TempDir::new().unwrap();
    let input = setup_input("~/repos", "", SetupViewer::AcceptDefault, &[], "y");
    let r = run_setup(tmp.path(), &input);
    assert!(r.success, "stderr: {}", r.stderr);

    let content = read_zshrc(tmp.path());
    assert!(content.contains("export GGROOT=~/repos"));
}

#[test]
fn setup_https_protocol() {
    let tmp = TempDir::new().unwrap();
    let input = setup_input("", "https", SetupViewer::AcceptDefault, &[], "y");
    let r = run_setup(tmp.path(), &input);
    assert!(r.success, "stderr: {}", r.stderr);

    let content = read_zshrc(tmp.path());
    assert!(content.contains("export GGHTTP=1"));
}

#[test]
fn setup_viewer_disabled() {
    let tmp = TempDir::new().unwrap();
    let input = setup_input("", "", SetupViewer::No, &[], "y");
    let r = run_setup(tmp.path(), &input);
    assert!(r.success, "stderr: {}", r.stderr);

    let content = read_zshrc(tmp.path());
    assert!(content.contains("export GGDIRVIEWER=-"));
}

#[test]
fn setup_viewer_custom() {
    let tmp = TempDir::new().unwrap();
    let input = setup_input("", "", SetupViewer::Custom("vim"), &[], "y");
    let r = run_setup(tmp.path(), &input);
    assert!(r.success, "stderr: {}", r.stderr);

    let content = read_zshrc(tmp.path());
    assert!(content.contains("export GGDIRVIEWER=vim"));
}

#[test]
fn setup_single_alias() {
    let tmp = TempDir::new().unwrap();
    let input = setup_input("", "", SetupViewer::AcceptDefault, &["ghg github.com"], "y");
    let r = run_setup(tmp.path(), &input);
    assert!(r.success, "stderr: {}", r.stderr);

    let content = read_zshrc(tmp.path());
    assert!(content.contains("-i zsh ghg github.com)\""));
}

#[test]
fn setup_multiple_aliases() {
    let tmp = TempDir::new().unwrap();
    let input = setup_input(
        "",
        "",
        SetupViewer::AcceptDefault,
        &["ghg github.com", "gmg github.com/marcelocantos"],
        "y",
    );
    let r = run_setup(tmp.path(), &input);
    assert!(r.success, "stderr: {}", r.stderr);

    let content = read_zshrc(tmp.path());
    assert!(content.contains("-i zsh ghg github.com)\""));
    assert!(content.contains("-i zsh gmg github.com/marcelocantos)\""));
}

#[test]
fn setup_no_append() {
    let tmp = TempDir::new().unwrap();
    let input = setup_input("", "", SetupViewer::AcceptDefault, &[], "n");
    let r = run_setup(tmp.path(), &input);
    assert!(r.success, "stderr: {}", r.stderr);

    // .zshrc should not exist
    assert!(!tmp.path().join(".zshrc").exists());
    assert!(r.stderr.contains("No changes made"));
}

#[test]
fn setup_print_mode() {
    let tmp = TempDir::new().unwrap();
    let input = setup_input("", "", SetupViewer::AcceptDefault, &[], "print");
    let r = run_setup(tmp.path(), &input);
    assert!(r.success, "stderr: {}", r.stderr);

    // .zshrc should not exist
    assert!(!tmp.path().join(".zshrc").exists());
    // stdout should have the block
    assert!(r.stdout.contains("# --- gg ---"));
    assert!(r.stdout.contains("-i zsh)\""));
    assert!(r.stdout.contains("# --- end gg ---"));
}

#[test]
fn setup_preserves_existing_content() {
    let tmp = TempDir::new().unwrap();
    let existing = "# My existing config\nexport PATH=$HOME/bin:$PATH\n";
    fs::write(tmp.path().join(".zshrc"), existing).unwrap();

    let input = setup_input("", "", SetupViewer::AcceptDefault, &[], "y");
    let r = run_setup(tmp.path(), &input);
    assert!(r.success, "stderr: {}", r.stderr);

    let content = read_zshrc(tmp.path());
    // Existing content preserved
    assert!(content.starts_with(existing));
    // New block appended
    assert!(content.contains("# --- gg ---"));
    assert!(content.contains("-i zsh)\""));
}

#[test]
fn setup_creates_zshrc() {
    let tmp = TempDir::new().unwrap();
    assert!(!tmp.path().join(".zshrc").exists());

    let input = setup_input("", "", SetupViewer::AcceptDefault, &[], "y");
    let r = run_setup(tmp.path(), &input);
    assert!(r.success, "stderr: {}", r.stderr);

    // .zshrc should now exist with the block
    assert!(tmp.path().join(".zshrc").exists());
    let content = read_zshrc(tmp.path());
    assert!(content.contains("# --- gg ---"));
}

#[test]
fn setup_block_markers() {
    let tmp = TempDir::new().unwrap();
    let input = setup_input(
        "~/myrepos",
        "https",
        SetupViewer::No,
        &["ghg github.com"],
        "y",
    );
    let r = run_setup(tmp.path(), &input);
    assert!(r.success, "stderr: {}", r.stderr);

    let content = read_zshrc(tmp.path());

    // Extract the block between markers
    let start = content.find("# --- gg ---").expect("missing start marker");
    let end = content
        .find("# --- end gg ---")
        .expect("missing end marker");
    let block = &content[start..end + "# --- end gg ---".len()];

    // Verify block structure
    let block_lines: Vec<&str> = block.lines().collect();
    assert_eq!(block_lines[0], "# --- gg ---");
    assert_eq!(*block_lines.last().unwrap(), "# --- end gg ---");

    // Verify all config is inside the block
    assert!(block.contains("export GGROOT=~/myrepos"));
    assert!(block.contains("export GGHTTP=1"));
    assert!(block.contains("export GGDIRVIEWER=-"));
    assert!(block.contains("-i zsh)\""));
    assert!(block.contains("-i zsh ghg github.com)\""));
}

#[test]
fn setup_full_config() {
    // Test a complete non-default setup
    let tmp = TempDir::new().unwrap();
    let input = setup_input(
        "~/code",
        "https",
        SetupViewer::Custom("subl"),
        &["gh github.com", "gl gitlab.com"],
        "y",
    );
    let r = run_setup(tmp.path(), &input);
    assert!(r.success, "stderr: {}", r.stderr);

    let content = read_zshrc(tmp.path());
    assert!(content.contains("export GGROOT=~/code"));
    assert!(content.contains("export GGHTTP=1"));
    assert!(content.contains("export GGDIRVIEWER=subl"));
    assert!(content.contains("-i zsh)\""));
    assert!(content.contains("-i zsh gh github.com)\""));
    assert!(content.contains("-i zsh gl gitlab.com)\""));
}

// --- Shell integration ---

#[test]
fn shell_integration() {
    // Skip if zsh is not available
    let zsh_check = Command::new("zsh").arg("-c").arg("true").output();
    match zsh_check {
        Err(_) => {
            eprintln!("zsh not found, skipping shell integration test");
            return;
        }
        Ok(out) if !out.status.success() => {
            eprintln!("zsh not working, skipping shell integration test");
            return;
        }
        _ => {}
    }

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let script = format!("{manifest_dir}/tests/integration.sh");
    let binary = binary_path();

    let output = Command::new("zsh")
        .arg(&script)
        .env("GG_BINARY", &binary)
        .env_remove("TERM_PROGRAM")
        .output()
        .expect("failed to run integration.sh");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        panic!("Shell integration test failed.\nstdout:\n{stdout}\nstderr:\n{stderr}");
    }
}
