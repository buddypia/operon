# Spec: draw Operon's own icon where the brand glyph is drawn now

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. The mark left of "Operon" in the top bar is the artwork from
   `assets/operon-icon-1024.png`, not a font glyph.
2. The mark on the first-run Projects page is the same artwork, larger.
3. The artwork ships as a bundled RGBA PNG of the ring with the icon's dark
   plate removed, so it sits on whatever surface it is drawn on rather than
   introducing a plate of its own. This is the shape approved at the screen gate.
4. The mark is decoded and uploaded to the GPU at most once per process, not per
   frame — the same cache the three agent marks already go through.
5. The top-bar mark stays one clickable control together with the word "Operon",
   keeps the hover text `ホーム`, and keeps navigating to `Page::Home`.
6. The top bar's height and the position of everything right of the mark are
   unchanged.
7. No `ICON_*` constant is left in the vocabulary with no call site: if
   `ICON_BRAND` has no reader after this change, it and its `ICON_VOCABULARY`
   row are removed.

## Behaviour

Nothing a person reads changes. `ホーム` stays the hover text on the brand
button and stays the label of the first navigation tab; the first-run page keeps
`まずはプロジェクトを開いてください`, `フォルダを選ぶと、その中で AI を起動できます。`,
and `プロジェクトフォルダを選択…` exactly as they are. The change is what the
mark above and beside them is drawn from.

Approved frames, from the screen gate:

```
Before ─ 現在（Phosphor の破線円を14ptの文字として描画）
┌──────────────────────────────────── 900px ─┐
│ ● ● ●    ◌  Operon   ⌂ ホーム  ▤ プロジェクト │
└────────────────────────────────────────────┘
          ↑ 文字と同じインクの輪郭線。O に見える

After ─ リングだけ（18px 透過）
┌──────────────────────────────────── 900px ─┐
│ ● ● ●    ◍  Operon   ⌂ ホーム  ▤ プロジェクト │
└────────────────────────────────────────────┘
          ↑ プレートなし。光るオレンジのリング

初回起動（プロジェクト未登録）
┌──────────────────────── 900px ─┐
│              ◍       ← 48px    │
│   まずはプロジェクトを開いてください│
│  フォルダを選ぶと、その中で AI を  │
│           起動できます。         │
│  ┌────────────────────────┐    │
│  │ プロジェクトフォルダを選択…│    │
│  └────────────────────────┘    │
└────────────────────────────────┘
```

States that are not the happy one:

- **The mark fails to decode.** The mark is bundled with the binary, so this can
  only mean a corrupt build, but the draw path still has to answer. The top-bar
  button falls back to the text `Operon` alone and stays clickable — the way
  `agent_restore_button` already falls back to a name rather than to a gap
  nothing can be clicked in. The first-run page draws no mark and keeps its
  heading and its button.
