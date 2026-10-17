# Spec: each project is one framed block, sessions are ruled apart, and a running session stops from its row

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

All in the session column, `ui_terminal_session_tabs`.

1. **A project is a block.** Each project shown — its header line and every
   session row under it — is drawn inside one `card_frame` (the home screen's
   card: fill `palette.raised`, 1px `palette.border_subtle`, corner
   `RADIUS_CARD`) with inner margin `SPACE_SM`, and `SPACE_SM` between two
   blocks. Two projects never share a block. The project's name is drawn at
   the `section` level (15px, strong, `palette.text_strong`), larger than any
   session title in it (13.5px). The test asserts: the header and every title
   of a project lie in one `raised` rect; two projects' rects do not
   intersect; the name is 15px and the titles are smaller.
   Test: `each_project_and_its_sessions_share_one_frame`.
1a. **The 要対応 queue is a block too.** When sessions are pinned above the
   projects, the 「要対応」 label and every pinned row lie in one `card_frame`
   that contains no project's header.
   Test: `the_needs_you_queue_is_its_own_block`.
2. **Sessions are ruled apart.** Inside a block, consecutive session rows are
   separated by one 1px `palette.border_subtle` horizontal rule spanning the
   row; none is drawn above the first row or below the last.
   Test: `sessions_in_a_project_are_ruled_apart`.
3. **The row's corner fits.** Nothing above the projects widens the column:
   the heading is laid out from the right — 「履歴」, then the grip — and the
   grip truncates only when 「履歴」 leaves it no room; the heading's 「n 件」
   goes, since 「すべて n」 directly below says the same, and its room is what
   keeps 「プロジェクト別セッション」 whole at the default width. The filter
   chips wrap onto a second line. In each row the corner is laid out first,
   right to left — `···` at the far right, then the time or, on the active row,
   ✕ and ■ — then `SPACE_XS`, and the title takes the width left, truncated.
   In a column dragged to `SIDEBAR_MIN_W`, no row's time runs past its clip,
   and it ends inside its block; at the default width the heading is whole and
   carries no count, and a too-long title ends at least `SPACE_XS` before the
   time. `···` does not move between a resting and an active row.
   Test: `a_session_rows_corner_stays_inside_its_block` (and the `···`
   position in `a_session_rows_menus_offer_stop_and_remove`).
3a. **The project header fits.** The header lays out ＋ first, right to left,
   then the count, and the name takes what is left, truncated with its full
   name and path on hover. With a name far wider than the column, the ＋ and
   「n 件」 still end inside the block.
   Test: `a_long_project_name_keeps_its_count_and_add_inside_its_block`.
3b. **Rename still works from the title.** Double-clicking a row's title in the
   column starts renaming that session, as today.
   Test: `a_column_title_double_click_renames_its_session`.
4. **Stop from the row.** On the active row (selected, or under the pointer) of
   a session whose state verb is 停止 (`session_verb` → `SessionVerbAction::Stop`:
   running, or a stop pending), a `small_icon_button` with `ICON_STOP` stands
   left of `✕`, its hover text the verb's hint
   (「実行中の CLI を停止します（記録は残ります）」, or the pending one).
   `···` stays rightmost.
   Clicking it calls `stop_session` — the same as the terminal header's 停止.
   A session that is not running shows no such button. The test observes
   `BackgroundKey::SessionStop(id)` in `background_tasks`; the suite's
   `tmux_command` speaks to its own `-L operon_suite` server, so no real
   session is touched.
   Test: `a_running_session_stops_from_its_row`.
5. **Stop and remove in the menus.** The row's `···` menu and its right-click
   menu draw one list, in one order: 名前を変更, worktree, 未読にする, then —
   only when the verb is 停止 — the verb's label (「停止」 or 「停止中…」),
   then 「セッションを削除」, then the restore targets as today. Choosing 停止
   calls `stop_session`; 「セッションを削除」 calls `request_session_removal`
   (a running session then shows the existing inline confirmation). The `···`
   hover text becomes 「その他の操作」, as on the home screen. A right click on
   the title itself — which takes clicks for its double-click rename — opens
   the same menu as a right click elsewhere on the row. The row's labels are
   not selectable, so a click or right click on its second line reaches the
   row (today it selects text and opens nothing). The test also chooses
   未読にする and 名前を変更 from a right click on the second line.
   Test: `a_session_rows_menus_offer_stop_and_remove`.

## Behaviour

Japanese strings a person reads — all existing message ids with EN/KO rows:

- 「停止」, 「停止中…」 and their hints — from `session_verb`, unchanged.
- 「セッションを削除」 — the ✕'s hover text, now also a menu item.
- 「その他の操作」 — the `···` hover, replacing
  「worktree · 名前を変更 · 復元 · 未読にする」 (left in the tables; no code path
  shows it afterwards).

