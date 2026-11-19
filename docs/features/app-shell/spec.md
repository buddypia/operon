# Spec: アプリの土台

## Behaviour
1. `main` first checks the first argument: `Operon knowledge search ...` (`KNOWLEDGE_CLI_SUBCOMMAND`) and `Operon agents ...` (`AGENTS_CLI_SUBCOMMAND`) run as CLIs and exit with their code without opening a window.
2. Otherwise startup runs in this order: `configure_command_path`, the single-instance locks, the legacy-store import, then `eframe::run_native` with `OperonApp::new_with_instance_locks`. A failed lock or import shows a native modal and returns before any window exists (details in the local-data feature).
3. `configure_command_path` appends to `PATH`, when they are existing directories not already present, `/opt/homebrew/bin`, `/usr/local/bin`, `$HOME/.local/bin` and `$HOME/.cargo/bin`. Existing entries keep their order and come first.
4. Window options: initial inner size 1280×820, minimum 760×540, full-size content view with the title bar and title hidden (the toolbar serves as the title bar), `persist_window` false, and an explicitly empty icon so the Dock keeps the bundle's icon.
5. Fonts (`configure_macos_fonts`): five bundled Sarasa faces (Mono J, Mono K, Term J, Fixed J, UI J) plus the Phosphor icon face (`ICON_FACE`). The Monospace family puts the chosen terminal font first and its fallback second; macOS's SF Mono is appended to Monospace and Apple Symbols to Proportional when readable. The Proportional chain is Sarasa UI J, Phosphor, Apple Symbols. Fonts reload when the chosen terminal font changes.
6. Per frame, `OperonApp::logic` runs first (also while the window is hidden): background results, hook events, terminal streams, session polling, attention, Dock badge, agent snapshot, notice expiry, and a repaint request at least every 2 s. `OperonApp::ui` runs only for shown frames: shortcuts, drops, the toolbar, the central panel, then toast and modals (palette, launch sheet, broadcast sheet, restore, launch, land, PR, run-script, port-stop).
7. Session state is polled every `SESSION_POLL_INTERVAL` (2 s) while the window is focused and every `SESSION_POLL_INTERVAL_UNFOCUSED` (6 s) otherwise; regaining focus reads at once. The pending store retry keeps its cadence while in the background.
8. The toolbar is `TOOLBAR_HEIGHT` (52) tall, filled with `palette.raised`. Left to right: the brand mark (click opens Home, tooltip 「ホーム」), page tabs 「ホーム」, 「プロジェクト」, 「セッション」, then a gear (「設定」), the search button 「検索・操作  {chord}」, the primary button 「新しいセッション  {chord}」, and 「要対応 {n}」 (with its chord, tooltip 「次の要対応セッションへ移動します」) when sessions wait. Chords are read from the live keymap, and are omitted when the action is unbound. The area right of `TRAFFIC_LIGHT_INSET` (78) and not occupied by a control drags the window.
9. `Page` is Home, Projects, Sessions or Settings. The central panel draws the page. Sessions, and Projects on the Files tab with a project selected, take the full width; every other page is centred in a column from `page_column_width` (the gutter is at most `PAGE_GUTTER`, and at most 4 % of the width on a narrow window).
10. Dropping folders on the window calls `route_dropped_folders`; while files hover, a scrim shows 「フォルダをドロップしてプロジェクトを追加」, or 「フォルダをドロップして、このセッションのプロジェクトにする」 when the launch sheet is open.
11. `apply_appearance` applies theme visuals and interface metrics once per theme change and fonts once per font change, clearing the terminal layout caches so open terminals re-colour. `apply_interface_metrics` fixes one control height (`CONTROL_HEIGHT` 28, `CONTROL_HEIGHT_SMALL` 24), button padding, spacing scale (`SPACE_XS` 4 … `SPACE_XL` 26), floating 8 px scrollbars and the type scale.
12. Shared parts in `src/ui/widgets.rs`: `page_header` (title 20 px, one muted line, hairline), `primary_button`, `secondary_button`, `quiet_button`, `icon_button`, `small_icon_button`, `icon_text_button`, `nav_tab`, `card_frame`, `clickable_card`, `status_chip`, `banner`, `empty_state`, `metric_tile`, `tool_status_row`, `hook_cell`, `theme_swatches`, `code_block`. Drawing code names a palette role (`Palette`), an `ICON_*` constant and a size from `DESIGN.md`, never a literal.
13. Notices: a success notice floats as a toast for `NOTICE_TOAST_SECONDS` (4); a failure stays until closed; the durability warning floats while on Sessions without a close button.
14. On exit: terminal streams shut down, the workspace layout is written, a model download in flight is stopped (its partial file kept to resume), the agent snapshot is finished and the shared embedder released.

## States
- Missing tmux or agent CLIs do not block startup; launch is refused with a pointer to Settings → エージェント.
- A CLI outside the four PATH additions is invisible to a Finder-launched app until started from a shell or linked into `~/.local/bin`; 「ツールを再確認」 re-reads PATH.
- Second instance: modal 「Operon は既に開いています」, exit. Unusable data directory: modal 「Operon はローカルデータを開けませんでした」, exit.
- A font file missing from the system (SF Mono, Apple Symbols) is skipped silently; the bundled faces still cover Latin, Japanese, Korean and icons.
- Window focus lost: slower polling; focus regained: immediate poll.
- Hidden window: `logic` still runs, so hooks, attention and the Dock badge stay current; no input is read there.

