.PHONY: gate hooks

# The delivery gate: the oracles CI already ran, on the shipped path.
# scripts/hooks/pre-push executes it and refuses the push when it is red, and
# CI runs the same recipe on master, on v* tags and on pull requests.
gate:
	cargo fmt -- --check
	cargo clippy --all-targets -- -D warnings
	cargo test

# Once per clone: make git run scripts/hooks instead of .git/hooks. A relative
# path resolves against the worktree the push runs from, so one setting covers
# every `git worktree`.
hooks:
	git config core.hooksPath scripts/hooks
	@echo "core.hooksPath -> scripts/hooks"
