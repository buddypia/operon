# Plan: dimmed text to 4.5:1, outlines to 3:1 everywhere, focus ring on custom rows

- **Spec**: `./spec.md`
- **Approved**: 2026-09-12
- **Status**: approved

This is the plan produced in plan mode and accepted before any file was edited.
If the implementation departs from it, update this file — an abandoned plan is
worse than no plan, because the next reader trusts it.

## Files that change

| File | Change |
|---|---|
| `src/theme.rs` | Four constants: Dark `text_faint` (118,124,133)→(147,153,162), Dark `border` (112,112,112)→(118,118,118), Light `text_faint` (122,128,135)→(95,101,108), Light `border` (143,136,124)→(135,127,114) |
| `DESIGN.md` | Same four values in the `colors` block, plus the prose line naming the old hover-outline value |
| `src/ui/widgets.rs` | Pure `focus_stroke` helper; used by `clickable_card` and `metric_tile` |
| `src/app/screens.rs` | Project rows paint the focus stroke on their full-rect response |
| `src/ui/diff.rs` | File rows and line note affordances paint the focus stroke |
| `src/tests.rs` | `text_faint` floor 3.0→4.5, outline checks over all six surfaces, new focus-helper test |

## Order of work

1. `src/theme.rs` + `DESIGN.md` together (the drift test compares them; neither
   lands without the other). No behaviour change beyond the four inks.
2. `src/ui/widgets.rs`: add `focus_stroke(has_focus, palette)` returning
   `Some(1px border-strong stroke)` when focused, `None` otherwise; call it in
   `clickable_card` (rect stroke, card radius) and `metric_tile`.
3. `src/app/screens.rs`: bind the project-row interact response and paint the
   focus stroke on its rect when focused.
4. `src/ui/diff.rs`: paint the focus stroke (inside, square) on the file row
   and the line note rect when focused.
5. `src/tests.rs`: raise the faint floor, extend `border`/`border-strong`
   checks to all six surfaces, add the `focus_stroke` both-arms test.
6. Mutation watch: sink each changed value below its floor and each helper arm
   the wrong way, confirm red, restore.
7. `cargo fmt --check`, `cargo test --locked`, `cargo clippy --locked -- -D warnings`.

The tree compiles between every step; steps 1–4 are paint-only, step 5 is
test-only.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A replacement value mis-typed (digits transposed) | wrong hue on screen, or a red suite | the contrast test measures the constants, not the intent; step 6 |
| `DESIGN.md` and constants drift apart | drift test goes red | step 1 lands them together; the suite |
| Focus stroke rect misaligned with the card frame | outline floats off the row edge | Tab walk in the running app (Acceptance) |
| Faint/muted hierarchy too compressed on Light | secondary lines read flat | Tab walk; faint `#5F656C` vs muted `#565C63` stays ordered (5.0:1 vs 5.7:1 on card) |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `every_theme_stays_readable` fails if any faint ink is sunk to its old value
  (watched in step 6), passes after.
- New `focus_stroke` test fails if either arm is inverted, passes after.
- In the running app: Tab through session list, overview metrics, project
  picker, and a diff — every stop shows the strong outline; dimmed lines on
  Dark and Light read without effort.

## Departures from the plan

Filled in during implementation. What changed, and why the plan was wrong.