## Invariants
- No per-frame filesystem, process or heavy parsing work in the draw path; such work runs in background tasks whose results the frame reads.
- Every child process goes through `src/exec.rs`.
- Every button has a documented height; every text size is a level of the `DESIGN.md` type scale; every palette ink meets WCAG 2.1 AA against every surface in all three themes; `DESIGN.md` documents exactly the colours the app paints.
- Every icon constant is listed in `ICON_VOCABULARY` and resolves from the bundled Phosphor face, not from the text face ahead of it.
- All four terminal fonts' CJK glyphs are exactly two ASCII cells wide.
- `CLAUDE.md` maps every module that exists.
- The Dock icon is the bundle's: `native_options` sets an explicitly empty `IconData`.
- User-facing text goes through `tr`/`tf!`; code, comments and docs stay English.

## Code and tests
- `src/main.rs::main`: CLI dispatch, startup order, lock and import modals, `run_native`.
- `src/main.rs::native_options`, `configure_macos_fonts`, `add_icon_face`, `proportional_ui_font_chain`, `configure_command_path`, `command_path_candidates`: window, fonts, PATH.
- `src/app.rs::OperonApp` (`new_with_instance_locks`, `from_state`, `apply_appearance`, `persist`) and its `eframe::App` impl (`logic`, `ui`, `on_exit`): the frame loop.
- `src/app.rs::Page`: the four pages; `ui_central_panel` draws the current one.
- `src/app/screens.rs::ui_topbar`: the toolbar.
- `src/app/toast.rs::ui_brief_notice_toast`: toasts and the durability warning.
- `src/app/launch_projects.rs::route_dropped_folders`: folder drops.
- `src/ui/widgets.rs::apply_interface_metrics`, `page_header`, `page_column_width`, `primary_button`, `icon_text_button`, `nav_tab`, `banner`, `tool_status_row`: the shared parts.
- `src/theme.rs::Palette`, `DARK_PALETTE`, `LIGHT_PALETTE`, `HIGH_CONTRAST_PALETTE`, `SPACE_*`, `CONTROL_HEIGHT`, `TOOLBAR_HEIGHT`, `TRAFFIC_LIGHT_INSET`: roles and metrics.
- `src/glyphs.rs::ICON_VOCABULARY`: the icon constants.
- `src/config.rs::SESSION_POLL_INTERVAL`, `SESSION_POLL_INTERVAL_UNFOCUSED`, `NOTICE_TOAST_SECONDS`: frame-loop timings.
- `src/util.rs`, `src/prelude.rs`: shared helpers and imports.
- Tests: `includes_standard_macos_agent_install_locations_in_command_path` pins the PATH additions.
- Tests: `the_app_icon_comes_from_the_bundle_alone` pins the empty icon.
- Tests: `bundled_terminal_fonts_keep_cjk_at_exactly_two_ascii_cells`, `bundled_ui_font_has_english_japanese_and_korean_glyphs`, `the_icon_face_is_registered_under_the_name_the_font_chain_names` pin fonts.
- Tests: `every_icon_resolves_from_the_bundled_icon_font` pins the icon vocabulary.
- Tests: `every_button_keeps_a_documented_height`, `every_text_size_is_a_level_on_the_type_scale` pin the control metrics.
- Tests: `design_md_documents_exactly_what_the_app_paints`, `every_theme_stays_readable`, `ansi_numbers_map_to_the_colours_they_name` pin the palettes.
- Tests: `topbar_wordmark_is_neutral_beside_the_primary_action` pins the toolbar's emphasis.
- Tests: `accepts_only_finder_folders_from_a_drop` pins which drops are accepted.
- Tests: `poll_275_front_window_reads_sessions_every_two_seconds`, `poll_275_background_window_waits_longer_between_reads`, `poll_275_returning_to_the_window_reads_at_once`, `poll_275_frame_reads_the_windows_focus`, `poll_275_pending_store_retry_keeps_its_cadence_in_the_background` pin the polling cadence.
- Tests: `instance_lock_rejects_a_second_process_and_releases_after_crash` pins the single-instance lock.
- Tests: `claude_md_maps_every_module_that_exists` pins the module map.

## Decisions inherited
- Change 041: body text must clear 4.5:1 and focus marks 3:1 in all three themes; the contrast suite pins it.
- Change 136: one focus look for every button; custom cards and rows paint a focus mark.
- Change 137: no raw `egui::Button`; buttons come from the shared helpers so height and padding are consistent, enforced by tests.
- Change 143: a button needs a visible resting affordance; a quiet text-only button is not an action.
- Change 221: wording and controls follow `DESIGN.md`; text that stays English when the language is switched is a bug.
- Change 223: the command palette anchors top-centre at a fixed offset, so it opens in a predictable place.
- Change 275: the 2 s poll runs only for a focused window; background windows poll every 6 s and the first frame back reads at once.
- Change 295: toolchain and egui are on 0.37, which split `update` into `logic` (always) and `ui` (shown frames); input is only read in `ui`.
- Change 276: a theme change keeps terminal rows on screen until the next capture, then lays them out again in the new colours.
