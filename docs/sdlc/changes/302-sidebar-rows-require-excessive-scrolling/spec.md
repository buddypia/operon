# Spec: compact vertical spacing and type scale in the session sidebar

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. On `Page::Sessions` rendered through `ui_central_panel` (where the outer layout sets `item_spacing = vec2(10.0, 10.0)` and `apply_interface_metrics` sets `interact_size.y = 28.0`), `ui_workspace_projects` tightens vertical spacing (`SPACE_XS` = `4.0` at column level and `SIDEBAR_ROW_GAP` = `2.0` inside each project frame), tightens frame margins (`Margin::symmetric(SPACE_XS as i8, 2)` on projects and `Margin::symmetric(SPACE_XS as i8, 1)` on session rows, with `1px` padding in `ui_sidebar_agent_segment`), trims `ui.horizontal` row minimum height to `CONTROL_HEIGHT_SMALL` on project headings and `COLUMN_SESSION_MARK_SIZE` on session rows (keeping `workspace_session_name_editor` at `CONTROL_HEIGHT_SMALL`), and removes the trailing `add_space(SPACE_SM)` between projects — so consecutive standard session rows sit at `<= 24.0px` center-to-center pitch with `<= 6.0px` vertical gap, the gap from a project heading to its first session row is `<= 12.0px`, and the gap from one project's last session row to the next project heading is `<= 14.0px`.
   Test: `sidebar_302_session_rows_and_project_headings_are_compact`.
2. Sidebar labels step down one notch on `DESIGN.md`'s existing typography scale: `12.0` (`caption`) for session row titles and project headings, and `11.5` (`chip`) for the `"プロジェクト"` column header, the filter summary count `"{count} 件"`, and per-project `status_count_chip` counts — keeping every `.size(...)` and `FontId` on the documented scale and every button at `CONTROL_HEIGHT` or `CONTROL_HEIGHT_SMALL`.
   Test: `sidebar_302_column_uses_compact_type_scale`.
3. With compact sidebar spacing and `SIDEBAR_ROW_GAP` wired to `ColumnDrag::Session` (and `SPACE_XS` to `ColumnDrag::Project`), open sessions appear in the exact same order in both the left sidebar and the center-top workspace tab strip, pinning/unpinning (`ピン留め` / `ピン留めを外す`) moves the session first with `ICON_PIN` in both places, and dragging a row across the `2.0px` gap reorders both the sidebar and the center-top tabs.
   Test: `sidebar_302_compact_rows_still_pin_reorder_and_sync_with_top_tabs`.

## Behaviour

- On `Page::Sessions`, the left sidebar draws `"プロジェクト"` (`11.5px`) and `"履歴"` at the top, followed by the scrollable filter band (`要対応` / `実行中` / `アイドル` tiles + CLI segment + hairline), then project headings (`12.0px` muted, strong when selected) with a `1px` hairline and compact session rows (`12.0px`, `~20px` tall, `2px` gap).
- Empty project (no open tabs): displays `"開いているセッションはありません"` inside the project block as today.
- Filtered empty state: when filters match zero sessions across projects, displays `"一致するセッションはありません。"` as today; when active filters show matching count, `"{count} 件"` renders at `11.5px` beside `"クリア"`.
- Pinned sessions: show `ICON_PIN` (`11.5px`) before the title with `"ピン留め中"` on hover and cannot be closed until unpinned (`"ピン留め中のセッションは閉じられません。ピン留めを外してから閉じてください。"`).
- Long titles / custom commands: truncate inside the column (`ui.available_width()`) with full text, state label, working-time hint, and `"ドラッグで並べ替え"` in hover tooltips.
- Order-save failure or interrupted drag: failed write surfaces `"並び順を保存できませんでした: {error}"` while keeping the in-memory order; `Esc` or pointer loss outside release cancels `workspace_column_drag` without modifying `session_arrangement`.
- Installed app parity: shipping from `main` after merge updates `/Applications/Operon.app` and launches it so the live app stops running the stale 2026-10-24 `/Applications/Operon Stable.app` (`5f2558b`) bundle that lacked pinning and the unified top tab strip.

## Design

Placement: `EXTEND` (route `modify`) — edits existing sidebar rendering and constants without adding new modules, persisted fields, external commands, or per-frame heap work beyond existing draw code:

