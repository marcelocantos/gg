use std::collections::HashMap;
use std::fs;
use std::process::Command;

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
    let output = Command::new(binary_path())
        .arg("--get")
        .args(args)
        .env("GGROOT", ggroot)
        .output()
        .expect("failed to run gg");

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
fn shorthand_url() {
    let tmp = setup_ggroot("github.com", "org");
    let r = run_gg(tmp.path(), &["github.com/org/repo"]);
    assert!(r.success);
    let p = r.parsed();
    assert_eq!(p["action"], "clone");
    assert!(p["git_url"].contains("github.com/org/repo.git"));
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
    // no host dir created under ggroot
    let r = run_gg(tmp.path(), &["github.com/org/repo"]);
    assert!(r.success); // exits 0, just warns
    assert!(r.stdout.is_empty()); // no output
    assert!(r.stderr.contains("not found"));
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
        panic!(
            "Shell integration test failed.\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
    }
}
