# Spec: The selected row takes the selection colour

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. **One selection colour.** In all three themes, the `row-selected` role
   carries the same value as the `selection` role the table already has, so a
   selected row and selected text are one language and the warm tint is gone.
   Test: `selected_row_takes_the_selection_colour` in `src/tests.rs`.
2. **A selected row stays readable.** The inks a selected row draws — its
   title, its second line, its time, its branch, its status word, its vendor
   word — are the palette's strong and ordinary text inks, chosen through
   `Palette::selected_ink` and `Palette::selected_secondary_ink`, and both
   measure at least 4.5:1 against `row-selected` in every theme. Status keeps
   its tone only in the dot, which is ringed in the selected ink so the mark
   is findable whatever the tone's luminance. Tests:
   `selected_row_inks_stay_readable` and `every_theme_stays_readable`.
3. **The toolbar's current page follows.** The chosen toolbar tab draws its
   words in `selected_ink`, not in the brand ink, which measured 3.36:1
   against the new surface. `selected_row_inks_stay_readable` measures the
   accessor, not the call site: that `nav_tab` takes it is covered only by
   the screen observation in Acceptance and by review, and the spec says so
   rather than letting the test's name imply more.
4. **Only a selection wears the selection colour.** The knowledge map's hover
   tint and the home page's attention banner, which borrow `row-selected` for
   things that are not selections, paint the neutral `card` surface instead;
   the banner keeps its warning ink and outline, which already read on
   `card`. Decided by the grep and the screen observation in Acceptance.

## Behaviour

No string a person reads changes, in any language. What changes is only what
things look like:

- Dark: the selected session row, pinned waiting row, and selected project
  card sit on the deep selection blue (`#005C80`); their words are the bright
  text inks; the status dot keeps its tone inside a bright ring.
- Light: the same rows sit on the pale selection blue (`#B4D6FA`); their words
  are the dark text inks.
- High contrast: the same rows sit on `#0A6E96` with the brightest inks.
- The toolbar's current page matches the rows; the orange button that starts
  a session is unchanged and becomes the only saturated warm fill on screen.
- The attention banner becomes a neutral card with its warning outline and
  words; the knowledge map's hovered row becomes a neutral card tint.
- Empty, loading, interrupted, and too-large states draw no selected row and
  are untouched. A session row that is being dragged or renamed keeps its
  existing behaviour; the rename field sits on `inset`, not on the row tint.

## Design

Placement verdict: **EXTEND** — the theme module grows two ink accessors; no
new module, no new role, no new colour value anywhere.

- `src/theme.rs` — `row_selected` takes `selection`'s value in all three
  palettes (the literal is repeated because a palette is a constant table;
  the test in requirement 1 is what keeps the two from drifting). Two new
  accessors on `Palette`: `selected_ink()` returning `text_strong` and
  `selected_secondary_ink()` returning `text`, named for where they land so a
  call site never picks a muted ink for a selected row again.
- `DESIGN.md` — the three `row-selected` values follow the binary (the drift
  test holds them together), and the prose that calls the tint warm, and the
  Controls paragraph that pairs the tint with the brand ink, are rewritten to
  say what is now true: one selection colour, bright inks on it, tone carried
  by the dot and the word together.
- `src/ui/widgets.rs` — `nav_tab` takes `selected_ink()` when chosen, and its
  comment stops claiming the selection blue belongs to text alone.
- `src/app/screens.rs` — the session column row, the pinned waiting row, and
  the selected project card switch their secondary, status, branch, vendor,
  and waiting inks to the two selected inks while selected, and ring the
  status dot in `selected_ink()`; the attention banner fills `card`.
