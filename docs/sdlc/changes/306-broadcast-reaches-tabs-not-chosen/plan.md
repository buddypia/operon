# Plan: the idle agents an instruction reaches are the ones ticked

- **Spec**: `./spec.md`
- **Approved**: 2026-11-17
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/app/broadcast.rs` | `BroadcastSheet.chosen`; `broadcast_recipients`; `send_broadcast` takes the choice; the sheet draws checkbox rows in a scroll area and the 「すべて選択」/「選択を解除」 control; `BROADCAST_NAMED` becomes `BROADCAST_VISIBLE_ROWS`. |
| `src/tests.rs` | Six `broadcast_306_` tests; change 282's send and sheet tests tick before sending. |
| `src/i18n_tables.rs` | Four new rows in each table; the rows for 「{p0} · 送信先 {p1} 件」 and 「全員に送る内容を入力」 removed. |
| `README.md`, `README.ja.md`, `README.ko.md` | The sentence says the line reaches the ticked agents. |
| `docs/MANUAL.md`, `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md` | The paragraph says the same, and names 「すべて選択」. |
| `docs/sdlc/changes/306-broadcast-reaches-tabs-not-chosen/` | `verification.yaml`, `mutations.py`, `review.yaml`, `approvals.log`. |

## Order of work

1. Write the six `broadcast_306_` tests against the new signatures and watch
   them fail (they do not compile until step 2; the failure watched is the
   compile error naming `chosen` / `broadcast_recipients`, then the assertion
   failures with stub bodies).
2. `BroadcastSheet.chosen`, `broadcast_recipients`, `send_broadcast(text,
   chosen)`; update the two 282 tests. The tree compiles from here.
3. The sheet: count line + toggle-all control, scroll area of checkbox rows,
   the button counting recipients, the hint.
4. Strings in both tables; remove the two dead rows.
5. The six documents.
6. `make q.fast`, `cargo test --locked broadcast_`, the mutation sweep.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A tick is applied while the row list is borrowed, or applied twice in one frame | a row that will not toggle, or toggles back | `broadcast_306_sheet_opens_with_nobody_chosen_and_a_row_ticks` clicks it once and reads the choice |
| The send still reads every target | an unticked tab receives the line — the very complaint | `broadcast_306_send_reaches_only_the_chosen_idle_agents`, and the mutation that drops the filter |
| A late-idle session lands ticked | the surprise of change 282 returns | `broadcast_306_an_agent_idle_after_opening_is_not_chosen` |
| The checkbox paints at a size or colour outside the design system | a guard about buttons, type scale, or roles fails | `every_button_keeps_a_documented_height`, `every_text_size_is_a_level_on_the_type_scale` in `make q.fast` |
| Rows past the scroll area's visible height are culled, so a test reading painted text misses them | a test that passes or fails for the wrong reason | requirement 6 is checked through the button's count and the send, not through painted rows |
| A dead message row is left in a table, or a used one removed | the message-table guard fails | the i18n table tests in the suite |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked broadcast_` — `13 passed; 0 failed`.
- `python3 docs/sdlc/changes/306-broadcast-reaches-tabs-not-chosen/mutations.py`
  — every mutation caught.
- The full suite on the branch push in CI.
- In the running app, from the packaged build: ⌘K → 「アイドル中のエージェントへ送信…」
  shows unticked rows and 「0 件に送信」 disabled; ticking one and Enter sets only
  that agent working.

## Departures from the plan

- **Two mutations escaped the first sweep** — the send button left enabled with
  nobody ticked, and the rows capped at six. Neither changes what is sent, so
  the tests had not looked. Requirement 4's test now reads the button's state
  from the accessibility tree (with a ticked positive control), and
  requirement 6's scrolls the list until its last row paints.
- **Finding F1** (empirical-verifier): an Enter that sent nothing — nobody
  ticked — kept the sheet and the line but left the field without the keyboard,
  so the next keystrokes were lost. The sheet now hands the keyboard back after
  such an Enter; `broadcast_306_f1_focus_stays_after_enter_with_nobody_chosen`
  pins it, and the sweep has a mutation for it.
- **The draw path filters the targets it already holds** instead of calling
  `broadcast_recipients`, which walked the sessions a second time each frame
  (rust-reviewer). The send path still calls it.
- Proof count: `cargo test --locked broadcast_` is 14 with F1.
