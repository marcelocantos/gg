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
