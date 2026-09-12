# gg

gg clones or fetches Git repos into a standardised directory tree
(e.g., `~/work/github.com/org/repo`), then `cd`s into the repo and
optionally opens it in an editor.

That clone/`cd`/viewer path is the **zsh function** (`eval "$(gg -i
zsh)"`), not the `gg` binary alone. The binary plans (`gg --get`);
the function runs git.

Shorthand URLs default to SSH (`git@host:org/repo.git`). Set `GGHTTP=1`
for HTTPS. New host directories are verified via `git ls-remote` before
creation.

## Installation

### From release binaries

Download the binary for your platform from the
[Releases](https://github.com/marcelocantos/gg/releases) page and place
it somewhere on your `PATH` (e.g., `/usr/local/bin` or `~/.local/bin`).

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

The installer appends a config block to **`~/.zshrc` only**, or prints
it for manual pasting. It does not install bash or fish.

`gg --help` lists `-i` possible values `zsh`, `bash`, and `fish`.
`gg -i zsh` emits the working function. `gg -i bash` and `gg -i fish`
print `not supported yet` and exit 0; they emit no integration
([issue #1](https://github.com/marcelocantos/gg/issues/1)).

## Usage

```sh
gg github.com/org/repo          # shorthand (SSH by default)
gg https://github.com/org/repo  # explicit HTTPS
gg git@github.com:org/repo      # explicit SSH
```

These commands work after the zsh function is loaded. If the repo is
already cloned, gg fetches instead. Either way, it clones or fetches
first, then `cd`s into the repo and opens your configured viewer —
unless `GGNOAUTOCD` is set or you are in VSCode's terminal, in which
case git still runs and only `cd`/viewer are skipped.

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

Tab-completion works for all cloned repos under the alias prefix.

### Environment variables

| Variable | Description | Default |
|----------|-------------|---------|
| `GGROOT` | Repo tree root | `~/work` |
| `GGHTTP` | Set to `1` for HTTPS URLs | SSH |
| `GGDIRVIEWER` | Editor command (`-` to disable) | `code` if the macOS VSCode binary exists |
| `GGNOAUTOCD` | If set, skip `cd` and viewer after clone/fetch (same skip when `TERM_PROGRAM=vscode`) | off |

## License

[Apache-2.0](LICENSE)
