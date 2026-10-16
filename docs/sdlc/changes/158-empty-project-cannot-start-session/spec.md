# Spec: an empty project offers 「セッションを開始」 in place of its empty line

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. In the session column (`ui_terminal_session_tabs`), a project with no
   sessions and none pinned above draws an `icon_text_button` with `ICON_ADD`
   and 「セッションを開始」 where the line 「セッションはまだありません」 was, and no
   longer draws that line. Clicking it opens the launch sheet with that project
   selected. Test: `an_empty_project_in_the_session_column_starts_a_session`.
2. In the history's 「プロジェクト別」 view (`ui_project_session_list`), a project
   with no sessions and no search draws the same button in place of
   「セッションはまだありません。」 and the small glyph beside it; clicking it opens
   the launch sheet for that project. With a search that matches none of the
   project's sessions, it draws 「一致するセッションはありません。」 and no button —
   new in this view, which today draws the empty line and glyph under a search
   too. The view's aggregate 「検索条件に一致するセッションはありません。」 at the
   bottom stays; when nothing matches anywhere both are drawn, the per-project
   line saying which project, the aggregate saying the whole search.
   Test: `an_empty_project_in_the_history_list_starts_a_session`.
3. In the history's 「グリッド」 view (`ui_session_grid`), a project with no
   sessions and no search draws the same button in place of
   「セッションはまだありません。」; clicking it opens the launch sheet for that
   project. A search with no match keeps 「一致するセッションはありません。」.
   Test: `an_empty_project_in_the_history_grid_starts_a_session`.
4. 「セッションを開始」 ("Start a session" / "세션 시작") and
   「一致するセッションはありません。」 ("No matching sessions." / "일치하는 세션이
   없습니다.") have rows in `EN_TABLE` and `KO_TABLE`; neither has one today, so
   both fall back to Japanese. `every_message_id_has_a_row_in_every_table`
   compares the tables with each other and would not notice a missing row, so
   the guard is requirement 1's test asserting `translation_for` for both
   strings in `Language::En` and `Language::Ko` — through the lookup, not the
   process-wide active language, which parallel tests share.
   `every_translation_table_is_sorted_by_message_id` guards the order.

## Behaviour

Japanese strings a person reads:

- 「セッションを開始」 — the button, preceded by the `ICON_ADD` glyph. The header's
  existing `ICON_ADD` glyph keeps 「セッションを開始」 as its hover text.
- 「一致するセッションはありません。」 — for a search that hides all of a
  project's sessions in the history views: unchanged in 「グリッド」, new per
  project in 「プロジェクト別」.

States:

- **Empty project, no filter**: the button, indented as the line was.
- **Empty because the status chips filter it** (session column): the project is
  skipped entirely, as today — the chips hide it, and offering a launch inside a
  filtered view would add rows the filter exists to remove.
- **Emptied by pinning** (session column): the project's sessions are drawn in
  the 要対応 section; no button, no line — as today.
- **Click while tmux is missing, the agent is not on `PATH`, or a worktree
  operation runs**: the sheet opens as it does from the header glyph, and its own
  footer says why the launch button is disabled. Nothing new.
- **Many empty projects**: one hairline button per project. Each is 28px, the
  height the line and its spacing already took within a few pixels.

Failure reaches the person nowhere new: the click only opens the sheet.

## Design

Placement verdict: **EXTEND**. `git grep -n "セッションはまだありません" src`
finds the three per-project empty lines in `src/app/screens.rs`
(`ui_terminal_session_tabs`, `ui_project_session_list`, `ui_session_grid`), and
each already reaches `open_project_session_setup(project.id)` from a header
control. Route `modify`.

- `src/app/screens.rs`: the three empty branches call
  `icon_text_button(ui, ICON_ADD, tr("セッションを開始"))` and, on click,
  `self.open_project_session_setup(project.id)`. In `ui_project_session_list`
  and `ui_session_grid` the branch splits on `needle.is_empty()`. The
  small `ICON_ADD` beside the old line in `ui_project_session_list` goes: the
  button replaces both.
- `src/i18n_tables.rs`: two rows in each of `EN_TABLE` and `KO_TABLE`.
- `src/tests.rs`: three tests; they draw through the `pub(crate)`
  `ui_terminal_session_tabs` and `ui_terminal_workspace` (with
  `session_library_open` set, which draws `ui_sessions`).
- No persisted-shape change, no external command, no background task. The draw
  path gains one button per empty project and no allocation beyond what `tr` and
  `icon_text_button` already do.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | No colour is named; `icon_text_button` takes the context's widget visuals. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | Reuses `ICON_ADD`, already in `ICON_VOCABULARY`, already the add-session mark in the header. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | No identifier; the label is a message id looked up through `tr`. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | Nothing persisted changes; the sheet is the existing one. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No child is spawned; the click opens the sheet, whose launch already passes the gates. |
| Documentation — user-facing docs change in all three languages together | No | `README.md` does not describe the empty line (checked: `grep -n "まだありません" README*`); the message tables gain EN and KO rows together. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | No | None added. |
| Budgets — any new scan or output path states its byte and item ceiling | No | No scan or output path. |

## Flagged concerns

- **Launch immediately, or open the sheet?** The request says 「すぐ」. A
  one-click start with the remembered settings would skip the dangerous-mode
  acknowledgement and the argument checks the sheet enforces, and every other
  start in the app goes through the sheet. Answered: the sheet, prefilled, which
  makes the start one click plus ⌘↩. Recorded under Rejected alternatives.
- **Two controls for one action on an empty project's two rows.** The header
  glyph stays because a project with sessions needs it and moving it between
  states would shift the header; the button below is the discoverable one.
  Answered: keep both; they call the same function.

## Acceptance

- `cargo test --locked an_empty_project_in_the` passes the three new tests, and
  `cargo test --locked a_project_emptied_by_pinning_does_not_claim_it_has_no_sessions`
  and `cargo test --locked every_` pass.
- `make q.fast` passes.
- In the packaged app, a newly added project shows 「セッションを開始」 under its
  name in the session column; clicking it opens the launch sheet for it.

## Rejected alternatives

- One-click launch with the last settings — bypasses the sheet's danger
  acknowledgement and checks (see Flagged concerns).
- A dashed full-width "ghost row" painted by hand — a new widget for one state;
  `icon_text_button` is the documented helper for a word with its icon.
- `quiet_button` (link text, no frame) — `DESIGN.md` reserves it for the action
  beside a heading; the hairline is what makes this read as a button at rest
  (change 143).
- Hiding the header glyph on an empty project — the header would change shape
  between states.
