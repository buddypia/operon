# Spec: file list sidebar on the left by default and draggable sidebar contents

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. On `Page::Sessions`, the left sidebar contains `InspectorTab::Files` and `InspectorTab::Sessions` tabs by default, with `InspectorTab::Files` as the initially selected active tab (`test_sidebar_defaults_left_files_and_right_conversation`).
2. On `Page::Sessions`, the right sidebar contains `InspectorTab::Conversation` and `InspectorTab::Changes` tabs by default, with `InspectorTab::Conversation` as the initially selected active tab (`test_sidebar_defaults_left_files_and_right_conversation`).
3. Each tab header in both sidebars is interactive and draggable. When a user drags a tab header and drops it onto the opposite sidebar drop zone, the tab is transferred to the destination sidebar and becomes the active tab there (`test_sidebar_tab_drag_between_sides`).
4. When a user drags a tab within the same sidebar header, the tab is reordered within that sidebar's tab list (`test_sidebar_tab_reorder_within_side`).
5. When `InspectorTab::Files` is selected in either sidebar, the file tree for the active session's project/worktree is rendered. When `InspectorTab::Sessions` is selected, the session list (`ui_terminal_session_tabs`) is rendered. When `InspectorTab::Conversation` is selected, the prompt timeline is rendered. When `InspectorTab::Changes` is selected, git diff changes are rendered (`test_sidebar_renders_selected_tab_content`).
6. If all tabs are dragged out of a sidebar, that sidebar automatically collapses to 0 width, allowing the terminal viewport to expand and utilize the reclaimed space (`test_sidebar_collapses_when_empty`).
7. The vertical splitters separating the left sidebar from the terminal and the terminal from the right sidebar can be dragged horizontally to resize the sidebar widths within the clamped range of 180.0 to 480.0 pixels (`test_sidebar_splitter_resize_clamp`).
8. The right sidebar can be toggled open and collapsed using the sidebar toggle button in the session header (`test_sidebar_toggle_visibility`).

## Behaviour

What the user sees and does:
- Opening the session workspace displays the file list (`ファイル`) on the left side of the terminal, showing the directory tree, file filter search input, and git status badges.
- Next to the 「ファイル」 tab in the left sidebar header is the 「セッション」 tab. Clicking 「セッション」 displays the session cards and filter chips in the left sidebar.
- On the right side of the terminal, the sidebar displays 「会話」 (prompt turns) and 「変更」 (git diff snapshot) tabs, with 「会話」 active by default.
- Dragging a tab displays a subtle drag highlight and preview. When the drag moves across the center of the terminal towards the opposite sidebar and is released, the tab smoothly docks into the target sidebar.
- Dragging the separator bar between a sidebar and the terminal changes the mouse cursor to a horizontal resize icon (`ResizeHorizontal`) and adjusts the sidebar width continuously.
- When no session is selected or when a project has no files, appropriate empty states are rendered (e.g. 「読み取り可能なファイルはありません」 or 「セッションはまだありません」).

Japanese user-facing text:
- 「ファイル」: Files tab label
- 「セッション」: Sessions tab label
- 「会話」: Conversation tab label
- 「変更」: Changes tab label
- 「ドラッグして移動」: Tooltip on tab headers explaining that tabs can be dragged to relocate them
- 「サイドパネルを表示 / 非表示」: Tooltip on toggle button for the side panel

## Design

Verdict: `EXTEND` — existing sidebar, inspector, and terminal workspace modules are extended to support flexible docking and dragging.

Modules and structures modified:
- `src/ui/session_tree.rs`:
  - Extend `InspectorTab` enum with `Sessions`:
    ```rust
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub(crate) enum InspectorTab {
        Files,
        Sessions,
        Conversation,
        Changes,
    }
    ```
  - Add `SidebarSide` enum:
    ```rust
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub(crate) enum SidebarSide {
        Left,
        Right,
    }
    ```
  - Provide pure helper functions for moving tabs between sides, reordering, and computing available layout widths.
