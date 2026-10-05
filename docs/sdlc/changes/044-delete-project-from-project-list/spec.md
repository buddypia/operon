# Spec: Delete project from the projects list view

- **Intent**: ./intent.md
- **Status**: approved

## Requirements

1. Each project card in the project list view (`Page::Projects`, when no project is selected) must provide a visible delete action button (`ICON_CLOSE`) on the right side of the card, positioned next to the Finder reveal button.
2. Right-clicking a project card in the project list view must open a context menu providing both `Finder で表示` and `プロジェクトを削除`.
3. Clicking the delete button or selecting `プロジェクトを削除` from the context menu must set `pending_project_removal = Some(project.id)`.
4. While a project has `pending_project_removal == Some(project.id)`, its row in the project list must render an in-place confirmation card with `palette.danger` border instead of the standard clickable project card.
5. The confirmation card must state the project name being deleted and explain that only records are removed while files, branches, and worktrees remain intact on disk.
6. The confirmation card must provide a danger action button `Operon から削除` that calls `remove_project(project.id)` and a `キャンセル` button that clears `pending_project_removal = None`.
7. While pending removal, clicking anywhere on the confirmation card must not trigger project selection or navigation into the project.
8. The project list must use `clickable_card` rather than `card` with manual `ui.interact`, guaranteeing that inner icon button clicks are never intercepted by the row background click handler.
9. All user-facing strings must use `tr()` or `tf!()` and have corresponding entries in `EN_TABLE` and `KO_TABLE` in `src/i18n_tables.rs`, sorted by message ID.

## Behaviour

### Normal Project List Row

In `Page::Projects` when `selected_project().is_none()`:

```
+-----------------------------------------------------------------------------------------+
| [ICON_PROJECT]  operon                                                [↗ Finder] [✕ 削除] |
|                 ~/dev/playground/operon                                                 |
+-----------------------------------------------------------------------------------------+
```

- Clicking anywhere on the card (except the buttons) opens the project overview.
- Hovering over `[↗ Finder]` shows tooltip `Finder で表示`. Clicking reveals the directory in Finder.
- Hovering over `[✕ 削除]` shows tooltip `プロジェクトを削除`. Clicking transitions the card to the confirmation state.
- Right-clicking the card opens a context menu:
  - `Finder で表示`
  - `プロジェクトを削除`

### In-Place Removal Confirmation State

When `pending_project_removal == Some(project.id)`:

```
+-----------------------------------------------------------------------------------------+
| 「operon」を Operon から削除しますか？                                                  |
| 記録だけを削除します。ファイル・ブランチ・worktree は残ります。                         |
|                                                                                         |
| [ Operon から削除 ]  [ キャンセル ]                                                     |
+-----------------------------------------------------------------------------------------+
```

- The frame border is highlighted in danger colour (`palette.danger`).
- Heading: `「{name}」を Operon から削除しますか？` in bold.
- Subtitle: `記録だけを削除します。ファイル・ブランチ・worktree は残ります。` in muted text.
- Primary danger button: `Operon から削除` (filled with danger colour). Clicking executes `self.remove_project(project.id)`.
- Cancel button: `キャンセル`. Clicking resets `self.pending_project_removal = None`.
- If an active session is currently running in that project, `remove_project` stops deletion and surfaces notice `実行中のセッションを停止または完了してください。`.

## Design

- `src/app/screens.rs`:
  - In `ui_projects(&mut self, ui: &mut egui::Ui)`:
    - In the project list loop (`for project in projects`):
      - Check `let is_pending = self.pending_project_removal == Some(project.id);`.
      - If `is_pending`:
        - Render confirmation card using `card_frame(palette).stroke(egui::Stroke::new(1.0, palette.danger))`.
        - Display confirmation copy:
          - `tf!("「{name}」を Operon から削除しますか？", name = &project.name)`
          - `tr("記録だけを削除します。ファイル・ブランチ・worktree は残ります。")`
        - Buttons:
          - `tr("Operon から削除")`: on click calls `self.remove_project(project.id)`.
          - `tr("キャンセル")`: on click sets `self.pending_project_removal = None`.
      - If not pending:
        - Use `clickable_card(ui, palette, ("project-row", project.id), true, |ui| ...)`
        - In the `right_to_left` layout inside the row:
          - First add `small_icon_button(ui, ICON_CLOSE, tr("プロジェクトを削除"))` (places it at the right edge).
          - Next add `small_icon_button(ui, ICON_OPEN_EXTERNAL, tr("Finder で表示"))`.
        - Add context menu to row response with `Finder で表示` and `プロジェクトを削除`.
        - If delete button or context menu delete is triggered, set `self.pending_project_removal = Some(project.id)`.
        - If finder button or context menu finder is triggered, call `request_system_action` with `reveal_path`.
        - If row was clicked and no action was triggered, call `self.select_project(Some(project.id))` and `self.project_tab = ProjectTab::Overview`.
- `src/i18n_tables.rs`:
  - Add missing translation keys to `EN_TABLE` and `KO_TABLE` in strictly sorted order:
    - `"「{name}」を Operon から削除しますか？"`
    - `"プロジェクトを削除"`
    - `"記録だけを削除します。ファイル・ブランチ・worktree は残ります。"`
    - `"Operon から削除"`
- `src/tests.rs`:
  - Add unit test verifying that project list card exposes deletion action and confirmation flow.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | Reuses existing palette tokens (`danger`, `border_subtle`, `text_strong`, `text_muted`, `accent_text`). |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | Uses existing icons from `src/glyphs.rs` (`ICON_CLOSE`, `ICON_OPEN_EXTERNAL`, `ICON_PROJECT`). |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | No new persisted or inter-process identifiers introduced. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | Deletion reuses the existing atomic `remove_project` method; no schema format changes. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | Finder reveal continues using existing safe `reveal_path` system action helper. |
| Documentation — user-facing docs change in all three languages together | No | UI text only; no markdown documentation changes required. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Operates purely on local state with zero network calls or unsafe blocks. |
| Budgets — any new scan or output path states its byte and item ceiling | No | No new directory scans or buffer allocations added. |

## Flagged concerns

- **Accidental double click on project card triggering deletion** — Placing `ICON_CLOSE` on the card could risk accidental clicks if hit targets overlap. Using `clickable_card` separates child button bounds from card bounds, and requiring a second confirmation click (`Operon から削除`) completely eliminates the risk of accidental project loss.

## Acceptance

- `cargo test --locked` passes, including all i18n ordering and completeness tests.
- `scripts/check-readiness.sh` on `docs/sdlc/changes/044-delete-project-from-project-list` returns Go (`0`).
- The project list displays `[✕ 削除]` on every project card.
- Clicking `[✕ 削除]` replaces the card with the danger confirmation card without navigating into the project.
- Clicking `キャンセル` restores the normal card.
- Clicking `Operon から削除` safely removes the project from the list and store.

## Rejected alternatives

- Modal dialog overlay: Rejected because an inline card replacement matches the established pattern used elsewhere in Operon (e.g. project overview and session deletion) and keeps focus directly on the target project without jarring window-level modality.
- Direct deletion without confirmation: Rejected because project removal clears session history and handoff records for that project. An irreversible action must always have confirmation.
