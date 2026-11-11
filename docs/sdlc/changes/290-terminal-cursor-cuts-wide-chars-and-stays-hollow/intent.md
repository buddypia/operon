# Intent: the terminal cursor splits wide glyphs in half and never fills as an active block

- **Status**: approved
- **Opened**: 2026-11-12

## Problem

The person's request, verbatim:
「カーソルの表現を業界標準にしたい。例えば英語とカーソル移動で重なるとぴったり合うが、日本語のように4Byte文字だと半分しか重ならないし、カーソルの背景は白じゃなくてもいいんだっけ？対応」

Today the terminal pane draws its cursor as a single-column translucent outline
even when the pane holds keyboard focus, and draws nothing when focus moves
elsewhere. Moving the cursor across ASCII characters frames one character
cleanly, whereas moving it onto a Japanese character, a four-byte ideograph or
emoji, or a character carrying a combining or variation mark frames only the
left half of the glyph. Because the focused cursor has no solid background fill
either, an active terminal reads like an unfocused one, while hardcoding a
literal white fill would vanish on the light theme.

## Who feels it, and when

Anyone moving the cursor with the arrow keys or editing Japanese and English
prompts in a terminal session, or comparing focused and unfocused panes in a
split workspace.

## Desired outcome

- Moving the cursor onto a narrow character or an empty cell frames one display
  column; moving it onto a full-width character — including three-byte and
  four-byte Japanese characters, emoji, and base characters with combining or
  variation marks — covers all two display columns of that character cleanly.
- When an active terminal pane holds keyboard focus, the cursor is a solid
  block in the current theme's terminal foreground colour (light on dark
  themes, dark ink on the light theme — never literal white) with the character
  underneath legible in the terminal background colour.
- When an active terminal pane does not hold keyboard focus, the cursor is a
  hollow one-pixel outline of the same cell width in the terminal foreground
  colour, leaving the character underneath in its own ink.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI with no per-frame heap
  churn or cache invalidation on unchanged output.
- Every painted colour comes from `Palette` roles across Dark, Light, and High
  Contrast themes; no colour literals at drawing call sites.

## Systems likely affected

The terminal pane's cursor sizing and drawing.

## Open questions

None.

## Not in scope

- User-configurable cursor shapes (beam or underline) or cursor blinking,
  neither of which was requested and blinking would force idle repaints.
- Changing how `tmux` captures panes or persisted session records.
