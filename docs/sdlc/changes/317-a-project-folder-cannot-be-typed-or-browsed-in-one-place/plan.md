# Plan: an in-app folder picker — type a path, walk the folders, or create the folder

- **Spec**: `./spec.md`
- **Approved**: 2026-11-25
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `folder_picker.rs` under `src/app/` | New. The pure path and listing functions, `FolderPicker`, its handlers and `ui_folder_picker`; the five project and workspace choosers moved from `src/app.rs`. |
| `src/app.rs` | `mod`/`use`; the `folder_picker` field; `BackgroundKey::{FolderListing, FolderCreate}`, `BackgroundResult::{FolderListing, FolderCreated}` and their arms; the `ui_folder_picker` call; the five moved functions removed. |
| `src/app/launch_projects.rs` | `choose_launch_project_folder` opens the picker; `launch_sheet_shows` is false while it shows; `route_dropped_folders` fills it. |
| `src/app/keymap.rs` | The picker counts as an open sheet. |
| `src/app/screens.rs` | The palette's project action closes the palette before opening the picker, if it does not already. |
| `src/config.rs` | `FOLDER_PICKER_ENTRY_LIMIT`. |
| `src/glyphs.rs` | `ICON_PARENT_FOLDER` and its `ICON_VOCABULARY` entry. |
| `src/i18n_tables.rs` | Rows in `EN_TABLE` and `KO_TABLE` for every new string. |
| `src/tests.rs` | The sixteen `folder_317_` tests; the three assertions that named the macOS dialog (`…_invoked` notices) now name the picker. |
| `CLAUDE.md` | The module-map line for `folder_picker.rs`. |
| `README.md` | The listing ceiling beside the other budgets. |
| `docs/MANUAL.ja.md`, `docs/MANUAL.md`, `docs/MANUAL.ko.md` | 「プロジェクトを追加する」 describes the picker, typing, walking, and creating a folder. |
| `docs/features/project-list/spec.md`, `docs/features/project-list/brief.md` | Behaviour of the picker; the invariant about writing to a project folder reworded; the new tests under `Pinned by`; the verify steps. |
| `docs/features/new-session/spec.md`, `docs/features/new-session/brief.md` | The sheet's two folder entries open the picker. |
| `docs/features/local-data/spec.md` | The listing ceiling among the constants of `src/config.rs` it lists. |
| `docs/features/index.yaml`, `docs/features/domains.yaml` | The new module claimed by `project-list` and its domain. |
| `docs/features/flow.yaml` | A sheet `folder-picker` with its `reach`, and the ways into it from `projects` and `new-session`; pictures re-taken with `python3 scripts/capture-screens.py`. |
| `docs/features/glossary.yaml` | Only if a suite check asks for the sheet's title as a term. |
| `docs/sdlc/changes/317-…/verification.yaml`, `mutations.py` | One obligation per requirement and the mutation sweep. |

## Order of work

The tree compiles after every step.

1. `src/config.rs` constant, `src/glyphs.rs` icon. Then in
   `folder_picker.rs` the pure layer only — `split_folder_input`,
   `folder_input_text`, `list_folder`, the completion helper — with tests for
   requirements 1–4 written first and watched failing (they fail to compile,
   then fail on a stub).
2. State and handlers without drawing: the field, the two background keys and
   results, `open_folder_picker`, `set_folder_picker_input`,
   `request_folder_listing`, `finish_folder_listing`, `submit_folder_picker`,
   `confirm_folder_creation`, `finish_folder_creation`, `close_folder_picker`.
   Tests 6, 7, 8, 10, 13, 15, 16 first, each watched failing. Tests drive the
   handlers and feed `finish_*` directly, as `apply_project_folder_choice`
   tests do, and use `tempfile` folders.
3. Move the five choosers out of `src/app.rs`; point `choose_project_folder`
   and `choose_launch_project_folder` at the picker; add
   `choose_folder_in_finder`. Test 5 and 12 first; update the three old
   assertions. `launch_sheet_shows`, `route_dropped_folders` and the keymap
   guard, with test 14 and the rest of 7.
