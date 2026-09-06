# gg

Rust CLI that clones or fetches a Git repo into `$GGROOT/<host>/<org>/<repo>`
(default `~/work/github.com/org/repo`), then `cd`s into it and opens a
viewer. The binary only plans; a generated zsh function executes.

## Build and test

| Task | Command |
|---|---|
| Build | `cargo build` (`--release` for a release binary) |
| Install locally | `cargo install --path .` |
| Run | `cargo run -- <args>` |
| Test | `cargo test` |
| Lint / format | `cargo clippy -- -D warnings`, `cargo fmt -- --check` |

CI (`.github/workflows/ci.yml`) runs clippy, `cargo test`, and the fmt
check on `ubuntu-latest` for pushes to `master` and every pull request.
Releases (`release.yml`, on a created GitHub release) build
`x86_64-unknown-linux-musl`, `x86_64-apple-darwin` (cross-compiled from
`macos-14`, tests skipped), and `aarch64-apple-darwin`, and upload
`gg-<os>-<arch>.tar.gz` plus `.sha256`.

Tests read the ambient environment: `tests/cli.rs` sets `GGROOT` but does
not clear `GGHTTP`, so `shorthand_url_ssh` fails on a machine that exports
`GGHTTP`. Run `env -u GGHTTP cargo test` there. The setup tests do clear
`HOME` and the `GG*` variables.

## Testing

`cargo test` runs 27 tests, all in `tests/cli.rs` (there are no unit tests
under `src/`):

- URL forms: shorthand SSH, shorthand HTTPS with `GGHTTP`, explicit HTTPS,
  explicit SSH, `.git` suffix stripping, trailing path.
- Clone vs fetch planning, `--prefix`, `--dry-run` (asserts stdout is
  empty and that the stderr line still names the host), invalid URL,
  missing host dir (hits a real `git ls-remote` failure).
- Fourteen installer tests that drive `gg` with piped stdin, `HOME`
  pointed at a `TempDir` and `GGROOT`/`GGHTTP`/`GGDIRVIEWER`/`GGNOAUTOCD`/
  `TERM_PROGRAM` removed from the child environment. `has_vscode()` makes
  the expected prompt sequence machine-dependent.
- `shell_integration`, which runs `tests/integration.sh` under zsh: the
  generated function clones, fetches, aliases, and honours `GGNOAUTOCD`
  against a local bare repo, with `url.<file>.insteadOf` redirecting
  `https://github.com/`. Four of its five assertions are on `PWD`; the
  fifth checks that the clone created `.git`. If `zsh` is missing or fails
  to run, the Rust test prints a note and passes anyway — a silent skip,
  not a failure.

## Architecture

All source is in `src/`; `main.rs` is the entry point.

- `cli.rs`: clap derive. Public flags `-n/--dry-run`, `-i/--install
  <zsh|bash|fish>`, `--version`, `--help`; hidden `--get` and `--prefix`.
- `getgit.rs`: the planner. A `LazyLock<Regex>` accepts
  `git@host:org/repo`, `http(s)://host/org/repo`, and the shorthand
  `host/org/repo` (also `host:org/repo`), each with an optional `.git`
  suffix or trailing path. The repo segment is `[^/\.]+`, so a dot
  anywhere in the repo name fails the match and the command exits 1 with
  `invalid path: <arg>`. A new host directory is verified with `git
  ls-remote` before creation. The `👉` summary goes to stderr
  unconditionally; the plan goes to stdout as `key=value` lines only when
  `--dry-run` is absent.
- `env.rs`: `GGROOT`, `GGHTTP`, home dir, executable path, `~`
  abbreviation.
- `setup.rs`: interactive installer. Prompts for GGROOT, protocol, viewer,
  and aliases, then appends a `# --- gg ---` block to `~/.zshrc` (or
  prints it). "Already installed" is detected by the substring `-i zsh)"`
  in `~/.zshrc`, not by the markers. Values are interpolated unquoted.
- `zsh.rs`: emits the `gg` shell function, alias functions, and `_gg`
  completion. Interpolated paths go through `shell::escape`.
- `shell.rs`: single-quote escaping.
- `bash.rs`, `fish.rs`: stubs that print `<shell> not supported yet` to
  stderr and return `Ok`, so `gg -i bash` exits 0 with empty stdout. Open
  as https://github.com/marcelocantos/gg/issues/1.

### Flow

1. `main::run` dispatches on `--get` first, then on `--install`, and
   otherwise falls through to `setup::setup`. So `--get <target>` runs the
   planner, `-i <shell>` emits integration code (with `TARGET ALIAS_PREFIX`
   it emits an alias instead, and a `TARGET` without a prefix errors with
   `missing prefix argument`), and anything else — including
   `gg -n github.com/org/repo` typed at the real binary rather than through
   the shell function — starts the interactive installer, silently ignoring
   the flag and the argument.
2. The installed function runs `'<exe>' --get "$@"`, parses the plan, and
   returns early if `TERM_PROGRAM` is `vscode` or `GGNOAUTOCD` is set.
   Otherwise it runs `git clone --recurse-submodules` or `git fetch --all
   --prune --jobs=10 --recurse-submodules=yes`, `cd`s, and runs the viewer.
3. Shorthand URLs are SSH unless `GGHTTP` is non-empty. Explicit
   `https://` and `git@` forms are preserved.

### Plan protocol (stdout of `gg --get`)

| Key | Meaning |
|---|---|
| `action` | `clone` or `fetch` |
| `git_dir` | On `clone`, the org directory to clone into. On `fetch`, the repo directory. |
| `git_url` | Clone URL; present only for `clone` |
| `cd_dir` | Repo directory plus any trailing path from the argument |

With `--dry-run` nothing is written to stdout, so the shell function does
nothing. The org directory is still created and a new host is still
verified.

### Environment variables

| Variable | Behaviour | Default |
|---|---|---|
| `GGROOT` | Repo tree root | `~/work` |
| `GGHTTP` | Non-empty: shorthand URLs use HTTPS | unset (SSH) |
| `GGDIRVIEWER` | Command run on `cd_dir` (repo dir plus any trailing path), not the repo root; `-` disables | `/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code` when that path passes `[ -x ]` in `zsh.rs` (`is_file()` in `setup.rs`), else none |
| `GGNOAUTOCD` | Non-empty: the function returns before git, `cd`, and viewer | unset |
| `TERM_PROGRAM` | `vscode` has the same effect as `GGNOAUTOCD` | set by the terminal |

Known behaviour worth knowing before changing it (see
`docs/audits/entropy-audit-2026-08-22.md`): `GGNOAUTOCD` and the VS Code
check skip clone and fetch as well as `cd`; repo names containing a dot
are rejected by the regex; the VS Code path is duplicated in `setup.rs`,
`zsh.rs`, and `tests/cli.rs`; `setup.rs` detects an existing install by the
substring `-i zsh)"` rather than by its own `# --- gg ---` markers, and
interpolates the user's answers into `~/.zshrc` unquoted.

## Conventions

- Read `~/.claude/rust.md` before changing Rust; `~/.claude/bash.md` before
  touching the generated shell or `tests/integration.sh`.
- Default branch is `master`. Squash-merge only.
- Keep the planner/executor split: the binary never runs `git clone`; the
  shell function never parses URLs.

## Delivery

Merged to `master`.
