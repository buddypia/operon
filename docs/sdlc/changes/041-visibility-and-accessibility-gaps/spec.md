# Spec: dimmed text to 4.5:1, outlines to 3:1 everywhere, focus ring on custom rows

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `text-faint` clears 4.5:1 on all six text surfaces (`panel`, `raised`,
   `card`, `inset`, `window`, `row-selected`) in all three themes. Dark
   `#767C85` becomes `#9399A2`; Light `#7A8087` becomes `#5F656C`;
   HighContrast is untouched (already 5.2:1 or better). The contrast test's
   `text_faint` floor rises from 3.0 to 4.5.
2. `border` (the hover outline) clears 3:1 on all six surfaces in all three
   themes, not just `panel`. Dark `#707070` becomes `#767676`; Light
   `#8F887C` becomes `#877F72`. The outline check in the contrast test runs
   over every surface.
3. `border-strong` (the pressed outline, and the focus outline in R5) is
   checked against all six surfaces at 3:1. It already passes everywhere
   (11.6:1 or better); the check pins it so the focus mark cannot fade.
4. Every custom element that can hold keyboard focus paints it with a
   `border-strong` outline while focused: the session rows
   (`clickable_card`), the metric tiles, the project rows, and the diff file
   rows plus diff line note affordances. Standard buttons need no change:
   egui 0.31 paints a focused widget with its `active` style, which already
   carries the `border-strong` outline.
5. The focus decision is a pure helper (`Some(border-strong stroke)` when
   focused, `None` otherwise) shared by all five sites, with a unit test
   pinning both arms.

## Behaviour

What changes on screen, all of it a recolor plus one new outline:

- Dimmed secondary lines (session metadata, file-tree marks, code language
  tags, plumbing labels, zero-value counts, separators) get slightly
  brighter on Dark and slightly darker on Light. Hierarchy is preserved:
  faint stays dimmer than muted, which stays dimmer than body text.
- Hovering a control on a selected row or on a Light card now shows a
  visible outline where the old one measured under 3:1.
- Tabbing onto a session row, metric tile, project row, or diff row draws
  the same strong outline a pressed button shows. Nothing else about the
  row changes: no fill change, no layout change, no new strings.
- HighContrast theme: no colour changes at all; only the new focus outlines
  appear there.
- No user-facing string is added or changed, so there is nothing new to
  translate and no empty/loading/error state to draw.

## Design

- `src/theme.rs`: four constant changes in two tables (`DARK_PALETTE` and
  `LIGHT_PALETTE`: `text_faint`, `border`). No new roles, no type changes,
  no process boundary crossed.
- `src/ui/widgets.rs`: one pure helper for the focus stroke, used by
  `clickable_card` and `metric_tile`.
- `src/app/screens.rs`: project rows paint the focus stroke on their
  full-rect click response when it holds focus.
- `src/ui/diff.rs`: file rows and line note affordances paint the focus
  stroke when their response holds focus.
- `src/tests.rs`: `every_theme_stays_readable` gains the raised faint floor
  and the all-surface outline checks; one new test pins the focus helper.
- `DESIGN.md`: the four changed values in the `colors` block (both
  directions, as the drift test demands), plus the prose line naming the
  old hover-outline value.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | Yes — values, not roles | No new role; the four changed values are updated in the `colors` block in both directions and clear the raised floors in the contrast test |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No new mark; the focus indicator is an outline, not a glyph |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | No new shared string |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | The store is not touched |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No new child process or launch path |
| Documentation — user-facing docs change in all three languages together | No | No user-facing string changes; `DESIGN.md` (English-only policy doc) is updated for the changed values |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | None added |
| Budgets — any new scan or output path states its byte and item ceiling | No | No new scan or output path |

## Flagged concerns

None. The audit behind this spec measured every pair with the same
luminance math the test uses: the four replacement values clear their
floors on all six surfaces (dark faint minimum 5.0:1, light faint minimum
4.9:1, dark border minimum 3.2:1, light border minimum 3.3:1), and
`border-strong` needs no change. The egui focus path (focused widget takes
the `active` style) was read out of the vendored egui 0.31.1 source rather
than assumed.

## Acceptance

- `cargo test --locked` passes, including `every_theme_stays_readable`
  with the raised floors and the new focus-helper test.
- Each new or raised guard is watched failing first: darken/lighten one
  value below its floor and confirm the suite goes red, then restore it.
- In the running app, Tab through the session list, the overview metrics,
  the project picker, and a diff: every stop shows the strong outline, and
  dimmed lines read without effort on Dark and Light.

## Rejected alternatives

- Swapping small dimmed text to `text-muted` instead of brightening
  `text-faint`: touches dozens of call sites and erases the faint/muted
  distinction the scale documents; four constants preserve it.
- A dedicated focus colour role: a fifth accent would compete with the
  status hues; `border-strong` already means "this is the one" and already
  clears 3:1 everywhere.
- Extending the text checks to `code-bg`: half the dark inks fail there and
  only body text lands on it, so the check would pin pairs nobody reads.
- Animated or filled focus marks: motion that delays an answer contradicts
  the design overview; an outline is enough.
