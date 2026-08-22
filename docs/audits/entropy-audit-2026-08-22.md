# Entropy audit — gg

**Mode:** full (entropy + hygiene)
**Date:** 2026-08-22
**Auditor:** entropy-audit owner (this campaign)

## Executive summary

- **Snapshot:** `/Users/marcelo/work/github.com/marcelocantos/gg`
  - Branch: `master` (tracking `origin/master`)
  - HEAD: `22e0c9faf48c081fc8631337e38382d89cd04c8c` — *Add reconstructed audit log (#4)*
  - Date of audit: 2026-08-22
  - **Initial dirty state** (user-owned; not modified by this audit):

    ```
    ## master...origin/master
     M src/cli.rs
     M src/main.rs
    ?? .claude/
    ?? src/agents-guide.md
    ```

    Dirty diff is WIP `--help-agent` plus untracked `src/agents-guide.md` (`include_str!`). HEAD does not contain that path. Cargo commands below ran on the dirty tree; architecture citations for `src/main.rs` and `src/cli.rs` are HEAD.

- **Scope:** the single binary crate `gg` (Rust 2021), zsh integration generator, interactive installer, GitHub Actions CI/release, README/CLAUDE.md. ~566 LOC committed production Rust under `src/` plus 572 LOC `tests/cli.rs` and 90 LOC `tests/integration.sh`.
- **Exclusions:** `target/` (build cache, gitignored). No vendored, generated, or fixture trees in source control. Untracked `.claude/settings.local.json` and `src/agents-guide.md` are user-owned WIP, not shipped.
- **Headline mechanism:** a small planner/executor split is healthy — the Rust binary plans (`--get` key=value) and the generated zsh function executes git/cd/viewer — but the executor’s single early-return folds “don’t cd” into “don’t clone,” and the `Shell` enum / installer pretend N shells exist while only zsh is real.
- **Highest-consequence findings:**
  - **ENT-001 (P1):** `GGNOAUTOCD` and `TERM_PROGRAM=vscode` return before `git clone`/`git fetch`, contradicting README/CLAUDE.md (“suppress auto-cd”).
  - **ENT-002 (P2):** `-i bash`/`-i fish` are clap `possible values`, print “not supported yet”, and exit 0; setup writes only `.zshrc`.
  - **ENT-003 (P2):** `--dry-run` still `create_dir_all`s the org directory.
- **Unverified residue:** live GitHub clone/auth; private-repo `git ls-remote` with stderr discarded; cargo-audit/advisory DB (tool not installed); GitHub Dependabot/secret-scan settings beyond the vuln-alerts API; whether owner intends the VSCode skip-clone behaviour.

No P0 current security or data-integrity failure was observed.

## Scope and exclusions

**In scope**

| Path | Role |
|---|---|
| `src/*.rs` | Binary crate (planner, installer, shell codegen) |
| `tests/cli.rs`, `tests/integration.sh` | Integration tests / zsh journey |
| `Cargo.toml`, `Cargo.lock` | Manifest + lockfile |
| `.github/workflows/{ci,release}.yml` | CI and release matrix |
| `README.md`, `CLAUDE.md`, `docs/audit-log.md`, `LICENSE` | Docs and license |
| `.gitignore` | `/target` only |

**Named exclusions**

- `target/` — cargo build artifacts (debug + release). Not source.
- Untracked `.claude/` and `src/agents-guide.md` — user-owned; dirty `src/cli.rs` / `src/main.rs` implement `--help-agent` against that untracked file. Not treated as shipped architecture.
- No `vendor/`, no generated protobufs/bindings, no snapshot corpus.

**Languages judged:** Rust (crate), zsh (generated function + `tests/integration.sh`). No Python/Go/C++/SQL/web frontend. Shell companion (`bash.md`) read because product-adjacent shell is in scope; the shipped function is zsh, not bash 3.2.

## Commands run

All commands from repo root. Toolchain: `rustc 1.96.0 (ac68faa20 2026-05-25) (Homebrew)`, `cargo 1.96.0`, `clippy 0.1.96`, `rustfmt 1.9.0`. `cargo-nextest`, `cargo-audit`, `cargo-machete`, `cargo-outdated`, `cargo-deny` **not installed** (not added).

| Command | Path | Exit | Relevant output / limitation |
|---|---|---|---|
| `git status --porcelain=v1 -b`; `git rev-parse HEAD` | snapshot | 0 | Initial dirty state recorded above. HEAD `22e0c9fa`. |
| `git log --stat`; name-only churn | auxiliary | 0 | Highest churn: `src/zsh.rs` (9), `src/main.rs` (9), `src/getgit.rs` (8). |
| `cargo tree --locked` | auxiliary | 0 | Direct deps: anyhow 1.0.65, clap 4.5.57, home 0.5.3, regex 1.6.0. Dev: tempfile 3.10.1. **No duplicate crate versions.** First run downloaded crates (lockfile present; local `target/` already had other graphs). |
| `cargo tree --duplicate --locked` | auxiliary | 0 | Empty (no duplicates). |
| `cargo fmt -- --check` | shipped (CI) | 0 | Clean on dirty tree. |
| `cargo clippy --locked --all-targets -- -D warnings` | auxiliary (stricter than CI) | 0 | Clean. |
| `cargo clippy -- -D warnings` | shipped (CI command) | 0 | Clean. CI does **not** pass `--all-targets` or `--locked`. |
| `cargo test --locked -- --nocapture` | shipped, dirty tree, ambient env | **101** | 26 passed, **1 failed:** `shorthand_url_ssh` — expected `git@github.com:org/repo.git`, got `https://github.com/org/repo.git` because ambient `GGHTTP=1` leaked into the test. |
| `env -u GGHTTP … cargo test --locked --test cli shorthand_url_ssh` | shipped, isolated | 0 | Passes when `GGHTTP` is unset. |
| Latest CI run `29147738846` (master, 2026-07-11) | shipped remote | success | `running 27 tests` / `27 passed`; `shell_integration ... ok`; fmt check ran. Ubuntu has zsh. |
| Behavioral probes (`cargo run -- --get -n …`, `-i bash`, `-i fish`, dotted repo) | shipped binary, dirty tree | see findings | `--dry-run` created `…/github.com/org`; `github.com/org/foo.bar` → `invalid path`; bash/fish print stub and **exit 0**; `--get` with no target prints Usage and **exit 0**. `--help` on dirty tree also showed uncommitted `--help-agent`. |
| `gh issue list`; `gh api …/branches/master/protection`; vuln-alerts | auxiliary | 404 on protection and vuln-alerts | Issues #1 (bash/fish) and #2 (tab completion) open. Branch **not protected**. `hasVulnerabilityAlertsEnabled: false`. |
| `~/.claude/skills/hygiene/hygiene_check.py` | n/a | **not run** | No `hygiene.yaml`. |

**Not run (unavailable, not installed):** `cargo audit`, `cargo nextest`, jscpd / clone detector, `cargo machete`.

## Dimension vector

No prior entropy report; **Change from baseline** is n/a (first full audit). Do not collapse to a scalar.

| Dimension | State | Evidence summary | Change from baseline |
|---|---|---|---|
| Architecture topology | healthy | Single binary; planner (`getgit`) decoupled from `Cli`; zsh executor consumes a four-key protocol; no import cycles; bash/fish are explicit stubs rather than a hidden second path. | n/a |
| Redundancy / sources of truth | concern | GGNOAUTOCD/VSCode contract disagrees across README, CLAUDE.md, `zsh.rs`, and the integration test’s name vs assertion. “Installed” is a substring, not the `# --- gg ---` markers. VSCode default-viewer path is copied three times. | n/a |
| Change amplification | concern | A new shell must touch `Shell` enum, `main` match, a new module, `setup.rs` (zsh-hardcoded), tests, and docs. Protocol keys are stringly duplicated in Rust writer and zsh parser. | n/a |
| Local code quality | healthy | `cargo fmt` and `clippy -D warnings` (including `--all-targets`) clean. Linear functions, `shell::escape` on generated zsh, `LazyLock` regex. `Box<dyn Fn>` squiggler is needless but local. | n/a |
| Correctness / verification | concern | 27 integration tests + zsh journey are real and green in CI. Local `shorthand_url_ssh` fails when `GGHTTP=1`. Dry-run mutates the filesystem. Dotted repo names rejected. GGNOAUTOCD oracle does not assert clone still happens. | n/a |
| Security / dependencies | concern | Post-2026-02 shell-injection fix is sound (quoted zsh, no `eval` of planner output). No cargo-audit/Dependabot; GitHub vuln alerts off; lockfile pins regex 1.6.0 / anyhow 1.0.65 / home 0.5.3 from 2022. Setup writes unquoted user strings into `.zshrc`. | n/a |
| Build / release / operations | concern | CI: clippy + test + fmt on `ubuntu-latest` (green). No `Swatinem/rust-cache`, no `RUSTFLAGS=-D warnings` on rustc, toolchain floats on `dtolnay/rust-toolchain@stable`. Release matrix: Linux musl, macOS arm (tested), macOS x86_64 cross-compile **tests skipped**. Branch unprotected. | n/a |
| Documentation / governance | concern | README + CLAUDE.md + reconstructed audit log + Apache-2.0 exist and mostly match the crate. GGNOAUTOCD/VSCode wording is wrong. No CODEOWNERS, SECURITY.md, CONTRIBUTING, `hygiene.yaml`, `bullseye.yaml`, branch protection. Single-maintainer bus factor. | n/a |

## Observed architecture

`gg` is a **planner binary** plus **generated shell executor**.

```
user shell
  └─ gg()  [emitted by `gg -i zsh`, lives in ~/.zshrc]
        ├─ $exepath --get "$@"     → stdout key=value
        ├─ git clone | git fetch   → uses action/git_dir/git_url
        ├─ cd $cd_dir
        └─ $viewer $cd_dir

gg binary (src/main.rs)
  ├─ no args        → setup::setup (interactive ~/.zshrc installer)
  ├─ -i <shell>     → zsh | bash | fish codegen
  └─ --get <target> → getgit::getgit (plan only; does not clone)
```

**Module direction (observed, no cycles)**

- `main` → `cli`, `env`, `getgit`, `setup`, `zsh`, `bash`, `fish`
- `getgit` → `env`
- `setup` → `env`
- `zsh` → `shell`
- `bash` / `fish` → (none; stubs)

`getgit` was decoupled from `Cli` in `5ccbe28` (2026-02-10). That boundary is still clean: `getgit(path, prefix, dry_run, ggroot)`.

**Public surfaces**

- CLI: clap derive (`version`, `--dry-run`, `-i <zsh\|bash\|fish>`, hidden `--get` / `--prefix`).
- Wire protocol (unversioned, stdout): `action=clone|fetch`, `git_dir=…`, `git_url=…` (clone only), `cd_dir=…`.
- Env: `GGROOT`, `GGHTTP`, `GGDIRVIEWER`, `GGNOAUTOCD`, plus read of `TERM_PROGRAM` in the zsh function.
- Installer: appends a marked block to `~/.zshrc`.
- Git: `git ls-remote` (new host dirs), `git clone --recurse-submodules`, `git fetch --all --prune --jobs=10 --recurse-submodules=yes`.

**Domain ownership**

| Concept | Owner |
|---|---|
| URL parse, host verify, plan | `getgit.rs` |
| Path/env/home | `env.rs` |
| argv | `cli.rs` |
| Interactive install | `setup.rs` (zshrc only) |
| Executor + completion | `zsh.rs` |
| Quote safety for generated shell | `shell.rs` |

**Declared vs observed rules**

| Rule | Status |
|---|---|
| Binary plans; shell executes git/cd/viewer | Declared (CLAUDE.md, commit `003acf4`) and observed |
| `--get` / `--prefix` hidden from `--help` | Declared and observed |
| Shorthand defaults to SSH; `GGHTTP` → HTTPS | Declared and observed (when env is not leaked into tests) |
| New host dirs verified with `git ls-remote` | Declared and observed |
| zsh implemented; bash/fish stubs (issue #1) | Declared in CLAUDE.md **and** contradicted by clap `possible values` + exit 0 |
| `GGNOAUTOCD` / VSCode suppress auto-cd | Declared in README/CLAUDE.md; **contradicted** by `zsh.rs` early return |
| `--dry-run` does nothing | Declared in `--help`; **contradicted** by `create_dir_all` |
| Installer block markers `# --- gg ---` | Observed write; **not** used for “already installed” |

**Inferred intent (not documented as a rule)**

- `git_dir` means org directory on clone and repo directory on fetch — the zsh function’s `git -C` matches that. Easy to misuse by a second consumer of the protocol.
- Host/org/repo is a three-level tree; GitLab nested groups would be parsed as `org/repo/tail` (cd-into-subdir feature, not nested-group clone).

**Unknown owner intent**

- Should `TERM_PROGRAM=vscode` skip clone, or only cd/viewer?
- Are bash/fish meant to stay visible stubs until issue #1, or hidden?
- Is dotted-repo rejection (`[^/\.]+`) leftover from `.git` suffix handling or a deliberate constraint?

## Findings

### ENT-001: VSCode / GGNOAUTOCD early-return skips clone and fetch

- **Priority:** P1
- **Dimensions:** Correctness / verification; Redundancy / sources of truth; Documentation / governance
- **Status:** observed fact (code + docs + history); inference that skip-clone is unintended given the env var name and README
- **Evidence:**
  - Executor returns before git:

    ```54:58:src/zsh.rs
        if [[ "${{TERM_PROGRAM:-}}" == "vscode" || -n "${{GGNOAUTOCD:-}}" ]]; then return; fi
        case "$action" in
            clone) git -C "$git_dir" clone --recurse-submodules "$git_url" || return ;;
            fetch) git -C "$git_dir" fetch --all --prune --jobs=10 --recurse-submodules=yes || return ;;
        esac
    ```

    (`zsh.rs` emits this via `write!`; those are the generated function lines.)
  - Docs say only auto-cd: `README.md:76`, `CLAUDE.md:57` — “disable auto-cd after clone/fetch (also auto-suppressed in VSCode)”.
  - `--get` never clones; it only prints a plan (`getgit.rs:117-129`). Skipping the executor therefore skips the only git clone/fetch.
  - History: pre-split `getgit` gated **git+cd+viewer** on `TERM_PROGRAM != vscode && !ggnoautocd()` (`a17fb55`). `003acf4` moved the same compound gate into zsh. The name has always been “no auto cd”; the implementation has always skipped git as well.
  - `tests/integration.sh:78-84` is named “GGNOAUTOCD suppresses operations” but **only asserts `PWD` stayed `/tmp`**. If clone/fetch still ran and cd were skipped, the test would still pass. It does not lock skip-clone.
- **Mechanism:** one boolean gate is reused for three product actions (git, cd, viewer). Docs and the env var name describe one of them. Default viewer is VSCode (`setup.rs:40-62`, `zsh.rs:60-63`), so the integrated terminal (`TERM_PROGRAM=vscode`) is a likely daily path: `gg org/repo` prints the pretty path, creates org dirs, and does not clone.
- **Blast radius:** every zsh user in VSCode’s terminal; anyone with `GGNOAUTOCD=1` who still expects fetch/clone. Silent no-op (exit 0 from the function after a successful `--get`).
- **Counterevidence checked:** integration test unsets `TERM_PROGRAM` so the happy-path clone test does not see this. CI green does not exercise the vscode branch. No comment in `zsh.rs` explaining skip-clone. Issue list does not mention it.
- **Smallest coherent remediation:** split the gate: always run clone/fetch; skip `cd` and viewer when `GGNOAUTOCD` or `TERM_PROGRAM=vscode`. Or, if skip-clone is intended, rename the var, fix README/CLAUDE.md, and assert in `integration.sh` that `.git` is not updated when the flag is set. Do not keep both stories.
- **Verification:** zsh journey: with `TERM_PROGRAM=vscode` and a missing repo dir, `gg host/org/repo` creates `repo/.git`. Fail the test if the dir is absent. Second case: `GGNOAUTOCD=1` still clones/fetches; `PWD` unchanged.
- **Ratchet candidate:** extend `tests/integration.sh` with those two assertions (shipped-path journey, not a doc grep).

### ENT-002: CLI advertises bash/fish; stubs exit 0; installer is zsh-only

- **Priority:** P2
- **Dimensions:** Architecture topology (extension seam); Change amplification; Correctness / verification; Documentation / governance
- **Status:** observed fact
- **Evidence:**
  - `src/cli.rs:38-43` (HEAD) `Shell { Zsh, Bash, Fish }` via `value_enum` → `--help` lists `possible values: zsh, bash, fish`.
  - `src/main.rs:61-65` (HEAD) dispatches to `bash()` / `fish()`.
  - `src/bash.rs:11-12` and `src/fish.rs:11-12`: `eprintln!("… not supported yet"); Ok(())` — probe: both **exit 0**.
  - `setup.rs:16` detects install via `content.contains("-i zsh)\"")`; `setup.rs:78` always emits `eval "$({exepath} -i zsh)"`; writes `home.join(".zshrc")` only (`setup.rs:11`).
  - Tracked as https://github.com/marcelocantos/gg/issues/1 (open). CLAUDE.md:45 names the stubs.
- **Mechanism:** clap’s enum is a published support matrix. A user who follows `--help` and runs `eval "$(gg -i bash)"` gets an empty integration and a successful status, so `.bashrc` “installs” a no-op. Completing issue #1 also requires rewriting `setup.rs`, not just `bash.rs`/`fish.rs` — the installer does not take `Shell` as a parameter.
- **Blast radius:** any non-zsh user following `--help` or README “Setup”; future bash/fish work fans out through installer + tests + docs.
- **Counterevidence checked:** README usage examples are zsh-only. Issue #1 honestly tracks the gap. Stubs are not dead code; `main` calls them. No test covers `-i bash` / `-i fish`.
- **Smallest coherent remediation:** until implemented, drop `Bash`/`Fish` from `value_enum` **or** return `Err` (exit 1) from the stubs. Parameterize setup by shell (rc path + `-i` argument) as part of issue #1, not after.
- **Verification:** `gg -i bash` exits non-zero (or is rejected by clap) in `tests/cli.rs`. When bash ships, a `tests/integration-bash.sh` journey analogous to `integration.sh`.
- **Ratchet candidate:** CLI test `install_unknown_or_stub_shell_fails`; later, hygiene item `correctness.bash-integration` once that journey exists.

### ENT-003: `--dry-run` still creates org directories

- **Priority:** P2
- **Dimensions:** Correctness / verification
- **Status:** observed fact
- **Evidence:**
  - `getgit.rs:114-117`: `create_dir_all(orgroot)` is **outside** `if !dry_run`.
  - Probe: `GGROOT=$tmp` with pre-existing `github.com/` host dir, `gg --get -n github.com/org/newrepo` exited 0 and created `$tmp/github.com/org`.
  - `--help`: “Print actions to perform but do nothing”.
  - `tests/cli.rs:178-191` `dry_run_no_stdout` only asserts empty stdout and that stderr contains `github.com`. Host/org dirs are pre-created by `setup_ggroot`.
- **Mechanism:** dry-run gates the key=value protocol, not the filesystem. Host verification (`git ls-remote`, `getgit.rs:80-103`) still runs (good); `create_dir_all` should not.
- **Blast radius:** anyone using `-n` to preview a new org path; leftover empty org dirs under `$GGROOT`.
- **Counterevidence checked:** dry-run still prints the pretty path on stderr (intentional). Host dir is not created by this call when it already exists; for a missing host dir, ls-remote runs first and `create_dir_all` would also create the host directory.
- **Smallest coherent remediation:** move `create_dir_all` under `if !dry_run`. Optionally skip ls-remote on dry-run or keep it (verification-only is consistent with “print actions”).
- **Verification:** test: empty ggroot+host, `--get -n host/org/repo` → org path does not exist; stdout empty.
- **Ratchet candidate:** that test in `tests/cli.rs`.

### ENT-004: URL regex rejects repository names containing dots

- **Priority:** P2
- **Dimensions:** Correctness / verification
- **Status:** observed fact
- **Evidence:**
  - `getgit.rs:22-24`: `(?P<repo>[^/\.]+)` then optional `(tail|/.*) | \.git`.
  - Probe: `gg --get github.com/org/foo.bar` → `gg: invalid path: github.com/org/foo.bar` (exit 1).
  - Real repos with dots exist (`vercel/next.js`, many `*.nvim` plugins). Org names allow dots (`[^/]+`).
- **Mechanism:** `.git` suffix stripping was encoded by forbidding dots in the repo capture, so `repo.git` matches `repo` + `.git`, but `next.js` cannot match.
- **Blast radius:** shorthand, HTTPS, and SSH forms; aliases that prefix into such names. Users fall back to hand-cloning, breaking the directory convention.
- **Counterevidence checked:** `dotgit_suffix_stripped` (`tests/cli.rs:115-123`) covers `.git`, not interior dots. No README claim that repo names are [A-Za-z0-9_-] only.
- **Smallest coherent remediation:** capture repo as `[^/]+?` and strip a trailing `.git` explicitly (or `(?P<repo>[^/]+?)(?:\.git)?` with tail still optional). Keep the three-component host/org/repo model.
- **Verification:** `tests/cli.rs` cases: `github.com/org/next.js` → `cd_dir` ends with `/next.js`; `…/next.js.git` → same, `git_url` ends with `next.js.git`.
- **Ratchet candidate:** those two CLI tests.

### ENT-005: CLI tests inherit `GGHTTP` (and are not hermetic)

- **Priority:** P2
- **Dimensions:** Correctness / verification
- **Status:** observed fact
- **Evidence:**
  - `tests/cli.rs:46-52` `run_gg_env` sets `GGROOT` and extra env, but does **not** `env_remove("GGHTTP")`.
  - `tests/cli.rs:71-78` `shorthand_url_ssh` asserts `git_url == git@github.com:org/repo.git`.
  - This audit’s `cargo test --locked` failed that test with ambient `GGHTTP=1` (left=`https://…`). `env -u GGHTTP` rerun passed.
  - Latest GitHub CI (run `29147738846`) passed 27/27 — ubuntu runners do not set `GGHTTP`.
  - Setup tests **do** remove `GGHTTP` (`tests/cli.rs:279`).
- **Mechanism:** the product reads process env for protocol selection (`getgit.rs:72-76`). Tests that pin SSH inherit the developer’s own `GGHTTP`, so the shipped-path oracle is green in CI and red on a machine that uses gg with HTTPS (this repo’s default for the owner).
- **Blast radius:** local false failures; more importantly, a future regression in SSH default can be hidden if a developer’s env is HTTPS and they only watch CI — wait, that’s the opposite: local fails, CI passes. The gap is **developers who export GGHTTP cannot trust `cargo test`**, and CI cannot catch env-leak bugs in the other direction (`shorthand_url_https_with_gghttp` sets the var explicitly and is fine).
- **Counterevidence checked:** `run_setup` hermeticizes HOME and GG\*. `shell_integration` sets `GGHTTP=1` itself (`tests/integration.sh:27`). Only `--get` URL tests are leaky.
- **Smallest coherent remediation:** in `run_gg_env`, `env_remove("GGHTTP")` (and `GGDIRVIEWER` / `GGNOAUTOCD` / `TERM_PROGRAM`) before applying the explicit `env` slice.
- **Verification:** `GGHTTP=1 cargo test --test cli shorthand_url_ssh` must pass.
- **Ratchet candidate:** that command in CI (`env GGHTTP=1 cargo test`) or the `env_remove` plus a comment. Prefer the code fix; CI `env GGHTTP=1` is a good extra ratchet.

### ENT-006: Installer is non-reentrant and interpolates unquoted values into `.zshrc`

- **Priority:** P2
- **Dimensions:** Local code quality; Security / dependencies; Change amplification
- **Status:** observed fact
- **Evidence:**
  - Already-installed: `setup.rs:14-18` `content.contains("-i zsh)\"")` then return. Markers written at `setup.rs:103` (`# --- gg ---` / `# --- end gg ---`) are unused for detection.
  - Unquoted exports: `setup.rs:30` `export GGROOT={ggroot}`; `setup.rs:56,73` `export GGDIRVIEWER={viewer}`; `setup.rs:78,97` `eval "$({exepath_str} -i zsh …)"` with raw `exepath` / alias tokens.
  - Tests cover append/print/no-op/markers (`tests/cli.rs:330-536`) but all paths use space-free values (`~/repos`, `vim`, `subl`).
- **Mechanism:** two notions of “installed” (eval substring vs markers) can drift; re-running `gg` cannot update protocol/viewer/aliases. Paths with spaces (`GGROOT`, `current_exe()` under some install prefixes) produce a broken rc line. User-supplied viewer/alias strings are copied into the rc file without `shell::escape` (the crate already has that helper for zsh codegen). This is self-targeting, not a remote injection, but it is the same class of bug `003acf4` fixed on the executor side.
- **Blast radius:** every installer user; first-run only; custom `GGROOT` with spaces; cargo/install paths with spaces.
- **Counterevidence checked:** `shell::escape` is used in `zsh.rs` for exepath/ggroot/prefix. Setup tests prove markers are written. False-positive detection would require the substring `-i zsh)"` elsewhere in `.zshrc` (possible in comments).
- **Smallest coherent remediation:** detect/replace the marked block; quote with `shell::escape` (or emit single-quoted assignments). Offer “already installed → reprint/replace”.
- **Verification:** setup test with `GGROOT=/tmp/my work` yields `export GGROOT='/tmp/my work'` (or equivalent) inside the markers; second run replaces the block rather than no-op, or a dedicated `--reinstall` path tested.
- **Ratchet candidate:** that setup test; optionally parse the block in a small snapshot assertion.

### ENT-007: No dependency scanning; lockfile pins 2022-era crates; GitHub vuln alerts off

- **Priority:** P2
- **Dimensions:** Security / dependencies; Build / release / operations
- **Status:** observed fact (tooling absence); needs verification (whether regex 1.6 / anyhow 1.0.65 have advisories)
- **Evidence:**
  - `Cargo.toml` pins `anyhow = "1.0.65"`, `home = "0.5.3"`, `regex = "1.6.0"` (caret, but **Cargo.lock** locks those exact 2022 versions).
  - `cargo tree --locked`: regex 1.6.0 → aho-corasick 0.7.19, regex-syntax 0.6.27. clap pulled forward to 4.5.57 (lockfile is not uniformly frozen in calendar time).
  - `gh` GraphQL: `hasVulnerabilityAlertsEnabled: false`, `isSecurityPolicyEnabled: false`. No `.github/dependabot.yml`. No `cargo-audit` / `cargo-deny` in CI (`.github/workflows/ci.yml`).
  - `cargo-audit` not installed locally; not added.
  - `home` 0.5.x README: `std::env::home_dir` restored in Rust 1.87; this toolchain is 1.96 — the crate is optional.
- **Mechanism:** a binary that shells out to git and writes `~/.zshrc` still has a Rust supply chain. Without Dependabot, cargo-audit, or GitHub advisories, known CVEs in regex/aho-corasick would not fail CI. Residue: this audit did not run an advisory DB.
- **Blast radius:** all release artifacts (`release.yml` musl + two macOS targets) built from this lockfile.
- **Counterevidence checked:** lockfile **is** committed (correct for a binary). No duplicate versions. CI uses `--` not `--locked` (separate drift: CI could resolve differently than `Cargo.lock` if lock is stale vs `Cargo.toml` — currently they match).
- **Smallest coherent remediation:** enable GitHub vulnerability alerts; add `cargo audit` (or `cargo deny`) to CI; `cargo update` of regex/anyhow/home (or replace `home` with `std::env::home_dir`) as a deliberate commit after audit output. Add `--locked` to CI cargo invocations.
- **Verification:** CI job that fails on RUSTSEC advisories; `cargo tree -i regex` shows a current 1.x after update.
- **Ratchet candidate:** CI step `cargo audit` + `--locked` on clippy/test. Hygiene item once `hygiene.yaml` exists.

### ENT-008: Default VSCode path is copied three times

- **Priority:** P3
- **Dimensions:** Redundancy / sources of truth; Change amplification
- **Status:** observed fact
- **Evidence:** identical path string in `setup.rs:164` (`vscode_installed`), `zsh.rs:62` (runtime default viewer), `tests/cli.rs:216` (`has_vscode`). `env.rs` used to own `ggdirviewer()` including this path; `003acf4` deleted it and split the copies.
- **Mechanism:** a VSCode install-location change (or wanting `code` on `PATH` on Linux/Windows) requires three edits; Linux CI never takes the true branch of `has_vscode()` / `vscode_installed()`.
- **Blast radius:** default-viewer behaviour on macOS; setup tests already branch on `has_vscode()`.
- **Counterevidence checked:** runtime default in zsh is independent of setup on purpose (setup omits `GGDIRVIEWER` when accepting VSCode). Linux users without that path simply get no viewer unless they set `GGDIRVIEWER` — documented-ish (“`code` if installed”).
- **Smallest coherent remediation:** one constant used by setup and tests; zsh generator interpolates the same escaped path. Optionally `command -v code` as a portable fallback.
- **Verification:** grep for the path string has a single owner (plus the generated zsh output test if one is added).
- **Ratchet candidate:** `rg` count in CI is fragile; a unit/integration assert on `gg -i zsh` output containing one agreed path is better.

### ENT-009: CI and release gates are thin relative to the Rust companion baseline

- **Priority:** P3
- **Dimensions:** Build / release / operations
- **Status:** observed fact
- **Evidence:**
  - `.github/workflows/ci.yml`: `ubuntu-latest` only; `dtolnay/rust-toolchain@stable` (unpinned); `cargo clippy -- -D warnings` then `cargo test` then `cargo fmt -- --check`; **no** `Swatinem/rust-cache@v2` (`rust.md` default for owned Rust CI); **no** `--locked`; **no** `--all-targets`.
  - `release.yml`: `run_tests: false` for `x86_64-apple-darwin` (cross-compile from `macos-14`). Arm macOS and Linux musl do run tests. No SBOM, no signing, no `cargo audit`.
  - `gh api repos/marcelocantos/gg/branches/master/protection` → 404 Branch not protected.
  - CI duration ~22–32s (run `29147738846` 32s) — cache is not a latency crisis.
- **Mechanism:** floating `stable` plus unlocked cargo can change clippy/fmt/dependency resolution without a repo commit. macOS x86_64 artifacts ship untested on that target. Unprotected `master` allows force-push / unreviewed merge (current practice is PRs: #3, #4).
- **Blast radius:** CI flakes vs local 1.96; release tarballs for Intel Macs.
- **Counterevidence checked:** clippy `--all-targets` was run in this audit and passed, so CI’s weaker clippy is not currently hiding failures. Release skip for x86_64 tests is explained by the runner arch (`docs/audit-log.md` 2026-02-11).
- **Smallest coherent remediation:** add rust-cache; `cargo * --locked`; pin via `rust-toolchain.toml` (or dtolnay with a version); run fmt first. Leave x86_64 test skip documented or add an Intel runner later.
- **Verification:** workflow file contains `Swatinem/rust-cache` and `--locked`; CI still green.
- **Ratchet candidate:** those workflow lines (hygiene `ci_step` / `ci_job` once declared).

### ENT-010: `--get` without a target prints usage and exits 0

- **Priority:** P3
- **Dimensions:** Correctness / verification
- **Status:** observed fact
- **Evidence:** `src/main.rs:49-52` (HEAD) `None => { eprintln!("Usage: gg <path>"); Ok(()) }`. Probe: `gg --get` exit 0. Internal flag, called from zsh as `gg --get "$@"`.
- **Mechanism:** a mistaken alias or empty `"$@"` is indistinguishable from success; the zsh function then hits `[ -z "$output" ] && return` (`zsh.rs:44`) and silently does nothing.
- **Blast radius:** empty `gg` with `--get` (the interactive installer is the no-arg path *without* `--get`). Low.
- **Counterevidence checked:** zsh wrapper passes `"$@"`; users without the wrapper who run `gg --get` are already on an internal API.
- **Smallest coherent remediation:** `bail!("missing path")` so exit 1.
- **Verification:** `tests/cli.rs` `gg --get` `success == false`.
- **Ratchet candidate:** that test.

## Redundancy and competing-source-of-truth inventory

| Fact | Copies | Can they disagree? | Disposition |
|---|---|---|---|
| GGNOAUTOCD / VSCode meaning | README:76, CLAUDE.md:57, `zsh.rs:54`, test name vs assertion in `integration.sh:78-84` | **Yes — they do** (ENT-001) | Converge on one contract |
| “gg is installed” | substring `-i zsh)"` vs `# --- gg ---` markers | Yes (ENT-006) | Detect via markers |
| Default VSCode CLI path | `setup.rs`, `zsh.rs`, `tests/cli.rs` | Yes if one is edited (ENT-008) | One constant |
| Supported shells | clap `Shell` enum, stubs, setup zsh-only, issue #1, CLAUDE.md | **Yes — they do** (ENT-002) | Hide or implement |
| Dry-run contract | `--help` “do nothing” vs `create_dir_all` | **Yes** (ENT-003) | Gate FS writes |
| SSH vs HTTPS default | README, `getgit.rs`, tests that forget to unset `GGHTTP` | Yes in test vs product env (ENT-005) | Hermetic tests |
| `git_dir` meaning | clone=org dir, fetch=repo dir (`getgit.rs:120-126`, `zsh.rs:56-57`) | Would disagree if a second parser assumed “always repo root” | Document in protocol; keep one consumer |
| Pretty `~` paths | `env::squiggler` vs `setup::squiggle_path` | Display-only; slightly different (`Path::new("~").join` vs format `~/`) | Optional merge; not a defect |
| bash.rs vs fish.rs | identical stub bodies | Only until one is implemented | Accept until issue #1 |
| Test count “27” | CLAUDE.md:24 vs `tests/cli.rs` | Currently match (CI: 27 passed) | Keep honest on next test add |

Deliberate duplication: zsh alias function vs `_gg` completion is two generated snippets with different `-W` roots — not the same fact.

## Healthy structure worth retaining

- **Planner/executor split** (`003acf4`): binary prints `action/git_dir/git_url/cd_dir`; zsh quotes `"$var"` and runs git. Shell injection in the executor path is fixed *by construction* (`src/shell.rs:1-4` used for codegen of exepath/ggroot/prefix). Do not revert to `eval $(gg --get)`.
- **`getgit` does not depend on `Cli`** (`5ccbe28`). Prefix and dry-run are data.
- **Hidden `--get` / `--prefix`:** user `--help` stays an installer/alias surface; the wrapper uses the internal API.
- **Installer tests override `HOME`** (`tests/cli.rs:275-304`) and do not touch the real `~/.zshrc`. Marker round-trip is tested (`setup_block_markers`).
- **Zsh journey** (`tests/integration.sh` via `tests/cli.rs:541-572`): whole binary + generated function + real `git` against a local bare repo with `url.*.insteadOf`. Clone, fetch, alias, cwd. CI actually runs it (`shell_integration ... ok` on ubuntu). This is the owner-visible product path, hermeticized — keep it, tighten ENT-001 on it.
- **Host verification before creating a new host dir** (`getgit.rs:80-103`) prevents typo-hosts from polluting `$GGROOT`.
- **`cargo fmt` + `clippy -D warnings` + tests in CI**; Apache-2.0 LICENSE; committed `Cargo.lock`; reconstructed `docs/audit-log.md`.
- **Module layout matches CLAUDE.md.** Docs-to-code for files/roles is accurate except GGNOAUTOCD and bash/fish UX.

## Hygiene posture

**Hygiene posture not declared.** No `hygiene.yaml` at repo root. The validator was not run (skill: do not initialize unless asked). No per-dimension held tiers, floors, or drift report.

Overlap with entropy findings that are machine-enforceable later (do **not** write them into a yaml in this audit):

| Future hygiene id (illustrative) | Would decide |
|---|---|
| `correctness.cli-tests` | `ci_job: ci.yml#test` already true |
| `correctness.zsh-journey` | same job + `file: tests/integration.sh` |
| `quality.fmt` / `quality.clippy` | existing CI steps |
| `deps.advisory-scan` | missing (ENT-007) |
| `build.locked` | missing `--locked` (ENT-007/009) |
| `security.vuln-alerts` | GitHub setting currently false |

Entropy findings ENT-001–ENT-006 are **not** hygiene drift; they are undeclared product properties.

## Oracle coverage and residue

| Property | Decided by | Notes |
|---|---|---|
| Shorthand / SSH / HTTPS / `.git` / tail parse | Shipped: `tests/cli.rs` `--get` | Fails locally if `GGHTTP` leaks (ENT-005). Dotted repos uncovered (ENT-004). |
| Clone vs fetch plan | Shipped: dir existence tests | Does not run `git clone`; only the plan. |
| Dry-run has no stdout | Shipped: `dry_run_no_stdout` | Does not decide FS non-mutation (ENT-003). |
| Invalid URL / missing host | Shipped: `invalid_url`, `missing_host_dir` | `missing_host_dir` hits real `git ls-remote` failure. |
| Installer flows | Shipped: 14 setup tests with `HOME` temp | Space quoting and reinstall uncovered (ENT-006). |
| Zsh clone, fetch, alias, cd | Shipped journey: `integration.sh` | Local `file://` rewrite, not live GitHub. GGNOAUTOCD only asserts cwd (ENT-001). |
| `TERM_PROGRAM=vscode` | **nothing** | No test sets it except unsetting it. |
| bash / fish integration | **nothing** (issue #1) | Stubs exit 0 (ENT-002). |
| Live `github.com` clone + SSH/HTTPS auth | **nothing** | Accepted cost if hermetic journey stays; owner-visible GitHub auth is residue. |
| Private repo + new host (`ls-remote` stderr discarded) | **nothing** | May fail closed or hang without a prompt (`getgit.rs:86-90`). |
| Release artifacts (musl, two macOS) | Auxiliary: `release.yml` on tag | x86_64 macOS untested (ENT-009). Not re-run here. |
| Clippy / fmt | Shipped CI + this audit | `--all-targets` only in this audit (passed). |
| Advisories / SBOM / signing | **nothing** | ENT-007. |
| `cargo test` unit tests in `src/` | n/a | Binary-only crate; 0 unit tests (CI log). Properties live in `tests/`. |

**Failed / skipped checks this audit**

- `cargo test --locked` failed `shorthand_url_ssh` under `GGHTTP=1`; isolated rerun passed.
- `cargo-audit` / nextest / jscpd not available.
- Hygiene validator not applicable.
- Did not `git stash` or rebuild at HEAD-only; dirty `--help-agent` WIP did not change the 27 tests except help text on `cargo run -- --help`.

**Owner residue (intent / taste only)**

1. Should VSCode / `GGNOAUTOCD` skip clone+fetch, or only cd/viewer?
2. Stay zsh-only in the CLI surface until bash/fish exist, or keep visible stubs?
3. Are GitLab nested groups in scope, or is host/org/repo the forever model?
4. Pin `rust-toolchain.toml` vs float on `stable`?
5. Onboard `hygiene.yaml` / Dependabot / branch protection for this small binary?

## Remediation sequence

1. **Oracle seam:** `env_remove("GGHTTP")` (and siblings) in `run_gg_env` (ENT-005). Optionally `GGHTTP=1 cargo test` in CI. Without this, local and CI disagree.
2. **Contract for GGNOAUTOCD/VSCode (ENT-001):** pick skip-cd-only vs skip-all; implement; assert clone on the zsh journey. Fix README/CLAUDE.md in the same commit.
3. **Dry-run FS (ENT-003) and dotted repos (ENT-004):** one-line gate + regex fix with tests. Independent of (2).
4. **CLI honesty (ENT-002, ENT-010):** stub shells exit 1 (or leave the enum); `--get` missing target exits 1. Full bash/fish remains issue #1 — do not implement in the same breath as the fail-loud fix.
5. **Installer (ENT-006):** quote with `shell::escape`; detect the marked block. Reinstall can wait if quoting lands first.
6. **Supply chain (ENT-007, ENT-009):** `--locked`, rust-cache, enable vuln alerts, `cargo audit`, then a deliberate `cargo update` of regex/anyhow/home. Do not silently expand `Cargo.lock` inside an unrelated fix.
7. **Ratchet** the new tests in CI (already runs `cargo test`). When requested, declare `hygiene.yaml` against those jobs — not as part of this audit.
8. **Re-run** this audit against the same finding IDs.

Do not start with a workspace split, a protocol rewrite, or implementing bash/fish as the entropy fix. The planner/executor boundary should stay.

## Comparison appendix

Not applicable — first full audit; no prior `docs/audits/entropy-audit-*.md`.
