# Spec: the 新しいセッション sheet adds a project in place

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. The sheet's project list ends, after a separator, with 「＋ 新しいプロジェクト…」;
   choosing it opens the folder picker (`choose_launch_project_folder`). Pinned
   by `launch_294_project_list_ends_with_new_project`, which opens the list by
   a click on the rendered sheet and clicks the row.
2. A chosen folder that is not a project yet is registered and becomes the
   sheet's project; the page, `launch_sheet_open`, the typed request and the
   session name stay. Pinned by
   `launch_294_chosen_folder_becomes_the_sheets_project_in_place`.
3. A chosen folder that is already a project is switched to, not duplicated and
   not refused; the page stays. Pinned by
   `launch_294_registered_folder_is_switched_to`.
4. A path that is not a folder leaves the sheet's project and the page as they
   were and says 「既存のフォルダを選んでください。」. Pinned by
   `launch_294_not_a_folder_leaves_the_sheet_as_it_was`.
5. With no project, 新しいセッション opens the sheet on the current page, and the
   sheet stays open drawing 「フォルダを選択…」; a chosen folder turns it into
   the form for that project. Pinned by
   `launch_294_without_projects_the_sheet_opens_and_offers_a_folder` (and the
   three existing tests that asserted the old jump, rewritten).
6. A folder dropped while the sheet is open becomes the sheet's project and the
   page stays; with the sheet closed a drop behaves as today. Pinned by
   `launch_294_folder_dropped_on_the_sheet_becomes_its_project`.
7. The list orders projects by their latest session start
   (`launched_at`, else `created_at`), newest first, and projects with no
   session by `added_at`, newest first, after them. Pinned by
   `launch_294_recent_projects_come_first`.
8. From `LAUNCH_PROJECT_FILTER_MIN` (8) projects the list shows a filter that
   keeps names containing the typed text, case-insensitively. Pinned by
   `launch_294_filter_narrows_by_name`.

## Behaviour

As drawn in `screen.md`. New strings (ja / en / ko):

| ja | en | ko |
|---|---|---|
| ＋ 新しいプロジェクト… (icon + 新しいプロジェクト…) | New project… | 새 프로젝트… |
| プロジェクトを絞り込み | Filter projects | 프로젝트 필터 |
| 一致するプロジェクトはありません。 | No matching project. | 일치하는 프로젝트가 없습니다. |
| 登録済みのプロジェクトに切り替えました。 | Switched to the existing project. | 등록된 프로젝트로 전환했습니다. |
| まだプロジェクトがありません。 | No project yet. | 아직 프로젝트가 없습니다. |
| セッションを始めるフォルダを選んでください。 | Choose a folder to start the session in. | 세션을 시작할 폴더를 선택해 주세요. |
| フォルダを選択… | Choose folder… | 폴더 선택… |
| Finder からフォルダをこのウィンドウにドロップしても追加できます。 | You can also drop a folder from Finder onto this window. | Finder에서 폴더를 이 창으로 드롭해도 추가할 수 있습니다. |
| フォルダをドロップして、このセッションのプロジェクトにする | Drop a folder to use it for this session | 폴더를 드롭하여 이 세션의 프로젝트로 사용 |
| {count} 件のフォルダを追加し、{name} を選びました。 | Added {count} folders and chose {name}. | 폴더 {count}개를 추가하고 {name}을(를) 선택했습니다. |

「先にプロジェクトを追加してください。」 is no longer shown and its rows go.
Failures reach the person through `notice` (not a folder, store not saved — the
existing messages); success through `notice_briefly`. A cancelled picker says
nothing.

## Design

Placement: `EXTEND` — the launch sheet (`ui_launch_sheet`, screens.rs) and
project registration (`try_add_project_path`, app.rs) exist. Route `modify`.

- New child module `src/app/launch_projects.rs` (both `app.rs`, 9920 lines,
  and `screens.rs`, 9887, sit under the 10000-line propose tier, so the new code
  and what it replaces move out): `register_project_folder(path) ->
  Result<RegisteredProject, ProjectAddition>` (the body of
  `try_add_project_path`, minus navigation; `RegisteredProject::{New, Existing}`),
  `adopt_launch_project_folder`, `apply_launch_project_folder`,
  `choose_launch_project_folder` (rfd under `cfg(not(test))`, a notice marker
  under test, as `choose_project_folder` does), `adopt_dropped_folders_in_sheet`,
  pure `launch_project_choices(projects, sessions, filter) -> Vec<(Uuid,
  String)>`, `LAUNCH_PROJECT_FILTER_MIN`, the header picker
  `ui_launch_sheet_project_picker`, the no-project body
  `ui_launch_sheet_without_project`, and `switch_launch_project` (moved from
  screens.rs). `CLAUDE.md` gains its map line, paid for by trimming others so
  `always_loaded_bytes` stays under its propose tier (8000).
- `try_add_project_path` keeps its contract and navigation, calling
  `register_project_folder`. `open_new_session` opens the sheet with no project
  instead of jumping. `update` routes a drop to the sheet while it is open; the
  overlay picks its text the same way.
- `OperonApp` gains transient `launch_project_filter: String`, cleared when the
  sheet opens and when a project is picked.
- The combo uses `PopupCloseBehavior::CloseOnClickOutside` (egui 0.31.1) so the
  filter can be clicked; a row closes it with `close_popup`. The choices are
  computed only while the popup is open. The picker runs after the sheet's
  `Area` closes, not inside it.
- Nothing persisted changes shape; the store write is the one
  `try_add_project_path` already does. No new process, thread, or `unsafe`.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | No new role | `text_muted` / existing roles via `palette` |
| Icons | No new mark | `ICON_ADD`, `ICON_SEARCH`, `ICON_FOLDER_OPEN` exist |
| Identifier SSOT | Yes | combo salt and filter threshold each one constant |
| Durability | Read | same `persist()` and rollback as `try_add_project_path`; no shape, version, or order change |
| Subprocess safety | No | no child; `rfd` already used the same way |
| Documentation | Checked | README mentions of adding projects checked in all three languages; changed only if they say the sheet cannot |
| Local-first | Yes | nothing leaves the machine |
| Budgets | Yes | choices bounded by the store's projects and sessions, built only while the list is open |

## Flagged concerns

None open.

## Acceptance

- `cargo test --locked launch_294_` (8 tests) and the three rewritten tests pass.
- In the app: open 新しいセッション, choose 「＋ 新しいプロジェクト…」, pick a
  folder; the sheet shows it and the page does not change.

## Rejected alternatives

- A mkdir / `git init` form: out of scope by the person's answer; the macOS
  picker can make a folder.
- Persisting a `last_used_at` on `Project`: a store-shape change for what the
  sessions already record.
- Putting the code in `launch_menu.rs`: that module is the tab strip's + menu,
  a different control.
