# Intent: streamline session list sidebar, card density, and header clutter

- **Status**: approved
- **Opened**: 2026-09-24

## Problem

1. **Sidebar instruction label noise**:
   The session sidebar permanently displays a static, multi-action instruction string ("ダブルクリックで名前を変更 · ⇆ 復元 · … worktree・名前を変更 · × 削除"). This takes up vertical space, adds cognitive clutter, and looks like prototype scaffolding rather than a polished tool.
2. **Session card density and premature title truncation**:
   Each session card in the sidebar permanently embeds a full combobox and button for restore target selection ("復元先 Codex CLI ▼ ⇆"). This dominates the card layout, adds visual noise for an action rarely used during routine browsing, and forces session and project titles like `example-webshop` to be prematurely truncated to `design-lookbo...`.
3. **Raw port numbers sprawled across header**:
   Active listening ports from workspaces (e.g. `:8791 node :49486 agy :49487 agy :52446 agy...`) are rendered as raw text links across the middle of the pane header, directly below breadcrumbs. When multiple agent processes or development servers are running, this line becomes excessively wide and distracting.
4. **Header metadata repetition**:
   The header renders project and agent information across two stacked lines with near-identical breadcrumb details, creating visual redundancy.

## Who feels it, and when

- Anyone using Operon with multiple sessions or workspaces who looks at the left session list and pane header, noticing visual clutter, truncated session titles, and sprawling lists of raw port numbers compared to modern development tools like VSCode and Cursor.

## Desired outcome

1. **Clean sidebar without static instruction text**:
   The static instruction string is eliminated from the sidebar. Operation affordances remain discoverable via hover tooltips, session card more-menus, and standard context actions.
2. **Compact, clear session cards**:
   Session cards focus cleanly on identity, branch, and status (e.g. `● IDLE`). The restore/handoff combobox is moved into the session's action menu or secondary controls, allowing session titles to display fully without premature ellipsis.
3. **Consolidated port indicator**:
   Active listening ports in the header are grouped into a neat, compact badge button (e.g. showing active port count with an icon), clicking which reveals the active ports and their browser links in a clean popover menu.
4. **Clean header layout**:
   The header clearly distinguishes session title, status chip, agent badge, project name, and git branch in an uncluttered single row hierarchy.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.
- No persisted store schema changes; store schema remains unchanged.
- All existing session capabilities (rename, restore/handoff, delete, worktree open) must remain fully functional.

## Systems likely affected

- `src/app/screens.rs` (session list rendering, session card layout, terminal panel header, port rendering)
- `src/app.rs` (session ports presentation helper)
- `src/ui/widgets.rs` (if new badge/chip widgets are added)
- `src/tests.rs` (tests covering session sidebar rendering, port display, and header actions)

## Open questions

All answered during design:
- Port accessibility: Ports are accessible via a dedicated compact badge with dropdown popover rather than taking up horizontal header bands.
- Restore target placement: The restore target combobox is accessible via the session card's `...` menu.

## Not in scope

- Complete rewrite to VSCode activity bar architecture (deferred to subsequent phase / proposal B).
- Modifying underlying tmux or agent execution process tracking.
