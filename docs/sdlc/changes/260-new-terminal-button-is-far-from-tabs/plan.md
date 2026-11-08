# Plan: The new-terminal button follows the last tab

- **Spec**: `./spec.md`
- **Approved**: 2026-11-09
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/app/screens.rs` | Tab region `ScrollArea` auto-shrinks horizontally; empty-workspace + laid out left to right. |
| `src/tests.rs` | `session_260_plus_follows_last_tab`, `session_260_empty_plus_sits_where_the_first_tab_appears`. |
| `docs/sdlc/changes/260-new-terminal-button-is-far-from-tabs/` | State, screen, verification, mutation sweep. |

## Order of work

1. Commit spec, screen and plan after the evaluator's verdicts and a readiness Go; no `src/` edit before.
2. Write the two `session_260_` tests and watch both fail on the unmodified source (the + at the right edge).
3. Change the two layout lines; the tests pass, and `session_254_` still passes.
4. Render fixture images at 1100px and 640px, with two tabs, many tabs, split panes and the empty workspace, and look at them.
5. `make q.fast`; mutation sweep; `verification.yaml`; an independent empirical verifier tries to break it.
6. Close `state.yaml`, commit, push, wait for CI, merge on main, clean up, package and install.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The + scrolls away or overlaps tabs once they overflow | + missing or drawn over a tab on a narrow pane | `session_254_terminal_action` (20 tabs, clip ends before the +) |
| Shrinking the region clips the selected tab's underline or the rename editor | Underline missing; editor cut | `session_254_selected_tab_rule_is_visible`, `session_253_` tests |
| The scroll area keeps a stale width for a frame after a tab is added | + overlaps the new tab for one frame | Rendered fixture after a press; egui measures content in the same frame |
| A split pane's + follows another pane's tabs | + far from its own tabs | Split-pane fixture image; same code path per pane |

## Proof of completion

- `make q.fast` — passes.
- `cargo test --locked session_260_` and `cargo test --locked session_254_` — all pass; the two new tests failed before step 3.
- Mutation sweep — both mutations caught.
- Fixture images show the + right after the last tab with few tabs and at the right edge with many.
- CI green on the branch head; the installed app contains the build from main.

## Departures from the plan

- The empty-workspace test failed for a reason the spec had not named: the
  heading 「セッションを選択」 was not on screen at all. The existing
  `with_layout(right_to_left(Align::Center))` row took the body's full height,
  so the + sat at the vertical centre of the right edge and the heading fell
  below the window. `Layout::left_to_right` would have kept that defect, so the
  row became `body.horizontal`; spec.md's Design says so and was re-evaluated.
- `session_260_plus_follows_last_tab` first measured from the close glyph,
  which sits 10px inside its button; it now measures from the last tab's frame,
  where the gap is the row's 8px item spacing — the same as between two tabs.
- Rendered fixture images (normal, many tabs at 1100px and 640px, split, light,
  empty, and the first frame after a tab is added) came from a temporary test
  that was removed before the commit; see verification.yaml.
- The empirical-verifier could not run in its isolated worktree: the worktree
  policy hook judged that tree as the trunk and refused `cargo`, `git fetch`
  and file writes. It returned four drafted probes instead; the author ran them
  in this worktree (tabs added one at a time to 30, scroll then close all but
  one, inline rename, split with an overflowing left pane) and all held. The
  first became `session_260_plus_stays_clear_at_every_tab_count`. CI run
  42502209570 on 5da4756 passed before it was added.
