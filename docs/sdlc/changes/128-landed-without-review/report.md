# Bugfix: three commits on main landed without a valid review, and the late review found defects

- **Route**: `bugfix` — the retroactive review findings are the report.
- **Skill**: `.claude/skills/root-cause/SKILL.md`

## What the stop gate found

`.claude/hooks/gate-stop.sh` refused three commits already on `main`:

| Commit | Why |
|---|---|
| `037405a` merge of change 122 | its `review.yaml` judged diff `27d6990371bc2426`; the merged diff is `b8ece15512ef5cf6` |
| `029779f`, `3ac57e7` (fix/124, tmux socket errors) | touch the `subprocess` surface with no change directory and no verdict |

They cannot be amended on `main`. The person chose to review them after the
fact (「後追い対応」), fix what the review finds here, and move the audit mark
past them with that record as the reason.

## Retroactive review, and what it found

`subprocess-safety-reviewer` and `rust-reviewer` on `029779f`;
`rust-reviewer` on `037405a^1..037405a`.

### 124 — two defects still on HEAD

1. **A failed stop drops the confirmed delete.** `SessionStopped` took
   `remove_after_close` up front; on an error that is not "server gone" (a
   timeout) the flag was not put back, so the 「停止」 the notice asks for only
   cancelled and the record stayed.
   Test: `failed_stop_keeps_the_requested_delete_for_the_retry`, watched failing:
   `the delete survives a stop that could not be confirmed`.
   Fix: put the flag back on that path (and on the same path in
   `TerminalClosed`).
2. **Gone server + unwritable store loops every frame.** The resize-failure arm
   rolled the store back to `Active` when the save failed, so the frame asked
   for another resize — a tmux child and a store write each frame.
   Test: `gone_resize_with_unwritable_store_stays_lost_and_retries`, watched
   failing: `left: Active right: Lost`.
   Fix: stay `Lost`, set `store_retry_pending`, say so in a notice.

### 122 — code defects already fixed, tests were not

The splitter clamp collapse and missing Escape cancel were fixed by 125, and
the layout reworked by 127. What remained on HEAD: tests named for
requirements that did not exercise the code. The drop decision is now
`sidebar_drop_side` in `src/ui/session_tree.rs`, called by the drawing code
and by `test_sidebar_tab_drag_between_sides` and
`test_sidebar_tab_drag_deadzone_and_bounds`.
`test_sidebar_tab_reorder_within_side` keeps its name (two specs cite it) and
now says it checks labels only: reordering was specified in 122 and never built.

## Left as they are

- 124: a gone-looking stop error marks any status `Cancelled`, including
  `Exited`; the `error connecting to` needle is broad. Behaviour the commit
  intended; reaching it needs a session that exited mid-stop.
- 122: `Serialize`/`Deserialize` derived on enums never persisted.
