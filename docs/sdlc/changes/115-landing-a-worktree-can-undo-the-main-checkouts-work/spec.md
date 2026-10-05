# Spec: landing a worktree can undo the main checkout's work

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Landing refuses, before running any merge, when the main checkout already has
   `MERGE_HEAD`, and `MERGE_HEAD` survives the refusal.
   `refuses_to_land_over_a_merge_already_in_progress`.
2. A conflicting landing is aborted, `HEAD` is back where it was, no `MERGE_HEAD`
   remains, the checkout is clean, and the error names each conflicting file.
   `a_conflicting_landing_is_aborted_and_names_the_files`.
3. A merge that started and then stopped for a reason other than a conflict (a
   `pre-merge-commit` hook refusing) is aborted the same way and reports git's
   output. `a_landing_stopped_by_a_hook_is_undone`.
4. Any failed merge — non-zero exit, timeout, or a spawn error — takes one path:
   probe `MERGE_HEAD`; if present, read the unmerged paths and run
   `merge --abort`; then, whether or not an abort ran, verify `HEAD` equals the
   commit recorded before the merge and `MERGE_HEAD` is gone. Only a verified
   state earns a sentence saying the checkout is unchanged or restored;
   otherwise the notice says the merge was left half done.
   `a_landing_that_cannot_be_undone_says_so` (a `pre-merge-commit` hook that
   takes `index.lock` and exits 1, so the abort fails — measured).
   `merge --abort` running only when `MERGE_HEAD` is present is defence in
   depth: with requirement 1's pre-check, any `MERGE_HEAD` after a failure is
   this landing's own, so no test can tell it from an unconditional abort.
5. Landing and Operon's commit, draft, and push on the same project exclude one
   another: `git_worktree_busy` counts a landing in flight; `land_worktree` and
   `push_project` refuse while it is true; the push button greys out on it as
   the commit button already does.
   `landing_and_the_projects_other_git_writes_wait_for_each_other` drives
   `land_worktree` and `push_project` with `GitMutation`, `CommitMessage`, and
   `LandWorktree` in flight, and checks `ui_push_row` gates the button on
   `git_worktree_busy`.
6. Landing refuses while a session is starting, active, queued, or unknown in the
   worktree **or** in the project checkout itself (a session of the project with
   no worktree). `landing_waits_for_a_session_in_the_main_checkout`.

## Behaviour

All failures reach the person through `notice`, prefixed by the existing
「マージに失敗しました: 」. New or changed sentences:

| When | Japanese |
|---|---|
| `MERGE_HEAD` exists | `メイン作業ツリーで別のマージが進行中のため取り込めません。そのマージを完了するか中止してから、もう一度お試しください。` |
| conflict, restored | `競合が発生したため取り込みを取り消し、{mainline} を元の状態に戻しました。競合したファイル: {files}` |
| stopped otherwise, restored | `マージを完了できなかったため取り込みを取り消し、{mainline} を元の状態に戻しました: {err}` |
| merge never started | `マージを開始できませんでした。{mainline} は変更されていません: {err}` |
| abort did not restore | `マージを取り消せませんでした。メイン作業ツリーにマージ途中の状態が残っています: {err}` |
| a session busy in the main checkout | `この worktree またはメイン作業ツリーで動作中・待機中のセッションがあります。停止・完了・キャンセルしてからマージしてください。` |
| another git write in flight | the existing `Git の操作が既に実行中です。` |

`{files}` is the unmerged paths joined by `、`, at most 10, then `ほか {n} 件`.
The old `マージに失敗したため操作を中断しました（コンフリクトまたは制約）: {err}`,
`worktree のマージ操作が既に実行中です。`, and the worktree-only session
sentence are retired with their table rows. No layout changes; the commit
button already greys out on `git_worktree_busy`.

## Design

Placement: `EXTEND` — `git_land_worktree` in `src/git.rs` and `land_worktree` in
`src/app.rs`, both from change 110. Route `modify`.

- `src/git.rs` `git_land_worktree`: check `git rev-parse -q --verify MERGE_HEAD`
  in the project before the dirty check; record `rev-parse HEAD` before the
  merge; on failure, if `MERGE_HEAD` exists read
  `git diff --name-only --diff-filter=U -z`, run `merge --abort`, then verify
  `HEAD` and `MERGE_HEAD`, and pick the sentence. The `MERGE_HEAD` probe and
  the unmerged-path read go through `git_output_allowing_limited` (exit 1
  accepted for the probe; 64 KiB ceiling, 20 s timeout); `HEAD` through
  `git_output`; the abort through `run_command_with_timeout` as today.
  A failing `rev-parse HEAD` before the merge refuses through `?`, which is
  safe: nothing has been written. A detached `HEAD` is already refused by the
  `current_branch != mainline` check (`--abbrev-ref` prints `HEAD`); an unborn
  one errors before it.
- `src/app/screens.rs`: `ui_push_row`'s push button reads `git_worktree_busy`.
- `src/models.rs`: `landing_blocked_by_session(sessions, project_id, worktree)`
  beside `worktree_has_reserved_session`, true for the worktree or for a session
  of `project_id` with `worktree_path == None`.
- `src/app.rs`: `git_worktree_busy` gains `BackgroundKey::LandWorktree(project)`;
  `land_worktree` uses it and `landing_blocked_by_session`; `push_project` reads
  `git_worktree_busy` instead of its own key check.
- `src/i18n_tables.rs`: EN and KO rows for the new ids, retired rows removed.
- No persisted shape, no new command, nothing in the draw path.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | No | No colour touched. |
| Icons | No | No glyph touched. |
| Identifier SSOT | Yes | The busy key list stays in `git_worktree_busy` only (`.claude/rules/identifiers.md`); no new shared string. |
| Durability | No | No store, sidecar, migration, or lock touched. |
| Subprocess safety | Yes | The new `git` calls go through `git_output_allowing_limited` / `git_output` (both `src/exec.rs` underneath); no new program, no shell. |
| Documentation | No | READMEs describe landing at the level 110 wrote; refusals are notices, not documented behaviour. |
| Local-first | Yes | No network, no `unsafe`. |
| Budgets | Yes | The unmerged-path read is capped at 64 KiB and 20 s; the notice names at most 10 paths. |

## Flagged concerns

None.

## Acceptance

- `cargo fmt --check`, `cargo clippy --locked -- -D warnings` clean.
- `cargo test --locked` passes with the six new tests, `6 ignored`.
- Each new guard goes red under its mutation (`plan.md`).

## Rejected alternatives

- Abort only on conflict and leave hook failures half merged — leaves the
  checkout in a state the notice does not describe.
- Ask the person before aborting — a conflicting landing is never something
  they asked to keep; restoring is what the 110 intent promised.