- **Light theme.** The plate is gone, so the ring sits directly on
  `palette.raised` (#FFFFFF) and on `palette.panel` (#F7F5F2). The artwork's own
  orange carries it; the approved shape is the one that was compared on all
  three themes.
- **The top bar at its narrowest.** The mark replaces a glyph of the same
  optical width in the same button, so no row that fits today stops fitting.

## Design

| Module | What it gains |
|---|---|
| `assets/operon-mark-128.png` | new. 128×128 RGBA, the master's ring with the plate keyed out. 128px is the size the three agent marks already ship at |
| `src/glyphs.rs` | `BRAND_MARK`, the `include_bytes!` for it, beside the three `AGENT_ICON_*` constants. `ICON_BRAND` and its `ICON_VOCABULARY` row are removed |
| `src/ui/widgets.rs` | `png_texture(ctx, name, bytes)` — decode-once, upload-once, keyed in `ctx.data`. This is `agent_icon_texture`'s body with the agent lookup lifted out. `brand_mark(ui, size)` draws the mark; `brand_mark_image(size)` hands back the `Image` the top-bar button is built from |
| `src/agents.rs` | `agent_icon_texture` becomes the agent lookup plus a call to `png_texture`; `decode_agent_icon` moves to `src/ui/widgets.rs` as `decode_png_mark` |
| `src/app.rs` | the top-bar button becomes `Button::image_and_text`; the first-run mark becomes an `Image` |
| `src/tests.rs` | the guards below |

`egui::Button::image_and_text` exists in the pinned egui 0.31.1
(`widgets/button.rs:56`), so mark and word stay one control and one hover
target rather than two widgets that happen to sit together.

Nothing crosses a process boundary. Nothing is persisted. No dependency is
added: `image` already decodes the three agent marks.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | No colour role is added or read. The mark's colour is pixels in a bundled PNG, the same as the three agent marks, and no drawing call site names a `Color32`. The glyph it replaces was drawn in `palette.accent_text`; nothing else in the row changes ink |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | Yes, in reverse | This change *removes* a vocabulary entry rather than adding one. `ICON_BRAND` exists because the mark was a Phosphor glyph; once it is artwork it is not part of the glyph vocabulary, and leaving a constant nothing reads would make `ICON_VOCABULARY` describe more than the app draws. The bundled-asset constant goes beside `AGENT_ICON_*`, which is where the crate already keeps marks that are pictures |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | The only new string is the texture cache key, which is written once in `src/ui/widgets.rs` and never compared against a literal elsewhere |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | Nothing is written. `src/store.rs` and `src/models.rs` are untouched |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No `Command::new`. `src/agents.rs` is touched only to delegate its texture cache; `is_safe_agent_command` and `is_safe_agent_option` are not reached |
| Documentation — user-facing docs change in all three languages together | No | No user-facing document describes the corner mark. `README.md`, `README.ja.md`, and `README.ko.md` show the bundle icon, which is unchanged |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | The mark is `include_bytes!`d into the binary. No network, no `unsafe` |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | One new bundled asset with a stated ceiling: 128×128 RGBA, under 40 KB on disk, one decode and one texture upload per process. `every_bundled_mark_is_small_enough_to_ship` pins the ceiling |

## Flagged concerns

- **The plate-keying recipe is not reproducible from the repository.** The
  shipped PNG is derived from `assets/operon-icon-1024.png` by removing the dark
  plate, and nothing in the repository can re-derive it: `sips` cannot key an
  alpha channel and the repository has no Python dependency to add one for.
  *Answer:* the asset is committed the way `assets/agent-icons/*.png` already
  are, its provenance is recorded beside them, and the guard checks the
  *properties* that make it the right asset — 128×128, corners fully
  transparent, an opaque orange ring — rather than pixel equality with a recipe.
  A property guard survives a re-export of the artwork; a hash would not, and a
  generator script depending on a toolchain nobody installed would rot unrun.

## Acceptance

- `cargo fmt --check` prints nothing; `cargo clippy --locked -- -D warnings`
  prints nothing past the compile lines.
- `cargo test --locked` passes with `6 ignored`, including
  `the_brand_mark_is_a_transparent_ring_not_a_plate`,
  `every_bundled_mark_is_small_enough_to_ship`, and
  `the_brand_is_drawn_from_artwork_and_not_from_the_glyph_vocabulary`.
- `bash scripts/check-bands.sh` reports no new breach.
- In the running app: the top-left corner shows the orange segmented ring, not a
  character; clicking it still goes to ホーム and hovering it still says ホーム;
  the ring reads the same on ダーク, ライト, and ミッドナイト; the first-run
  Projects page shows the same ring above まずはプロジェクトを開いてください.

## Rejected alternatives

- **Keep the plate (option A at the screen gate).** Matches the Dock and the
  three agent plates exactly, but puts the only black tile on screen in the
  light theme. Rejected by the person at the gate.
- **Embed the 1024×1024 master and key the plate at runtime.** No new asset, but
  1.1 MB in the binary and a 1024×1024 decode on the frame that first draws the
  corner — a hitch in a path the rules say must not do work.
- **Tint a Phosphor glyph, or draw the ring with `Painter` arcs.** Both keep the
  mark a drawing of the icon rather than the icon, which is the complaint.
- **A generator script for the asset.** Would need a Python imaging dependency
  this repository does not have, to regenerate a file that changes when the
  brand does. Provenance in a comment plus a property guard costs less and rots
  less.
