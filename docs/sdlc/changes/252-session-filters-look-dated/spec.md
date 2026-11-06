# Spec: Compact, counted sidebar filters

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. The sidebar draws the four status choices as tiles in a two-column block at the default width, each with its group's mark, word and a count; no 「CLI」, 「ステータス」, 「すべて」 or 「承認・回答待ち」 words remain in the column's filters; clicking a tile toggles that group and the rows follow, as before (`sidebar_252_status_tiles_count_and_toggle`).
2. Tile counts follow the CLI choice; the shown-session count and a 「クリア」 action appear only while a filter is on, and 「クリア」 clears both dimensions without changing the selected session or workspace tab (`sidebar_252_counts_follow_cli_and_clear_only_while_filtering`).
3. 「承認・回答待ち」 is no longer a separate choice: a session waiting on a permission or an answer is shown by 要対応, and 要対応's hover names how many of its sessions are waiting (`sidebar_252_needs_you_covers_waiting`).
4. At the default 268px column the tiles sit two per row; at the 180px minimum column (`SIDEBAR_MIN_W`) they stack one per row, and the CLI row wraps its count and 「クリア」 under the marks, so every filter control stays inside the column (`sidebar_252_tiles_fit_narrow_column`).
5. Every filter change, including 「クリア」, still cancels a pending row-removal confirmation, as change 251 required (`sidebar_252_counts_follow_cli_and_clear_only_while_filtering`).

## Behaviour

Status tiles, in the order 要対応, 実行中, 指示待ち, 終了: the group's existing mark in its status colour, its word, and the count right-aligned. At rest a tile is a hairline; under the pointer it fills; selected, it is washed in its status colour with a hairline and count in that colour. A zero count draws faint and stays clickable. Each count is the number of sessions the tile would show if chosen alone, so 終了 counts ended sessions that the unfiltered live-only list does not draw. Hover text is the group's existing description; 要対応 adds a second line 「承認・回答待ち {count} 件」.

CLI choices: Codex, Claude Code and Antigravity marks and a 「その他」 mark (the existing more glyph) in one hairline segment, with full names on hover and as accessible names. While any is selected, unselected marks fade.

While filtering, the CLI row ends with 「{count} 件」 (sessions that match) and a quiet 「クリア」 with the close mark. Otherwise neither is drawn. When the marks, the count and 「クリア」 do not fit on one row, the count and 「クリア」 wrap onto the next line. The live-only default, ended sessions when 終了 is chosen, 「一致するセッションはありません。」 for no match, 「プロジェクトがありません」 for an empty store, and the history list's own chips are unchanged. Home's group shortcuts still open the list narrowed to one group. No IO, process, background task or failure path is added; counts are one pass over the sessions per frame, as `status_group_counts` already is.

## Design

Placement: EXTEND `ui_sidebar_session_filters` in `src/app/screens.rs` and the filter state in `src/app.rs`; new drawing helpers in `src/ui/widgets.rs`. `waiting_filter` and `toggle_waiting_filter` are removed (memory-only, never persisted), and the history list's 「すべて」 chip and pinning rule stop reading it. `status_group_counts` gains a sibling that counts only sessions passing the CLI choice. New helpers: `status_filter_tile` (custom-painted, `CONTROL_HEIGHT`, palette roles `border_subtle`, `control_hovered`, `status(tone)`, `text`, `text_muted`, `text_faint`, focus via `paint_button_focus`, selected widget info) and `agent_filter_toggle` (mark or glyph, `CONTROL_HEIGHT_SMALL` inside a 28px hairline frame, faded via `multiply_opacity`). Two tile columns need `STATUS_TILE_MIN_W` (a new layout constant in `src/config.rs`, 104px: two fit at 268px, not at 180px) per tile; below that the block is one column. `session_filter_button` is removed with its last caller. Strings 「{count} 件」, 「クリア」 (if absent) and 「承認・回答待ち {count} 件」 are added to every table; rows that lose their last caller are removed. No persisted shape changes.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — DESIGN.md | yes | Only existing roles: border_subtle, control_hovered, row_selected, status tones, text inks; washes use gamma_multiply on a role as existing drop zones do. |
| Icons — src/glyphs.rs | yes | Existing group glyphs, ICON_MORE and ICON_CLOSE; bundled vendor marks. No new glyph. |
| Controls — DESIGN.md | yes | The Controls table row for sidebar filters names the two new helpers at 28px / 24px. |
| Identifier SSOT — src/config.rs | no | No identifier, path or command. |
| Durability | no | No store, cancellation, migration or byte limit. |
| Subprocess safety | no | No child process. |
| Local-first — CONTRIBUTING.md | yes | Local memory only. |
| Budgets | no | No scan or output path. |

## Flagged concerns

None. The person chose proposal A and the folded waiting choice after viewing the mockups.

## Acceptance

- `cargo test --locked sidebar_252_` and `cargo test --locked sidebar_251_` pass; `make q.fast` passes.
- The mutation sweep catches each requirement.
- In the running app at 268px and at the narrowest column: tiles, counts, faded marks, クリア, no-match state.

## Rejected alternatives

- Proposal B, a filter button with a popover: an extra click per switch.
- Proposal C, a mailbox-style list: as tall as today and multi-select hidden behind ⌘-click.
- Keeping 「承認・回答待ち」: it duplicates part of 要対応; the person chose to fold it.
