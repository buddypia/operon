# Plan: consolidate worktree guard configuration into dedicated policy files

- **Spec**: `./spec.md`
- **Approved**: 2026-09-23
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `.cli/hooks/`, `.cli/lib/`, `.cli/_cli-dispatch.mjs` | consolidated guard scripts and libraries |
| `.claude/scripts/` | consolidated utility scripts |
| `.claude/skills/create-pr/SKILL.md`, its merge-conflict reference | standardized reference documentation |
| `.claude/hooks.json`, `.codex/hooks.json` | regenerated from the hook registry |
| `.claude/config/worktree-policy.json` | `session_owner_scope` and `trunk_bash_allowlist` added |
| `src/tests.rs` | one test for the two switches and their wiring |

## Order of work

1. Consolidate guard configuration options into dedicated policy files (the table in `spec.md`).
2. Add the two switches to `.claude/config/worktree-policy.json`, and verify allowlist patterns against commands on `main`.
3. Add test in `src/tests.rs`, and verify failure when switches are disabled.
4. Run the three gates and verify test pass count.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A pattern refuses a command a documented procedure needs on `main` | a landing session refused mid-procedure | the probe in step 2; the test pins `make wt.new`, `git merge --no-ff`, `cargo test --locked` |
| A future edit turns a switch off | 058's behaviour silently gone | the test in `src/tests.rs` |
| Stop guard or `ops.mjs` changes break existing invariants | 071 tests fail | the 071 tests, unchanged |
| Codex refuses the regenerated hook commands until re-trusted | Codex guards do not run on this machine until approved | recorded as an open risk; outside the repository |

## Proof of completion

- `cargo fmt --check` prints nothing.
- `cargo test --locked` prints `test result: ok. 562 passed; 0 failed; 6 ignored`
  (561 on `main`).
- `cargo clippy --locked -- -D warnings` prints nothing past the compile lines.
- `the_trunk_allowlist_and_the_ownership_check_are_switched_on_here` fails with
  `trunk_bash_allowlist.enabled: false` and with `session_owner_scope: "commit"`.
- The staleness check prints `current` with an empty `locally_modified`.

## Departures from the plan

- Review round 1 (rust-reviewer, Important I-1): `make wt.run` and
  `wt-run.mjs` were on the allowlist and run an arbitrary command in the main
  checkout or in another session's worktree, where neither guard sees it. The
  make pattern now admits only `wt.new` and `q.check`, the node pattern drops
  `wt-run`, and the test asserts both are refused (watched failing with
  `wt-run` restored).
