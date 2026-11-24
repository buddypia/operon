# Spec: an in-app folder picker — type a path, walk the folders, or create the folder

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

Every test is in `src/tests.rs`.

1. A typed path splits into the folder to list and the partial name being
   typed. Text ending in `/` (or exactly `~`) lists that folder with no
   partial; otherwise the folder is the parent and the partial is the last
   component. `~` alone and a leading `~/` are the home folder. Any other
   text that does not start with `/` — `dev/x`, `~name`, empty text — is
   refused: no listing is asked for it. — `folder_317_input_splits_into_a_folder_and_a_partial_name`
2. A listing holds the child folders only, sorted by name ignoring case. A
   symlink to a folder is listed. Names starting with `.` are shown only when
   the partial starts with `.`. A partial keeps the names that start with it,
   ignoring case; the partial is applied while the folder is read. — `folder_317_listing_holds_child_folders_only`
3. A listing reads at most `FOLDER_PICKER_ENTRY_LIMIT` (5 000) directory
   entries, counted as they are read from the open handle, and is marked
   truncated when it stopped there; whether the typed path itself is a folder
   is asked separately, so a folder past the ceiling still opens. — `folder_317_listing_stops_at_its_entry_limit`
4. A listing says which of four things the typed path is: an existing folder,
   a path that does not exist, something that is not a folder, or a folder
   that cannot be read. — `folder_317_listing_says_what_the_typed_path_is`
5. Every way that opened the macOS folder dialog for a project opens the
   picker instead, and none of them opens the macOS dialog: the projects
   page's 「プロジェクトフォルダを選択…」 and 「フォルダを選択…」, ⌘O, the
   palette's project action, and the sheet's 「新しいプロジェクト…」 and
   「フォルダを選択…」. — `folder_317_every_entry_opens_the_picker`
6. Opened from the projects side, 「開く」 on an existing folder does what
   `try_add_project_path` does today (added and opened, or the registered one
   opened) and closes the picker. The picker closes whenever registration was
   attempted, whatever it answered; a refusal reaches the person through the
   existing notice. — `folder_317_opening_a_typed_folder_adds_the_project`
7. Opened from the 新しいセッション sheet, 「開く」 does what
   `apply_launch_project_folder` does today: the sheet takes the project, the
   page under it does not change, the request typed so far stays. While the
   picker shows the sheet is not drawn; it is back when the picker closes.
   — `folder_317_the_launch_sheet_takes_the_typed_folder`
8. A path that does not exist is created only after a second, explicit
   confirmation; the first press creates nothing. Confirmed, the folder and
   its missing parents exist, are empty, and the folder is the project.
   Editing the path withdraws a pending confirmation. The folder is made on
   a background thread: until it answers the sheet shows 「作成中…」, its
   buttons are off, and Enter, Esc and a second press do nothing.
   — `folder_317_a_missing_folder_is_created_only_after_confirmation`
9. Clicking a row makes that folder the typed path (ending in `/`) and lists
   it; 「上のフォルダへ」 lists the parent; Tab extends the partial to the
   longest prefix all matching names share, and adds `/` when one name is
   left. — `folder_317_rows_descend_and_tab_completes`
10. A listing that arrives for text that is no longer in the field is not
    shown, and the listing for the current text is asked for.
    — `folder_317_a_stale_listing_is_not_shown`
11. 「キャンセル」, ✕ and Esc close the picker and change nothing: no project, no
    folder, no notice. With a confirmation pending, Esc withdraws the
    confirmation first. — `folder_317_cancel_changes_nothing`
12. 「Finder で選ぶ…」 closes the picker and opens the macOS dialog the entry
    used before this change. — `folder_317_finder_is_still_reachable`
13. A relative path, or a path that is a file, opens nothing and creates
    nothing; the primary button is off and Enter does nothing.
    — `folder_317_a_relative_path_or_a_file_opens_nothing`
14. A folder dropped on the window while the picker shows becomes the typed
    path, ending in `/` (the first folder when several are dropped); nothing
    is registered until 「開く」.
    — `folder_317_a_dropped_folder_fills_the_path`