- `src/config.rs`: add `pub(crate) const SIDEBAR_ROW_GAP: f32 = 2.0;` right after `SIDEBAR_DEFAULT_W`.
- `src/ui/widgets.rs`: in `status_count_chip`, change `egui::FontId::proportional(12.5)` to `egui::FontId::proportional(11.5)`.
- `src/app/screens.rs`:
  - `ui_sidebar_agent_segment`: `.inner_margin(egui::Margin::same(1))` (was `2`).
  - `ui_sidebar_filter_summary`: `RichText::new(...).size(11.5)` (was `12.0`).
  - `ui_workspace_projects`:
    - Override `ui.spacing_mut().item_spacing.y = SPACE_XS;` at function start (overriding `ui_central_panel`'s `10.0`).
    - `"プロジェクト"` column header label `.size(11.5)` (was `12.5`).
    - Project `egui::Frame::new()`: `.inner_margin(egui::Margin::symmetric(SPACE_XS as i8, 2))` (was `Margin::same(SPACE_SM as i8)`), and inside `.show(ui, |ui| { ... })` set `ui.spacing_mut().item_spacing.y = SIDEBAR_ROW_GAP;` and `ui.spacing_mut().interact_size.y = CONTROL_HEIGHT_SMALL;` for the project heading row.
    - Project heading `RichText::new(&project.name)` (both selected and unselected branches): `.size(12.0)` (was `12.5`).
    - Session row `egui::Frame::new()`: `.inner_margin(egui::Margin::symmetric(SPACE_XS as i8, 1))` (was `Margin::symmetric(SPACE_SM as i8, 2)`), and before its `ui.horizontal` set `ui.spacing_mut().interact_size.y = COLUMN_SESSION_MARK_SIZE;` and inside `ui.horizontal` set `ui.spacing_mut().item_spacing.x = SPACE_XS;` (also clamp fallback command label allocation to `egui::vec2(width, COLUMN_SESSION_MARK_SIZE)`).
    - Session title `RichText::new(&title)`: `.size(12.0)` (was `12.5`).
    - Selected row left accent bar: `row.rect.top() + 1.0` .. `row.rect.bottom() - 1.0` (was `+/- 2.0`).
    - Remove `ui.add_space(SPACE_SM);` after `targets.push(project.id, card.response.rect);`.
    - Wire `ColumnDrag` drop gap: `ColumnDrag::Project(_) => (&targets, SPACE_XS)`, `ColumnDrag::Session { .. } => (&session_targets, SIDEBAR_ROW_GAP)`.
- `src/tests.rs`:
  - Update `session_column_project_is_a_heading_not_a_card` to assert `.size(12.0)` on `&project.name`.
  - Add `sidebar_302_session_rows_and_project_headings_are_compact`, `sidebar_302_column_uses_compact_type_scale`, and `sidebar_302_compact_rows_still_pin_reorder_and_sync_with_top_tabs`.
- `src/app/screens.rs` line budget: stays below `10000` (`propose` ceiling in `docs/sdlc/bands.yaml`).
- Persisted shape, external commands, background work: none.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | Uses existing `Palette` roles (`text`, `text_strong`, `text_muted`, `row_selected`, `accent`, `border_subtle`); checked `.claude/rules/palette-and-glyphs.md`. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | Reuses existing `ICON_PIN`, `ICON_ADD`, `ICON_DISCLOSURE_*`, and `session_badge` (`COLUMN_SESSION_MARK_SIZE`). |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | Adds `SIDEBAR_ROW_GAP` once in `src/config.rs` and uses it for both project frame spacing and `ColumnDrag::Session` drop-line gap; checked `.claude/rules/identifiers.md`. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | No store, migration, cancellation, or tmux code changes; `session-order.json` format unchanged. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | No child processes or launch paths. |
| Documentation — user-facing docs change in all three languages together | no | No user-facing documentation describes sidebar pixel gaps or font sizes. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | no | Immediate-mode drawing only; zero `unsafe` or network calls. |
| Budgets — any new scan or output path states its byte and item ceiling | no | No scan or output path; draw path adds zero heap allocations (checked `.claude/rules/rust.md`). |
| Controls & Type Scale — `.claude/rules/palette-and-glyphs.md` | yes | `12.0` (`caption`) and `11.5` (`chip`) are existing levels in `DESIGN.md`; no button drops below `CONTROL_HEIGHT` / `CONTROL_HEIGHT_SMALL`, verified by `every_button_keeps_a_documented_height` and `every_text_size_is_a_level_on_the_type_scale`. |

## Flagged concerns

None.

## Acceptance

- `cargo test --locked sidebar_302_` passes (3 tests, watched failing before code edits).
- `cargo test --locked session_column_project_`, `sidebar_251_`, `sidebar_252_`, `session_285_`, `workspace_project_`, `every_button_keeps_a_documented_height`, `every_text_size_is_a_level_on_the_type_scale`, `design_md_documents_exactly_what_the_app_paints`, and `every_theme_stays_readable` pass.
- `python3 docs/sdlc/changes/302-sidebar-rows-require-excessive-scrolling/mutations.py` catches every mutation.
- `make q.fast` passes and CI passes before merging to `main`.
- After shipping `/Applications/Operon.app` from `main`, the launched app displays the compact sidebar, session pinning (`ピン留め`), and the unified center-top session tab strip; the user is informed why `/Applications/Operon Stable.app` previously hid pinning and tabs.

## Rejected alternatives

- Introduce new `11.0px` or `10.5px` font levels in `DESIGN.md`: unnecessary when `12.0` (`caption`) and `11.5` (`chip`) already sit on the scale and stay legible on HiDPI and standard displays.
- Shrink `status_filter_tile` (`28px`) or `agent_filter_toggle` (`24px`) button heights below `CONTROL_HEIGHT` / `CONTROL_HEIGHT_SMALL`: violates `DESIGN.md` Controls and `every_button_keeps_a_documented_height`.
- Overwrite or delete `/Applications/Operon Stable.app` automatically: violates Change 265 / destructive-action policy; the user is asked first.
