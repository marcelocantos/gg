# Audit Log

Chronological record of audits, releases, documentation passes, and other
maintenance activities. Append-only — newest entries at the bottom.

## 2022-09-18 — open-sourcing (reconstructed)

- **Commit**: `b48f32c`
- **Outcome**: Initial open-source release. Added README.md and LICENSE (Apache 2.0). Tagged as v0.1. Release workflow added same day.

## 2026-02-10 — major-refactor (reconstructed)

- **Commit**: `a17fb55`
- **Outcome**: Significant multi-commit refactor session: fixed shell injection and panics, decoupled `getgit` from `Cli` struct, removed `Box<dyn Write>` plumbing, improved CLI discoverability, improved error messages with `gg:` prefix, fixed Clippy warnings, implemented GGHTTP/SSH URLs and smart host dir creation, added integration tests, replaced static help with interactive installer, and upgraded clap v3→v4 with error handling fixes.

## 2026-02-11 — documentation-pass (reconstructed)

- **Commit**: `8628580`
- **Outcome**: Updated CLAUDE.md to reflect current architecture after the major refactor.

## 2026-02-11 — ci-setup (reconstructed)

- **Commit**: `d444d6c`
- **Outcome**: Added CI and release workflows. Updated README to match. Fixed Rust toolchain action, formatting, and macOS x86_64 cross-compilation across several follow-up commits same day.

## 2026-02-11 — release/v0.1.0 (reconstructed)

- **Commit**: `bafcc70`
- **Outcome**: Tagged v0.1.0 after CI and release workflow fixes landed. First release with working automated cross-compilation for macOS x86_64.

## 2026-09-06 — documentation pass (burst 2026-09)

- **Commit**: branch `burst/docs`
- **Outcome**: Replaced the standalone `CLAUDE.md` with a canonical
  `AGENTS.md` plus a thin `CLAUDE.md` adapter. Corrected the `GGNOAUTOCD`
  / VS Code description (the shell function returns before clone and
  fetch, not just before `cd`), the `GGDIRVIEWER` default (VS Code's
  bundled binary at a fixed path, not `code` on `PATH`), and `GGHTTP`
  (any non-empty value). README gained a flags table, the release asset
  names, the zsh-only note, and a limitations section (dotted repo
  names, `--dry-run` still creating the org directory). Every claim was
  checked against `src/` and `tests/`.

## 2026-09-06 — documentation pass, second sweep (burst 2026-09)

- **Commit**: branch `burst/docs`
- **Outcome**: Re-audited every behavioural claim against `src/` and
  `tests/`. Corrected the `GGDIRVIEWER` target (`cd_dir`, which includes a
  trailing path, not the repo root) and its detection (an `[ -x ]` test on
  the bundled binary path, not the `.app` bundle's existence); `--dry-run`
  (the `👉` line goes to stderr on every run, not only dry runs); the
  bash/fish stubs (stderr, exit 0, empty stdout); the `-i CMD` alias form
  without a prefix (`missing prefix argument`); and the installer's
  `-i zsh)"` substring detection plus its unquoted interpolation. Added the
  dispatch gotcha — the raw binary without `--get` ignores `-n` and its
  argument and runs the installer. Fixed the AGENTS.md test descriptions
  (`shell_integration` also asserts `.git` and skips silently without zsh;
  the setup tests override `HOME` rather than clearing it). Flagged the
  2026-08-22 entropy audit as a dated snapshot.
