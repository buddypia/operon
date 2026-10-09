# Plan: every button at a documented height, every text size on the scale

- **Spec**: `./spec.md`
- **Approved**: 2026-10-09
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/tests.rs` | `every_button_keeps_a_documented_height`, `every_text_size_is_a_level_on_the_type_scale` |
| `src/app.rs`, `src/app/screens.rs`, `src/ui/diff.rs` | raw `small_button` / `.small()` / literal heights → `secondary_button`, `small_icon_button`, `CONTROL_HEIGHT(_SMALL)` |
| `src/app.rs`, `src/app/screens.rs`, `src/ui/*.rs` | off-scale `.size(N)` / `FontId` literals → nearest `DESIGN.md` level |
| `DESIGN.md` | Controls: which helper draws which button, no egui `small` variant; Typography: icons take the same scale |
| `.claude/rules/palette-and-glyphs.md`, `CLAUDE.md` | the button and type-size rule beside colour and icons; the table row says so |
| `docs/sdlc/lessons.md` | the lesson and its guard |

## Order of work

1. Write the two tests; run them and watch both fail on the current tree.
2. Move each call site; the tree compiles after each file.
3. Docs and rule.
4. `make q.fast`, the named tests, mutation sweep (put one `small_button` and one
   `.size(13.0)` back, watch red), `bash scripts/package-macos.sh`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| `small_icon_button` needs `palette` where the call site has none | compile error | the compiler |
| a disabled icon button stops looking disabled | arrow drawn at full ink with no matches | `add_enabled_ui` greys it; checked in the running app |
| a 28px button grows a tight row | row height changes | screenshot of the project page and sidebar |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked every_button_keeps_a_documented_height` and
  `every_text_size_is_a_level_on_the_type_scale` — pass, and fail before step 2.
- CI: `test result: ok. N passed; 0 failed; 6 ignored`.
- The project page's "パスを手入力" at 28px with inset label.

## Departures from the plan
