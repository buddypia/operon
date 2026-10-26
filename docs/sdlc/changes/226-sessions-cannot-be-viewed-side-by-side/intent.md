# Intent: two sessions cannot be watched side by side

- **Date**: 2026-10-26
- **Route**: `feature`
- **Risk tier**: `low` (`autonomy: full`, `screen: yes`)
- **Status**: approved

## The request, as the person wrote it

> セッションを移動して分割して利用できるようにしたい。既存セッションのターミナルにD&Dにしフォーカルが当たっと箇所は視覚的にわかるようにして置いておくと位置によっては上下で配置されたり、左右で配置されたり。よくあるVSCodeの開いているエディターの位置調整みたいな感じ。

## Problem

The session workspace draws exactly one terminal: `ui_terminal_workspace` shows
the selected session in the centre and nothing else. A person running two agents
at once — one implementing, one reviewing, or a dev server beside the agent that
edits it — has to click back and forth in the session list to see either, and
loses sight of the other each time. Output that arrives in the hidden one is
only noticed through the unread mark.

## Desired outcome

- A session dragged from the session list, or a terminal's own header, onto a
  terminal that is already showing is placed next to it: dropped on that
  terminal's left or right part the two sit side by side, on its top or bottom
  part they are stacked, and dropped on its middle it replaces what that
  terminal shows — the way an editor tab is dropped in VS Code.
- While the drag is held, the part of the target terminal the dropped session
  would take is highlighted, so where it lands is visible before the release.
- Every terminal on screen keeps updating, and the one last clicked holds the
  keyboard and drives the side panels, as the single terminal does today.
- A terminal can be taken off the screen again without stopping its session.

## Non-goals

- Keeping the arrangement across restarts: the panel docking it sits beside is
  not kept either (`DockLayout` is in memory).
- Tabs inside one split, or dragging a session out into a separate window.
- Showing one session in two places at once.
