# Spec: The new-terminal button follows the last tab

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. In every pane's tab strip, the + 「空のターミナルを開く」 sits directly to the right of the last tab, separated only by the row's item spacing, and with two short tabs it is nowhere near the pane's right edge. Pressing it still starts a plain terminal in that pane. Test: `session_260_plus_follows_last_tab`.
2. When the tabs are wider than the pane, the + stays inside the pane at its right edge and the scrolling tabs clip before it (change 254's behaviour, unchanged). Test: `session_254_terminal_action`.
3. In an empty workspace (no pane), the + sits at the left edge of the workspace, where the first tab will appear, and keeps its disabled and missing-tmux behaviour. Tests: `session_260_empty_plus_sits_where_the_first_tab_appears`, `session_254_empty_workspace`.

## Behaviour

Only the position of the existing + changes. Its icon (ICON_ADD), its 28px size,
its hover and accessible name 「空のターミナルを開く」, and what a press does are
unchanged: 「ターミナルを起動しています…」, then the new tab in the starting,
active or failed state, opened in the clicked pane's project and worktree.

- Few tabs: `[● セッション1 ×][● セッション2 ×][+]`, then empty strip.
- Tab added: the new tab takes the +'s place and the + moves to its right.
- Many tabs: the tabs scroll horizontally inside the pane and clip before the +,
  which stays at the pane's right edge.
- Split panes: each pane's + follows that pane's own last tab.
- Empty workspace: the + is at the left above 「セッションを選択」; disabled
  with no project; with no tmux the existing notice
  「tmux が必要です。「設定」を開いてください。」 and no session.
- Long names still truncate at 132px with the full name on hover; rename, close
  and drag keep working.

## Design

Placement: `EXTEND` — `ui_tab_workspace` in `src/app/screens.rs` already draws
the strip and the + (change 254, `session_254_terminal_action` pins it). No new
behaviour, only its position.

- The tab region's `egui::ScrollArea::horizontal()` takes
  `auto_shrink([true, true])` instead of `[false, true]`. In egui 0.31.1
  (`containers/scroll_area.rs` line 934) a horizontally scrolling area with
  horizontal auto-shrink takes `min(available, content)` width, and the
  enclosing `allocate_ui_with_layout` advances only by its child's `min_rect`,
  so the + that follows in the same horizontal row lands right after the last
  tab. The region's maximum width stays `available - CONTROL_HEIGHT - SPACE_SM`,
  so with overflow the + keeps its right-edge slot and the clip that stops
  before it is unchanged.
- The empty-workspace + is drawn in a `horizontal` row instead of
  `with_layout(Layout::right_to_left(Align::Center))`. That layout took the
  body's whole height, centred the + vertically at the right edge, and pushed
  「セッションを選択」 and its line below the window — measured in the test
  fixture, not visible in change 254's mock.
- No persisted type, string, icon, colour, size, dependency, process, scan or
  output budget changes. Nothing new is computed per frame: the scroll area
  already measures its content each frame.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | No colour is touched; the strip keeps `row_selected`, `card` and `accent`. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | yes | The existing `ICON_ADD`; no new glyph. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | no | No identifier is introduced. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | Nothing persisted, cancelled, migrated or budgeted changes. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | The press still calls `launch_workspace_terminal`; no launch code changes. |
| Documentation — user-facing docs change in all three languages together | no | `README.md` does not describe the +'s position; read to confirm. No string changes, so `src/i18n_tables.rs` is untouched. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | None added. |
| Budgets — any new scan or output path states its byte and item ceiling | no | No scan or output path. |

## Flagged concerns

None. The person's request names the place ("タブ名の右側"); keeping the + at the
right edge on overflow is what keeps the change-254 guarantee that it never
scrolls away, and the empty-workspace position follows from "where the next tab
appears".

## Acceptance

- `make q.fast` passes, and `cargo test --locked session_260_` and `cargo test --locked session_254_` pass.
- The mutation sweep catches the tab region returned to full width and the empty-workspace + returned to the right edge.
- Rendered fixture images at 1100px and 640px show the + right after the last tab with few tabs, and at the pane's right edge with many.
- CI passes before landing; package from main and install.

## Rejected alternatives

- Putting the + inside the scrolling tab row: it scrolls out of view when tabs overflow, which change 254 rejected.
- Measuring the tab row on the previous frame and sizing the region from it: a frame of lag and app state for what the scroll area already reports.
- A second + at the right edge as well: two buttons for one action.
