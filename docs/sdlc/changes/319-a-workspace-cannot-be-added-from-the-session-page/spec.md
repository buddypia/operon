# Spec: add a session or a project from the session page's column, and give its project names their own size

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. The column's heading carries a + (`small_icon_button`, `ICON_ADD`, hover
   「追加」) that opens a menu of 「新しいセッション…」 and
   「プロジェクトを追加…」. Pinned by
   `column_319_the_heading_plus_offers_a_session_and_a_project`.
2. A secondary click on the column's empty space below the last project opens
   the same two items. Pinned by
   `column_319_a_right_click_on_empty_space_offers_the_same_items`.
3. 「新しいセッション…」 does what `open_new_session` (⌘N) does. Pinned by
   `column_319_new_session_opens_the_sheet_as_command_n_does`.
4. 「プロジェクトを追加…」 opens the folder picker with a new target,
   `FolderPickerTarget::SessionColumn`. A folder opened from it — typed, walked,
   or chosen through 「Finder で選ぶ…」 — is registered as
   `register_project_folder` does; then the page stays `Page::Sessions`, the
   project is in `SessionArrangement::unfolded` (saved), it is the workspace's
   project (`select_workspace_project`), and the notice is
   「プロジェクトを追加しました。」. Pinned by
   `column_319_a_project_added_from_the_column_stays_on_the_session_page`.
5. A folder already registered, opened from that target, adds nothing, and is
   unfolded and made the workspace's project the same way, with the notice
   「そのプロジェクトは追加済みのため、開きました。」. Cancel changes nothing and
   says nothing, as for the other targets. Pinned by
   `column_319_a_registered_folder_is_unfolded_and_chosen`.
6. A secondary click on a project's heading opens 「セッションを開始」
   (`open_project_session_setup` on that project) and 「プロジェクトを追加…」
   (as in R4). Pinned by
   `column_319_a_heading_right_click_offers_its_project_actions`.
7. Project names in the column are drawn at the `control` level of the type
   scale (13.5px), strong, in `palette.text` (`text_strong` when it is the
   selected project), above the session titles' 12px. The column's heading
   「プロジェクト」 is drawn at the same `control` level, strong, muted. Pinned
   by `column_319_project_names_read_larger_than_their_rows`.
8. 「履歴」 is a `quiet_button`, as the action beside a heading is in
   `DESIGN.md` → Controls. Pinned by
   `column_319_the_heading_plus_offers_a_session_and_a_project`, which finds it
   in the link ink.
9. With no project, under 「プロジェクトがありません」 a `secondary_button`
   「プロジェクトを追加…」 opens the picker with the column's target. Pinned by
   `column_319_an_empty_column_offers_to_add_a_project`.

## Behaviour

Laid out in `screen.md`. Strings a person reads, new ones marked *:

