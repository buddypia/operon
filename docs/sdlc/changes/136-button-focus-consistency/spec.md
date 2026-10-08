# Spec: one keyboard focus mark for every button

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Every button helper in `src/ui/widgets.rs` that paints a frameless or
   primary button shows the same focus mark when it has keyboard focus: a 1px
   stroke in `border_strong`, drawn on the control's rectangle at
   `RADIUS_CONTROL` (6).
2. The mark is present only while the button has keyboard focus, and absent
   otherwise. A pointer hover or press does not add it.
3. The fill of each button is unchanged in every state. The focus mark adds an
   outline and nothing else.
4. The secondary button keeps egui's own focus outline, which is the same mark.
5. The focus decision is a single pure function, and a unit test pins its
   output for every theme: a stroke of width 1 in `border_strong` when focused,
   and nothing when not.

## Behaviour

- Tab onto a primary action: a 1px `border_strong` outline appears around the
  filled accent button. The accent fill and the on-accent label are unchanged.
- Tab onto a quiet button, an icon button, a small icon button, a nav tab, or
  a tab with a count: the same 1px outline appears around the control, over
  its existing hover or selected fill.
- Tab onto a selected nav tab or tab item: the outline appears, and the
  selected fill and the accent underline are unchanged.
- Focus leaving the button removes the outline at once.
- Mouse-only use shows no outline on any button.
- No string is added or changed; all text is as before.

## Design

`focus_stroke` already decides the mark for custom rows and cards. This change
adds one painting helper beside it in `src/ui/widgets.rs`, which takes the
response and the palette, calls `focus_stroke`, and paints the result on
`response.rect` with `RADIUS_CONTROL` and `StrokeKind::Inside`. `Inside` is what
egui uses for its own `active` outline, so a painted ring and a native one sit
in the same place and do not spill into a neighbouring tab or toolbar cell.

The helper is called at the end of each button helper, after `ui.add`, for:
`primary_button`, `quiet_button`, `nav_tab`, `tab_item_with_count`,
`icon_button`, and `small_icon_button`. `secondary_button` and
`icon_text_button` are unchanged because they keep egui's native outline.

`verb_button` (`src/ui/widgets.rs`) takes the ring in its danger form only. A
plain verb already takes egui's native outline, which the app theme sets to
`border_strong`, so it needs no call. A danger verb overwrites that outline with
`palette.danger` while hovered or pressed, so the helper is called again over it
when `danger` is set.

Three buttons are built inline in `src/app/screens.rs` rather than through a
helper, and each sets `.stroke(egui::Stroke::NONE)`: the brand button in the
toolbar and the two "Operon から削除" danger buttons. Each calls the same helper
after its `ui.add`, so they take the ring too.

Three more are built inline with `.frame(false)`, which suppresses the stroke
the same way: the session row's close (×) button and its "…" menu in
`src/app/screens.rs`, and the running-port badge menu in `src/app.rs`. Each
calls the helper on its response (for a menu, the `.response` of
`menu_custom_button`), so they take the ring too.

`focus_stroke` and its callers for cards and the stat tile are unchanged.

No palette row, no icon, no string, and no `DESIGN.md` change: the role used
(`border_strong`) already exists and is already documented.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | Uses the existing `border_strong` role; no colour literal and no new role. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | No icon is added or changed. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | no | No persisted or shared string is touched. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | Drawing only; nothing is persisted. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | No child process is started. |
| Documentation — user-facing docs change in all three languages together | no | The user-facing behaviour described in `DESIGN.md` is unchanged; no palette role changes. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | no | No network, no `unsafe`. |
| Budgets — any new scan or output path states its byte and item ceiling | no | No scan or output path is added. |

## Flagged concerns

- **Tabs and the count tab are buttons, so they take the ring too** — the
  nav tabs and the in-page tab bar are egui buttons that were drawn without a
  stroke, so they fall within the rule "every button". Answered: they take the
  ring, because a tab a keyboard user cannot see is the same failure as an
  invisible primary action. The selected accent underline is not changed.
- **Inline buttons outside the helpers** — the brand button and the two
  danger "Operon から削除" buttons in `src/app/screens.rs` are plain
  `egui::Button`s that suppress their stroke, so keyboard focus would be
  invisible on them. Answered: they take the ring, because the request is every
  button, and a removal button a keyboard user cannot see is the same failure.
  The danger fill and the brand mark are unchanged.
- **Frameless inline buttons** — the session close (×), the session "…" menu,
  and the running-port badge are `.frame(false)` buttons, so egui draws no
  outline on focus either. Answered: they take the ring, for the same reason as
  the danger buttons above. The plan did not list them; the review that found
  them is the reason they are here.
- **Danger verb overwrites egui's outline** — `verb_button` with `danger` set
  replaces the active outline with `palette.danger` (hover and press), so the
  focused danger verb would show the danger colour, not the shared ring. Answered:
  the helper is called again over the danger verb, so it shows the shared ring.
  A plain verb is not changed. The plan did not list `verb_button`; this entry
  records the decision made after review.
- **Inside or Outside stroke** — `clickable_card` uses `Outside`, but a ring
  drawn outside a button would overlap the neighbouring cell in the toolbar and
  tab rows, which are packed at small spacing. Answered: `Inside`, which matches
  egui's native `active` outline on the secondary button, so every button's
  ring sits in the same place.

## Acceptance

- `cargo fmt --check` prints nothing.
- `cargo clippy --locked -- -D warnings` prints nothing past the compile lines.
- `cargo test --locked` passes, including
  `focus_ring_on_a_button_is_the_strong_outline_at_control_radius`, which fails
  before the helper exists and passes after it.
- `bash scripts/check-bands.sh` prints `bands: N metrics within their bands`.
- `bash scripts/check-transcript-vocabulary.sh` prints `N kinds observed locally, all classified`.
- In the running app, Tab through the toolbar, a page with a primary action,
  and a tab bar: each focused control shows the same 1px outline at the same
  corner radius, and no control with the pointer off it shows one.

## Rejected alternatives

- **Accent-coloured ring on every button** — the person chose the uniform
  `border_strong` outline; an accent ring would also collide with the accent
  fill of the primary action.
- **Change egui's `active` style globally** — would change hover and press
  looks on every widget, including ones this change does not cover.
- **Radius 10 to match cards** — the button rule is `RADIUS_CONTROL`; cards are
  a separate surface and their radius is a separate question.