15. The picker opens at the parent of the most recently added project, unless
    macOS guards that parent (`is_in_protected_folder`): then, and with no
    project, it opens at `~/`. — `folder_317_opens_at_home_rather_than_in_a_guarded_folder`
16. The picker does not open while a launch or restore progress modal owns the
    window. Opened from the palette, the palette is closed first.
    — `folder_317_opens_over_no_other_modal`

## Behaviour

`screen.md` draws it. A sheet 640px wide titled 「プロジェクトフォルダを開く」:

- **Path field**, focused when the sheet opens, hint `~/path/to/project`. It
  opens holding the parent of the most recently added project, written with
  `~` and a trailing `/`; with no project, or when macOS guards that parent,
  `~/`; with no home folder, `/`.
- **Line under the field**, one of: nothing (an existing folder);
  「このフォルダはまだありません。作成して開けます。」;
  「これはフォルダではありません。」;
  「/ または ~ で始まるパスを入力してください。」.
- **Location row**: an icon button 「上のフォルダへ」 (off at `/`) and the
  listed folder's path.
- **List** of child folders, each a row with the folder icon; scrolls. In its
  place, when there are no rows: nothing (refused text, for which no listing
  is asked); 「読み込み中…」 (a listing for this text was asked and is not back
  yet); 「フォルダはありません。」 (empty); 「一致するフォルダはありません。」 (a
  partial matched nothing); 「このフォルダは読み取れません。」 (denied);
  「この場所はまだありません。」 (the folder to list does not exist). Under a
  listing that hit the ceiling:
  「項目が多いため、先頭の {count} 件だけを読みました。パスを最後まで入力すれば開けます。」.
- **Footer**: 「Finder で選ぶ…」 on the left; 「キャンセル」 and the primary
  button on the right. The primary is 「開く」 for an existing folder and
  「フォルダを作成…」 for a path that does not exist; it is off while the
  listing is loading, and for empty text, refused text or a path that is not
  a folder. ✕ in the title row is 「キャンセル」.
- **Confirmation** (after 「フォルダを作成…」 or Enter on a missing path): the
  footer is replaced by 「「{path}」を作成しますか？」,
  「途中のフォルダも、まだなければ作成します。」, 「作成して開く」 and 「やめる」.
- **Keys**: Enter is the primary button (and 「作成して開く」 while the
  confirmation shows); Esc is 「やめる」 then 「キャンセル」; Tab completes.
  While the picker shows, ⌘O, ⌘K and the page shortcuts do nothing, as with
  the other sheets.
- **Failures**: the folder cannot be created →
  `notice` 「フォルダを作成できませんでした: {error}」, the picker stays open with
  its footer back, and nothing is registered. The project cannot be saved →
  the picker closes, with the existing rollback and its existing notice; a
  folder just created stays on disk, empty. A folder that was there when
  listed and is gone at 「開く」 → the picker closes with the existing
  「既存のフォルダを選んでください。」.
- **Other modals**: requirement 16. The picker and the 新しいセッション sheet
  are never drawn together (requirement 7); the broadcast sheet cannot be
  opened while the picker shows, as with the other sheets.
- **Interrupted**: closing the picker or quitting while a listing is being
  read discards the listing; nothing was written. The picker cannot be closed
  while a folder is being made (requirement 8), so its answer always finds it.
- **Too large**: the ceiling of requirement 3, reported, never shown as a
  complete listing. Long names and long paths are
  one line each, truncated.
- **Protected folders**: walking or typing into Desktop, Documents, Downloads
  or `/Volumes` lists them, so macOS may ask the person once in Operon's name.
  The picker never lists a guarded folder the person did not type or click
  into: it does not open in one (requirement 15), and the home folder's own
  listing names those folders without reading them.
- Results reach the person as today: 「プロジェクトを追加しました。」,
  「そのプロジェクトは追加済みのため、開きました。」 (projects side),
  「登録済みのプロジェクトに切り替えました。」 (sheet side). Cancelling the
  picker says nothing; cancelling the macOS dialog reached through
  「Finder で選ぶ…」 says what it says today.

