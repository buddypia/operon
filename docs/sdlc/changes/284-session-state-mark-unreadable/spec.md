# Spec: a session's state reads as a shape on its mark

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. The state badge on a session's CLI mark is one size for every state —
   `STATUS_DOT_RATIO` of the mark, about 9px on a 14px tab mark — filled in
   `Palette::status` of the state's tone, ringed in the surface, and holds the
   state's `StatusMark` glyph (the generated vector vocabulary the circled
   state mark already uses, without its enclosure) drawn in the surface
   colour. — `session_284_badge_holds_the_state_mark`
2. Two states whose marks differ draw different strokes inside the badge:
   `Idle` (pause bars), `Done` (check) and `Waiting` (exclamation) are told
   apart with the colour removed. — `session_284_states_differ_in_shape`
3. A running plain terminal still draws no badge (change 283's
   `session_mark_speaks` is unchanged). — `session_283_running_terminal_says_nothing`
4. A project row in the left column counts its sessions per state group with
   the group's generated `StatusMark` in the group's tone, followed by the
   count as text — no font glyph — and a click on it still opens the first
   session of the group. — `workspace_project_header_counts_sessions_by_status_group`,
   `workspace_project_count_click_opens_first_session_of_its_group`
5. A session row in the left column with no CLI mark (a custom command) draws
   the generated circled state mark, not a font glyph. —
   `session_284_markless_row_draws_the_generated_mark`
6. Hovering still names the CLI and the state (unchanged). —
   `session_283_mark_names_cli_and_state_on_hover`

## Behaviour

- Tab: `[mark with badge] タイトル ×`; the badge is about 9px (14px mark) or
  10px (16px column mark), over the mark's lower right corner, with a 1.5px
  ring in the tab's or row's fill.
- Badge glyphs, by state: 実行中/作業中 ⟳ arrows, 起動中 power, アイドル ‖,
  要対応 !, 前提完了 ▶, 順番待ち hourglass, 完了 ✓, 失敗 ×, 停止 ■,
  ロスト −, 確認中 ?. The mapping is `SessionStatusKind::status_mark`, which
  the circled mark already uses.
- Project row counts: `(!)1 (⟳)2` — a 12px circled mark in the group's tone,
  then the count in the same tone; hover says the group and 「N 件」 as today.
- Custom-command row: the 14px circled state mark, then the command label.
- Empty, error, too-large, interrupted: none — painting only.

## Design

Placement: **EXTEND** — `paint_status_dot` (`src/ui/widgets.rs`) paints the
badge; `StatusMark::paint` (`src/glyphs.rs`) the vocabulary; the project
counts and the markless row are drawn in `src/app/screens.rs`.

- `src/glyphs.rs`: `StatusMark::paint` splits into the enclosure and
  `paint_glyph(painter, center, scale, ink)`; `StatusMark::paint_badge(painter,
  center, radius, ink)` scales the glyph so its 8-unit body spans about 55% of
  the badge. `STATUS_DOT_RATIO` becomes 0.64; `STATUS_DOT_ATTENTION_RATIO`
  is removed. New `STATUS_COUNT_MARK_SIZE = 12.0`.
- `src/ui/widgets.rs`: `paint_status_dot` paints ring, disc and
  `kind.status_mark().paint_badge` in the surface colour, replacing the
  hand-drawn exclamation. `SessionStatusGroup::status_mark()` beside `icon()`.
  `SessionStatusGroup::status_mark()` maps NeedsYou → Attention, Free →
  Pause, Busy → Progress, Finished → Check, the same states its `icon()`
  names. New `status_count_chip(ui, palette, group, count) -> Response`:
  allocates mark plus count text, click-sensing, paints both.
- `src/app/screens.rs`: the project counts call `status_count_chip`; the
  markless row calls the existing `session_status_icon`, which already paints
  the circled `StatusMark` at 14px with the state as its accessible label.
- `DESIGN.md`: the session-tab paragraph describes the badge and its glyph.
- The count text is formatted once per chip per frame as today (`format!` was
  already there); no persisted shape, no child process, no background work.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | Yes | The badge reads `Palette::status(tone)` and the surface roles already used; no new role. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | Yes | No new glyph: the marks are the existing `StatusMark` geometry; font glyphs in two places are replaced by it. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs` | No | No new identifier. |
| Durability — schema change means a `STORE_SCHEMA_VERSION` bump | No | Nothing is stored. |
| Subprocess safety — new children through `src/exec.rs` | No | No child, no `unsafe`. |
| Documentation — user-facing docs change in all three languages together | Yes | README and manual sentences describe the mark and its state; reworded in all three if they name the dot. |
| Local-first — no telemetry, accounts, cloud calls | No | Painting only. |
| Budgets — any new scan or output path states its ceiling | No | None. |

## Flagged concerns

- **Glyphs at about 5px inside the badge may blur** — resolved by checking a
  screenshot of the built app; if a glyph is unreadable, its stroke is
  thickened, not removed.

## Acceptance

- `cargo test --locked session_284_ session_283_ workspace_project_` passes.
- `make q.fast` passes.
- `python3 docs/sdlc/changes/284-session-state-mark-unreadable/mutations.py`
  catches every mutation.
- A screenshot of the built app shows the tab strip and the left column as
  `screen.md` draws.

## Rejected alternatives

- Scale the font glyph into the 5px dot — unreadable below about 8px.
- Write the state word after the title — readable but widens every tab.
