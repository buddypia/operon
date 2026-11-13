# Plan: Rust 1.100.0, egui 0.37, and every direct dependency on its newest release

- **Spec**: `./spec.md`
- **Approved**: 2026-11-13 (see `approvals.log`)
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `rust-toolchain.toml`, `Cargo.toml`, `.github/workflows/ci.yml`, `.github/workflows/harness.yml`, `.github/workflows/scheduled-scan.yml` | toolchain `1.100.0` |
| `Cargo.toml`, `Cargo.lock` | eframe 0.37, egui-phosphor 0.14, rfd 0.18, rusqlite 0.42, toml 1; `cargo update` for the rest |
| `src/main.rs` | Phosphor face registered from `font_bytes()`; window/native options on the 0.37 API |
| `src/app.rs`, `src/app/*.rs`, `src/ui/*.rs` | egui 0.37 call sites; new clippy findings |
| other `src/*.rs` | new clippy findings only |
| `src/tests.rs` | R1 and R3 tests; egui test harness (`run_ui`/`run_pass`) and source-shape tests moved to 0.37 names |
| `CLAUDE.md` | "egui 0.31" → "0.37" |

## Order of work

1. Write `every_toolchain_pin_names_one_version` (tightening: it checks a
   superset of the pins the old test checked), run it against a deliberately
   unbumped `harness.yml` pin, watch it fail; bump every pin to 1.100.0; watch it
   pass.
2. In the same commit — the commit gate runs clippy `-D warnings`, so the pin
   bump cannot land without them — fix the 1.100 compiler and clippy findings at
   their sites. `make q.fast` green. Commit.
3. Bump rfd, rusqlite, toml and `cargo update` the compatible rest; fix any
   fallout; run the store and knowledge tests by filter. Commit.
4. Capture "before" screenshots from a build of this commit (old egui) at a
   fixed window size: main screen, terminal tab, diff, knowledge tab, launch
   sheet. The tree does not compile between steps 5 and 6.
5. Bump eframe 0.37 (features as spec R2) and egui-phosphor 0.14; write
   `the_icon_face_is_registered_under_the_name_the_font_chain_names` and
   register the face from `font_bytes()`.
6. Move every call site to the 0.37 API, reading each replacement from the
   0.37 source; then the tests. `make q.fast` green, then the named tests and
   the egui-driven test modules by filter. Commit.
7. "After" screenshots at the same size; correct any difference back toward
   `main`. Record both in `screen.md`.
8. verification.yaml, verifiers, close, push, CI, land, package.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| egui default spacing/text layout moved | widgets shift, wrap or clip differently | step 7 screenshot comparison |
| a menu/popup replacement closes or opens at a different moment | a menu stays open after a click, or a popup never shows | existing menu/launch tests; exercised in the running app in step 7 |
| `App::update` split changes when `on_exit`/logic run | state saved late or not at all | `on_exit` source tests; exercising quit in step 7 |
| rusqlite 0.42 bundled SQLite reads older files differently | knowledge DB fails to open | knowledge store tests; opening the real DB in step 7 |
| tests rewritten to the new API lose an assertion | the suite stays green while coverage drops | `gate-commit.sh` test-count check; diff review of `src/tests.rs` |
| two egui versions linked | icon font mismatch | R3 tests |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- CI `cargo test --locked` — `test result: ok`, no new ignored.
- `every_toolchain_pin_names_one_version` watched failing on one stale pin.
- `the_icon_face_is_registered_under_the_name_the_font_chain_names` watched
  failing with the registration removed.
- Before/after screenshots in `screen.md` matching.

## Departures from the plan

- **egui-phosphor is registered by hand.** 0.14 is built against egui 0.36, so
  its `add_to_fonts` takes the other egui's `FontDefinitions`. `src/main.rs`
  inserts `Variant::Regular.font_bytes()` under `ICON_FACE` itself, and
  `the_icon_face_is_registered_under_the_name_the_font_chain_names` checks the
  name the font chain asks for is the one registered. The duplicate egui stays
  until egui-phosphor targets 0.37, as approved.
- **`App::update` became `logic` + `ui`.** eframe 0.37 calls `logic(&Context)`
  once per frame and then `ui(&mut Ui)`. Everything that does not draw moved to
  `logic`; panels now show on the root `Ui`. The source-shape check in
  `the_app_sweeps_expired_handoff_archives_once_at_startup` reads `fn logic`.
- **Menus.** `menu_custom_button` in `src/ui/widgets.rs` wraps 0.37's
  `MenuButton::from_button(..).ui(..).0`, so the four call sites keep one shape;
  the tab strip's `+` menu is an `egui::Popup` anchored to its button.
- **Button text keeps `TextStyle::Button`.** 0.37's default button theme sets a
  plain label in `TextStyle::Body` (14 px, not 13.5). `apply_interface_metrics`
  registers `ButtonTextAtButtonSize`, which takes egui's own button style and
  changes only the font. The theme API is behind egui's `experimental` feature,
  so `Cargo.toml` names `egui` directly for that one feature — eframe's own
  egui, no new crate in `Cargo.lock`.
- **Animation pace.** 0.37 raised `animation_time` from 1/12 s to 0.2 s; the
  app sets 1/12 s back.
- **Three draw fixes 0.37 needed.**
  - The terminal caret's containment check got the half-pixel slack vertically
    too: 0.37 rounds the scroll offset to a whole pixel, which pushed a fully
    visible caret a third of a pixel past the clip and hid it.
  - The history row's title sits in a scope keyed by the row, so its `Id` no
    longer counts the corner's widgets; hovering swaps the time for ✕, and 0.37
    forgets a press on a widget whose `Id` moved before the release.
  - The notice banner wraps its message in a column beside the mark: a label
    that starts partway along a wrapping row gets a first-row indent that 0.37
    subtracts again on a later row with nowhere to break, which put a long path
    outside the frame.
- **Tests.** `Context::run` became `run_ui`/`run_pass`; a `CentralPanel` shows
  on the root `Ui`. A test-only `TestDroppedFile` implements 0.37's
  `DroppedFile` trait. The test profile turns off epaint's debug assertions:
  0.37 asserts every frame's texture upload reached a renderer, and the suite
  runs frames with no renderer by design. Tests that clicked typed coordinates
  in the toolbar now read where the frame drew the tab: the headless default
  fonts lay CJK out at new widths under 0.37.
  `repaint_loop_idle_interval_is_not_polling_100ms` warms up one more frame:
  0.37 asks for one more pass to style a button it had not seen.
- **Screenshots at 1280×820**, the default window, not 1440×900 —
  `screen.md` has the comparison.
- **Disk.** The test build ran the disk out of space once; the branch's own
  `target/` output for `operon` and a scratch probe target were cleaned and
  rebuilt.
