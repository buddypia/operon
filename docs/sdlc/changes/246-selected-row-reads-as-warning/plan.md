# Plan: The selected row takes the selection colour

- **Spec**: `./spec.md`
- **Approved**: recorded in `approvals.log` when the evaluator's verdict lands
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/tests.rs` | `selected_row_takes_the_selection_colour` and `selected_row_inks_stay_readable` added; `every_theme_stays_readable` checks the two selected inks against `row-selected` instead of every ink |
| `src/theme.rs` | `row_selected` takes `selection`'s value in the three palettes; `selected_ink()` and `selected_secondary_ink()` accessors on `Palette` |
| `DESIGN.md` | The three `row-selected` values; the warm-tint sentence in Contrast; the tinted-selection paragraph in Controls |
| `src/ui/widgets.rs` | `nav_tab` draws the chosen tab in `selected_ink()`; comment updated |
| `src/app/screens.rs` | Session column row, pinned waiting row, and selected project card switch to the selected inks while selected and ring the status dot; the attention banner fills `card` |
| `src/app/knowledge.rs` | The map panel row's hover fills `card` instead of `row_selected` |

## Order of work

1. Add `selected_row_takes_the_selection_colour` alone — it names only
   fields that already exist, so the tree compiles — and watch its assertion
   fail: `cargo test --locked selected_row_takes_the_selection_colour`.
2. Add the two accessors and `selected_row_inks_stay_readable` against them.
   It passes on the old surface too (the bright inks read on the tint as
   well); its job is the guard in step 7, not this step's red.
3. Move the three values in `theme.rs` and mirror them in `DESIGN.md`. The
   first test passes; `every_theme_stays_readable` now fails against the
   new surface on the muted and tone inks and on the `border` outline.
4. Restructure that test exactly as `spec.md` Design describes: the shared
   `surfaces` array loses `row-selected` (ink, diff-sign, and outline loops
   narrow together), a selected block checks the two selected inks at
   4.5:1 against it, and `border_strong` is checked against it at 3:1.
5. Switch the drawing call sites: `nav_tab`, the two session rows, the
   project card, the banner, the map hover.
6. Rewrite the two `DESIGN.md` prose passages and the `nav_tab` comment.
7. Watch the guard fail: point `selected_secondary_ink` at `text_muted`,
   run `cargo test --locked selected_row_inks_stay_readable`, see it red,
   restore.
8. `make q.fast`, then the named tests. Package with
   `bash scripts/package-macos.sh` and swap per the ship skill, because the
   change touches `src/` beyond `src/tests.rs`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A muted or tone ink still lands on the new surface somewhere the plan did not read | A dim second line or status word on the blue row, visible only in the running app | The Acceptance grep plus the running-app observation in all three themes |
| The restructured readability test quietly claims less than the old one | A future call site draws a muted ink on a selected row and nothing goes red | The mutation in step 7, watched failing; the two selected inks are named in `DESIGN.md` prose so review can hold a call site to them |
| The attention banner loses its warning read on the neutral card | The banner looks like an ordinary card | It keeps its warning outline and warning words, which measure 9.59:1 (dark) on `card`; the running-app observation names the banner explicitly |
| The pinned waiting row's waiting-time ink was a status tone, not a text ink | The wait reads dim on the blue row | It switches to `selected_ink()` in step 5; the row is in the Acceptance observation |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked selected_row` — the two new tests pass.
- `cargo test --locked every_theme_stays_readable` — passes.
- `make q.fast` — passes.
- The mutation from step 7 watched failing, then restored.
- The running packaged app, in all three themes, shows what Acceptance
  describes.

## Departures from the plan

- A third test, `only_selections_wear_the_selection_colour`, was added
  during implementation so requirement 4 is a guard the mutation sweep can
  break, not only a grep a person runs once. It counts `palette.row_selected`
  at the call sites that may wear it (three selections in
  `src/app/screens.rs`, the toolbar page in `src/ui/widgets.rs`, none in
  `src/app/knowledge.rs`). The plan had named only the two palette tests.
- Review and the empirical verifier each found inks still landing on the
  selection colour that the first pass missed, and all were fixed rather
  than carried: the vendor chip's fallback word (`agent_chip_with_ink` in
  `src/agents.rs`), the session rows nested inside a selected project card
  and that card's empty line, and the removal confirmation inside the
  selected row (`src/app.rs`). The titles that already held the selected
  values now take them through the accessors where they are selected.
- Change 245 landed while this change was in review and turned the project
  card into a heading with no fill, so the selected-project-card site this
  plan and the screen describe no longer exists. Merging it in removed
  that site rather than adapting to it: the project name, its status
  counts, and the rows under it keep 245's inks on the panel, the switches
  added for the blue card came back out, and the borrow guard's count for
  `src/app/screens.rs` went from three selections to two.
