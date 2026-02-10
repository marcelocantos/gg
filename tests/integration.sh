#!/usr/bin/env zsh
set -euo pipefail

# GG_BINARY must be set by the caller (the Rust test harness)
if [[ -z "${GG_BINARY:-}" ]]; then
    echo "GG_BINARY not set" >&2
    exit 1
fi

# --- Setup ---

export GGROOT=$(mktemp -d)
export GGDIRVIEWER=-
unset GGNOAUTOCD
unset TERM_PROGRAM

trap 'rm -rf "$GGROOT"' EXIT

# Stub out compdef (not available in non-interactive zsh)
compdef() { :; }

# Create a bare repo to clone from
mkdir -p "$GGROOT/github.com/testorg"
git init --bare "$GGROOT/github.com/testorg/testrepo.git" >/dev/null 2>&1

# Use HTTPS URLs so the URL rewriting below works with shorthand input.
export GGHTTP=1

# Redirect https://github.com/ URLs to local bare repos via git config.
# This lets the shell function's `git clone <https-url>` resolve locally,
# and also makes `git ls-remote` succeed for new host dir verification.
export GIT_CONFIG_GLOBAL="$GGROOT/.gitconfig"
git config --file "$GIT_CONFIG_GLOBAL" \
    "url.file://$GGROOT/github.com/.insteadOf" "https://github.com/"

# Install gg shell function
eval "$("$GG_BINARY" -i zsh)"

pass=0
fail=0

check() {
    local name="$1"
    local ok="$2"
    if [[ "$ok" == "true" ]]; then
        echo "PASS: $name"
        (( pass++ )) || true
    else
        echo "FAIL: $name"
        (( fail++ )) || true
    fi
}

# --- Test 1: Clone ---
gg github.com/testorg/testrepo 2>/dev/null

[[ -d "$GGROOT/github.com/testorg/testrepo/.git" ]] && ok=true || ok=false
check "clone creates repo dir" "$ok"

[[ "$PWD" == "$GGROOT/github.com/testorg/testrepo" ]] && ok=true || ok=false
check "clone cd into repo" "$ok"

# --- Test 2: Fetch (same repo, already cloned) ---
cd /tmp
gg github.com/testorg/testrepo 2>/dev/null

[[ "$PWD" == "$GGROOT/github.com/testorg/testrepo" ]] && ok=true || ok=false
check "fetch cd into repo" "$ok"

# --- Test 3: Alias ---
eval "$("$GG_BINARY" -i zsh gto github.com/testorg)"
cd /tmp
gto testrepo 2>/dev/null

[[ "$PWD" == "$GGROOT/github.com/testorg/testrepo" ]] && ok=true || ok=false
check "alias cd into repo" "$ok"

# --- Test 4: GGNOAUTOCD ---
export GGNOAUTOCD=1
cd /tmp
gg github.com/testorg/testrepo 2>/dev/null

[[ "$PWD" == "/tmp" || "$PWD" == "/private/tmp" ]] && ok=true || ok=false
check "GGNOAUTOCD suppresses operations" "$ok"
unset GGNOAUTOCD

# --- Summary ---
echo ""
echo "$pass passed, $fail failed"
[[ $fail -eq 0 ]]
