# Spec: icon-only status on session tabs

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. R1: generate native vector status badges with a common circular enclosure, 18px footprint and distinct central marks for running, idle, attention, starting, ready, queued, done, failed, stopped, lost and unknown. Status meaning must survive monochrome and all themes. `session_tab_status_badges_are_distinct_vectors` checks painted geometry for all fourteen kinds.
2. R2: session tabs show the badge immediately before the title and no permanent state word, including live transitions, inactive tabs and long titles. `session_tabs_show_live_state_before_title` checks the actual workspace output.
3. R3: hovering the badge displays the current localized state name and precise existing explanation; leaving dismisses it. Existing selection, drag, close and document tab behavior remain intact. `workspace_session_tab_shows_its_status_glyph` exercises hover and live changes; existing `workspace_pointer_` tests cover interactions.

## Behaviour

Before: a small glyph, 実行中 / 要対応 / 指示待ち / 終了, then the title.
After: one circular status badge, then the title. The tooltip retains the existing Japanese state label and hint, e.g. 実行中 and ターミナル稼働中, or 要対応 and the explanation of the pending response. No new user-facing strings.

All states use the same outer circle. Distinct inner marks identify the state: circular progress strokes, pause bars, exclamation, power, play, hourglass, check, cross, stop square, minus and question. Cancellation in progress and stopped share the stop mark but keep their existing different tones and hover explanation. Working and running share the progress mark; waiting and blocked share attention. No animation is needed to understand a state.

Empty workspaces and missing sessions show no fabricated badge. Unknown state has a question mark and its existing explanation. Very long titles truncate as before, and many tabs scroll. Each frame reads the latest in-memory status; hover reads the same current session. No new I/O, commands, persisted state, dependencies, background work or failure paths.

## Design

Placement: EXTEND. `ui_tab_workspace` in `src/app/screens.rs` currently draws a state glyph and permanent `kind.label()` before each session title (change 248). Replace those with `session_status_icon` from `src/ui/widgets.rs`. Define the generated vector vocabulary and painter in `src/glyphs.rs`, which already owns the icon vocabulary; do not rasterize an 18px icon or depend on a font fallback. Reuse `SessionStatusKind` and `Palette::status` rather than classify twice. Other status chips retain their text and existing glyphs.

The badge occupies 18 logical pixels in a `CONTROL_HEIGHT_SMALL` hover target. Its common enclosure and interior are painted with the current palette's status tone. The helper adds accessible localized state text to widget metadata without putting it in the visible tab. No new module or asset decoder.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — DESIGN.md | Yes | Existing palette.status(kind.tone()); no new colour literals. |
| Icons — src/glyphs.rs | Yes | Dedicated named vector vocabulary lives beside ICON_VOCABULARY; document the generated badge family in DESIGN.md. Existing Phosphor vocabulary remains intact. |
| Identifier SSOT — src/config.rs | No | No external identifier introduced. |
| Durability | No | No persistence, cancellation behavior, migration or byte limit changes. |
| Subprocess safety | No | No process lifecycle or external commands in product. |
| Documentation — CONTRIBUTING.md | Yes | Internal DESIGN.md icon guidance updated; no public command or capability changes. |
| Local-first — CONTRIBUTING.md | Yes | Pure local drawing, no dependencies, network or unsafe. |
| Budgets | No | Fixed-size vector geometry from current in-memory kind. |

## Flagged concerns

None.

## Acceptance

R1–R3 tests and existing workspace pointer tests pass. Mutation sweep catches missing enclosure, collapsed status marks, permanent state text, stale hover label and removed explanation. make q.fast and band checks pass, then full CI before merging. Package from main and install when Operon is not running; a running user's app is not terminated without authorization.

## Rejected alternatives

Permanent state words contradict the request. Plain coloured dots cannot convey state without colour. Raster image generation reduces legibility at this size and adds theme/scale handling; native vector generation produces exact, crisp badges at every display scale. Reusing bare action icons would not establish a common status enclosure.
