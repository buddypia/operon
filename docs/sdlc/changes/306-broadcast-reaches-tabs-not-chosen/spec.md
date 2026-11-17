# Spec: the idle agents an instruction reaches are the ones ticked

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. The sheet opens with no session chosen. It lists every idle session of the
   project in view (who may be chosen: change 282's requirement 1, unchanged),
   each row a checkbox, its agent's mark and its title; ticking the box, or
   clicking the title, chooses that session, and doing it again unchooses it.
   The line under the title reads 「{project} · 選択 {n} / {m} 件」 — chosen
   and listed — and the field's hint 「選んだ相手に送る内容を入力」. —
   `broadcast_306_sheet_opens_with_nobody_chosen_and_a_row_ticks`
2. Sending queues the line, trimmed, then Enter, through
   `queue_terminal_input` only for the sessions that are both chosen and still
   idle, in list order; an idle session left unticked, and a ticked one that
   has since started working or waiting on a prompt, receive nothing; the
   notice 「{n} 件のエージェントに送信しました」 counts who received it. —
   `broadcast_306_send_reaches_only_the_chosen_idle_agents`
3. A session that falls idle while the sheet is open is listed unticked, and
   receives nothing until it is ticked. —
   `broadcast_306_an_agent_idle_after_opening_is_not_chosen`
4. With nobody chosen, the button reads 「0 件に送信」 and is disabled, Enter in
   the field sends nothing and says nothing, and the sheet stays open with what
   was typed. — `broadcast_306_nobody_chosen_sends_nothing_and_keeps_the_line`
5. 「すべて選択」 chooses every listed session; while every listed session is
   chosen the same control reads 「選択を解除」 and unchooses them all. —
   `broadcast_306_choose_all_then_clear`
6. Every idle session can be chosen: none is counted away as 「ほか N 件」, and
   past six rows the list scrolls rather than growing the sheet. —
   `broadcast_306_every_idle_agent_can_be_chosen`

Change 282's tests that sent to every idle agent without a choice
(`broadcast_282_send_types_into_every_idle_agent`,
`broadcast_282_sheet_names_who_it_reaches_and_sends_on_enter`) now tick the
sessions first; what they pin besides — the trim, the Enter, the notice, the
IME rule, closing after a send — is unchanged.

## Behaviour

Strings, new: 「{project} · 選択 {n} / {m} 件」; 「すべて選択」; 「選択を解除」;
the hint 「選んだ相手に送る内容を入力」. Kept: the title
「アイドル中のエージェントへ送信」, 「アイドル中のエージェントはいません」,
「キャンセル」, 「{n} 件に送信」 (n is now the chosen and idle count), and the
notice 「{n} 件のエージェントに送信しました」. Gone from this sheet:
「{project} · 送信先 {n} 件」, 「ほか {n} 件」 and 「全員に送る内容を入力」.

- **Opening**: every row unticked; the field has the keyboard, so a line can be
  typed before or after choosing.
- **Empty** (nobody idle): 「アイドル中のエージェントはいません」 in place of the
  count line, no rows and no 「すべて選択」, the field and the button disabled —
  as in change 282.
- **Changing while open**: the rows are the idle sessions at this frame. A
  session that falls idle appears unticked. A ticked session that starts
  working leaves the list and the count; its tick is kept, so if it finishes
  again while the sheet is open it returns ticked — the person chose it. A
  closed session is simply gone.
- **Nobody chosen**: 「0 件に送信」 disabled; Enter does nothing visible and the
  line stays.
- **Too large**: every idle session is a row; past six the list scrolls inside
  the sheet, whose width stays 460px and whose height stops growing.
- **Error**: a send tmux refuses is reported per session by the existing notice
  「ターミナルに入力を送れませんでした: …」.
- **Interrupted**: キャンセル or Escape closes the sheet and forgets the choice
  and the line; nothing was sent. A click on the scrim does nothing.

## Design

Placement: **EXTEND** — `src/app/broadcast.rs` already owns the sheet
(`git grep -n "fn send_broadcast"`); route `modify`.

- `BroadcastSheet` gains `chosen: Vec<Uuid>`, empty on open. Not persisted.
- `broadcast_targets()` stays: who may be chosen.
- New `broadcast_recipients(&self, chosen: &[Uuid]) -> Vec<Uuid>`: the targets
  that are in `chosen`, in target order.
- `send_broadcast(&mut self, text: &str, chosen: &[Uuid]) -> usize` sends to
  `broadcast_recipients(chosen)` instead of every target.
- `ui_broadcast_sheet`: the count line and a `quiet_button` for
  「すべて選択」/「選択を解除」 share one row; the recipients are drawn in an
  `egui::ScrollArea::vertical()` whose `max_height` is
  `BROADCAST_VISIBLE_ROWS` (6, replacing `BROADCAST_NAMED`) rows; each row is
  `ui.checkbox(&mut ticked, "")`, `agent_chip`, and the title as an
  `egui::Label` that senses a click. A toggle is applied after the rows are
  drawn, as `screens.rs`'s changed-file list does.
- `src/i18n_tables.rs`: the four new strings in the English and Korean tables;
  the two that no caller uses any more are removed.
- `README.md`, `README.ja.md`, `README.ko.md`, `docs/MANUAL.md`,
  `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md`: the sentence and the paragraph say
  the line reaches the ticked agents.
- No new external command; input still goes through `queue_terminal_input`.
  The draw path, while the sheet is open, computes the targets (as before) and
  one `contains` per row against a `Vec` of a few ids; nothing when closed.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | Existing roles only: `raised`, `border`, `scrim`, `text_strong`, `text_muted`, and `link` through `quiet_button`; the checkbox paints with the visuals `apply_appearance` sets, as the commit list's does. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No glyph is written: `agent_chip` and `ICON_START` exist, and the tick is egui's checkbox shape. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | No identifier is shared; the scroll area's id salt is used in one place. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | Nothing is stored; the choice dies with the sheet. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No new child; the existing input queue sends. |
| Documentation — user-facing docs change in all three languages together | Yes | The README and the manual, in English, Japanese and Korean. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | No | None. |
| Budgets — any new scan or output path states its byte and item ceiling | No | No scan or output path. The rows are the project's idle sessions, already walked every frame while the sheet is open; the scroll area bounds the height, not the count. |

## Flagged concerns

- **The person must now tick before sending** — one more click for the common
  case of telling everyone. Resolved: 「すべて選択」 is one click, and the
  person chose an empty start over a full one when shown both.
- **A re-arranged modal** — a person's to approve. Resolved: the person was
  shown the sheet with checkboxes and three default choices, and chose
  「何も選ばない」 (approvals.log).

## Acceptance

- `cargo test --locked broadcast_` passes: the six `broadcast_306_` tests and
  change 282's seven, thirteen in all.
- `make q.fast` passes; the message-table, button-height, type-scale and
  three-language guards pass.
- `python3 docs/sdlc/changes/306-broadcast-reaches-tabs-not-chosen/mutations.py`
  catches every mutation.
- In the running app: with two idle agents, ⌘K → the row shows both unticked
  and 「0 件に送信」 disabled; ticking one and Enter sets only that one working.

## Rejected alternatives

- Every idle tab ticked on opening — the person did not choose it, and it keeps
  the surprise for a tab just opened.
- Only the tab in view ticked — the person did not choose it.
- Choosing by multi-selecting tabs in the tab strip — a second selection model
  on the strip, for an act that starts in the palette.
- Leaving out sessions whose agent has never had a turn — it guesses which tabs
  the person means, and still sends to a tab they did not pick.
