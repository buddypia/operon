# Plan: each project is one framed block, sessions are ruled apart, and a running session stops from its row

- **Spec**: `./spec.md`
- **Approved**: 2026-10-17
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/app/screens.rs` | `ui_terminal_session_tabs`: a frame per project, the 15px name, rules between rows, the row's first line laid out right to left first, ■ on the active running row, one shared menu list for `···` and right-click; `ui_pinned_sessions`: the same frame |
| `src/tests.rs` | eight tests from the spec and their helpers |

Test helpers, beside change 166's (`workspace_texts_at_click_size`,
`app_with_an_empty_second_project`):

- `column_fixture` — `app_with_two_sessions` (which makes both Active; the
  fixture sets `second` to Exited), all created three hours ago so the time
  reads 「3時間前」, plus a project `r` with one finished session `third`,
  which is the open one so the terminal header paints no 「停止」 of its own.
- `painted_shapes(output)` — every `Text`, `Rect`, and `LineSegment` a frame
  painted, flattened out of `Shape::Vec`.
- `block_around(rects, point)` — the smallest `raised`-filled rect containing
  a point.
- The column's own copy of a title is the leftmost painted text equal to it
  (the terminal header repeats the selected session's title further right).

## Order of work

1. Write the eight tests against today's code and watch each fail for its
   stated reason:
   - `each_project…`: no `raised` rect around the project header (asserted
     `Some`, so a missing frame cannot pass);
   - `the_needs_you_queue…`: no `raised` rect around 「要対応」;
   - `sessions_in_a_project…`: no `border_subtle` rule inside a block;
   - `a_session_rows_corner…`: the time's right edge past the right edge of
     the column heading's 「履歴」 — checked before the block, so it fails today
     on the clipped time, not on the missing frame; then `Some(block)` and
     inside it;
   - `a_long_project_name…`: with a 200-character name, 「1 件」 and the header ＋
     past 「履歴」's right edge;
   - `a_column_title_double_click…`: passes today — it pins the rename
     gesture through the re-layout (the only existing rename test,
     `renaming_a_session_is_durable`, is store-level);
   - `a_running_session_stops…`: no `ICON_STOP` on the `first` row;
   - `a_session_rows_menus…`: no 「停止」 / 「セッションを削除」 in the opened
     `···` menu; a right click on the title opens nothing.
2. Frame and header size (`each_project…`, `sessions_in_a_project…` pass after
   the rules too).
3. Rules between rows.
4. The row's first line, right to left first (`a_session_rows_corner…`).
5. The header, right to left first (`a_long_project_name…`).
6. ■ on the active row (`a_running_session_stops_from_its_row`); its
   observable is `BackgroundKey::SessionStop(first)` in `background_tasks`, and
   the suite's `tmux_command` runs on its own `-L operon_suite` server.
7. `session_row_menu` shared by both menus and the title's right click
   (`a_session_rows_menus…`).
8. The pinned queue's frame (`the_needs_you_queue…`).
9. `make q.fast`; the named tests; `cargo test --locked an_empty_project_in_the`
   and `every_`; a mutation sweep with `docs/sdlc/templates/mutations.py`;
   a screenshot of the dev build against a copied store, compared with
   `screen.md`.

The tree compiles after every step.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The row's own click sense (its `scope_builder`, not `clickable_card`) swallows ■ or the menu | stop opens the terminal instead | `a_running_session_stops_from_its_row` asserts `SessionStop(first)` was spawned |
| Right-to-left first changes which widget owns the title's double click (rename) | rename stops working | `a_column_title_double_click_renames_its_session` |
| The frame nests a second scroll width | horizontal overflow in the column | test 3's right-edge assertion; the screenshot |
| change 166's header ＋ moves off its row | the empty project's ＋ is not found | `an_empty_project_in_the_session_column_starts_a_session` |
| `session_verb` per row per frame | allocation in the draw path | called only on the active row and inside an open menu |

## Proof of completion

- The eight named tests — `8 passed`; `an_empty_project_in_the` — `3 passed`.
- `make q.fast` — passes.
- Mutation sweep: drop the project frame, drop the queue frame, shrink the
  name to 12px (`each_project…` asserts 15px), drop the rule, put the title
  first again, put the header name first again, drop ■, drop 停止 from the
  menu, drop セッションを削除 from the menu, drop the title's context menu —
  each turns a test red.
- Screenshots After, dark and light, match `screen.md`.
- `bash scripts/package-macos.sh`, swapped in after landing.

## Departures from the plan
