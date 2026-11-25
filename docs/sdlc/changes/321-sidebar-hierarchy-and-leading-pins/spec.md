# Spec: quiet session-list label, 15px project names, and leading pin marks

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Left column top label (`ui_workspace_projects`): draws `DockPanel::Sessions.label()` (`tr("セッション一覧")`) at `12.5px` (`label` level in `DESIGN.md`) in `palette.text_muted`, without `.strong()`, keeping `+` (`small_icon_button`, `ICON_ADD`, hover `tr("追加")`) and `履歴` (`quiet_button`) on the right.
   Test: `column_321_panel_label_is_quiet_session_list`.
2. Project names in the left column (`ui_workspace_projects`): drawn at `15.0px` (`section` level in `DESIGN.md`), `.strong()`, in `palette.text` (`palette.text_strong` when selected), outranking both the `12.5px` panel label and the `12.0px` session rows, while folded project headings keep at most `28.0px` top-to-top pitch.
   Test: `column_321_project_names_use_section_scale` (and existing `column_311_folded_projects_sit_closer_together`).
3. Pinned session rows in the left column (`ui_workspace_projects`): when `self.is_session_pinned(id)` is true, the row's left inner margin shifts left to `SIDEBAR_ROW_INDENT - SESSION_PIN_SLOT_W - SPACE_XS` (`42 - 14 - 4 = 24px`) and draws `ICON_PIN` (`11.5px`, `palette.selected_secondary_ink()` when selected else `palette.text_muted`, hover `tr("ピン留め中")`) in a `SESSION_PIN_SLOT_W` (`14.0px`) × `COLUMN_SESSION_MARK_SIZE` (`16.0px`) slot **before** `session_badge` / `session_status_icon_sized`, so the AI/status mark's left edge stays aligned at `SIDEBAR_ROW_INDENT` (`42px`) across pinned and unpinned rows, under the project name; the trailing `ICON_PIN` after the mark is removed.
   Test: `column_321_sidebar_pin_sits_in_gutter_before_aligned_agent_mark`.
4. Pinned session tabs in the central workspace (`ui_tab_workspace`): for `WorkspaceTab::Session(id)` when `self.is_session_pinned(*id)` is true, `ICON_PIN` (`11.5px`, `palette.selected_secondary_ink()` when active else `palette.text_muted`, hover `tr("ピン留め中")`) is drawn in a `SESSION_PIN_SLOT_W` (`14.0px`) × `CONTROL_HEIGHT_SMALL` (`24.0px`) slot **before** `session_badge` (which renders the CLI mark or, for custom commands with no mark, `session_status_icon`); the trailing `ICON_PIN` on the right is removed, and pinned session tabs continue to omit `ICON_CLOSE` (`×`).
   Test: `column_321_tab_pin_sits_left_of_agent_mark_without_close_button`.
5. English translation of `セッション一覧` in `src/i18n_tables.rs` is `"Session list"` (capitalized like `"Files"`, `"Conversation"`, `"Changes"`, while keeping `"Move Session list to the {edge}"` grammatical in `dock_drop_hint` and preserving strict Japanese-key sort order).
   Test: `column_321_session_list_label_is_capitalized_in_english`.

## Behaviour

- Left column heading: reads 「セッション一覧」 in muted 12.5px text; `+` and 「履歴」 stay at the right edge; right-click and `+` menus are unchanged.
- Project headings: each project name reads 「operon」 etc. in 15.0px strong text (`text`, or `text_strong` when selected), with disclosure arrow on the left and status counts + `+` on the right.
- Pinned session rows: `📌` sits in the left indent gutter (`24..38px`) before the AI/status mark (`42px`), leaving the 2px `accent` selection bar (`row.rect.left() + 1px`) unobstructed; hovering `📌` shows 「ピン留め中」. Selected pinned rows draw `📌` in `selected_secondary_ink()` to preserve contrast against `row_selected`. Unpinned rows keep `42px` left margin so every row's AI mark and title align vertically under the project name.
- Legacy custom-command session rows (`session_mark_texture` is `None`): `📌` sits in the same left gutter (`24..38px`) before `session_status_icon_sized` (`42px`), leaving the exact same title width as unpinned rows even at minimum `180px` sidebar width.
- Central workspace tabs: a pinned session tab reads `📌 [AI] タイトル` with no `×` close button (`selected_secondary_ink()` on the active tab, `text_muted` on inactive tabs); an unpinned session tab reads `[AI] タイトル ×`; a document tab is unchanged.
- Empty, filtered, narrow (`180px`), rename-in-place (editor remains right of the mark), order-save error notice (「並び順を保存できませんでした: {error}」), and drag-reorder states behave identically to today.

## Design

Placement: `EXTEND` (route `modify`) of `session-column` (`ui_workspace_projects` in `src/app/screens.rs`) and `terminal-pane` (`ui_tab_workspace` in `src/app/screens.rs`).

