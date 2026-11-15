# Spec: ⌘-click opens a terminal path in Operon; right-click offers where to open it; bare names resolve

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. A primary click with ⌘ (or ⌥) on an openable path in the terminal pane opens
   it inside Operon (`open_terminal_target`), not with the default application.
   A URL still opens in the browser. Pinned by
   `terminal_293_cmd_and_option_click_open_a_path_inside_operon`.
2. A secondary click on an openable path — with or without ⌘ — opens the path
   menu anchored at the pointer, whose items are, in order: 「Operon で開く」
   「既定のアプリで開く」「Finder で表示」「パスをコピー」. Pinned by
   `terminal_293_path_menu_lists_four_choices_in_order` and
   `terminal_293_secondary_click_opens_the_menu_at_the_pointer`.
3. 「Operon で開く」 from the menu opens the file as a document tab; the other
   items hand the file to `open_path` / `reveal_path` / the clipboard. Pinned by
   `terminal_293_menu_open_in_operon_opens_a_document`.
4. A token with no `/` that does not resolve against the worktree or project
   root resolves to the single scanned file of that name under the session's
   root (`session_file_cache` for a worktree session, `file_cache` for a
   project session). Zero or several matches leave it unresolved. Pinned by
   `terminal_293_bare_name_resolves_to_the_only_file_with_that_name` and
   `terminal_293_bare_name_with_two_matches_stays_unresolved`.
5. A bare name that missed because the scan was not there yet resolves once it
   lands, with no new output: the probe drain requests the scan the session's
   root needs (`request_session_files` for a worktree session, `request_files`
   for a project session; `spawn_background` already dedups by key) when a
   `/`-free candidate misses and no scan is cached, and both scan handlers
   (`BackgroundResult::Files`, `BackgroundResult::WorktreeFiles`) forget every
   cached miss (`None` entries in `resolved_paths` and `terminal_path_caches`,
   and `negative_path_probed_at`) so the next hover probes again. Pinned by
   `terminal_293_probe_requests_the_scan_a_bare_name_needs` and
   `terminal_293_bare_name_missed_before_the_scan_resolves_after_it`.
6. The menu closes on a choice, on Escape, and on a primary press outside it.
   Pinned by `terminal_293_menu_closes_on_escape_or_outside_click` (pure
   `terminal_path_menu_closes(chosen, escape, pressed_outside)`), alongside the
   existing tests that clear `terminal_path_menu` on session/project removal.

## Behaviour

- Hover underline, pointing-hand cursor with ⌘/⌥, and plain click = focus are
  unchanged.
- The menu is a small borderless window placed at the click position; it
  closes on a choice, on Escape, or on a click outside it. The first line shows
  the resolved path in muted 12px text, as today. The old 「閉じる」 button is
  dropped (outside-click and Escape close it).
- A pptx or other file the editor cannot show opens as a tab that says so and
  offers the existing 「既定のアプリで開く」 button (existing `Unopenable` path).
- Before the session's file scan has arrived a bare name is a miss and is not
  underlined; the miss asks for the scan, and when the scan lands the miss is
  forgotten, so the next hover underlines it (R5).
- Failures reach the person exactly as today: `notice` 「ファイルが見つかりません: …」.

New string: 「Operon で開く」 (en "Open in Operon", ko "Operon에서 열기").

## Design

Placement: `EXTEND` — the terminal link feature exists (`src/ui/terminal.rs`
`terminal_targets`, `src/app/screens.rs` click dispatch, `src/app.rs`
`resolve_session_terminal_file`, `ui_terminal_path_menu`). Route `modify`.

- `src/app/screens.rs`: primary click with an open modifier always calls
  `open_terminal_target`. Secondary click on a path stores the pointer position
  in a new transient field `terminal_path_menu_at: Option<egui::Pos2>` beside
  `terminal_path_menu`.
- New module `app::terminal_paths` (`NEW_MODULE` for this part: `screens.rs`
  is at 9936 lines against the 10000 `largest_module_lines` propose tier, so
  the menu moves out rather than grows there): `ui_terminal_path_menu`
  (moved), drawing the items from `TERMINAL_PATH_MENU: [(&str,
  TerminalPathMenuChoice); 4]` at `fixed_pos`, `apply_terminal_path_menu_choice`,
  and `resolve_terminal_path_with_scan`. `CLAUDE.md` gains its map line.
- `src/app.rs`: the probe drain and `resolve_session_terminal_file` call
  `resolve_terminal_path_with_scan` (defined in the new module) =
  `resolve_terminal_session_path`, then for a `/`-free token
  `unique_scanned_file` over the session root's cached scan. The drain asks
  for a missing scan, and the two scan handlers call
  `forget_terminal_path_misses` (new module) (R5).
- `is_terminal_cmd_modifier` stays: `is_terminal_open_modifier` and its tests
  use it; only the dispatch stops branching on it. New pure `unique_scanned_file(scan: &FileScanSnapshot, name) ->
  Option<&Path>` in `src/files.rs`.
- Nothing persisted changes; no new process; no new thread. The basename lookup
  runs in the probe drain (outside the draw path) over at most
  `FILE_TREE_LIMIT` in-memory paths per candidate, at most
  `TERMINAL_PATH_PROBE_LIMIT` candidates per frame.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | No new role | menu reuses `text_muted` via palette |
| Icons | No | no new mark |
| Identifier SSOT | Yes | menu items in one array used by drawing and test |
| Durability | No | no store, no persisted shape (`terminal_path_menu_at` is transient) |
| Subprocess safety | No new child | `open_path`/`reveal_path` already exist in `src/sys.rs` |
| Documentation | Checked | README mentions of ⌘-click updated in all three languages if present |
| Local-first | Yes | nothing leaves the machine; no `unsafe` |
| Budgets | Yes | lookup bounded by `FILE_TREE_LIMIT` × `TERMINAL_PATH_PROBE_LIMIT` |

## Flagged concerns

None open.

## Acceptance

- `cargo test --locked terminal_293_` (9 tests) passes; existing terminal link tests pass in CI.
- In the app: ⌘-click `dist/…pptx` or a bare `Operon_LT_10min.pptx` opens a tab;
  right-click shows the four items at the pointer.

## Rejected alternatives

- Swap ⌥ to "default app": two modifiers with different meanings is what the
  person tripped on; one gesture opens in Operon, the menu holds the rest.
- Guess among several same-named files: an underline that opens the wrong file
  is worse than no underline (the parser's own rule).
