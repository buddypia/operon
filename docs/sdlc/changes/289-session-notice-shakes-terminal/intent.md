# Intent: an interrupting notice on the Sessions page pushes the whole terminal workspace down
 
- **Status**: approved
- **Opened**: 2026-11-12

## Problem

The person's request, verbatim:
「セッションページのお知らせの割り込みでにターミナル画面全体がが下へ移動してしまい画面が揺れるように見えるから直したい。」

While watching or typing in the Sessions page, only six short success messages
float in the bottom-right corner; every other notice — a progress message such
as opening a file, a session launch or native-resume readiness message, a closed
tab or background scan announcement, an error, or a durability warning — is
inserted at the top of the page above the workspace. When it arrives, the entire
left session column, tab strip, side panel, and terminal viewport drop down by
roughly fifty pixels (shrinking the terminal's height as well), and when it
closes or is replaced by a brief completion toast a moment later, the whole
screen snaps back up, making the terminal screen appear to shake.

## Who feels it, and when

Anyone working on the Sessions page whenever an action, background task, or
error posts a notice or warning while a terminal is on screen.

## Desired outcome

- On the Sessions page, the top edge and height of the workspace (column, tabs,
  side panels, and terminal viewport) stay fixed when any notice or durability
  warning appears, changes between brief and persistent, or is dismissed.
- Every notice and durability warning on the Sessions page remains visible and
  readable over the bottom-right corner: brief notices still clear themselves
  after the toast timeout, non-brief notices still stay until dismissed with `✕`
  or replaced, and a durability warning still stays without a close button until
  persistence confirms.
- Other pages keep their existing notice presentation.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI; no extra work per frame
  when nothing is shown.
- Colours come from `Palette` roles and glyphs from `ICON_*` constants; no new
  colour role, glyph, or persisted field.
- User-facing text is Japanese; code, comments, and docs are English.

## Systems likely affected

Top-level page and toast drawing in `src/app.rs`.

## Open questions

None.

## Not in scope

- Reclassifying individual call sites between `notice` and `notice_briefly`.
- Changing how notices or durability warnings are drawn on Home, Projects, or
  Settings.
