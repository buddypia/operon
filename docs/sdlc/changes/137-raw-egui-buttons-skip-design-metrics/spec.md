# Spec: every button at a documented height, every text size on the scale

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. No production source calls egui's `small_button` or `Button::…small()`.
   Both zero the vertical button padding and drop the 28px minimum height that
   `apply_interface_metrics` sets, which is exactly the cramped "パスを手入力".
   Pinned by `every_button_keeps_a_documented_height`.
2. Every `min_size(egui::vec2(_, h))` on a control names `CONTROL_HEIGHT` or
   `CONTROL_HEIGHT_SMALL` for `h`, never a number. Pinned by the same test.
3. Every literal text size in production source — `.size(N)`,
   `FontId::proportional(N)`, `FontId::monospace(N)`, `FontId::new(N, …)` — is a
   `fontSize` in `DESIGN.md`'s `typography` block. The test reads the document,
   so the document stays the single source. Pinned by
   `every_text_size_is_a_level_on_the_type_scale`.
4. `DESIGN.md` says which helper in `src/ui/widgets.rs` draws which kind of
   button, and that icons take the type scale too; `.claude/rules/palette-and-glyphs.md`
   carries the same rule so it arrives when an agent opens a drawing file.

## Behaviour

No string is added or changed. What a person sees:

- "パスを手入力" on the project page, "キャンセル" and "削除" in the sidebar's
  session-removal prompt, and "削除" / "解決" / "再開" on a diff note are
  ordinary 28px buttons with the shared padding, like every other button.
- The find bar's previous / next arrows and the copy button in the running-port
  menu are 24px frameless icon buttons with the shared focus ring, like the
  other icon buttons in rows.
- The session row's close (×) and "…" controls and the running-port badge are
  24px; the launch button is 28px like every other button (it was 34px).
- Off-scale text moves to the nearest documented level: 10 and 11 → 11.5,
  13 → 13.5, 17 → 16, 18 → 20, 24 → 20, 26 and 30 → 28. Differences are half a
  pixel to two pixels; nothing reflows beyond that.

## Design

Placement: `EXTEND` — `src/ui/widgets.rs` already holds every button helper
(`secondary_button`, `small_icon_button`, …); call sites move onto them. No new
helper is needed. Disabled icon buttons wrap `small_icon_button` in
`ui.add_enabled_ui`.

Call sites that change: `src/app.rs` (session-removal prompt, find bar, port
badge and its copy button, drop overlay and progress text sizes),
`src/app/screens.rs` (manual path toggle, launch button, session row controls,
off-scale sizes), `src/ui/diff.rs` (note actions, sizes), `src/ui/files.rs`,
`src/ui/session_tree.rs`, `src/ui/markdown_view.rs`, `src/ui/widgets.rs`
(sizes only).

The two tests live in `src/tests.rs` and scan `source_files()` minus
`src/tests.rs`. No persisted shape, no external command, nothing new in the
draw path.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | No colour changes; existing roles only. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | No icon added; existing `ICON_*` constants only change size. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | The type scale lives once, in `DESIGN.md`; the test reads it rather than restating it. Heights stay `CONTROL_HEIGHT` / `CONTROL_HEIGHT_SMALL` in `src/theme.rs`. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | Drawing only; nothing persisted. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | No child process. |
| Documentation — user-facing docs change in all three languages together | no | `DESIGN.md` is English-only and has no translations; README text is unchanged. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | no | None added. |
| Budgets — any new scan or output path states its byte and item ceiling | no | The tests reuse `source_files()`, which already caps each file at `MAX_HARNESS_FILE_BYTES`. |

## Flagged concerns

- **The launch button loses 6px** — it was 34px, the only button above 28.
  Answered: `DESIGN.md` says every clickable thing is 28px or 24px, and the
  filled accent already marks it as the one primary action; a third height is the
  inconsistency the person reported.
- **Menu items stay `ui.button`** — buttons inside an open popup menu are laid
  out by egui as a menu list. Answered: out of scope (intent), and `ui.button`
  already takes the shared padding and 28px height, so it is not a source of the
  defect; the test does not forbid it.
- **Line-based detection of `.small()` on a button** — `RichText::small()` is a
  documented level (12px) and must stay legal. Answered: the test flags a line
  that names `Button` and calls `.small()`, plus every `small_button(`; rustfmt
  keeps the builder call on the constructor's line for the forms in this crate,
  and the mutation sweep shows it going red.

## Acceptance

- `cargo fmt --check` prints nothing; `cargo clippy --locked -- -D warnings`
  prints nothing past the compile lines.
- `cargo test --locked every_button_keeps_a_documented_height` and
  `cargo test --locked every_text_size_is_a_level_on_the_type_scale` pass, and
  each fails when one `small_button` or one `.size(13.0)` is put back.
- `cargo test --locked design_md_documents_exactly_what_the_app_paints` passes.
- In the running app, "パスを手入力" is as tall as the "フォルダを選択…" button
  beside its section, with its label inset like any other button.

## Rejected alternatives

- A new `small_text_button` helper at 24px: YAGNI — every cramped site sits
  where a 28px button already fits.
- Named type-scale constants replacing every `.size(N)`: touches ~200 call
  sites for the same guarantee the test gives by reading `DESIGN.md`.
- A rule document without a test: the rule only loads when a file is read, and
  `CLAUDE.md` says the test is what enforces.
