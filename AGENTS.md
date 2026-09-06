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
- Clone vs fetch planning, `--prefix`, `--dry-run` (asserts empty stdout
  only), invalid URL, missing host dir (hits a real `git ls-remote`
  failure).
- Fourteen installer tests that drive `gg` with piped stdin under a
  temporary `HOME`.
- `shell_integration`, which runs `tests/integration.sh` under zsh: the
  generated function clones, fetches, aliases, and honours `GGNOAUTOCD`
  against a local bare repo, with `url.<file>.insteadOf` redirecting
  `https://github.com/`. It asserts only on `PWD`.

## Architecture

All source is in `src/`; `main.rs` is the entry point.

- `cli.rs`: clap derive. Public flags `-n/--dry-run`, `-i/--install
  <zsh|bash|fish>`, `--version`, `--help`; hidden `--get` and `--prefix`.
- `getgit.rs`: the planner. A `LazyLock<Regex>` accepts
  `git@host:org/repo`, `https://host/org/repo`, and `host/org/repo`, each
  with an optional `.git` suffix or trailing path. A new host directory is
  verified with `git ls-remote` before creation. Writes the plan to stdout
  as `key=value` lines.
- `env.rs`: `GGROOT`, `GGHTTP`, home dir, executable path, `~`
  abbreviation.
- `setup.rs`: interactive installer. Prompts for GGROOT, protocol, viewer,
  and aliases, then appends a `# --- gg ---` block to `~/.zshrc` (or
  prints it). "Already installed" is detected by the substring `-i zsh)"`
  in `~/.zshrc`, not by the markers. Values are interpolated unquoted.
- `zsh.rs`: emits the `gg` shell function, alias functions, and `_gg`
  completion. Interpolated paths go through `shell::escape`.
- `shell.rs`: single-quote escaping.
- `bash.rs`, `fish.rs`: stubs that print `not supported yet` and return
  `Ok`, so `gg -i bash` exits 0 with no output. Open as
  https://github.com/marcelocantos/gg/issues/1.

### Flow

1. No arguments: `setup::setup`. `-i <shell>`: emit integration code.
   `--get <target>`: `getgit`.
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
| `GGDIRVIEWER` | Command run on the repo dir; `-` disables | VS Code's bundled `code` if `/Applications/Visual Studio Code.app` exists, else none |
| `GGNOAUTOCD` | Non-empty: the function returns before git, `cd`, and viewer | unset |
| `TERM_PROGRAM` | `vscode` has the same effect as `GGNOAUTOCD` | set by the terminal |

Known behaviour worth knowing before changing it (see
`docs/audits/entropy-audit-2026-08-22.md`): `GGNOAUTOCD` and the VS Code
check skip clone and fetch as well as `cd`; repo names containing a dot
are rejected by the regex; the VS Code path is duplicated in `setup.rs`,
`zsh.rs`, and `tests/cli.rs`.

## Conventions

- Read `~/.claude/rust.md` before changing Rust; `~/.claude/bash.md` before
  touching the generated shell or `tests/integration.sh`.
- Default branch is `master`. Squash-merge only.
- Keep the planner/executor split: the binary never runs `git clone`; the
  shell function never parses URLs.

## Delivery

Merged to `master`.
