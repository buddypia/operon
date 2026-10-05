# Intent: the mark at the top left of the window reads as a letter, not as Operon

- **Status**: approved
- **Opened**: 2026-09-03

## Problem

The small mark in the top-left corner of the window, the one immediately left of
the word "Operon", does not read as a logo. It reads as a character — an `O`, a
degree sign, something typed rather than something drawn. It is a monochrome
outline at text size, set in the same ink as the word beside it, so the eye
groups the two and takes the pair for one piece of text.

The application already has a real mark: the icon macOS shows in the Dock, in
the Finder, and in the app switcher. That mark is a segmented orange ring on a
dark plate, and the person using Operon has already seen it several times before
the window opens. The window then declines to show it and draws an approximation
instead.

Every other product named inside Operon is named with its own icon. A session
row shows Claude Code's mark, Codex's mark, Antigravity's mark — each the real
one, each a coloured plate. Operon is the only product in its own window that
gets an outline instead of a logo, which reads as an oversight rather than as a
choice.

## Who feels it, and when

Every launch, in the first second, before anything else on screen has been read.
The corner mark is the first thing above the navigation and it is what says
which application this window belongs to; the person who reported it looked at
it and had to ask whether it was text.

It is also felt on first run, on the empty Projects page, where the same mark is
drawn much larger — big enough that "this is a letter, not a logo" is
unmistakable — above the invitation to open a folder.

## Desired outcome

The corner shows Operon's actual icon: the same artwork the Dock shows, in
colour, recognisable as the same thing at a glance. The person who has seen the
Dock icon sees the same mark in the window and does not have to decide whether
it is a character.

It stays legible on all three themes, it does not grow the row it sits in, and
it stays clickable with the same hover wording it has now.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- The draw path runs every frame: a mark decoded or uploaded per frame is a
  stutter, not a detail.
- User-facing text is Japanese; code, comments, and docs are English.

## Systems likely affected

`src/glyphs.rs` (the icon vocabulary and the bundled marks), `src/app.rs` (the
top bar and the first-run empty state), `src/ui/` (where a reusable drawing
helper would go), `src/agents.rs` (which already owns the only PNG-to-texture
path in the crate).

## Open questions

- Does the mark keep the artwork's dark plate, the way the Dock icon and the
  three agent marks all do, or is it the ring alone with the plate dropped? A
  person decides this; it is the screen-approval gate.

## Not in scope

- The bundle icon itself. `assets/operon-icon-1024.png`, `assets/Operon.icns`,
  and `scripts/make-icns.sh` are correct and are not redrawn.
- The other fifty-odd marks in the icon vocabulary. Phosphor outlines are right
  for actions; this is about the one mark that is a logo and not an action.
- The window title bar and the traffic lights.