- `src/app/knowledge.rs` — the map panel row's hover fills `card`.
- `src/tests.rs` — the two new tests; in `every_theme_stays_readable` the
  shared `surfaces` array loses `row-selected`, which narrows three loops at
  once and each narrowing is the honest one: the every-ink loop (muted and
  tone inks no longer land there as text), the diff-sign loop (a sign is
  never drawn on a selected row), and the outline loop (the hover outline
  `border` is not drawn on a selected row either — measured, it would fail
  at 1.63:1 / 2.63:1 / 1.47:1). What replaces them is stated where the claim
  now lives: a selected block checking the two selected inks at 4.5:1
  against `row-selected`, and `border_strong` — the focus outline a
  selected row actually gets — checked at 3:1 against it (7.39 / 11.34 /
  5.70). Every other surface keeps the full ink and outline lists.

No persisted shape changes. Nothing runs on a background thread. No external
command is added.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | Yes | No new role and no new value: an existing role's value moves, in three palette rows plus `DESIGN.md` together. AA is re-measured by `every_theme_stays_readable` as restructured and by `selected_row_inks_stay_readable`; the surfaces a muted ink no longer lands on stop being claimed for it, and the claim that replaces that is itself tested. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No new mark. The status dot gains an enclosure drawn with the painter, which is what the Iconography section already prescribes for status marks. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | No identifier is added or agreed in two places. The colour's one home stays `src/theme.rs`, with `DESIGN.md` held to it by the existing drift test. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | The store, its schema, cancellation, and every byte budget are untouched. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No child process, launch path, or command is added or changed. |
| Documentation — user-facing docs change in all three languages together | No | No user-facing document or string changes. `DESIGN.md` is the internal design source and is English by construction; it changes because the colour table it mirrors changes. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | None added; the change is constants, two accessors, and drawing calls. |
| Budgets — any new scan or output path states its byte and item ceiling | No | No scan or output path is added. |

## Flagged concerns

- **Status tones measure low against the dark selection blue as fills**
  (danger 2.37:1, sand 2.66:1). Resolved in the design rather than carried:
  on a selected row the tone survives only in the dot, ringed in
  `selected_ink()` (6.7:1 on the dark surface), and the word beside it is
  drawn in `selected_secondary_ink()`, so the state is carried by glyph,
  word, and hue together as `DESIGN.md` already requires.
- **`every_theme_stays_readable` changes its premise.** Resolved: the test
  keeps the full ink list for every surface muted inks still land on, and
  checks the two selected inks against `row-selected`. Pointing
  `selected_secondary_ink` at `text_muted` must turn it red; that mutation is
  run before the change is believed (see `plan.md`).
- **The prescribed 2px accent bar is still missing after this change.**
  Resolved as out of scope in `intent.md`: it is a separate drawing change,
  and this change's screen is judged without it.

## Acceptance

- `cargo test --locked selected_row` passes, including
  `selected_row_takes_the_selection_colour` and
  `selected_row_inks_stay_readable`, and `cargo test --locked
  every_theme_stays_readable` passes.
- `grep -rn row_selected src --include='*.rs'` outside `src/theme.rs` and
  `src/tests.rs` lists only selections: the toolbar tab, the two session
  rows, and the project card.
- In the running app, in each of the three themes: the selected session row
  is blue and its title, second line, and time are readable at a glance; the
  current toolbar page matches it; the attention banner reads as a warning
  on a neutral card; hovering a knowledge-map row no longer paints it the
  selection colour.

## Rejected alternatives

- **Keep the tint, add only the prescribed accent bar** — separates "colour
  or presentation" cleanly, and stays the follow-up; the person chose the
  colour change after seeing both, and the bar without it leaves the warm
  family collision in place.
- **Abolish the tint; show selection by surface step and bar alone** — the
  quietest reading, but it rewrites the design document's selection language
  for a weaker at-a-glance signal, and its surface step measured 1.03:1.
- **A new blue mixed for this purpose** — a colour nobody's role names; the
  table already carries the selection colour, and inventing a neighbour is
  how the warm tint happened in the first place.
- **Import another editor's selection hex** — another product's token in
  this table, with no role here that means it.