States:

- **Project with no sessions**: its block is the header line alone (change 166).
- **Status chips filtering a project empty**: no block, as today.
- **Selected row**: `row_selected` fill inside the block; the rules beside it
  stay.
- **Stop pending**: the corner button and the menu say 停止中…; pressing again
  retries, as the header verb does.
- **Starting / queued / finished session**: no stop button, no 停止 item;
  「セッションを削除」 stays (queued cancels, finished closes, starting is
  refused with the existing notice).
- **Stop fails**: the existing notice from `stop_session`'s background result.
- **Narrow column**: the title truncates first; the corner never leaves the
  block.

## Design

Placement verdict: **EXTEND** — `ui_terminal_session_tabs` and
`ui_pinned_sessions` in `src/app/screens.rs` draw the column today
(`git grep -n "terminal-session-card" src/app/screens.rs`). Route `modify`.

- `src/app/screens.rs`:
  - each project's header and rows go inside `card_frame(palette)` with
    `inner_margin(SPACE_SM)`; the pinned queue likewise;
  - the header: `right_to_left` (＋, then 「n 件」), then a nested
    `left_to_right` with the icon and the name
    `RichText::size(15.0).strong().color(text_strong)` as a truncating label;
  - the title label's response gets the same `context_menu` as the card;
  - before every row but the first, `ui.painter().hline` in `border_subtle`
    across the row's width, with `SPACE_XS` above and below;
  - the row's first line: `right_to_left` first (time, or ✕ / ■ / ···), then a
    nested `left_to_right` for the dot and the truncated title in what is left
    — the order `home_session_row` already uses;
  - `session_verb(&candidate, self.cancellation_pending(candidate.id))` read
    once per row, only on the active row and when a menu is open;
  - a private `session_row_menu(ui, …)` draws the shared list for both menus
    and returns which item was chosen.
- No new type, palette role, glyph, message id, persisted shape, external
  command, or background task. `stop_session` already runs `tmux kill-session`
  on a background thread through `run_command_with_timeout`.
- Per frame: one more frame and N−1 line segments per project; `session_verb`
  allocates one hint `String` for the active row only.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | Yes | Existing roles only, through `card_frame`: `raised` ("cards and grouped sections that sit above the panel"), `border_subtle` ("the hairline around a group"), `text_strong`. In the dark theme `raised` (25) is a step below `panel` (27), so the block reads by its hairline — as the home screen's cards already do in the person's theme; DESIGN.md *Controls*: "A control at rest is a hairline, not a slab". `every_theme_stays_readable` covers ink × surface; the hairline is judged on the dark and light screenshots. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | Yes | `ICON_STOP`, already the 停止 verb's mark. |
| Controls / type scale — `DESIGN.md` *Controls*, `typography` | Yes | `small_icon_button` (list row, 24px) for ■; 15px is `section`. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs` | No | No identifier. |
| Durability — schema change means a `STORE_SCHEMA_VERSION` bump | No | Nothing persisted changes; stop goes through the existing `persist_cancellation_intent`. |
| Subprocess safety — new children go through `src/exec.rs` | No | No new child; `stop_session` is reused. |
| Documentation — user-facing docs change in all three languages together | No | `grep -n "停止\|区切り" README*.md` names no sidebar layout. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | No | None. |
| Budgets — any new scan or output path states its byte and item ceiling | No | No scan or output path. |

## Flagged concerns

- **Dark theme.** The block's fill is barely distinct from the panel there; the
  hairline carries it. Answered: judged on the dark-theme screenshot (the
  person's theme) and on the light one.
- **Stop beside delete.** ■ and ✕ sit side by side on the active row. Answered:
  removing a running session already asks first (inline confirmation), and
  stopping keeps the record and can be resumed — the same verb the header
  shows without confirmation.

## Acceptance

- The eight tests named above pass; `cargo test --locked an_empty_project_in_the` and
  `every_` still pass.
- `make q.fast` passes.
- A screenshot of the dev build against a copied store shows three framed
  blocks, ruled rows, whole times, and ■ on a hovered running row, in the dark
  and the light theme.

## Rejected alternatives

- Only a larger project name and more space — space alone was the problem: the
  gaps were already there and did not read as grouping.
- A full-width rule between projects instead of a frame — the rule separates
  but does not contain; with two-line rows the eye still has to count.
- Collapsible projects — a new interaction the person did not ask for.
- The stop as a worded `verb_button` on every running row — a column of
  「停止」 words loud on every row; the word is in the menu and the hover.
