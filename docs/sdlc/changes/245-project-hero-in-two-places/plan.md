# Plan: the session column's project becomes a heading

- **Spec**: `./spec.md`
- **Approved**: 2026-11-05
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/app/screens.rs` | `ui_workspace_projects`: flat heading group (no `card_frame` fill), demoted top 「プロジェクト」 heading, heading drops `ICON_PROJECT`, hairline, strong session titles; `session_crumb` helper; `ui_terminal_pane`: crumb row above the one header draw both single and split panes share, header drops the repeated project name |
| `src/tests.rs` | Four tests from the spec (`session_column_project_is_a_heading_not_a_card`, `session_column_project_heading_keeps_its_operations`, `session_crumb_names_project_then_title`, `terminal_pane_draws_the_crumb_once`), each watched failing first |
| `docs/sdlc/changes/245-project-hero-in-two-places/` | This change's records, `mutations.py`, `verification.yaml`, `review.yaml` |

## Order of work

1. Tests first: `session_crumb_names_project_then_title` (pure helper),
   `terminal_pane_draws_the_crumb_once` (the pane draw), then the two
   source-reading guards for the heading and its kept operations. Watch
   each fail.
2. `session_crumb` + the pane crumb row (single and split share the draw);
   drop the header's project label.
3. The column: flat group, demoted top heading, no `ICON_PROJECT` in the
   group heading, hairline, strong session rows.
4. `make q.fast`, the named tests, then the mutation sweep.
5. `verification.yaml` from the spec, verifiers, `review.yaml`, commit,
   push, CI, merge, cleanup, package.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Drag reorder breaks without the card rect | Dropping a project does nothing or lands wrong | The group rect is recorded in `ColumnTargets` exactly where the card rect was; the heading-keeps-operations guard reads it |
| Fold click area shrinks | A folded project cannot be reopened | Disclosure button unchanged in size; only the name's text role changes |
| Crumb duplicates the workspace tab title | Two lines saying the same session | The pane header keeps the title; the crumb is project-first and muted — accepted in the approved screen |
| Session row hover/click shifts | Rows move as fills change | Row frame and interact id unchanged |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked session_column_project_is_a_heading_not_a_card`,
  `cargo test --locked session_column_project_heading_keeps_its_operations`,
  `cargo test --locked session_crumb_names_project_then_title`, and
  `cargo test --locked terminal_pane_draws_the_crumb_once` — the four new
  tests pass, watched failing before the fix.
- `make q.fast` — passes.
- The running app shows the muted headings and the crumb; the Projects page
  is pixel-identical.

## Departures from the plan

- Verification (empirical-verifier): the selected heading at `text_strong`
  broke requirement 3 in that group, and the guards read roles from anywhere
  in the column. Spec requirement 1 now keeps the selected heading in
  `text_muted` (strong only), the guards read the name, title, and crumb
  labels themselves, and the sweep gained the three role mutants. Recorded
  as F1 and F2 in `verification.yaml`; sweep is 8 of 8 caught.
