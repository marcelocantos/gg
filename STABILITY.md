# Stability

gg is pre-1.0 (`v0.1.0`). This catalogue is the public interaction
surface as implemented at HEAD — not the help text or README
aspiration.

## Stability commitment

Once 1.0 ships, breaking changes to the zsh executor contract, the
`--get` key=value protocol, or the documented env vars require a
major version bump (or a new product name). The pre-1.0 period exists
to get those right.

## Interaction surface

Snapshot as of v0.1.0.

### CLI — `gg`

| Form | What it actually does | Stability |
|------|------------------------|-----------|
| `gg` (no args) | Interactive installer. Writes a marked block to `~/.zshrc` only (append / print / no-op). Never writes `.bashrc`, fish config, or any other rc. | Needs review |
| `gg <url>` | **Not** handled by the binary. Requires the zsh function from `gg -i zsh`. Bare `gg <url>` without that function is parsed as installer/`-i` args and will not clone. | Needs review |
| `-i zsh` / `--install zsh` | Emits the `gg()` function, `_gg` completion, and optional alias functions on stdout. | Needs review |
| `-i zsh <cmd> <prefix>` | Emits a prefixed alias function plus completion rooted at `$GGROOT/<prefix>`. Errors if prefix is missing. | Needs review |
| `-i bash` / `-i fish` | **clap `possible values` include these.** Each stub prints `<shell> not supported yet` on stderr, writes nothing to stdout, and **exits 0**. `eval "$(gg -i bash)"` therefore "succeeds" and installs a no-op. | Fluid — stubs until https://github.com/marcelocantos/gg/issues/1 |
| `-n` / `--dry-run` | With `--get`: prints the pretty path on stderr, **creates the org directory**, suppresses key=value stdout. Does not clone/fetch. Help text says "do nothing"; the FS write is not gated. | Needs review |
| `--version` | Prints crate version. | Stable |
| `--help` | clap help. Lists `-i` possible values `zsh`, `bash`, `fish`. Does not mention that bash/fish are stubs. | Needs review |
| `--get <path>` | Internal. Plans only: writes `action` / `git_dir` / `git_url` / `cd_dir` on stdout. Does not run git. Hidden from `--help`. | Needs review |
| `--get` (no path) | Prints `Usage: gg <path>` on stderr and **exits 0**. | Needs review |
| `--prefix <path>` | Internal. Prepended to the `--get` target (alias invocations). Hidden from `--help`. | Needs review |

There is **no** `--help-agent` flag at HEAD.

### Shells

| Shell | Installer | `-i` | Status |
|-------|-----------|------|--------|
| zsh | `~/.zshrc` block with `eval "$(<exe> -i zsh)"` | Working function + completion | Implemented |
| bash | not written | stderr stub, exit 0 | Not implemented |
| fish | not written | stderr stub, exit 0 | Not implemented |

`--help` advertising bash/fish is not an install path.

### Environment variables

| Variable | Actual effect | Default |
|----------|---------------|---------|
| `GGROOT` | Tree root for host/org/repo | `~/work` |
| `GGHTTP` | Non-empty → shorthand URLs use HTTPS | empty (SSH) |
| `GGDIRVIEWER` | Viewer command; `-` disables. Unset → zsh tries the macOS VSCode `code` binary path | unset |
| `GGNOAUTOCD` | If set, skip `cd` and viewer after clone/fetch | unset |
| `TERM_PROGRAM` | `vscode` skips `cd` and viewer the same way; git still runs | unset |

`--get` never clones. `GGNOAUTOCD` / VSCode skip only `cd` and the
viewer; the generated function still runs `git clone` / `git fetch`.

### URL parse (`--get`)

Accepted shapes: `git@host:org/repo`, `https://host/org/repo`,
`host/org/repo`, optional trailing `.git`, optional `/tail` after the
repo name.

Constraints that are easy to miss:

- Repo capture is `[^/\.]+`. Names with interior dots (`next.js`) are
  `invalid path`.
- Host, org, repo, and tail components `.` and `..` are `invalid path`.
  Emitted `git_dir` / `cd_dir` stay under `$GGROOT` after normalization.
- Tail is **not** ignored: it is appended to `cd_dir`.
- New host directories are created only after `git ls-remote`
  succeeds (stderr discarded). `--dry-run` still runs that check and
  still `create_dir_all`s the org path.

### `--get` stdout protocol

Unversioned `key=value` lines. Empty stdout (dry-run, or the zsh
function's failed/`[ -z "$output" ]` path) means the executor does
nothing.

| Key | Clone | Fetch |
|-----|-------|-------|
| `action` | `clone` | `fetch` |
| `git_dir` | **org** directory (parent of the new repo) | **repo** directory |
| `git_url` | present | omitted |
| `cd_dir` | repo path + optional tail | same |

A second consumer that treats `git_dir` as "always the repo root"
will clone into the wrong place.

### Zsh executor git lines

- clone: `git -C "$git_dir" clone --recurse-submodules "$git_url"`
- fetch: `git -C "$git_dir" fetch --all --prune --jobs=10 --recurse-submodules=yes`

### Installer detection

"Already installed" is `~/.zshrc` containing the substring `-i zsh)"`,
not the `# --- gg ---` / `# --- end gg ---` markers the installer
writes. Re-running `gg` with that substring present is a no-op.

## Gaps before a 1.0 lock-in

- [ ] bash/fish: drop from clap, fail non-zero, or implement
      (https://github.com/marcelocantos/gg/issues/1)
- [x] Split GGNOAUTOCD/VSCode so clone/fetch still run; only `cd` and
      viewer are skipped
- [ ] Gate `create_dir_all` on `!dry_run`
- [ ] `--get` with no target should exit non-zero
- [ ] Repo names containing `.`
- [ ] `--help-agent` (CLI convention; not shipped)
