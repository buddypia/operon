# Spec: streamline session list sidebar, card density, and header clutter

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. The static operation guide label ("ダブルクリックで名前を変更 · ⇆ 復元 · … worktree・名前を変更 · × 削除") is completely removed from the session list sidebar (`ui_terminal_session_tabs`).
2. Session cards in the sidebar display session name, status badge, branch, and secondary action button without embedding the full restore combobox by default.
3. The restore/handoff action remains fully accessible via the session card's `...` action menu ("他の CLI へ復元...").
4. Active listening ports from workspaces are consolidated into a compact badge button (`🔌 {N} ポート` / `🔌 {port}` if 1 port) in the pane header instead of printing a sprawling horizontal line of text. Clicking the badge opens an egui menu listing each port with its process name and one-click "ブラウザで開く" action.
5. If no ports are active, no port indicator is rendered.
6. The terminal panel header displays session title, status chip, agent badge, project name, and git branch in an orderly, compact layout without duplicated breadcrumbs.

## Behaviour

- **Session sidebar**:
  - The status filter chips (`要対応`, `空き`, `実行中`, `終了`) remain interactive at the top.
  - Below the filter chips, the project list begins immediately without the static instruction label.
  - Each session card shows:
    - Top row: Session title (with more horizontal space) and `···` more actions menu button.
    - Second row: Agent icon, status chip (`● IDLE`, `⟳ 実行中`, etc.), and Git branch name.
    - The `···` menu offers:
      - "名前を変更"
      - "worktree を作成"
      - "他の CLI へ復元（Codex / Claude / Antigravity）..."
      - "セッションを削除"
  - Hovering a session card shows a tooltip explaining double-click to rename and right-click or `···` for actions.
- **Pane header**:
  - When ports are open, a neat badge appears in the metadata row: e.g. `🔌 11 ポート`.
  - Clicking the badge opens a dropdown listing each active port (e.g. `:8791 (node)`, `:49486 (agy)`), allowing the user to click any port to open it directly in the default browser.
  - The sprawling list of raw port text across the header is eliminated.

## Design

- In `src/app/screens.rs`:
  - In `ui_terminal_session_tabs`:
    - Remove the instruction label `RichText::new(tf!("ダブルクリックで名前を変更..."))`.
    - Adjust session card layout to omit the inline restore selector and instead provide full width for the session title and concise status details.
    - Ensure session card `···` menu includes the restore/handoff option (`self.open_restore_modal(session.id)`).
  - In `ui_terminal_panel`:
    - Refactor `self.ui_session_ports` call to render as a compact popover menu button inside the header metadata strip.
- In `src/app.rs`:
  - Refactor `ui_session_ports` to render an egui menu button with an icon and count, displaying the port items in the menu dropdown rather than sprawling horizontally.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | Applies | Uses existing `palette.text_muted`, `palette.accent_text`, `palette.raised`, and `palette.control` colors without inventing untracked palette roles. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | Applies | Uses existing `ICON_RESTORE`, `ICON_MORE`, and `ICON_CLOSE` glyphs from `src/glyphs.rs`. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Does not apply | No persisted identifier shapes or config strings are changed. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Does not apply | No change to the local store schema, persistence, or process locks. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Does not apply | No new subprocesses or CLI launch paths introduced. |
| Documentation — user-facing docs change in all three languages together | Does not apply | Internal UI layout cleanup; user documentation does not reference the removed instruction sentence. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Applies | Completely local UI refactoring; zero remote calls or unsafe code. |
| Budgets — any new scan or output path states its byte and item ceiling | Does not apply | No new scans or output file operations added. |

## Flagged concerns

None. All interactions have been validated against the existing egui immediate-mode architecture.

## Acceptance

- `cargo test --locked` passes, including tests asserting the absence of the static instruction label and asserting the consolidated port popover.
- Running the application shows an uncluttered session list with full session titles visible, no verbose static hint text, and ports consolidated into a clean badge menu.

## Rejected alternatives

- *Hiding ports entirely*: Rejected because developers frequently need to know which local servers their agents have launched (e.g. Next.js, Vite dev servers) to open them in the browser.
- *Keeping the inline restore selector*: Rejected because handoff/restore is an occasional migration action, not a high-frequency action that justifies 40% of every card's height.
