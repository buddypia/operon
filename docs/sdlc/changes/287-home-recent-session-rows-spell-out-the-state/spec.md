# Spec: a Home session row shows its state as a mark, named on hover

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. A session row drawn by `home_session_row` draws no text containing the
   state's word while the pointer is away.
   Test: `home_287_row_draws_no_state_word`.
2. Resting the pointer on the row's state mark shows the state's word, and
   the sentence that says which state it is, in a tooltip.
   Test: `home_287_state_word_appears_on_hover`.
3. A click on the state mark itself still opens the session.
   Test: `home_287_clicking_the_mark_opens_the_session`.

## Behaviour

- The row begins with the state's generated mark (the circled shape of
  change 284: pause for アイドル, progress for 実行中, exclamation for 要対応,
  check for 終了, and so on for all fourteen states) in the state's colour.
  The word that followed it is gone; the title moves left into the space.
- Hover on the mark: two lines, the state's word (`アイドル`) and its existing
  hint (`ターンを終え、次の依頼を出せます`). Both strings exist today.
- `home_session_row` draws three lists — 要対応 and 最近のセッション on Home, and
  the sessions of a project on its page — and all three change together: one
  row, one look. The person named the recent list; the other two are the same
  row and would otherwise differ from it for no reason a reader could see.
- There is no empty, error, too-large or interrupted state of its own: every
  session has exactly one state, and every state has a mark.

## Design

Placement: `EXTEND`. `git grep -n "status_chip("` finds four callers in
`src/app/screens.rs`; only the one in `home_session_row` changes. It becomes
`session_status_icon(ui, palette, status.kind)` — the passive mark with a
full-height hover target that `src/ui/widgets.rs` already documents as
"wording accessible without making it a permanent label" — with
`.on_hover_ui` drawing `status.label` and then `status.hint`, the word before
the sentence, as the session tab's hover does. `status_chip` stays for its three other callers. No new symbol,
string, constant, persisted shape, external command, or background work; the
draw path gains no allocation (`status_view` already builds both strings).

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | `session_status_icon` paints with `palette.status(kind.tone())`, an existing role (`.claude/rules/palette-and-glyphs.md` read) |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | the generated `StatusMark` of change 284 is reused; no glyph is written |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | no | no identifier is added |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | nothing persisted is read or written differently |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | no process is started |
| Documentation — user-facing docs change in all three languages together | no | no user-facing document describes the row's state word |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | no | drawing only |
| Budgets — any new scan or output path states its byte and item ceiling | no | no scan or output path |

## Flagged concerns

None.

## Acceptance

- `cargo test --locked home_287_` passes: three tests.
- `make q.fast` passes.
- In the packaged app, Home's 最近のセッション rows begin with a state mark and
  no word; resting the pointer on the mark shows the word.

## Rejected alternatives

- Keep the font glyph and only drop the word: the tabs and the column moved
  to generated marks in change 284, and Home would be the last place a
  session's state is a font glyph.
- Change only the recent list: the same function draws 要対応 and the project
  page, and a flag to keep the word there adds a difference nobody asked for.
- Hover shows the word alone: the hint is what tells fourteen states apart
  under four words, and it is on hover today.
