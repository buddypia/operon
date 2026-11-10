# Intent: the Dock icon does not say that agents wait

- **Status**: approved
- **Opened**: 2026-11-10

## Problem

The person's request, verbatim: 「AIを主に利用するIDEです。利便性やパフォーマンスなど多様な観点で改善を行ってください。」
Asked which candidates to carry out, they chose 「Dockアイコンに待機数バッジ」.
Asked again before `src/sys.rs` was edited, because the badge needs an
`unsafe` block there, they answered 「追加してよい」.

Operon says how many sessions wait for the person in its toolbar (要対応 N)
and sends a notification when one starts waiting. With another application in
front the toolbar is out of sight and a notification is gone in seconds, so a
person who was away for a minute cannot tell from the Dock whether any agent
is stopped on a question.

## Who feels it, and when

Someone who starts agents and works in a browser or an editor while they run.

## Desired outcome

- The Dock icon shows how many sessions wait for the person, and shows
  nothing when none does.
- The number is the one the toolbar's 要対応 N shows with no filter chosen.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- No new crate: the Objective-C runtime is reached as `src/sys.rs` already
  reaches it for notifications.
- A new `unsafe` block carries a `// SAFETY:` argument and is read by the
  subprocess-safety reviewer.
- No persisted shape changes.

## Systems likely affected

`src/sys.rs` (the AppKit call), `src/app.rs` (the frame).

## Open questions

None.

## Not in scope

- A setting to turn the badge off.
- Bouncing the Dock icon.
