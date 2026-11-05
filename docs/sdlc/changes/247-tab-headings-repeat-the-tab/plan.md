# Plan: tab contents stop repeating their tab as a heading

- **Spec**: `./spec.md`
- **Approved**: 2026-11-05
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/app/screens.rs` | `ui_git` / `ui_worktrees` / `ui_pull_requests`: drop the repeated `ui.heading`; `ui_sessions`: drop the repeated heading; `ui_terminal_session_tabs`: title demoted to 12.5 `text_muted`; `ui_skills` / `ui_rules`: headings become 15.0 strong `text_strong`; `ui_settings`: sidebar 設定 becomes 12.5 `text_muted` |
| `src/tests.rs` | Five tests from the spec, each watched failing first |
| `docs/sdlc/changes/247-tab-headings-repeat-the-tab/` | This change's records, `mutations.py`, `verification.yaml`, `review.yaml` |

## Order of work

1. Tests first: the five source-reading guards from the spec. Watch each
   fail.
2. Remove the repeated headings (requirements 1–2), keeping every control
   on their rows.
3. Demote the three remaining labels (requirements 3–5).
4. `make q.fast`, the named tests, then the mutation sweep.
5. `verification.yaml` from the spec, verifiers, `review.yaml`, commit,
   push, CI, merge `main` in first if change 245 has landed, merge,
   cleanup, package.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A removed heading was the only name of a panel | The docked session panel opens unnamed | Requirement 3 demotes rather than removes that title; the guard pins the label's presence |
| A control shared the heading's row and is lost with it | Refresh or search missing from a tab | `tab_headings_keep_their_controls` reads each row's controls in the source |
| Existing painted-text tests expect a heading string | Suite failures outside the new guards | Run the named tests plus the dock/session-list tests that read these strings before committing |
| Change 245 lands first and moves nearby lines | Merge conflict in `screens.rs` | This change touches none of 245's regions; merge `origin/main` into the branch and re-run the guards before landing |

## Proof of completion

- `cargo fmt --check` — no output.
- The five named tests pass, watched failing before the fix (one filter
  per invocation).
- `make q.fast` — passes.
- The running app shows tabs opening on their controls, section-strength
  スキル／ルールと指示, and a quiet settings sidebar label; tabs and page
  headers unchanged.

## Departures from the plan

- The first CI run failed two tests. `every_message_row_is_one_the_code_shows`
  named the cause: the removed PR heading was the only draw of the
  `プルリクエスト` message id, so its EN/KO rows were orphaned; the rows are
  removed with the heading (spec's Documentation row corrected to match).
  `a_column_title_double_click_renames_its_session` lost its click when the
  demoted list title moved the first row up: the press landed in the same
  pass as the pointer's arrival, before the row's hover layout was drawn.
  That test's scene now rests the pointer on the row for one pass first —
  the arrangement `column_hovered` already used — and the guarded
  behaviour (double-click renames) is asserted unchanged.
