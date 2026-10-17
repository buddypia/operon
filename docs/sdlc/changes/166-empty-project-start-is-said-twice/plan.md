# Plan: one ＋ per project header, and nothing else, starts a session

- **Spec**: `./spec.md`
- **Approved**: 2026-10-17
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/app/screens.rs` | the three empty branches stop drawing the framed button; 「プロジェクト別」's header glyph becomes `ICON_ADD` / 「セッションを開始」; the grid header gains a right-aligned `ICON_ADD` |
| `src/tests.rs` | `click_start_session` becomes a helper that clicks the `ICON_ADD` level with a named project's rightmost header; the three tests assert no painted text ends in 「セッションを開始」 |

The new helper reads `text_positions` at the `click_at` size, takes the rightmost
painted text equal to the project's name (with the history open, the history is
right of the session column), and clicks the `ICON_ADD` glyph whose `y` is within
half a control height of it and whose `x` is greater. It returns whether one was
found. In 「プロジェクト別」 it also asserts no `ICON_SHOW` on that row, so the old
glyph cannot survive beside the new one.

## Order of work

1. Change the three tests and the helper; run them against today's code. Each
   fails on the new assertion that nothing painted ends in 「セッションを開始」
   (today's framed button paints exactly that); the list and grid tests also
   fail to find an `ICON_ADD` on the history header row. Watch each fail.
2. Change the session column's branch; its test passes.
3. Change 「プロジェクト別」 and 「グリッド」; their tests pass.
4. `make q.fast`; the named tests; a mutation sweep from
   `docs/sdlc/templates/mutations.py`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The framed button survives in one view | two starts for one project | each test asserts no painted text ends in 「セッションを開始」 |
| The header ＋ opens the wrong project | the sheet for the project above | each test asserts `selected_project` is the empty one, drawn below a non-empty one |
| The grid's ＋ lands on the project-page glyph's behaviour | the page changes instead of the sheet opening | the grid test asserts `launch_sheet_open` |
| The search case regresses | a no-match project reads as empty | the history tests keep their search assertions |

## Proof of completion

- `cargo test --locked an_empty_project_in_the` — `3 passed`.
- `cargo test --locked a_project_emptied_by_pinning` — `1 passed`.
- `cargo test --locked every_` — all pass.
- `make q.fast` passes.
- Mutation sweep: restore the framed button in each branch, keep `ICON_SHOW` in
  「プロジェクト別」, drop the grid ＋, drop a click — each turns a test red.
- `bash scripts/package-macos.sh`, swapped in after landing.

## Departures from the plan
