# Spec: 「閉じる」 in the session row's and the session tab's right-click menu

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. R1: the right-click menu of a session row in the session page's left column
   offers 「閉じる」 when the session is not pinned. Choosing it closes that
   session's tab: the row and the tab are both gone, the session stays in the
   store with its status unchanged, and ⇧⌘T reopens it.
   `session_286_row_menu_closes_the_tab`.
2. R2: the right-click menu of a session tab's name offers 「閉じる」 when the
   session is not pinned, with the same effect as that tab's ×.
   `session_286_tab_menu_closes_the_tab`.
3. R3: on a pinned session neither menu offers 「閉じる」.
   `session_286_a_pinned_session_offers_no_close`.
4. R4: closing, from the column, a session whose project is not the one shown
   closes its tab in that project's workspace: its row goes, the shown
   project's tabs are untouched, and the shown project does not change.
   `session_286_closing_a_row_of_another_project_keeps_the_shown_one`.

## Behaviour

- Right-click a row → 「名前を変更」, 「ピン留め」, 「閉じる」. Choose 「閉じる」:
  the menu closes, the row and its tab disappear, the next tab of that group
  comes to the front (what × does today).
- Right-click a tab's name → the same three entries, the same effect.
- Pinned session: 「名前を変更」, 「ピン留めを外す」 only.
- The string is 「閉じる」, already in every table of `src/i18n_tables.rs`
  (Close / 닫기).
- Empty: closing a project's last row leaves 「開いているセッションはありません」.
- Error, too-large, interrupted: none — closing a tab runs no command, writes
  no file, and is one in-memory step. Nothing is reported to the person because
  nothing can fail.

## Design

Placement: `EXTEND`. `git grep -n context_menu src/app/screens.rs` shows the
row's menu (`row.context_menu`, session page column) and the tab's menu
(`label.context_menu`, tab strip), each with 名前を変更 and ピン留め;
`close_workspace_tab` in `src/app.rs` is the one close a person asks for and
`session_285_a_pinned_tab_does_not_close` pins its refusal.

- `src/app/screens.rs`: both menus gain `quiet_button(ui, palette, tr("閉じる"))`
  when `!pinned`. The tab's sets the strip's existing `close` to the tab; the
  column's sets a new local `close: Option<Uuid>` handled after the column is
  drawn, beside `pin` and `open`.
- `src/app/session_order.rs`: `close_session_tab(&mut self, id: Uuid)`. When
  the shown workspace holds the tab it calls `close_workspace_tab`; otherwise,
  unless pinned, it closes the tab in the owning entry of `project_workspaces`
  and remembers it for ⇧⌘T.
- `src/app.rs`: the lines that remember a closed tab become
  `remember_closed_tab`, called from both.
- No persisted shape changes, no external command, nothing on a background
  thread; the draw path gains one `bool` read per open menu.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | no colour is named; `quiet_button` paints with existing roles |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | the entry is text only |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | no | no identifier is added |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | nothing is persisted; the store, cancellation, and tmux paths are not touched |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | no child process |
| Documentation — user-facing docs change in all three languages together | no | the README files do not describe these menus (`git grep -n ピン留め -- 'README*'` is empty) |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | none added |
| Budgets — any new scan or output path states its byte and item ceiling | yes | the remembered closed tabs keep `CLOSED_TAB_LIMIT` through the shared helper |

## Flagged concerns

None.

## Acceptance

- `cargo test --locked session_286` passes: the four tests above.
- `cargo test --locked session_285` still passes.
- `make q.fast` passes.
- In the app: right-click a row and a tab name, choose 「閉じる」, the tab and
  row go; pin one, the entry is absent.

## Rejected alternatives

- A × button on each row: the row is one click-and-drag target, and a button
  inside it would take clicks meant to open or drag the row; the person asked
  for the tab's 「右クリックで出てくる閉じる」 and "同じく" for the row.
- Showing 「閉じる」 on a pinned session and refusing with the notice: an entry
  that never works; the tab's × is already absent there.
- 「タブを閉じる」 as the label: the person's word is 閉じる, and in the column
  there is no tab under the pointer.
