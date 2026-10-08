# Screen: keyboard focus on a button

- **Status**: approved

## What the person chose

In the conversation that opened this change, the person chose "全ボタン共通の1px枠（推奨）":
every button shows keyboard focus the same way. The fill of each button is unchanged.

## The state this screen shows

- **Focused by keyboard** (`response.has_focus()`): a 1px outline in the palette
  role `border_strong`, drawn at `RADIUS_CONTROL` (6), with `StrokeKind::Inside`
  so the ring stays inside the control's rectangle.
- **Not focused, hovered, pressed, or selected by pointer**: no ring. Only the
  existing fill and accent treatment shows, as before.
- **Layout**: no control moves, resizes, or changes its label. The ring is a
  paint-only overlay on the control's own rectangle.

## Controls that get the ring

`primary_button`, `quiet_button`, `nav_tab`, `tab_item_with_count`, `icon_button`,
`small_icon_button`, and the danger form of `verb_button`. `secondary_button`,
the plain `verb_button`, and `icon_text_button` already show the egui outline
(the app theme sets it to `border_strong`) and are not changed by this diff.
The focus test covers the plain verb too, so a theme change that breaks it fails.

Inline buttons that take the ring as well: the brand button and the two "Operon
から削除" danger buttons (`src/app/screens.rs`), the session close (×) button and
the session "…" menu (`src/app/screens.rs`), and the running-port badge menu
(`src/app.rs`). The close and menu buttons are frameless, so without the ring
they show no focus at all.

## What does not change

- No new palette role, colour literal, or glyph.
- No new screen, modal, or layout.
- Radius and fill of every control.

## Verified in code

`focus_ring_on_a_button_is_the_strong_outline_at_control_radius` (src/tests.rs)
checks that a focused control paints exactly one ring at `RADIUS_CONTROL` and an
unfocused one paints none.

## Not yet verified on screen

A person looks at the running app: a focused primary, secondary, quiet, and icon
button show the same ring (`focus-visible-on-every-button` in `state.yaml`).
