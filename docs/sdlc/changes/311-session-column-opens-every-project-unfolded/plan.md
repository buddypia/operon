# Plan: the session column is a list of projects, each folded until opened

- **Spec**: `./spec.md`
- **Approved**: 2026-11-18
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/ui/session_tree.rs` | `SessionArrangement.folded` becomes `unfolded` |
| `src/app/screens.rs` | `ui_workspace_projects` folds by default, shows matches while filtering, drops the folded heading's rule and the frame's vertical margin; `toggle_project_fold` edits `unfolded` |
| `src/app/session_order.rs` | `commit_session_arrangement` retains `unfolded` |
| `src/i18n_tables.rs` | 「フィルタ中はすべて表示されます」 in every table |
| `src/tests.rs` | four `column_311_*` tests; fixtures that read rows unfold their project; the two older fold tests rewritten to the new default |
| `README.md`, `README.ja.md`, `README.ko.md`, `CHANGELOG.md` | one sentence each |
| `docs/sdlc/changes/311-session-column-opens-every-project-unfolded/` | `mutations.py`, `verification.yaml`, `state.yaml` |

## Order of work

1. Rename the field to `unfolded` everywhere it is named, so the new tests
   compile, while the draw path and `toggle_project_fold` still treat the list
   as the folded projects: the tests of step 2 then fail on behaviour, not on
   a missing field. The meaning moves in step 3.
2. Add the four `column_311_*` tests and watch each fail for its own reason.
3. Change `ui_workspace_projects`, `toggle_project_fold`,
   `commit_session_arrangement`; add the string.
4. Run the whole suite once locally to find every test that read rows of a
   project it never unfolded; unfold in their fixtures, rewrite the two older
   fold tests. No test is removed.
5. READMEs and changelog; `mutations.py`; `make q.fast`;
   `bash scripts/check-bands.sh`; `verification.yaml`; the verifier; commit,
   push, CI, land, ship.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Tests that read session rows now see none | failures across `session_285_`, `sidebar_302_`, `sidebar_251_`, `workspace_project_` | the local full run in step 4, then CI |
| An older order file fails to parse and loses the order and pins | the column falls back to default order after the update | `column_311_an_older_order_file_keeps_its_order_and_folds_every_project` |
| The filter's forced unfold gets saved | a project stays unfolded after 「クリア」 | `column_311_a_filter_lists_matches_of_a_folded_project` |
| A drag of a project lands wrong with the tighter frame | drop line off by the old margin | `sidebar_302_compact_rows_still_pin_reorder_and_sync_with_top_tabs` (project drag) |
| `src/app/screens.rs` crosses `10000` lines | `largest_module_lines` at `propose` | `bash scripts/check-bands.sh` |
| The person keeps opening the October app | the change looks absent | the final report names the app to open and asks about the old bundle |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked column_311_` — 4 passed, each watched failing before step 3.
- `cargo test --locked` — `0 failed` locally once, and in CI on the push.
- `python3 docs/sdlc/changes/311-session-column-opens-every-project-unfolded/mutations.py` — every mutation caught.
- `/Applications/Operon.app` shipped from `main`; `bash scripts/check-installed-build.sh` says current.

## Departures from the plan

- The disk filled (under 1 GiB free) before step 1 could be built, so steps 1
  to 3 were written together without a build: the four `column_311_*` tests
  are watched failing through the mutation sweep, which removes each piece of
  the change and runs them, rather than before the change existed.
- The base fixture `app_with_two_sessions` unfolds its one project, so the
  tests built on it read rows as before; the full local run of step 4 waits
  for disk space.
- The tooltip reads 「フィルタ中はすべて表示されます」, the app's spelling.
- Disk space came back; the first build read a folded pitch of 29px, not 28:
  under the app's button padding (`5px`) the heading's 14px icon button is
  25px tall, not `CONTROL_HEIGHT_SMALL`. The heading row takes `SPACE_XS` as
  its vertical button padding, so the button is 24px there; `small_icon_button`
  elsewhere is untouched. The sweep holds it (`heading-button-keeps-the-app-padding`).
- The full local run of step 4 failed five tests: four read rows of a second
  project they never unfolded and now unfold it in their setup; the project
  drag of `sidebar_302_compact_rows_still_pin_reorder_and_sync_with_top_tabs`
  expected the drop line 2px higher, the frame margin this change removes.
- `column_311_a_filter_lists_matches_of_a_folded_project` also reads the
  disclosure's tooltip under a filter, and the sweep mutates that sentence.
