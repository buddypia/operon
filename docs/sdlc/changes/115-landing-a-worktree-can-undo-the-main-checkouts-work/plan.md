# Plan: landing a worktree can undo the main checkout's work

- **Spec**: `./spec.md`
- **Approved**: 2026-10-04
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/git.rs` | `git_land_worktree`: `MERGE_HEAD` refusal first; pre-merge `HEAD`; every merge failure (exit, timeout, spawn error) goes through one `undo_failed_landing` path: probe, unmerged paths, abort when `MERGE_HEAD` is present, verify `HEAD` and `MERGE_HEAD`, pick the sentence; the 10-path cap in a small helper |
| `src/models.rs` | `landing_blocked_by_session` beside `worktree_has_reserved_session` |
| `src/app.rs` | `git_worktree_busy` gains `LandWorktree`; `land_worktree` uses it and `landing_blocked_by_session`; `push_project` uses `git_worktree_busy` |
| `src/app/screens.rs` | `ui_push_row`'s push button reads `git_worktree_busy` |
| `src/i18n_tables.rs` | EN/KO rows for the six new ids; the three retired rows removed |
| `src/tests.rs` | the six tests named in the spec |

## Order of work

1. The six tests, watched failing. The git tests must fail on assertions
   against today's `git_land_worktree`; the two that name new symbols fail to
   compile first, then on assertions once a stub exists.
2. `src/git.rs`, `src/models.rs`, `src/app.rs`, `src/app/screens.rs` until they
   pass.
3. i18n rows; `cargo fmt`, the three gates.
4. Mutation sweep (copied from `docs/sdlc/templates/mutations.py`), each
   expected to turn a named test red:
   1. drop the `MERGE_HEAD` pre-check → `refuses_to_land_over_a_merge_already_in_progress`;
   2. claim "元の状態に戻しました" without the `HEAD`/`MERGE_HEAD` verification →
      `a_landing_that_cannot_be_undone_says_so`;
   3. drop the unmerged-path list from the sentence →
      `a_conflicting_landing_is_aborted_and_names_the_files`;
   4. skip the abort when the merge stops for a hook →
      `a_landing_stopped_by_a_hook_is_undone`;
   5. drop `LandWorktree` from `git_worktree_busy`;
   6. `land_worktree` back to its own `LandWorktree` key check;
   7. `push_project` back to its own `GitMutation` key check;
   8. `ui_push_row` back to `contains(GitMutation)` —
      5–8 → `landing_and_the_projects_other_git_writes_wait_for_each_other`;
   9. drop the project-checkout arm of `landing_blocked_by_session` →
      `landing_waits_for_a_session_in_the_main_checkout`.
   Not mutated: aborting only when `MERGE_HEAD` is present. With mutation 1's
   pre-check in place that gate is unobservable — defence in depth, untested.
5. `rust-reviewer` plus whatever `scripts/check-review.sh` widens to (the diff
   touches `src/git.rs`, so `subprocess-safety-reviewer`); `review.yaml`;
   commit; merge into `main`; clean up the worktree; package and swap.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The abort removes a merge somebody else started | a resolution vanishes | `refuses_to_land_over_a_merge_already_in_progress` asserts `MERGE_HEAD` survives |
| The notice claims a restore that did not happen | person trusts a half-merged checkout | `a_landing_that_cannot_be_undone_says_so`; mutation 2 |
| A merge timeout skips the abort | `MERGE_HEAD` left with no notice saying so | the spawn `Err` joins the same path; read in review (a 60 s hang is not a test fixture) |
| A commit and a landing share the index | `index.lock` errors, a lost commit | `landing_and_the_projects_other_git_writes_wait_for_each_other` |
| Push now also waits for a draft and a landing | push greyed while a message is drafted | intended: the button and the action read one predicate |
| A fixture hook does not run under a global `core.hooksPath` | the hook tests pass vacuously | the fixtures set a local absolute `core.hooksPath`, and assert the hook's effect |
| A retired message id still referenced | Japanese fallback in EN/KO | `every_message_id_has_a_row_in_every_table` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- The six new tests pass, and each mutation in step 4 turns its test red.

## Departures from the plan

- **Retired rows: two, not three.** The old conflict sentence
  (`マージに失敗したため操作を中断しました…`) never had a row in either table, so
  only `worktree のマージ操作が既に実行中です。` and the worktree-only session
  sentence were removed. The new sentences all have EN and KO rows.
- **The overflow count reuses `ほか {p0} 件`**, an existing id, rather than adding
  a seventh.
- **The tests share a `LandFixture`** (project, linked worktree, a hooks
  directory outside both trees set as a local absolute `core.hooksPath`),
  cleaned up on drop.
- **The status set moved into `session_reserves_its_directory`** so
  `worktree_has_reserved_session` and `landing_blocked_by_session` read one list
  (`.claude/rules/identifiers.md`).
- **Review round 1 added a seventh test and a tenth mutation.**
  `subprocess-safety-reviewer` found that "unchanged" and "restored" checked
  `HEAD` and `MERGE_HEAD` but not the tree: a merge killed after writing the
  tree and before writing `MERGE_HEAD` reported "unchanged" over a dirty
  checkout. The verification now also requires an empty `status --porcelain`.
  `a_landing_that_left_the_tree_changed_is_not_called_unchanged` produces the
  state with a hook that SIGKILLs the merging git (measured; a failing hook
  alone cannot, because git writes `MERGE_HEAD` after the hook refuses).
- **Sweep result**: 10 of 10 caught, each by the test step 4 names, mutation 10
  by the new test (`mutations.py` in this directory).
- **This change's lesson lands with the close, not here.** `docs/sdlc/lessons.md` is
  outside the paper trail, so adding it would move the reviewed diff past this
  route's two review rounds.