## Design

**Placement: `EXTEND`** of feature `project-list` (and the entry of
`new-session`), as a new child module of `src/app.rs`. Evidence:
`docs/features/project-list/spec.md` behaviour 2, 6, 7 and 12 — the folder
dialog, the bare path field, and the shared `register_project_folder` all
exist; no in-app listing and no folder creation exist
(`git grep -n create_dir -- src/app.rs src/app` finds none on a project path).
The route is `feature` because folder creation is behaviour that does not
exist.

- **`folder_picker.rs` under `src/app/`** (new, `mod` and `pub(crate) use` in
  `src/app.rs`, a line in `CLAUDE.md`'s map, the module added to
  `docs/features/index.yaml` and `docs/features/domains.yaml`):
  - `FolderPickerTarget { Projects, LaunchSheet }`.
  - `FolderInput { dir: PathBuf, partial: String }` and
    `split_folder_input(text, home) -> Option<FolderInput>`;
    `folder_input_text(path, home) -> String` (the `~/…/` spelling).
  - `FolderTarget { Folder, Missing, NotAFolder }` and
    `FolderListing { input: String, dir: PathBuf, names: Vec<String>,
    dir_state: Listed | Missing | Unreadable, truncated: bool,
    target: FolderTarget }`; `list_folder(text, home, limit) -> Option<FolderListing>`
    is the only function that touches the file system for a listing, and runs
    on a background thread.
  - `FolderPicker { target, input, listing: Option<FolderListing>,
    rows: Vec<String>, confirm_create: bool, creating: bool,
    focus_field: bool }` held as
    `OperonApp::folder_picker: Option<FolderPicker>`. `rows` (the names that
    match the partial, each already joined with its icon) is rebuilt when the
    text changes or a listing arrives, never in the frame.
  - `open_folder_picker(target)`, `set_folder_picker_input(text)`,
    `request_folder_listing()`, `finish_folder_listing(listing)`,
    `folder_picker_complete()`, `submit_folder_picker()`,
    `confirm_folder_creation()`, `close_folder_picker()`,
    `ui_folder_picker(ctx)`.
  - `add_project`, `choose_project_folder`, `apply_project_folder_choice`,
    `choose_workspace_folder`, `apply_workspace_folder_choice` move here from
    `src/app.rs` unchanged in behaviour, so the largest module does not grow.
    `choose_project_folder` and `choose_launch_project_folder` now open the
    picker; the macOS dialog they held becomes `choose_folder_in_finder(target)`.
- **`src/app.rs`**: the field; `BackgroundKey::FolderListing` and
  `BackgroundResult::FolderListing(FolderListing)`; the arm in
  `process_background_results`, which drops a result whose `input` is not the
  field's text and asks again (the key admits one job at a time); the
  `ui_folder_picker` call beside the other sheets.
- **`src/app/launch_projects.rs`**: `launch_sheet_shows` is false while the
  picker shows; `route_dropped_folders` hands the first dropped folder to the
  picker when it shows.
- **`src/app/keymap.rs`**: the picker counts as an open sheet.
- **`src/config.rs`**: `FOLDER_PICKER_ENTRY_LIMIT: usize = 5_000`.
- **`src/glyphs.rs`**: `ICON_PARENT_FOLDER = icons::ARROW_UP` with its
  `ICON_VOCABULARY` entry. `ICON_CHANGE_PREVIOUS` already draws that glyph for
  another meaning; a second name for one glyph is the file's own practice
  (`ICON_BACK` and `ICON_DEPENDS_ON`).
- **`src/i18n_tables.rs`**: a row in `EN_TABLE` and `KO_TABLE` for every string
  in Behaviour, held by the existing
  `every_message_the_code_shows_has_a_row_in_every_table`.
