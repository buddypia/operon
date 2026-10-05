# Plan: one entry point for a development request

- **Spec**: `./spec.md`
- **Status**: done

## Steps

1. Write `.claude/skills/feature-pilot/SKILL.md` and its reference
   `.claude/skills/feature-pilot/references/rust-build-chain.md`.
2. Remove the entry claim from `.claude/skills/sdlc/SKILL.md` and add the
   worktree step to its §3. Name the entry skill in `CLAUDE.md` and `AGENTS.md`.
3. Add `development-skills.md` (removed by change 114).
4. Add the three tests to `src/tests.rs`. Watch each fail under a mutation:
   - `AGENTS.md`, `CLAUDE.md`, and the `sdlc` description each pointed back at
     `sdlc`;
   - the worktree step removed from `sdlc` and from the entry skill;
   - a skill dropped from the catalog.
5. `make q.check`, then `bash scripts/check-bands.sh`.

## Proof of completion

- `cargo test --locked skill` — the three new tests pass, and each failed under
  its mutation (six mutations, six caught).
- `make q.check` — `cargo fmt --check` silent, `cargo test --locked` with
  `0 failed`, `cargo clippy --locked -- -D warnings` clean.
- `bash scripts/check-bands.sh` — exit 0; `always_loaded_bytes` below 13000.

## Departures from the plan

None.
