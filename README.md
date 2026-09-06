# gg

gg clones or fetches Git repos into a standardised directory tree
(e.g., `~/work/github.com/org/repo`), then `cd`s into the repo and
optionally opens it in an editor.

Shorthand URLs default to SSH (`git@host:org/repo.git`). Set `GGHTTP=1`
for HTTPS. New host directories are verified via `git ls-remote` before
creation.

Shell integration is zsh only. `gg -i bash` and `gg -i fish` are accepted
by the CLI but print `not supported yet` and emit nothing (tracked in
https://github.com/marcelocantos/gg/issues/1).

## Installation

### From release binaries

Download the tarball for your platform from the
[Releases](https://github.com/marcelocantos/gg/releases) page
(`gg-linux-amd64`, `gg-macos-amd64`, `gg-macos-arm64`, each a `.tar.gz`
containing the `gg` binary plus a `.sha256` file), extract it, and place
`gg` somewhere on your `PATH` (e.g., `/usr/local/bin` or `~/.local/bin`).

On macOS, you may need to allow the binary in
**System Settings > Privacy & Security** on first run.

### From source

```sh
cargo install --path .
```

### Setup

Run `gg` with no arguments to launch the interactive installer. It will
walk you through configuring:

- **GGROOT** — where repos live (default `~/work`)
- **Git protocol** — SSH (default) or HTTPS
- **Directory viewer** — editor to open after clone (default: VSCode if
  installed)
- **Aliases** — shorthand commands like `ghg github.com`

The installer appends a config block to `~/.zshrc`, or prints it for
manual pasting.

## Usage

```sh
gg github.com/org/repo          # shorthand (SSH by default)
gg https://github.com/org/repo  # explicit HTTPS
gg git@github.com:org/repo      # explicit SSH
```

If the repo is already cloned, gg fetches (`git fetch --all --prune`,
with submodules) instead of cloning. Either way, it `cd`s into the repo
and opens your configured viewer. A trailing path after the repo
(`gg github.com/org/repo/some/dir`) is appended to the `cd` target.

### Flags

| Flag | Purpose |
|---|---|
| `-n`, `--dry-run` | Print the resolved path to stderr and stop. Nothing is cloned, fetched, or opened. A new host is still verified with `git ls-remote`, and the org directory under `GGROOT` is still created. |
| `-i`, `--install <zsh\|bash\|fish>` | Emit shell integration code for `eval`. With two further arguments (`CMD PREFIX`) it emits an alias command instead. |
| `-V`, `--version` | Print version |
| `-h`, `--help` | Print usage |

`gg` with no arguments runs the interactive installer.

### Aliases

Aliases prefix their argument with a path:

```sh
# In ~/.zshrc (added by the installer):
eval "$(gg -i zsh ghg github.com)"
eval "$(gg -i zsh gmg github.com/marcelocantos)"

# Then:
ghg org/repo      # → gg github.com/org/repo
gmg gg            # → gg github.com/marcelocantos/gg
```

Tab-completion offers the directories under the alias prefix (or under
`GGROOT` for plain `gg`), so every cloned repo completes.

### Environment variables

| Variable | Description | Default |
|----------|-------------|---------|
| `GGROOT` | Repo tree root | `~/work` |
| `GGHTTP` | Any non-empty value makes shorthand URLs use HTTPS | unset (SSH) |
| `GGDIRVIEWER` | Command run on the repo directory after clone or fetch; `-` disables it | VS Code's bundled `code` binary, if `/Applications/Visual Studio Code.app` exists; otherwise nothing |
| `GGNOAUTOCD` | Any non-empty value makes the shell function stop after resolving the path: no clone, no fetch, no `cd`, no viewer. The same happens automatically when `TERM_PROGRAM` is `vscode` (VS Code's integrated terminal). | unset |

### Limitations

- Repository names containing a dot (for example `next.js`) are rejected
  with `invalid path`; only a trailing `.git` is stripped.
- Only the three-level `host/org/repo` layout is supported. Anything after
  `repo` is treated as a directory inside the repo to `cd` into, not as a
  nested group.

## License

[Apache-2.0](LICENSE)