- **`README.md`**: the listing ceiling, beside the other budgets.
- **Background and draw path**: the frame reads `folder_picker` only. Listing
  (`read_dir`, one `metadata` per symlink entry, `is_dir` on the typed path)
  runs in `spawn_background` under `BackgroundKey::FolderListing`. Folder
  creation (`fs::create_dir_all`) runs in `spawn_background` under
  `BackgroundKey::FolderCreate`, answering
  `BackgroundResult::FolderCreated { path, error: Option<String> }`;
  `FolderPicker::creating` is the pending state. Registration then runs once
  in `process_background_results`, outside any draw, through the unchanged
  `register_project_folder`. `open_folder_picker` reads
  `is_in_protected_folder` (pure, no disk) for requirement 15.
- **No persisted shape changes. No external command. No new dependency.**

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no new role | The sheet uses `raised`, `border`, `scrim`, `text_strong`, `text_muted`, as `ui_broadcast_sheet` does; `.claude/rules/palette-and-glyphs.md` read. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | yes | `ICON_FOLDER_CLOSED` and `ICON_CLOSE` exist; `ICON_PARENT_FOLDER` is added with its vocabulary entry (Design says why it is a second name for `ARROW_UP`). |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | The ceiling is one constant in `src/config.rs`, read by the listing and by the test; `.claude/rules/identifiers.md` read. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | yes, its budget rules | A new scan budget, `FOLDER_PICKER_ENTRY_LIMIT`: documented in `README.md` in this change, counted on the open handle while reading, and a truncated listing is reported as truncated (requirement 3). No store shape, write order or migration changes; registration goes through the unchanged `register_project_folder` and its rollback. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | No child process; the macOS dialog is the existing `rfd` call, moved. |
| Documentation — user-facing docs change in all three languages together | yes | `docs/MANUAL.ja.md` 「プロジェクトを追加する」 and its English and Korean counterparts gain the picker; `docs/features/project-list` and `new-session` spec and brief, `flow.yaml` (a new sheet and two ways). |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | Local file-system reads and one `create_dir_all`; no `unsafe`. |
| Budgets — any new scan or output path states its byte and item ceiling | yes | 5 000 directory entries per listing, one folder deep, never recursive. |

## Flagged concerns

- **The feature invariant 「Adding or removing a project never writes to the
  project folder」.** Answered: the person chose folder creation
  (「フォルダ作成も含む」). The invariant becomes: nothing is written inside a
  project folder; a folder that does not exist is created, empty, only after
  the person confirms. `docs/features/project-list/spec.md` is updated so.
- **macOS permission prompts for protected folders.** Answered in Behaviour:
  only a folder the person typed or clicked into is ever listed.

## Acceptance

- `cargo test --locked folder_317_` passes: the sixteen tests above.
- `cargo test --locked launch_294_` and
  `cargo test --locked starts_with_onboarding_until_a_project_exists` still
  pass, with the assertions that named the macOS dialog now naming the picker.
- `make q.fast` passes; `bash scripts/check-bands.sh` shows nothing at propose
  and `largest_module_lines` no higher than on `main`.
- In the built application (`python3 scripts/verify-app.py run project-list`):
  プロジェクト → 「フォルダを選択…」 shows the sheet as `screen.md` draws it;
  typing narrows the list; a row descends; a missing path asks before it
  creates; the same sheet opens from 新しいセッション → 「新しいプロジェクト…」.

## Rejected alternatives

- **A path field only on the 新しいセッション sheet** — offered to the person,
  who chose the unified picker.
- **Dropping the macOS dialog** — it reaches iCloud, tags and recent places the
  picker does not; one button keeps it.
- **A recursive tree with disclosure triangles** — every expanded level is
  another listing to keep fresh, and a path field plus one level says the same
  thing with one listing.
- **Listing in the frame** — forbidden by `.claude/rules/rust.md`; a network
  volume would stall the window.
- **Creating the folder in the press's own handler** — it is not a draw path,
  but a slow volume would stall the window all the same.
- **Creating the folder on the first Enter** — one mistyped character would
  leave a folder behind.
- **`git init` in the new folder** — offered, not chosen.
