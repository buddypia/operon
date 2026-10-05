# Spec: Home shows what waits first, and one line per session

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Home always draws the 要対応 line directly under the page header, above the
   tiles. With `attention_count() == 0` it is a card holding `ICON_STATUS_DONE`
   in `palette.success`, 「あなたを待っているセッションはありません」, and muted
   「要対応になるとここに先頭表示されます」. With n > 0 it is today's warning card
   (sentence and 「要対応のセッションを見る」 unchanged), followed inside the card
   by one recent row (req 3) per session for which
   `session_in_status_group(session, NeedsYou)` holds, newest first, at most
   five.
2. Recent sessions (newest first, at most six, as today) are drawn with the
   recent row, not `session_card`. `session_card` stays for its other callers.
3. The recent row is a new method `home_session_row(&mut self, ui, session,
   list: &'static str)`: one `clickable_card` salted with
   `("home-row", list, session.id)` — a waiting session is usually also a
   recent one, so it is drawn twice on one frame, and the list name keeps the
   two rows' ids (card, ··· menu, removal confirmation) apart. The waiting
   rows pass `"waiting"`, the recent rows `"recent"`. Its content is a single
   horizontal line — status
   chip, title (strong, truncated to the space left), agent chip, project name
   (muted), branch with `ICON_BRANCH` in `palette.branch` when present,
   relative time (muted); right-aligned: the action, then a `ICON_MORE` menu.
   Clicking the row outside its controls calls `open_session_in_terminal`, as
   the card did.
4. The action is `session_verb(session, cancellation_pending)` when its action
   is `Start` or `ResumeNative` (drawn with `verb_button`, hover = its hint,
   dispatched as `session_card` dispatches); otherwise a secondary 「開く」
   calling `open_session_in_terminal`.
5. The ··· menu (hover 「その他の操作」) holds, in order: the `Stop` verb when
   `session_verb` returns one (same label and dispatch); 「ターミナルを閉じる」 /
   「残ったターミナルを閉じる」 for `Exited` / `Failed`, calling
   `close_completed_terminal`; 「セッションを削除」 calling
   `request_session_removal`; then, as small weak labels, what the card showed
   below its first line: the display goal when it differs from the title,
   `tmux: <name>`, `cwd: <path>`, the native resume line (same text and helper
   as the card) or 「ネイティブ再開 ID を待機中」, the dependency count, and the
   blocked-by line. `session_removal_confirmation` is drawn under the row.
6. Beside the recent list, a right column 300 px wide headed
   「プロジェクトから始める」 lists every project in store order: `ICON_PROJECT`
   and the name, and a secondary `ICON_ADD` 「セッション」 button calling
   `open_project_session_setup(project.id)`. With no project the column is not
   drawn. When the available width is under 720 px the column is drawn under
   the recent list instead of beside it.
7. The empty states are today's, drawn in the left column.
8. Nothing is persisted; no store field changes.

## Behaviour

See `./screen.md`. New message ids, each with EN and KO rows:
「あなたを待っているセッションはありません」, 「要対応になるとここに先頭表示されます」,
「プロジェクトから始める」, 「開く」, 「その他の操作」 (row added if missing).
「セッション」 already has rows.

## Design

- `src/app/screens.rs`: `ui_home` restructured (要対応 line always, tiles,
  two columns); new `home_session_row`; `session_card` untouched.
- `src/i18n_tables.rs`: the new rows, in byte order.
- `src/tests.rs`: new tests below.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | yes | No new role: `success`, `warning`, `branch`, `text_muted` exist. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | yes | Existing constants only (`ICON_STATUS_DONE`, `ICON_MORE`, `ICON_BRANCH`, `ICON_PROJECT`, `ICON_ADD`). |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | no | No new identifier; actions reuse `session_verb` and existing methods. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | Nothing persisted changes. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | No new child; the row calls the same methods the card called. |
| Documentation — user-facing docs change in all three languages together | yes | README Home description checked in all three; changed together if it describes the cards. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | no | UI only. |
| Budgets — any new scan or output path states its byte and item ceiling | no | No scan; rows are capped at six recent and five waiting. |
| Draw path — no I/O, no avoidable per-frame allocation | yes | The row reads in-memory state only; the project-name lookup is per row as in the card; the ··· details are built only while the menu is open. |

## Flagged concerns

- **Controls on a card that are gone from its face** — close-terminal, delete,
  stop move into ···. Resolved: the mock puts internal information and rare
  actions behind ···; each keeps its label and handler, and the session screen
  still carries stop.
- **Tests that draw `ui_home`** — `a_group_tile_filters_the_session_list_to_that_group`
  finds the tile's count by the text "3"; the recent rows draw no bare "3", so
  it stays unambiguous.

## Acceptance

- `cargo test --locked` passes, including new tests
  `home_always_says_whether_anything_waits` (Home drawn headless with no
  waiting session draws 「あなたを待っているセッションはありません」; with one
  session marked unread through `mark_session_unread` — so it is in 要対応 and
  also among the recent six — it does not, and draws the warning sentence and
  that session's title twice, once inside the 要対応 card and once in the
  recent list, with no egui id clash warning in the frame's output),
  `a_recent_session_is_one_line_with_its_details_behind_the_menu` (Home drawn
  with a session carrying a tmux name and cwd: no drawn text starts with
  `tmux:` or `cwd:`; the title, project name, and 「開く」 are drawn at one
  height within 4 px; clicking 「開く」 selects that session's terminal), and
  `starting_from_a_project_on_home_opens_that_projects_launch_form` (two
  projects, the second's 「セッション」 button clicked: `selected_project` is the
  second, `project_tab` Overview, page Projects).
- `cargo clippy --locked -- -D warnings` and `cargo fmt --check` are clean.
- In the installed app, Home matches `./screen.md`.

## Rejected alternatives

- Collapsing the existing card behind a disclosure — still one card per
  session, still tall when opened, and two shapes for one list.
- Dropping the internal details — they are how a person attaches from a shell;
  the mock keeps them behind ···.
