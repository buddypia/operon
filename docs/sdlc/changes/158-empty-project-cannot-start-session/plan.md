# Plan: an empty project offers 「セッションを開始」 in place of its empty line

- **Spec**: `./spec.md`
- **Approved**: 2026-10-17
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/app/screens.rs` | the three per-project empty branches draw `icon_text_button(ICON_ADD, 「セッションを開始」)` and open the sheet; the history views split on the search |
| `src/i18n_tables.rs` | 「セッションを開始」 and 「一致するセッションはありません。」 rows in `EN_TABLE` and `KO_TABLE`, sorted |
| `src/tests.rs` | three tests named in the spec, and a draw-and-click helper they share |

The helper cannot reuse `render_workspace_output`: that forces
`session_library_open = false`, and the history tests need it true. It draws
`ui_terminal_workspace` itself at 900x600 — the screen `click_at` hard-codes, so
the positions it reads are the positions the click lands on — finds the text
that `ends_with("セッションを開始")` (the painted label is the `ICON_ADD` glyph,
two spaces, then the word), and clicks it. `app_with_two_sessions` has one
project; each test adds a second, empty project drawn after it, so a click that
selected the first project would fail.

## Order of work

1. Write the three tests and the helper. They compile against today's code
   (they only draw and click) and fail: no 「セッションを開始」 text is drawn under
   an empty project, so the click lands on nothing and `launch_sheet_open` stays
   false. Watch each fail.
2. Change the session column's branch; its test passes.
3. Change the two history branches; their tests pass.
4. Add the four table rows; `every_translation_table_is_sorted_by_message_id`
   and `every_message_id_has_a_row_in_every_table` pass, and the
   `translation_for` assertions (EN and KO, both strings) pass.
5. `make q.fast`; the named tests; mutation sweep.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The button appears when the project's sessions are pinned above | a launch offer on a project that has sessions on screen | `a_project_emptied_by_pinning_does_not_claim_it_has_no_sessions`, extended to assert no 「セッションを開始」 label |
| A search hiding sessions reads as an empty project | 「セッションを開始」 under a project whose sessions are filtered out | the history tests type a non-matching search and assert 「一致するセッションはありません。」 and no button |
| The click selects the wrong project | the sheet opens for another project | each test asserts `selected_project` is the empty one, with a non-empty project drawn above it |
| A table row out of order | the translation silently not found | `every_translation_table_is_sorted_by_message_id` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked an_empty_project_in_the` — `3 passed`.
- `cargo test --locked a_project_emptied_by_pinning` — `1 passed`.
- `cargo test --locked every_` — all pass.
- Mutation sweep: restore the old label in each branch, drop the click, drop the
  `needle` split — each turns a test red.
- `bash scripts/package-macos.sh`, then in the app: a project with no sessions
  shows 「セッションを開始」 in the column; clicking it opens the sheet.

## Departures from the plan

- **The history tests reopen the history before the search case.** Clicking
  「セッションを開始」 selects the empty project, and `select_project` closes the
  history whenever the project changes — as the history's existing header
  controls already did. The first run read the search case off the terminal
  workspace and failed; the tests now set `session_library_open` again. Not a
  product change: the sheet opens over whatever is on screen.
- **The click helper returns how many 「セッションを開始」 it saw.** With the
  history open, the session column draws its own button for the same empty
  project, so "a button exists" would pass on the column's button even with the
  history's removed. The history tests assert two, and the rightmost (the
  history's) is the one clicked; `list-click-does-nothing` and
  `grid-opens-nothing` in `mutations.py` confirm the click lands there.
- **`mutations.py`**: 8 of 8 caught, including the claim mutations (the column
  saying 「セッションはまだありません」 again, and the list offering a start under a
  search).
