# Intent: resizing the window makes every session terminal redraw over and over

- **Status**: approved
- **Opened**: 2026-11-07

## Problem

The person's request, verbatim: 「セッションのターミナルを縮小・最大などウィンドウを調整するたびに再レンダリングされているようだが、これパフォーマンス悪いからもっと効率よくパフォーマンスも出るやり方にして。」

While the window is being resized — dragged smaller, maximized, restored, or
minimized to the Dock — the session terminal is told its new size at almost
every intermediate step. Each time, the agent CLI running in it redraws its whole
screen and Operon reads and lays out the pane again, so one resize gesture costs
many full redraws instead of one, and the app stutters while it happens.

## Who feels it, and when

Anyone with an agent session open who resizes, maximizes, restores, or minimizes
the Operon window — most visibly with Claude Code or Codex, whose full-screen
interfaces reprint their whole conversation on every size change.

## Desired outcome

- One resize gesture (a drag that ends, a maximize, a restore) makes the agent
  redraw once, at the final size, not once per intermediate size.
- While the window is changing size and the terminal's text has not changed, the
  pane does not lay its text out again every frame.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- A newly opened session still gets its correct size at once: the first size it
  is told must not wait.

## Systems likely affected

`src/app.rs` (resize requests), `src/app/screens.rs` (where the pane asks),
`src/ui/terminal.rs` (the pane's layout cache), `src/config.rs` (the delay).

## Open questions

None: the delay is a recoverable tuning value, decided in the spec.

## Not in scope

- Making tmux or the agent CLI redraw more cheaply once it is told a new size.
- The in-app divider and tab drags beyond sharing the same rule.
- A separate rule for minimizing to the Dock: egui does not report a macOS
  window's minimized state after startup, so minimize gets only the rule every
  other size change gets.
