# Bug: a long session tab shows its title twice on hover

Reported 2026-10-30: on the session page, hovering a CLI session tab whose title
is too long for its 160px slot shows the same text twice in the tooltip.

## Cause

`egui::Label::truncate()` in egui 0.31 already attaches the full text as a
tooltip when the galley is elided (`egui-0.31/src/widgets/label.rs:256`). The
central tab strip (`ui_terminal_workspace`, `src/app/screens.rs`) then adds
`.on_hover_text(hover)`, and for a session tab `workspace_tab_title` returns the
title itself as `hover`, so egui stacks two identical lines. A document tab's
hover is its absolute path, which differs from the name and is kept.

The same pattern repeats the visible text in three more truncated labels: the
project name on the project page and in the session column, and the session
title in the terminal header when the session has no goal.

## Fix

Add a hover only when it says something the label does not. No persisted
shape, command, or string changes.

## Requirement

1. A truncated session tab, hovered, paints its full title once more than when
   not hovered — `a_truncated_session_tab_names_its_title_once_on_hover`.
