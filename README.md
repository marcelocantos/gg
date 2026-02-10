# gg

gg clones or fetches Git repos into a standardised directory tree
(e.g., `~/work/github.com/org/repo`), then `cd`s into the repo and
optionally opens it in an editor.

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

The installer appends a config block to `~/.zshrc`, or prints it for
manual pasting.

## Usage

```sh
gg github.com/org/repo          # shorthand (SSH by default)
gg https://github.com/org/repo  # explicit HTTPS
gg git@github.com:org/repo      # explicit SSH
```

If the repo is already cloned, gg fetches instead. Either way, it `cd`s
into the repo and opens your configured viewer.

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

Tab-completion works for all cloned repos.

### Environment variables

| Variable | Description | Default |
|----------|-------------|---------|
| `GGROOT` | Repo tree root | `~/work` |
| `GGHTTP` | Set to `1` for HTTPS URLs | SSH |
| `GGDIRVIEWER` | Editor command (`-` to disable) | `code` if installed |
| `GGNOAUTOCD` | Set to `1` to suppress auto-cd | off (also suppressed in VSCode) |

## License

[Apache-2.0](LICENSE)
