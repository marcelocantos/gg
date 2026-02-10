# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**gg** is a Rust CLI tool that clones or fetches Git repos into a standardised directory tree (e.g., `~/work/github.com/org/repo`), then `cd`s into the repo and optionally opens it in an IDE. It provides shell integration (zsh fully implemented; bash/fish are stubs).

## Build & Run Commands

| Task | Command |
|------|---------|
| Build (debug) | `cargo build` |
| Build (release) | `cargo build --release` |
| Install locally | `cargo install --path .` |
| Run | `cargo run -- <args>` |
| Run tests | `cargo test` |
| Check (no codegen) | `cargo check` |
| Format | `cargo fmt` |
| Lint | `cargo clippy` |

## Testing

Tests live in `tests/cli.rs` (Rust integration tests) and `tests/integration.sh` (zsh shell integration). All tests run via `cargo test`. There are currently 27 tests:

- **URL format tests** — shorthand SSH/HTTPS, explicit SSH/HTTPS, `.git` suffix stripping, tail paths
- **Clone vs fetch** — clone when repo missing, fetch when repo exists
- **Prefix, dry-run, error cases** — prefix flag, dry-run suppresses stdout, invalid URL, missing host dir
- **Setup (interactive installer)** — 14 tests covering all setup paths: already installed detection, default/custom GGROOT, protocol, viewer, aliases, append/print/no-op modes, .zshrc preservation, block markers
- **Shell integration** — end-to-end zsh test (clone, fetch, alias, GGNOAUTOCD) using local bare repos

Tests use `tempfile::TempDir` for isolation. Setup tests override `HOME` to a temp dir to avoid touching real config files.

## Architecture

All source lives in `src/`. The binary entry point is `main.rs`.

- **cli.rs** — CLI argument parsing via clap derive macros (`Cli` struct, `Shell` enum). Internal flags (`--get`, `--prefix`) are hidden from `--help`.
- **getgit.rs** — Core logic: parses a Git URL/shorthand with a static `LazyLock<Regex>`, resolves host/org/repo, verifies new hosts via `git ls-remote`, and outputs key=value data (action, git_dir, git_url, cd_dir) to stdout.
- **env.rs** — Wrappers for environment variables and home dir (`GGROOT`, `GGHTTP`, home dir, exe path, squiggler for `~` display).
- **setup.rs** — Interactive installer: walks the user through GGROOT, protocol, viewer, and aliases, then appends a marked config block to `~/.zshrc` or prints it.
- **zsh.rs** — Generates zsh shell function (parses key=value output, runs git, cd, viewer), aliases, and `_gg` tab-completion.
- **shell.rs** — Single-quote escaping for safe shell interpolation.
- **bash.rs / fish.rs** — Stub shell integrations (not yet implemented, tracked in GitHub issue #1).

### Flow

1. `main.rs` parses CLI args. With no args, runs the interactive installer. With `-i <shell>`, emits shell integration code. With `--get`, delegates to `getgit`.
2. The shell function (installed via `eval "$(gg -i zsh)"`) wraps the binary: it calls `gg --get <url>`, parses the key=value stdout, then runs git clone/fetch, cd, and viewer.
3. `getgit` uses a regex to accept multiple URL formats (`git@host:org/repo`, `https://host/org/repo`, `host/org/repo`), resolves paths under `$GGROOT`, and writes key=value pairs to stdout. Shorthand URLs default to SSH; set `GGHTTP=1` for HTTPS. New host directories are verified via `git ls-remote` before creation.

### Key environment variables

- `GGROOT` — repo tree root (default `~/work`)
- `GGHTTP` — set to `1` to prefer HTTPS URLs over SSH (default: SSH)
- `GGDIRVIEWER` — IDE/editor command (default: `code` if VSCode installed; `-` to disable)
- `GGNOAUTOCD` — disable auto-cd after clone/fetch (also auto-suppressed in VSCode)
