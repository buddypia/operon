# Spec: Rust 1.100.0, egui 0.37, and every direct dependency on its newest release

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. **One toolchain, named everywhere.** `rust-toolchain.toml`'s `channel`,
   `Cargo.toml`'s `rust-version`, and every `toolchain:` pin in
   `.github/workflows/*.yml` name `1.100.0`, and a test refuses any of them
   disagreeing — today the test reads only `ci.yml` and the manifest, so a
   missed pin in `harness.yml`, `scheduled-scan.yml` or `rust-toolchain.toml`
   would pass.
   — `every_toolchain_pin_names_one_version` (replaces
   `cargo_manifest_declares_msrv_matching_ci`, which it subsumes)
2. **Newest direct dependencies.** `Cargo.toml` asks for `eframe 0.37`,
   `egui-phosphor 0.14`, `rfd 0.18`, `rusqlite 0.42`, and (dev) `toml 1`; every
   other direct requirement already admits its newest release and `Cargo.lock`
   resolves each to it. eframe keeps 0.31's rendering: `default-features =
   false` with `accesskit`, `clipboard`, `default_fonts`, `glow`, `links` — 0.37's
   defaults switch the renderer to wgpu and add `system_fonts`, a system font
   fallback that would let glyphs Sarasa lacks come from another face, which
   `src/main.rs` deliberately prevents. Held back, with the reason in `Cargo.toml`:
   `llama-cpp-2` (fork, change 157) and `libc` (1.0 is alpha only).
   — `cargo update --locked` would refuse a stale lock; checked by the build.
3. **Icons render from Phosphor on egui 0.37.** `egui-phosphor 0.14` is built
   against egui 0.36, so its `add_to_fonts` cannot take egui 0.37's
   `FontDefinitions`. Startup registers the face from
   `Variant::Regular.font_bytes()` itself under the name `phosphor`, which
   `proportional_ui_font_chain` already names.
   — `every_icon_resolves_from_the_bundled_icon_font` (existing) and
   `the_icon_face_is_registered_under_the_name_the_font_chain_names` (new)
4. **Nothing a person sees or keeps changes.** The whole suite passes on the
   new versions, with tests that read source shape (`fn update`, panel and menu
   calls) moved to the new API names and nothing weakened, and the running app
   matches the installed one screen for screen.
   — `cargo test --locked` in CI; the screenshot comparison in `screen.md`

## Behaviour

No new string, state, or interaction. Every Japanese string, empty, error,
too-large and interrupted state is the one that exists on `main`. A failure to
read a font file stays silent as today (the `let Ok(bytes) … else continue`).

## Design

- **Placement:** EXTEND — no new behaviour; dependencies, toolchain, and the
  call sites their API changes reach.
- **Toolchain:** `rust-toolchain.toml`, `Cargo.toml`, `ci.yml` (four pins),
  `harness.yml`, `scheduled-scan.yml` to `1.100.0`. Compiler and clippy
  findings new since 1.88 (about 170: mostly `float_literal_f32_fallback`,
  `field_reassign_with_default`, `needless_borrow`) are fixed at the site, not
  allowed, except where a fix would change behaviour — each such `allow` is
  argued in a comment.
- **egui 0.31 → 0.37 API moves** (measured on a scratch build, ~340 errors):
  `Context::run` → `run_ui`/`run_pass` in tests; `Context::style`/`style_mut` →
  `global_style`/`global_style_mut`; `screen_rect` → `content_rect`;
  `SidePanel`/`TopBottomPanel` → `Panel`; `menu::menu_custom_button`,
  `Ui::close_menu`, `popup_below_widget`, `show_tooltip` and the private popup
  helpers → the `Popup`/`MenuButton` API; `RawInput::modifiers` moved;
  `eframe::App::update` → the 0.37 trait methods; `eframe::glow` path.
  Each replacement is read from the 0.37 source, not recalled
  (`.claude/rules/rust.md`).
- **Persisted shape:** none. rusqlite 0.42 opens the same SQLite files; the
  bundled SQLite version moves, which reads every older file.
- **External commands:** none added.
- **Draw path:** no new per-frame work; renamed calls only.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | no | no palette role added; colours pass through unchanged APIs |
| Icons | yes | no mark added; the Phosphor face is registered under its existing name (R3) |
| Identifier SSOT | yes | the toolchain version is one value checked across every file that names it (R1) |
| Durability | yes | read `.claude/skills/durability-invariants/SKILL.md`: no store shape, write order, cancellation, migration or byte budget changes; rusqlite's bump keeps the file format |
| Subprocess safety | no | no `Command::new`, no `unsafe` added |
| Documentation | yes | `CLAUDE.md`'s "egui 0.31" line and comments naming 0.31 behaviour are updated where the behaviour still holds; no user-facing doc names a version |
| Local-first | yes | no new direct crate; new transitive crates (person-approved, see Flagged concerns) are egui 0.37's own text and raster stack and platform shims, none networked; `system_fonts` and wgpu are left off |
| Budgets | no | no scan or output path |

## Flagged concerns

- **New transitive crates.** With the features above, the lock gains egui
  0.37's text/raster stack (`skrifa`, `read-fonts`, `font-types`, `harfrust`,
  `vello_cpu`, `vello_common`, `kurbo`, `peniko`, `color`, `fearless_simd`,
  `glifo`, `guillotiere`, `euclid`, `polycool`, `linebender_resource_handle`,
  `unicode-general-category`, `unicode-properties`, `utf_types`, `self_cell`,
  `send_wrapper`, `phf*`, `siphasher`), `accesskit_ios`, `windows-*` shims,
  `toml_writer` (dev), and a second `egui`/`epaint`/`emath` 0.36 through
  egui-phosphor. Resolved: a new crate is a person's decision (approval.md);
  recorded in `approvals.log`.

- **egui-phosphor builds a second egui (0.36).** It costs compile time and a few
  hundred KB of binary for one unused function. Resolved: accepted until
  egui-phosphor publishes for 0.37; the comment in `Cargo.toml` says when to
  drop the workaround.
- **egui's defaults moved across six releases** (spacing, text layout,
  scrolling). Resolved: R4's screenshot comparison catches it; any visible
  difference is corrected back toward `main`'s rendering, not accepted.

## Acceptance

- `cargo fmt --check`, `cargo clippy --locked -- -D warnings` clean on 1.100.0.
- `cargo test --locked` passes in CI, including
  `every_toolchain_pin_names_one_version`,
  `the_icon_face_is_registered_under_the_name_the_font_chain_names`, and
  `every_icon_resolves_from_the_bundled_icon_font`.
- The new build's main screen, a terminal tab, a diff, the knowledge tab and the
  launch sheet match `main`'s at the same window size (screenshots in
  `screen.md`).

## Rejected alternatives

- egui left at 0.31 for a later change — offered; the person chose all at once.
- Forking egui-phosphor to egui 0.37 — a second fork to maintain, for one
  function the app can do in four lines.
- `#[allow]` on the new clippy lints crate-wide — hides them from every later
  change; fixed at the site instead.
