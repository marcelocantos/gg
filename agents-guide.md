# gg — Agent Guide

This guide is for AI agents and tools that interact with `gg` as it
exists at HEAD. Do not assume bash/fish integration or `--help-agent`.

## What gg does

`gg` manages Git repositories in a standardised directory tree rooted
at `$GGROOT` (default `~/work`). Canonical path:
`$GGROOT/<host>/<org>/<repo>`.

The **binary** only plans. The **zsh function** (`eval "$(gg -i zsh)"`)
runs git clone/fetch, `cd`, and the viewer. Without that function,
`gg github.com/org/repo` is not a clone command.

## CLI reference

```
gg [TARGET] [ALIAS_PREFIX]
```

| Flag / Arg | Description |
|---|---|
| `TARGET` | With `--get`: repo URL or path. With `-i zsh`: alias command name. With no `-i`/`--get`: unused (no-arg `gg` is the installer). |
| `ALIAS_PREFIX` | Path prefix for an alias (`-i zsh <cmd> <prefix>`). |
| `-n`, `--dry-run` | With `--get`: no key=value stdout. Still creates the org directory. Does not clone. |
| `-i`, `--install <SHELL>` | `zsh` emits integration. `bash` and `fish` print `not supported yet` on stderr, emit nothing, **exit 0**. |
| `--version` | Print version |
| `--help` | Print help (lists bash/fish as possible `-i` values; those are stubs) |

There is no `--help-agent` flag.

### Internal flags (hidden from --help)

| Flag | Description |
|---|---|
| `--get` | Print a clone/fetch **plan** (key=value). Does not run git. |
| `--prefix <PATH>` | Prepended to the `--get` target for aliases |
| `--get` with no path | Prints `Usage: gg <path>`, exits 0 |

## URL formats accepted

- `github.com/org/repo` — shorthand (SSH unless `GGHTTP` is set)
- `https://github.com/org/repo` — explicit HTTPS
- `git@github.com:org/repo` — explicit SSH
- Trailing `.git` is stripped
- `/tail` after the repo name is appended to `cd_dir` (not discarded)
- Repo names containing `.` (e.g. `org/next.js`) are rejected

## Environment variables

| Variable | Description | Default |
|---|---|---|
| `GGROOT` | Repo tree root | `~/work` |
| `GGHTTP` | Non-empty → HTTPS shorthand | unset (SSH) |
| `GGDIRVIEWER` | Editor command; `-` to disable | macOS VSCode `code` path if that file exists |
| `GGNOAUTOCD` | If set, skip `cd` and viewer after clone/fetch | unset (same skip when `TERM_PROGRAM=vscode`) |

## Shell integration

```sh
eval "$(gg -i zsh)"
eval "$(gg -i zsh ghg github.com)"       # ghg org/repo → gg github.com/org/repo
eval "$(gg -i zsh gmg github.com/mc)"    # gmg repo    → gg github.com/mc/repo
```

Do **not** `eval "$(gg -i bash)"` or `eval "$(gg -i fish)"`. Those
succeed with empty stdout. The interactive installer only updates
`~/.zshrc`. Tracked as https://github.com/marcelocantos/gg/issues/1.

Tab-completion (`_gg` / alias `compdef`) lists directories under
`$GGROOT` or the alias prefix.

If `GGNOAUTOCD` is set or `TERM_PROGRAM` is `vscode`, the function
still clones or fetches, then returns before `cd` and the viewer.
`--get` itself never clones.

## Key=value output protocol (`--get`)

| Key | Value |
|---|---|
| `action` | `clone` or `fetch` |
| `git_dir` | Org directory on **clone**; repo directory on **fetch** |
| `git_url` | Resolved git URL (**clone only**) |
| `cd_dir` | Directory to cd into (repo + optional tail) |

## Architecture (repo)

- `cli.rs` — clap derive
- `getgit.rs` — URL parse, host verify, key=value
- `setup.rs` — interactive installer (`~/.zshrc` only)
- `env.rs` — env / home / exe
- `zsh.rs` — zsh function + completion
- `bash.rs` / `fish.rs` — stubs, exit 0
- `shell.rs` — quoting for generated zsh

## Building

```sh
cargo build
cargo build --release
cargo test
cargo clippy
cargo fmt
```
