# Screen — 223-command-palette-anchors-at-top-center

## Affected surface
- Command palette overlay (`OperonApp::ui_command_palette`, `src/app/screens.rs:446-520`)

## Layout and hierarchy
- Window anchor: `egui::Align2::CENTER_TOP` with offset `[0.0, 68.0]`
- Default width: `520.0` px (`collapsible(false)`, `resizable(false)`)
- Contents:
  1. Single-line search field (`hint_text(tr("操作・セッション・プロジェクトを検索"))`, `desired_width(f32::INFINITY)`)
  2. Selectable result rows with right-aligned keyboard chords (`palette.text_muted`)

## Conformance to `DESIGN.md`
- Positions the command palette in the upper-center of the window so it never collides with the left navigation rail (`230.0` px) or window controls.