4. `ui_folder_picker`: check `egui` 0.37 for `TextEdit`, key consumption and
   `ScrollArea::show_rows` in context7 before writing. Tests 9 and 11 drive
   real frames with the suite's `click_at` and `drawn_text_rect` helpers.
5. Strings into both language tables; `make q.fast`; the named tests;
   `cargo test --locked launch_294_ starts_with_onboarding`.
6. Documents: module map, README, manuals, feature specs and briefs,
   index, domains, flow.
7. Build and run in the application per
   `.claude/skills/feature-pilot/references/verify-in-the-app.md`; compare
   with `screen.md`; `python3 scripts/capture-screens.py`.
8. `verification.yaml`, `mutations.py` (each guard watched red, including a
   claim: the confirmation sentence and the ceiling message), the two
   verifier agents, then close, commit, push, CI, land, package.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A file-system call slips into the frame | A stall on a slow volume | `ui_folder_picker` reads only `FolderPicker`; rust-reviewer; a mutation that puts `is_dir` in the draw is not catchable by test, so the reviewer is the catcher and the spec says so |
| `spawn_background` drops a listing asked while one is in flight | The list shows the folder for older text | Test 10: the stale result is dropped and the current one asked |
| Tab is taken by egui's focus traversal before the picker sees it | Tab moves focus instead of completing | Test 9 in a real frame; consume the key before the field is drawn |
| Esc reaches the launch sheet under the picker | Both close at once | The sheet is not drawn while the picker shows; test 7 and 11 |
| A created folder with a registration that fails | An empty folder left behind | Stated in the spec as the outcome; the notice says what failed |
| `largest_module_lines` grows | `check-bands.sh` | The five choosers leave `src/app.rs`; the band is read before the commit |
| The flow and feature suites refuse a sheet they do not know | `cargo test` on the docs tests | Step 6 before step 8; run `cargo test --locked flow glossary feature` |
| The app cannot be photographed on this machine (screen permission) | `capture-screens.py` fails | Record `awaiting-user` for that picture only; no stubbed picture |
| ⌘O used to answer while a progress modal showed | A person presses ⌘O and nothing happens | Requirement 16 makes it deliberate; the manual says the picker opens from the pages and the sheet |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked folder_317_` — `16 passed; 0 failed`.
- `cargo test --locked launch_294_` — all passed.
- `bash scripts/check-bands.sh` — nothing new at propose;
  `largest_module_lines` at or below 9968.
- `python3 docs/sdlc/changes/317-a-project-folder-cannot-be-typed-or-browsed-in-one-place/mutations.py` — every mutation caught.
- In the built application: the sheet as `screen.md` draws it, from both
  entries; a missing path created only after 「作成して開く」.
- CI green on the pushed branch.

## Departures from the plan

- **`local-data` is a third feature.** `a_change_that_alters_a_feature_updates_its_spec` ties `src/config.rs` to `local-data`, whose spec lists the scan ceilings, so `state.yaml` names it and its spec gains the picker's ceiling (the row added to the table above). The plan had read the constant as nobody's.
- **Tests were not watched failing against stubs.** The module and its sixteen tests were written together; each test is watched failing in `mutations.py` instead, which breaks the line it guards.
- **Three findings fixed after the build**, each with its test (`F1`–`F3` in `verification.yaml`): Tab on a bare `~` wrote `~name/` (empirical-verifier); a path the disk would not answer for was offered for creation, so `FolderTarget` gained `Unreadable` and the line 「このパスは確認できません。」 (rust-reviewer); the Enter that confirms a Japanese conversion opened the folder, so the sheet reads Enter through `find_bar_step` (rust-reviewer, lesson 016). From the same review: Tab does nothing on a listing cut at the ceiling, a row click hands the keyboard back to the field, and the creation failure names the path — 「「{path}」を作成できませんでした: {error}」 where the spec wrote 「フォルダを作成できませんでした: {error}」.
- **Not fixed: a listing that hangs.** One listing is read at a time, so a read that never returns (a dead network mount) leaves the sheet on 「読み込み中…」 until it does; 「キャンセル」 and 「Finder で選ぶ…」 still work.
- **`FolderPicker` holds `question: Option<String>`** where the spec wrote `confirm_create: bool`: the confirmation's sentence is built once when asked, so the frame formats nothing.
