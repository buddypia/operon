# Spec: tab contents stop repeating their tab as a heading

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. In `ui_git`, `ui_worktrees`, and `ui_pull_requests`, the large heading
   that repeats the tab (`ui.heading("Git")`, `ui.heading("worktree")`,
   `ui.heading(tr("プルリクエスト"))`) is gone. The rest of each header row
   stays: the Git view switches and refresh, the worktree refresh, and the
   PR row's 「認証済み GitHub CLI 経由」 note with its refresh. Pinned by
   `tab_content_no_longer_repeats_its_tab_as_a_heading`.
2. On the Sessions page, `ui_sessions` drops `ui.heading(tr("セッション"))`;
   the refresh, the プロジェクト別／グリッド switch, and the search field
   stay on the row. Pinned by the same test as requirement 1.
3. The session list title in `ui_terminal_session_tabs` is demoted, not
   deleted: it is also the docked session panel's title, where no tab above
   it names it. `ui.heading(tr("プロジェクト別セッション"))` becomes a
   12.5 `text_muted` label, the panel-label form change 245 set. Pinned by
   `session_list_title_is_a_panel_label_not_a_heading`.
4. Inside エージェント設定, `ui_skills` and `ui_rules` draw スキル and
   ルールと指示 as section headings: 15.0 strong in `text_strong` (the
   `DESIGN.md` section level), not `ui.heading`. Their refresh buttons stay.
   Pinned by `agent_settings_sections_use_section_headings`.
5. The settings sidebar's 設定 label is demoted from 20.0 strong to the
   panel-label form (12.5, `text_muted`): the page title on that screen is
   the `page_header` beside it. Pinned by
   `settings_sidebar_label_is_a_panel_label`.
6. Nothing else moves: tabs and their selected marks, `page_header`,
   empty-state headings, the store, and every control's behaviour are
   unchanged. No string is added or removed from what a person reads, except
   the repeated headings in requirements 1–2. The kept controls are pinned
   by `tab_headings_keep_their_controls`.

## Behaviour

- Happy path: each project tab opens directly with its controls and
  content; the Sessions page opens with its switch and search row; skills
  and rules read as two sections under エージェント設定; the settings
  sidebar reads as a panel label over its section list.
- Empty and loading states are unchanged: they never carried these
  headings as their message, and the empty-state headings elsewhere
  (「まずはプロジェクトを開いてください」) are untouched.
- Long text: no heading remains to truncate in the changed rows; the PR
  note 「認証済み GitHub CLI 経由」 already draws small beside the refresh.
- The person approved this classification in the proposals HTML
  (keep / remove / demote) and said 「実装を進めて」; `screen.md` records
  the same layout in text.

## Design

Placement verdict: `EXTEND`. The behaviour exists in
`src/app/screens.rs`; the heading level it converges on is the one
`page_header` (20px page title) and change 245's column label
(12.5 muted panel label) already established. No new module, type,
string, icon, or persisted shape.

- Headings removed in requirements 1–2 are `ui.heading` calls in a
  horizontal row; the row and its remaining controls stay, so no layout
  reflows beyond the heading's own width.
- Demotions use `RichText` size and `Palette` roles only: 15.0 strong
  `text_strong` for in-page sections (requirement 4), 12.5 `text_muted`
  for panel labels (requirements 3 and 5).
- Requirement 3 is a recorded departure from the proposals' "remove"
  list: `ui_terminal_session_tabs` is drawn both under the Sessions
  page's (removed) heading and as the docked session panel, where the
  title is the only name the panel has. Demoting keeps one name at panel
  strength in both places.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No new role | Only existing `Palette` roles (`text_strong`, `text_muted`) and sizes already on the `DESIGN.md` scale (15.0 section, 12.5 label). |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No new mark | No icon is added or removed; refresh buttons keep `ICON_REFRESH`. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Does not apply | No identifier is introduced; two repeated heading strings stop being drawn. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Does not apply | No store, migration, cancellation, or byte-limit surface is touched. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Does not apply | No child process or launch path changes; refresh buttons call the same request functions. |
| Documentation — user-facing docs change in all three languages together | Does not apply | No manual or README sentence changes. Four repeated headings stop being drawn (`Git`, `worktree`, `プルリクエスト`, `セッション`). The `プルリクエスト` message id has no other call site, and `every_message_row_is_one_the_code_shows` forbids a row nothing draws, so its EN and KO rows are removed with the heading. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Applies | None added; drawing only. |
| Budgets — any new scan or output path states its byte and item ceiling | Does not apply | No scan or output path changes; the draw path reads the same state as today. |

## Flagged concerns

None.

## Acceptance

- `cargo test --locked tab_content_no_longer_repeats_its_tab_as_a_heading`,
  `cargo test --locked session_list_title_is_a_panel_label_not_a_heading`,
  `cargo test --locked agent_settings_sections_use_section_headings`,
  `cargo test --locked settings_sidebar_label_is_a_panel_label`, and
  `cargo test --locked tab_headings_keep_their_controls` pass, each watched
  failing first (one filter per invocation).
- `make q.fast` passes.
- In the running app: project tabs open without repeating their name, the
  Sessions page opens with its switch/search row, スキル／ルールと指示 read
  as sections, and the settings sidebar label is quiet. Tabs, page headers,
  and the session column (change 245) are unchanged by this diff.

## Rejected alternatives

- Demote every removed heading to a small label instead of removing it:
  keeps a second copy of the tab's name at reading distance; the column's
  lesson (change 245) was that the repetition itself, not only its size,
  is what buries the content.
- Apply the panel-label form to tabs themselves: tabs already carry a
  selected mark (accent fill, accent rule, selected button); a second
  label system on top would say the same thing twice.
- Shrink `page_header` too: it is the one page title, with subtitle and
  hairline; removing it leaves pages with no title at all.
