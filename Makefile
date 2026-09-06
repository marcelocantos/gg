.PHONY: gate hooks

# The delivery gate: the oracles CI already ran, on the shipped path.
# scripts/hooks/pre-push executes it and refuses the push when it is red, and
# CI runs the same recipe on master, on v* tags and on pull requests.
# gg reads its behaviour out of the environment (GGHTTP picks the clone
# protocol, GGDIRVIEWER the post-clone viewer, GGROOT the repo root), so tests
# run under a deliberately hostile env: every one of those is set to the wrong
# value. A test that leaks the ambient environment into the gg it spawns fails
# here instead of passing on CI and failing on the owner's laptop.
GATE_HOSTILE_ENV = \
	GGHTTP=1 \
	GGROOT=/nonexistent-gate-ggroot \
	GGDIRVIEWER=/nonexistent-gate-viewer \
	GGNOAUTOCD=1 \
	TERM_PROGRAM=gate

gate:
	cargo fmt -- --check
	cargo clippy --all-targets -- -D warnings
	env $(GATE_HOSTILE_ENV) cargo test

# Once per clone: make git run scripts/hooks instead of .git/hooks. A relative
# path resolves against the worktree the push runs from, so one setting covers
# every `git worktree`.
hooks:
	git config core.hooksPath scripts/hooks
	@echo "core.hooksPath -> scripts/hooks"
