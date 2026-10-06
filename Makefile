# --- worktree-isolation (auto-managed targets) ---
.PHONY: wt.new wt.run

wt.new:
	@if [ -z "$(BR)" ]; then \
		echo "❌ wt.new: BR=<branch> required. Example: make wt.new BR=feature/<task>"; \
		exit 2; \
	fi
	@node .claude/scripts/worktree-new.mjs --branch $(BR) $(if $(BASE),--base $(BASE)) $(if $(DRY),--dry-run)

wt.run:
	@if [ -z "$(CMD)" ]; then \
		echo "❌ wt.run: CMD=\"<command>\" required. Example: make wt.run CMD=\"npm run test\""; \
		exit 2; \
	fi
	@node .claude/scripts/wt-run.mjs $(CMD)

.PHONY: q.check q.fix

# The same three, in the same order, as AGENTS.md, CONTRIBUTING.md, CLAUDE.md,
# the ship skill, .claude/hooks/gate-commit.sh, CI and the pull-request
# template. This target is the eighth copy of that list, so it is held by the
# same test as the other seven: every_document_that_names_the_gates_names_the_
# same_three in src/tests.rs reads this file too. A `make q.check` that ran a
# different set would report a pass the commit gate then refuses — or worse,
# refuse what the commit gate would have taken.
#
# `--locked` is not decoration. Without it a stale Cargo.lock is updated in
# place and the gate reaches the network, which is the promise CONTRIBUTING.md
# calls "Local-first means local-first".
q.check:
	cargo fmt --check
	cargo test --locked
	cargo clippy --locked -- -D warnings

# What the commit gate runs (change 128). The suite runs in CI on the pushed
# branch, and the merge into main waits for it; locally, run the tests a change
# names with `cargo test --locked <filter>` beside this.
.PHONY: q.fast
q.fast:
	cargo fmt --check
	cargo clippy --locked -- -D warnings

q.fix:
	cargo fmt
	cargo clippy --locked --fix --allow-dirty --allow-staged

.PHONY: prune.target
prune.target:
	@bash scripts/prune-target.sh $(if $(DRY),--dry-run)

# --- end worktree-isolation ---
