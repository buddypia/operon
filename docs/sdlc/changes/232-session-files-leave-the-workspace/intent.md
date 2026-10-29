# Intent: Opening a session file leaves the session workspace

- **Status**: draft
- **Opened**: 2026-10-29

## Problem

Opening a file from a session moves the person to the project page. Switching
conversations and comparing a conversation with files requires navigating away.

## Who feels it, and when

The person working across coding-agent sessions, each time they click a session
file or compare conversations and files.

## Desired outcome

Files open as new central tabs in the session workspace. Sessions also occupy
central tabs, with projects alone in the left column. Tabs switch like VSCode or
Chrome and move between groups split into left/right or top/bottom halves.
Closing a session tab only hides its view and never stops the agent.

The user explicitly requests “セッションページの中央ウィンドウに新しいタブとして表示されるべき”
and “左プロジェクトのみ表示されてセッションはセッションタブを中央ウィンドウに追加”.

## Constraints this change inherits

- macOS; egui 0.31; local-first; Japanese UI strings.
- Preserve unsaved editor buffers and existing file read/save protections.
- No session-store format, process lifecycle, or dependency changes.

## Systems likely affected

`src/app.rs`, `src/app/screens.rs`, and `src/ui/session_tree.rs`.

## Open questions

None: the person supplied the target layout and interactions.

## Not in scope

Detached native macOS windows and persistence of editor group arrangement.