- `src/config.rs`: add `pub(crate) const SESSION_PIN_SLOT_W: f32 = 14.0;` next to `SIDEBAR_ROW_INDENT` (`42.0`).
- `src/i18n_tables.rs`: change `("セッション一覧", "the session list")` to `("セッション一覧", "Session list")` in `EN_TABLE` (`KO_TABLE` `"세션 목록"` stays unchanged).
- `src/app/screens.rs`:
  - `ui_workspace_projects` heading: `RichText::new(DockPanel::Sessions.label()).size(12.5).color(palette.text_muted)`.
  - `ui_workspace_projects` project name: `.size(15.0).strong()`.
  - `ui_workspace_projects` session row: conditional `left` inner margin (`SIDEBAR_ROW_INDENT - SESSION_PIN_SLOT_W - SPACE_XS` when `pinned`, else `SIDEBAR_ROW_INDENT`); render `ICON_PIN` (`11.5px`, `palette.selected_secondary_ink()` when selected else `palette.text_muted`) in `vec2(SESSION_PIN_SLOT_W, COLUMN_SESSION_MARK_SIZE)` before `session_badge` / `session_status_icon_sized`; remove the old `if pinned` block after the mark.
  - `ui_tab_workspace` tab frame: for `WorkspaceTab::Session(id)` when `self.is_session_pinned(*id)`, render `ICON_PIN` (`11.5px`, `palette.selected_secondary_ink()` when active else `palette.text_muted`) in `vec2(SESSION_PIN_SLOT_W, CONTROL_HEIGHT_SMALL)` before `session_badge`; on the right side, draw `small_icon_button(..., ICON_CLOSE, ...)` only when `!matches!(tab, WorkspaceTab::Session(id) if self.is_session_pinned(*id))` and drop the trailing `ICON_PIN`.
- `src/tests.rs`: add the five `column_321_*` tests and update existing assertions that expected `"プロジェクト"` as the column's top label (`test_sidebar_renders_selected_tab_content`, `test_stacked_sidebar_shows_sessions_and_files_together`, `test_dock_*`) or `13.5px` project names / `"プロジェクト"` column heading (`session_column_project_is_a_heading_not_a_card`, `heading_320_the_column_heading_outranks_the_project_names`, `sidebar_302_column_uses_compact_type_scale`, `column_319_project_names_read_larger_than_their_rows`).
- Persisted shape, external commands, background work: none.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | yes | No new role: reuses `palette.text_muted`, `palette.text`, `palette.text_strong`, `palette.selected_secondary_ink()`, and `palette.accent`; selected/active `ICON_PIN` switches to `selected_secondary_ink()` (`>= 4.5:1` on `row_selected` per `DESIGN.md` Contrast). |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | yes | Reuses existing `ICON_PIN` (`icons::PUSH_PIN`) already in `ICON_VOCABULARY`. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | `DockPanel::Sessions.label()` (`tr("セッション一覧")`) is reused directly as the panel label; `SESSION_PIN_SLOT_W` and `SIDEBAR_ROW_INDENT` live in `src/config.rs`. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | Draw path and i18n only; no store or arrangement file format changes. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | No child process spawned or changed. |
| Documentation — user-facing docs change in all three languages together | yes | `docs/features/session-column/{brief,spec}.md`, `docs/features/terminal-pane/{brief,spec}.md`, `CHANGELOG.md`, and `EN_TABLE` in `src/i18n_tables.rs` updated together; `README*.md` do not specify font sizes or pin icon slot order. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | Local UI layout change only. |
| Budgets — any new scan or output path states its byte and item ceiling | no | No scan or output path added. |

## Flagged concerns

None.

## Acceptance

- `cargo test --locked column_321_ heading_320_ column_320_ column_319_ column_311_ sidebar_302_ session_285_ session_286_ session_column_project_is_a_heading_not_a_card test_sidebar_renders_selected_tab_content test_stacked_sidebar_shows_sessions_and_files_together test_dock_` passes, with `column_321_*` watched failing before implementation.
- `cargo test --locked every_text_size_is_a_level_on_the_type_scale design_md_ translation feature_behaviours_` passes.
- `python3 docs/sdlc/changes/321-sidebar-hierarchy-and-leading-pins/mutations.py` catches every mutation.
- `make q.fast` and `bash scripts/check-bands.sh` pass.
- `python3 scripts/verify-app.py run session-column` passes and the screenshot confirms 「セッション一覧」 (`12.5px` muted), larger project names (`15px`), and leading `ICON_PIN` placement.

## Rejected alternatives

- Placing 「プロジェクト」 below the filter tiles instead of 「セッション一覧」 at the top: leaves the panel without a top label or adds an extra row of vertical chrome.
- Putting `ICON_PIN` inline at `SIDEBAR_ROW_INDENT` (`42px`) before `[AI]`: shifts the AI icon and session title right on pinned rows so the vertical line of AI icons under the project name breaks.
- Keeping `13.5px` project names and shrinking session titles below `12.0px`: `11.5px` (`chip`) is reserved for micro-badges/counts and harms readability of session names.
