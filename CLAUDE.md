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
| Check (no codegen) | `cargo check` |
| Format | `cargo fmt` |
| Lint | `cargo clippy` |

There are currently no tests in the project.

## Architecture

All source lives in `src/`. The binary entry point is `main.rs`.

- **cli.rs** — CLI argument parsing via clap derive macros (`Cli` struct, `Shell` enum).
- **getgit.rs** — Core logic: parses a Git URL/shorthand with a single regex, resolves the host/org/repo components, and emits shell commands to clone or fetch + cd + open viewer.
- **env.rs** — Wrappers for environment variables and platform detection (`GGROOT`, `GGDIRVIEWER`, `GGNOAUTOCD`, `GGHTTP`, home dir, exe path, shell type).
- **zsh.rs** — Generates zsh shell function, aliases, and `_gg` tab-completion.
- **bash.rs / fish.rs** — Stub shell integrations (not yet implemented).
- **help.rs** — First-run help/setup instructions printed when `gg` is invoked with no arguments.

### Flow

1. `main.rs` parses CLI args. With no args, prints setup help. With `--install <shell>`, emits shell integration code. With `--get`, delegates to `getgit`.
2. The shell function (installed via `eval "$(gg -i zsh)"`) wraps the binary: it calls `gg --get <url>` and `eval`s the stdout (clone/fetch + cd + viewer commands).
3. `getgit` uses a regex to accept multiple URL formats (`git@host:org/repo`, `https://host/org/repo`, `host/org/repo`), resolves paths under `$GGROOT`, and writes shell commands to stdout.

### Key environment variables

- `GGROOT` — repo tree root (default `~/work`)
- `GGDIRVIEWER` — IDE/editor command (default: `code` if available)
- `GGNOAUTOCD` — disable auto-cd after clone/fetch
- `GGHTTP` — prefer HTTPS URLs over SSH
