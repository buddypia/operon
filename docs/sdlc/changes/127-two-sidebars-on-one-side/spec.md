# Spec: one left sidebar — sessions above, files / conversation / changes below

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. When the side panel is open and docked left (the default), the session list
   and the side panel are drawn in one column of width `session_list_w`: the
   session list on top, the side panel below it, both visible in the same frame
   (`test_stacked_sidebar_shows_sessions_and_files_together`).
2. Docked left, the terminal pane is not narrowed by the side panel: only the
   one column sits left of it
   (`test_stacked_sidebar_returns_column_width_to_terminal`).
3. The split between the two halves is a horizontal splitter; its position is a
   fraction of the column height, default `SIDEBAR_SPLIT_DEFAULT` (0.4), and
   each half keeps at least `SIDEBAR_SECTION_MIN_H` (120px) whenever the column
   is tall enough for both; a non-finite fraction falls back to the default
   (`test_stacked_sidebar_heights_clamp`).
4. With the side panel folded the session list takes the whole column height
   and the panel's files are not drawn (`test_sidebar_collapses_when_empty`).
5. Docked right, the side panel is its own column on the right edge of the
   terminal as before (`test_sidebar_defaults_left_files_and_right_conversation`).
6. Dragging a side-panel tab and releasing it past the centre of the workspace
   (with the existing 24px dead zone) moves the panel to the other side; the
   drop zone is the whole workspace, so the left column itself is a valid target
   for a panel dragged from the right (`test_sidebar_tab_drag_between_sides`).
7. The unreachable `InspectorTab::Sessions` variant, which drew a second copy of
   the session list inside the side panel, is removed
   (`test_sidebar_tab_reorder_within_side`).

## Behaviour

- Opening the session workspace: the left column reads, top to bottom,
  「プロジェクト別セッション」 with its filter chips and grouped sessions, a
  thin horizontal separator, then the 「ファイル」「会話 n」「変更 n」 tab row and
  the selected tab's content. The terminal starts right after the column.
- Hovering the horizontal separator shows the vertical-resize cursor; dragging
  it moves the split; the halves never shrink below 120px each.
- 「サイドパネルを畳む」 folds the lower half; the session list grows to the full
  column. 「サイドパネルを表示」 in the session header brings it back.
- 「サイドバーを右側に移動」 (or dragging a tab to the right half) moves the lower
  half to the right edge of the terminal as its own column; 「サイドバーを左側に
  移動」 brings it back under the session list.
- History view (「履歴」) or no selected session: no lower half; nothing to show.
- No new Japanese strings; every label already exists.

## Design

Verdict: `EXTEND` — `ui_terminal_workspace` (`src/app/screens.rs`) already owns
the left column and `ui_session_inspector` (`src/ui/session_tree.rs`) the panel;
evidence: `git grep -n "fn ui_terminal_workspace\|fn ui_session_inspector"`.

- `src/config.rs`: `SIDEBAR_SPLIT_DEFAULT: f32 = 0.4`,
  `SIDEBAR_SECTION_MIN_H: f32 = 120.0`.
- `src/ui/session_tree.rs`: pure `stacked_sidebar_heights(total_h, split) ->
  (f32, f32)`; `InspectorTab::Sessions` and its draw arm removed; layout doc
  comment updated.
- `src/app.rs`: `session_list_split: f32` on `OperonApp`, default
  `SIDEBAR_SPLIT_DEFAULT`.
- `src/app/screens.rs`: `ui_terminal_workspace` draws the stacked column when the
  panel is docked left, open, and a session is selected outside the history
  view; `ui_terminal_panel` draws the panel only when docked right; the tab drag
  handler moves from `ui_terminal_panel` into a workspace-level method using the
  workspace rect.
- Draw path work per frame: one clamp and two multiplications; no allocation.
- No persisted shape, no subprocess, no new dependency.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | yes | splitter line reuses `palette.border_subtle`; no literal |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | no new mark |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | the split default and minimum live in `src/config.rs` and are read by the helper and its test |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | in-memory field only; store untouched |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | no process spawned |
| Documentation — user-facing docs change in all three languages together | no | the READMEs do not describe the column layout |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | none added |
| Budgets — any new scan or output path states its byte and item ceiling | no | no new scan |

## Flagged concerns

None.

## Acceptance

- `cargo test --locked` passes, including the tests named in Requirements.
- `cargo clippy --locked -- -D warnings` and `cargo fmt --check` are clean.
- In the packaged app: the session list and the file tree are in one left
  column; dragging the separator between them resizes both; moving the panel to
  the right makes it a separate right column.

## Rejected alternatives

- **One sidebar with four tabs (セッション / ファイル / 会話 / 変更), VS Code
  activity-bar style** — saves the same width, but hides the session list while
  a file or diff is open. Operon's job is supervising several agents; the
  status of the others (要対応) must stay in view, and a tab badge is a weaker
  signal than the row itself.
- **Sessions as horizontal tabs above the terminal (browser / tmux style)** —
  wins width, but loses project grouping and the status filter, and stops being
  readable past roughly eight sessions, which is a normal count here.
- **Sessions left, side panel right by default (Conductor, Codex app, Apple HIG
  inspector on the trailing edge)** — a sound convention, but it keeps two
  columns and undoes change 122, where the person asked for files on the left.
  It stays one click away (「サイドバーを右側に移動」).
- **Side panel above, sessions below** — the file tree and diff need the height
  more, and the eye starts at the top: the navigation that decides what the rest
  shows belongs first (VS Code Explorer keeps its open-editors list above the
  tree for the same reason).
- **Status quo** — two columns, ~540px, both half empty.
