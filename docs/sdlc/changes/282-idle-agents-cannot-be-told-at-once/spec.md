# Spec: one instruction to every idle agent of the project

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. The instruction reaches the sessions of the project in view — the
   workspace's, else the selected one — whose agent has finished its turn
   (`SessionStatusKind::Idle`, a state only a running terminal has); not one that works, waits on a prompt, has
   ended, or belongs to another project; nobody when no project is in view. —
   `broadcast_282_reaches_only_the_projects_idle_running_agents`
2. Sending queues the text, trimmed, then Enter, for each of them through
   `queue_terminal_input`, says 「N 件のエージェントに送信しました」, and leaves
   the screen where it was. — `broadcast_282_send_types_into_every_idle_agent`
3. An empty text, or nobody idle, sends nothing and says nothing. —
   `broadcast_282_nothing_to_say_or_nobody_idle_sends_nothing`
4. The command palette has the row 「アイドル中のエージェントへ送信…」, and
   choosing it opens the sheet. — `broadcast_282_palette_opens_the_sheet`
5. The sheet shows its title, the project with the number of recipients, and
   the recipients' titles; its field has the keyboard when it opens; Enter
   sends and closes it; the Enter that confirms an IME conversion does
   neither. — `broadcast_282_sheet_names_who_it_reaches_and_sends_on_enter`
6. Escape closes the sheet without sending; with nobody idle the sheet says
   so, its Enter sends nothing, and it stays open. —
   `broadcast_282_sheet_closes_on_escape_and_waits_when_nobody_is_idle`
7. While the sheet is open the app-wide chords that the launch sheet holds
   back are held back for it too — the palette, a new session, the page
   chords — so a half-typed line is not covered or left behind. —
   `broadcast_282_f1_page_shortcuts_do_not_reach_past_the_sheet`

## Behaviour

Strings: the palette row 「アイドル中のエージェントへ送信…」; the title
「アイドル中のエージェントへ送信」; 「{project} · 送信先 {n} 件」; up to six
recipients by title with their agent's mark, then 「ほか {n} 件」; the field's
hint 「全員に送る内容を入力」; the buttons 「キャンセル」 and 「{n} 件に送信」;
the notice 「{n} 件のエージェントに送信しました」.

- **Empty** (nobody idle): 「アイドル中のエージェントはいません」 in place of the
  count, the field and the send button disabled; the sheet stays until
  cancelled, and shows recipients as soon as an agent becomes idle.
- **Changing while open**: the list is read each frame, so the count on the
  button is who the send reaches at the moment it is pressed.
- **Error**: a send tmux refuses is reported per session by the existing
  notice 「ターミナルに入力を送れませんでした: …」.
- **Too large**: one line; more than six recipients are counted, not listed.
- **Interrupted**: closing the sheet (キャンセル, Escape) drops what was
  typed; nothing was sent. A click outside the sheet does nothing.

## Design

Placement: **NEW_IN_MODULE** — a new sheet beside the launch sheet
(`git grep -n ui_launch_sheet`), written in a new file under `src/app/`
because `src/app.rs` and `src/app/screens.rs` stand near the
`largest_module_lines` ceiling.

- `src/app/broadcast.rs`: `BroadcastSheet { text, focus }`;
  `BROADCAST_NAMED = 6`; `impl OperonApp { open_broadcast_sheet(),
  broadcast_project() -> Option<&Project>, broadcast_targets() -> Vec<Uuid>,
  send_broadcast(text) -> usize, ui_broadcast_sheet(ctx) }`. The sheet is
  drawn as the launch sheet is: a scrim `Area` that takes clicks, a
  foreground `Area` with a `Frame` in `palette.raised`; buttons through
  `primary_button` and `secondary_button`; the Enter rule is change 279's
  `reply_enter_sends`.
- `src/app.rs`: field `broadcast_sheet: Option<BroadcastSheet>` (not
  persisted); `update` calls `ui_broadcast_sheet` after `ui_launch_sheet`;
  `mod broadcast`.
- `src/app/keymap.rs`: `handle_app_shortcuts` treats the open sheet as it
  treats the open launch sheet, in its three guards.
- `src/app/screens.rs`: `palette_actions` gains the row (eleven rows) and
  `activate_palette_entry` its arm.
- `src/i18n_tables.rs`: seven strings in the English and Korean tables
  (「キャンセル」 and 「ほか {p0} 件」 exist).
- `CLAUDE.md`: the module map names the new file.
- `README.md`, `README.ja.md`, `README.ko.md`, `docs/MANUAL.md`,
  `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md`: a sentence or paragraph each.
- No persisted shape; no new external command — `queue_terminal_input`'s
  existing `tmux send-keys`, whose text handling change 280 corrected. While
  the sheet is open the frame computes the recipients (one pass over the
  sessions and a small `Vec`); nothing when it is closed.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | Existing roles only: `raised`, `border`, `scrim`, `text_strong`, `text_muted`. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | `ICON_START`, already the mark of the button that starts work. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | The palette's dispatch name is written in the list and in the arm, as every row's is; the test finds the row by its label and activates the name the list gives. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | Nothing is stored. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No new child; the existing input queue sends. |
| Documentation — user-facing docs change in all three languages together | Yes | The README and the manual, in English, Japanese and Korean. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | No | None. |
| Budgets — any new scan or output path states its byte and item ceiling | No | None. |

## Flagged concerns

- **An instruction sent to more agents than meant** — one Enter types into
  several terminals. Resolved: the sheet names the recipients and the button
  carries their number before anything is sent; only the project in view is
  reached; working and prompting agents are never typed into.
- **A new modal** — a person's to approve. Resolved: the person was shown the
  sheet and chose it (approvals.log).

## Acceptance

- `cargo test --locked broadcast_282_` passes (seven tests).
- `make q.fast` passes; the palette, message-table, module-map and
  three-language guards pass.
- `python3 docs/sdlc/changes/282-idle-agents-cannot-be-told-at-once/mutations.py`
  catches every mutation.
- In the running app: with two idle agents, ⌘K → the row → a line and Enter
  sets both working.

## Rejected alternatives

- Every project at once — a wider reach than the person chose.
- A field in the toolbar — always on screen for an occasional act.
