# Intent: keyboard focus looks different on every button

- **Status**: approved
- **Opened**: 2026-10-08

## Problem

The app's buttons do not agree on what keyboard focus looks like. Tabbing onto
a button shows a different mark depending on which helper drew it: some get a
1px outline, some only a grey fill, and the primary action shows nothing at
all. The person who reported it read the difference as a size mismatch of the
focus "shadow", and asked for the whole UI to be made consistent.

## Who feels it, and when

Anyone who works the app from the keyboard. Tab moves focus from the toolbar
into a page, and at the primary action on the page the focus is invisible, so
the next Enter press lands on something the person cannot see. On the icon
buttons in the toolbar and in session rows the only sign of focus is a faint
fill, which is easy to miss against a hovered row.

## Desired outcome

Every button in the app shows keyboard focus with the same mark: a 1px outline
in the strong border colour, at the same corner radius as the control. The fill
of each button is unchanged, so resting, hovered, pressed, and selected looks
stay as they are. A person tabbing across a page can see where focus is on
every button, including the primary action.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Drawing code names roles, not colours: the outline uses the existing
  `border_strong` role and adds no palette row.
- User-facing text is Japanese; no new strings are added by this change.

## Systems likely affected

`src/ui/widgets.rs` (the button helpers and the focus decision), `src/tests.rs`
(the focus tests). `DESIGN.md` is expected to need no change because no palette
role is added.

## Open questions

None. The treatment was chosen by the person: a uniform 1px border on every
button, recommended over an accent ring.

## Not in scope

- Focus on custom-drawn rows and cards (`clickable_card`, the stat tile) and on
  the diff and terminal focus rectangles. Their outline is already a 1px stroke
  in `border_strong`; their corner radius differs (10 or 0), which is a
  separate consistency question and is not part of this change.
- Changing button fills, sizes, or hover states.
- Any colour, icon, or text change.
