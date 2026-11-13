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

Filled in during implementation.