- 「追加」 (hover of the heading +; exists).
- 「新しいセッション…」*, 「プロジェクトを追加…」* (menu items and the empty
  column's button).
- 「セッションを開始」 (exists).
- Notices: 「プロジェクトを追加しました。」, 「そのプロジェクトは追加済みのため、
  開きました。」 (exist). A folder that cannot be registered says what
  `register_project_folder` already says, and nothing else changes. When the
  fold cannot be saved after the project is registered, the project stays
  registered and unfolded for this run, and the notice is
  「並び順を保存できませんでした: {error}」 (exists), as `toggle_project_fold`
  says today.
- Empty: no project — section 3 of `screen.md`.
- Filtering: a project added while a filter is on is unfolded in the saved
  arrangement, and is listed only if it matches the filter, as every project
  is while filtering.
- Interrupted: closing the picker, or Esc on a menu, changes nothing.
- Too large: nothing new is listed or read; the picker keeps its own ceiling.

## Design

Placement: **EXTEND** `session-column` (`docs/features/session-column/spec.md`
behaviour 1 and 5: the heading has only 「履歴」, a project heading only its
controls) with an entry point of `project-list` (change 317's picker, which
has `Projects` and `LaunchSheet` targets only). Route `modify`.

- `column_add.rs` under `src/app/` (new child of `src/app.rs`, because `src/app.rs` is
  within 21 lines of its size limit and `src/app/screens.rs` holds the draw):
  `ColumnAdd { Session, Project }`, `COLUMN_ADD_CHOICES`,
  `OperonApp::apply_column_add`, `OperonApp::ui_column_add_items` (the two
  `quiet_button` rows shared by the + popup and the empty-space menu; the
  heading menu draws its own 「セッションを開始」 for that project and then the
  same 「プロジェクトを追加…」 row through `ui_column_add_project_item`), and `OperonApp::apply_column_project_folder` (R4, R5).
- `src/app/folder_picker.rs`: `FolderPickerTarget::SessionColumn`, routed in
  `open_folder_from_picker` and `choose_folder_in_finder` to
  `apply_column_project_folder`.
- `ui_workspace_projects` in `src/app/screens.rs`: the heading row (R1, R7, R8),
  the empty state (R9), the heading's `context_menu` (R6), the name sizes
  (R7), and, after the last project inside the scroll area, the rest of the
  column allocated with `Sense::click()` carrying the `context_menu` (R2). The session rows keep their own menus:
  egui gives the secondary click to the topmost response under the pointer,
  and the empty space lies below every row.
- `src/app.rs`: the `mod column_add;` line.
- `src/i18n_tables.rs`: two rows per table.
- No persisted shape; the fold writes `session-order.json` through the
  existing `toggle_project_fold` path. No external command.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | yes, read `.claude/rules/palette-and-glyphs.md` | No new role: `text`, `text_strong`, `text_muted`, `link` exist and are already checked against `panel`. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | yes | `ICON_ADD` exists; no new mark. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | read `.claude/rules/identifiers.md` | The two column items are one list (`COLUMN_ADD_CHOICES`) drawn by one function for the + popup and the empty-space menu; the heading menu reuses the project row's function. No string is shared across processes. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | yes, read | No store shape; registration is `register_project_folder` unchanged; the fold is saved by the existing atomic write. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no new child | Sessions start through the existing sheet and its gates. |
| Documentation — user-facing docs change in all three languages together | yes | `docs/features/session-column` and `project-list` brief and spec; `docs/features/flow.yaml` gains the column's ways to `folder-picker` and `new-session`. The session-page paragraph of `docs/MANUAL.ja.md`, `docs/MANUAL.md` and `docs/MANUAL.ko.md` gains one sentence on the column's + and right-click, in all three. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | None added. |
| Budgets — any new scan or output path states its byte and item ceiling | no new scan | The picker's listing ceiling (change 317) is unchanged. |
| Type scale and buttons — `DESIGN.md` typography and Controls | yes | 13.5 (`control`) for the heading and the names; `small_icon_button`, `quiet_button`, `secondary_button`. |

## Flagged concerns

- **The word 「プロジェクト」** — the person suggested 「ワークスペース」. Left
  to change 320 so one word changes everywhere at once; the strings added here
  use 「プロジェクト」 and will be renamed with the rest.

## Acceptance

- `cargo test --locked column_319_` passes, with the related groups
  `folder_317_`, `column_311_`, `sidebar_302_`, `session_285_`, and the
  translation, type-scale, button-height and flow tests.
- In the debug build on the session page: the heading shows 「履歴」 and +; the
  + menu's 「プロジェクトを追加…」 adds a folder and it appears unfolded in the
  column with the window still on the session page; a right-click on empty
  space and on a heading shows the menus in `screen.md`; project names read
  larger than the session titles.

## Rejected alternatives

- Reusing `FolderPickerTarget::Projects` and switching the page back: the
  projects page would flash and the new project would arrive folded.
- Putting the + menu in the tab strip's + (change 273): that menu starts a CLI
  in a pane; a project is not a pane's choice.
- 「Finder で表示」 and fold items in the heading menu: not asked for; the
  disclosure arrow and the projects page already do both.
- A larger size for the heading 「プロジェクト」 than the names: it would
  compete with the names it heads.