- `src/app.rs`:
  - Add sidebar configuration and drag state fields to `OperonApp`:
    - `left_sidebar_tabs: Vec<InspectorTab>`
    - `right_sidebar_tabs: Vec<InspectorTab>`
    - `left_sidebar_tab: InspectorTab`
    - `right_sidebar_tab: InspectorTab`
    - `left_sidebar_w: f32`
    - `right_sidebar_w: f32`
    - `show_left_sidebar: bool`
    - `dragging_sidebar_tab: Option<(InspectorTab, SidebarSide)>`
- `src/app/screens.rs`:
  - Refactor `ui_terminal_workspace` to render:
    - Left sidebar (if `show_left_sidebar` and `!left_sidebar_tabs.is_empty()`) using `left_sidebar_w`.
    - Resizable vertical splitter for the left sidebar.
    - Central terminal workspace taking remaining width.
    - Resizable vertical splitter for the right sidebar.
    - Right sidebar (if `show_session_inspector` and `!right_sidebar_tabs.is_empty()`) using `right_sidebar_w`.
  - Draw draggable tab headers with drag start detection, drop target zones, and tab content switching.
- `src/i18n_tables.rs`:
  - Register any new Japanese, English, and Korean UI tokens.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | yes | Reuses existing palette tokens (`palette.raised`, `palette.border_subtle`, `palette.accent_soft`, `palette.text_strong`, `palette.text_muted`) without adding new colour literals |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | yes | Reuses established icons (`ICON_FOLDER_CLOSED`, `ICON_TERMINAL`, `ICON_DISCLOSURE_OPEN`, `ICON_DISCLOSURE_CLOSED`) |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | Min and max sidebar widths (`SIDEBAR_MIN_W: f32 = 180.0`, `SIDEBAR_MAX_W: f32 = 480.0`, `SIDEBAR_DEFAULT_W: f32 = 268.0`) reside in `src/config.rs` |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | No persisted store schema change; sidebar layout preferences live in in-memory `OperonApp` state |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | No new subprocesses or command executions are introduced |
| Documentation — user-facing docs change in all three languages together | no | Internal UI layout improvement conforming to standard expectations; no manual documentation changes required |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | Operates strictly locally in-memory with pure immediate-mode egui widgets; no network or telemetry calls |
| Budgets — any new scan or output path states its byte and item ceiling | no | No new scanning or output paths; reads existing cached file tree and session collections |

## Flagged concerns

- **Drag interactions in immediate-mode egui without interfering with text selection or scrolling** — egui's `Sense::click_and_drag()` on tab pill handles captures drag movements when the pointer moves beyond the drag threshold while pressed, ensuring that clicking still selects the tab normally and clicks within the underlying scroll area are never swallowed.
- **Handling empty sidebars when all tabs are transferred to one side** — If a user moves all four tabs to the left sidebar or all to the right sidebar, the emptied sidebar cleanly collapses to width 0 so the terminal expands to fill the space without leaving an awkward blank gutter.

## Acceptance

How the finished change is judged, as commands and observations:

- `cargo test --locked` passes, including `test_sidebar_defaults_left_files_and_right_conversation`, `test_sidebar_tab_drag_between_sides`, `test_sidebar_tab_reorder_within_side`, `test_sidebar_renders_selected_tab_content`, and `test_sidebar_splitter_resize_clamp`.
- In the running app (`Operon.app`), opening the session workspace shows the file list on the left sidebar by default, with the 「セッション」 tab next to it.
- Dragging the 「ファイル」 tab from the left sidebar to the right sidebar smoothly moves it to the right sidebar.
- Dragging the vertical splitters resizes the sidebars smoothly with the horizontal resize cursor.

## Rejected alternatives

- Storing dock layouts in the persisted `Store` — Rejected because changing the store requires a schema version bump and disk migration, whereas in-memory session layout in `OperonApp` is lightweight, robust, and zero-risk.
- Restricting tabs to fixed sidebars (left only / right only) — Rejected because the user explicitly asked for the ability to move sidebar contents by dragging.
- Floating modal windows for file list — Rejected because dockable sidebars preserve the integrated single-window workspace experience.
