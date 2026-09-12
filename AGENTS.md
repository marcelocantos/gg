# AGENTS.md

Shared instructions for agents working in this repository.

**gg** is a Rust CLI that plans a clone or fetch into a standardised
directory tree (`$GGROOT/<host>/<org>/<repo>`, default
`~/work/github.com/org/repo`). A generated **zsh** function is the
executor: it runs `gg --get`, then git clone/fetch, `cd`, and an
optional viewer. bash and fish are clap enum values, not working
integrations.

## Build & run

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

Tests live in `tests/cli.rs` (Rust integration tests) and
`tests/integration.sh` (zsh journey). All tests run via `cargo test`.
There are currently 31 tests:

- **URL format tests** — shorthand SSH/HTTPS, explicit SSH/HTTPS,
  `.git` suffix stripping, tail paths
- **Clone vs fetch** — clone plan when repo missing, fetch plan when
  repo exists (`--get` does not run git)
- **Prefix, dry-run, error cases** — prefix flag, dry-run suppresses
  stdout (org dirs are still created), invalid URL, missing host dir,
  `.` / `..` path escape rejected
- **Setup (interactive installer)** — 14 tests: already-installed
  detection, default/custom GGROOT, protocol, viewer, aliases,
  append/print/no-op, `.zshrc` preservation, block markers
- **Shell integration** — zsh journey (clone, fetch, alias,
  GGNOAUTOCD, `TERM_PROGRAM=vscode`) against local bare repos.
  The skip-cd cases assert `.git` exists and `PWD` is unchanged.

Tests use `tempfile::TempDir`. Setup tests override `HOME`. URL tests
inherit ambient `GGHTTP` unless the test sets it.

## Architecture

All source lives in `src/`. Binary entry: `main.rs`.

- **cli.rs** — clap derive (`Cli`, `Shell`). Internal `--get` /
  `--prefix` are hidden from `--help`. `Shell` possible values are
  `zsh`, `bash`, `fish`.
- **getgit.rs** — parses a Git URL/shorthand with a static
  `LazyLock<Regex>`, resolves host/org/repo, verifies new hosts via
  `git ls-remote`, writes key=value to stdout. Repo names with `.`
  (e.g. `next.js`) do not match. Host/org/repo/tail components `.`
  and `..` are `invalid path`; emitted `git_dir` / `cd_dir` stay
  under `$GGROOT`. `--dry-run` skips stdout but still
  `create_dir_all`s the org directory.
- **env.rs** — `GGROOT`, `GGHTTP`, home dir, exe path, `~` display.
- **setup.rs** — interactive installer. Detects install by substring
  `-i zsh)"`. Always emits `eval "$(… -i zsh)"`. Writes only
  `~/.zshrc` (or prints the block).
- **zsh.rs** — generates the zsh function (parse key=value, git, cd,
  viewer), aliases, and `_gg` completion.
- **shell.rs** — single-quote escaping for generated zsh.
- **bash.rs / fish.rs** — stubs: print `bash not supported yet` /
  `fish not supported yet` to stderr and return `Ok(())` (exit 0).
  Tracked in https://github.com/marcelocantos/gg/issues/1.

### Flow

1. `main.rs` parses CLI args. No args → interactive installer. `-i
   <shell>` → zsh codegen or bash/fish stub. `--get` → `getgit`.
   `--get` with no target prints `Usage: gg <path>` and exits 0.
2. The zsh function (`eval "$(gg -i zsh)"`) calls `gg --get <url>`,
   parses stdout, then always clone/fetch. `GGNOAUTOCD` or
   `TERM_PROGRAM=vscode` skip only `cd` and the viewer.
3. Shorthand URLs default to SSH; `GGHTTP=1` selects HTTPS. New host
   directories are verified with `git ls-remote` before creation.

`--get` never clones; the zsh function is the only clone/fetch path.

### `--get` protocol (stdout)

| Key | When | Meaning |
|-----|------|---------|
| `action` | always | `clone` or `fetch` |
| `git_dir` | always | **org directory** on clone; **repo directory** on fetch |
| `git_url` | clone only | resolved git URL |
| `cd_dir` | always | repo path plus optional tail after the repo name |

### Environment variables

- `GGROOT` — repo tree root (default `~/work`)
- `GGHTTP` — set to `1` to prefer HTTPS URLs over SSH (default: SSH)
- `GGDIRVIEWER` — IDE/editor command (default: the macOS VSCode
  `code` binary if that path exists; `-` to disable)
- `GGNOAUTOCD` — if set, skip `cd` and viewer after clone/fetch.
  The same skip applies when `TERM_PROGRAM=vscode`.

Public CLI and env contracts: [STABILITY.md](STABILITY.md).
User-facing usage: [README.md](README.md).
Agent-oriented CLI notes: [agents-guide.md](agents-guide.md).
