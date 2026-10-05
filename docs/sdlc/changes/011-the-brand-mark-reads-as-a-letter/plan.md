# Plan: draw Operon's own icon where the brand glyph is drawn now

- **Spec**: `./spec.md`
- **Approved**: 2026-09-03
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `assets/operon-mark-128.png` | new. 128×128 RGBA, derived from `assets/operon-icon-1024.png` by keying the dark plate out of the alpha channel |
| `assets/agent-icons/SOURCES.md` | records where each bundled mark came from; gains the row for the brand mark, since that is where a reader already looks |
| `src/glyphs.rs` | `BRAND_MARK` beside the three `AGENT_ICON_*` constants; `ICON_BRAND` and its `ICON_VOCABULARY` row removed |
| `src/ui/widgets.rs` | `decode_png_mark`, `png_texture`, `brand_mark_image`, `brand_mark` |
| `src/agents.rs` | `decode_agent_icon` and the cache body leave; `agent_icon_texture` becomes the agent lookup plus `png_texture` |
| `src/app.rs` | top bar uses `Button::image_and_text`; first-run page uses `egui::Image` |
| `src/tests.rs` | three guards; `decode_agent_icon` call site follows the rename |

## Order of work

The tree compiles between every step except 3, where the two call sites still
name a constant that step 4 removes.

1. Generate `assets/operon-mark-128.png` and check it by eye at 18px and 48px on
   all three surface colours before any Rust changes. A wrong asset caught here
   costs one command; caught after the call sites it costs the whole loop.
2. `src/ui/widgets.rs`: add `decode_png_mark` and `png_texture` — `agents.rs`'s
   existing body with the agent lookup lifted out — then `brand_mark_image` and
   `brand_mark` on top of them.
3. `src/agents.rs`: delete `decode_agent_icon`, make `agent_icon_texture`
   delegate. `src/tests.rs`'s one call site follows the rename. Tree compiles.
4. `src/glyphs.rs`: add `BRAND_MARK`. Tree compiles.
5. `src/app.rs`: both call sites. Tree compiles.
6. `src/glyphs.rs`: remove `ICON_BRAND` and its vocabulary row, which is now
   dead. Doing this last means the compiler, not a grep, says whether it was.
7. `src/tests.rs`: the three guards. Watch each one fail by mutation before
   trusting it.
8. `cargo fmt`, the three gates, `scripts/check-bands.sh`, then the running app.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The keying leaves a faint dark halo where the plate met the glow, so the mark shows a grey box on the light theme | Only at small sizes, only on light — the size and theme least likely to be looked at | `the_brand_mark_is_a_transparent_ring_not_a_plate` asserts the corner pixels are fully transparent; step 1 looks at 18px on white before any code is written |
| `Button::image_and_text` sizes its content differently from `Button::new`, so the top bar's first control changes width and shifts the nav tabs | A row that no longer lines up, at a width nobody tested | The `min_size` on the button is kept, and the app is looked at — this is the class of bug lesson 005 says only a person catches |
| `png_texture`'s cache key collides between the brand mark and an agent mark | The wrong picture in one of the two places, deterministically | The key is `("png-mark", name)` with `name` unique per asset; `every_launchable_agent_ships_a_decodable_mark` still decodes all three, and the brand guard decodes the fourth |
| Removing `ICON_BRAND` leaves a caller behind | Compile error | The compiler, because step 6 is last |
| The 1.1 MB master gets bundled by accident instead of the 128px mark | Binary grows, first frame hitches | `every_bundled_mark_is_small_enough_to_ship` fails over 40 KB |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`, with
  N three higher than before this change.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-bands.sh` — `bands: N metrics within their bands`, or the
  same breaches this tree already had.
- The three new guards, each watched failing first:
  `the_brand_mark_is_a_transparent_ring_not_a_plate` (invert the alpha floor),
  `every_bundled_mark_is_small_enough_to_ship` (drop the ceiling under the real
  size), `the_brand_is_drawn_from_artwork_and_not_from_the_glyph_vocabulary`
  (put `ICON_BRAND` back).
- In the running app: the top-left corner is the orange segmented ring, not a
  character; it still clicks to ホーム and still says ホーム on hover; the same
  ring on ダーク, ライト, and ミッドナイト; the first-run Projects page shows it
  at 48px above まずはプロジェクトを開いてください.

## Departures from the plan

- **The texture cache key is `"brand-mark"`, not `"operon-mark"`.** The obvious
  key opens with `MANAGED_TMUX_PREFIX`, and
  `the_session_prefix_is_written_in_exactly_one_place` refuses any literal
  starting with it outside `src/config.rs` — including, on the second attempt,
  one written inside the comment explaining the first. The guard is right to be
  blunt: it cannot tell a cache key from a session name, and lesson 004 is
  exactly what happens when it tries. The key names the asset instead.
- **`brand_mark(ui, size)` was not added.** `brand_mark_image` alone serves both
  sites — the top bar needs the `Image` to build a button from, and the
  first-run page is one `ui.add`. A wrapper with one caller is a name to learn
  for nothing.
- **Provenance went into the `BRAND_MARK` doc comment, not
  `assets/agent-icons/SOURCES.md`.** That file is scoped to the three vendor
  marks and their licences; Operon's own mark is neither a vendor's nor in that
  directory. The comment is what a reader of the constant actually finds.
- **The two sizes are literals at the call sites, 18.0 and 48.0.** That is what
  `agent_icon` and `agent_icon_slot` already do; hoisting only these two into
  `src/theme.rs` would make the brand the one mark whose size is spelled
  somewhere else.

Measured across the change: `steering_bytes` 141780 → 141780 (the standing
`diagnose` breach is change 009's and untouched), `largest_module_lines`
8760 → 8768 against a `warn` at 8800, `tests_in_ci` 319 → 322.
