# Intent: Application logo not displayed in macOS notifications

- **Status**: in-progress
- **Opened**: 2026-09-15

## Problem

Operon delivers system notifications on turn completion, input requests, and session status changes.
Currently, `send_macos_notification` in `src/sys.rs` implements notification dispatch by shelling out to `/usr/bin/osascript` with:
`display notification "<body\>" with title "Operon"`

Because macOS Notification Center (`usernoted`) attributes notifications to the executing process (`/usr/bin/osascript`), the notification is owned by `com.apple.ScriptEditor2` (AppleScript / Script Editor) rather than Operon (`local.operon`).
AppleScript's `display notification` command provides no mechanism to specify a custom application logo or icon, causing macOS notifications to display the generic Script Editor icon (quill and paper) instead of Operon's branded application logo (`Operon.icns`).

## Who feels it, and when

Any user who has enabled notifications in Operon (`ターンの完了・入力待ち・終了を通知する`) and receives notifications when a coding agent session finishes, requests input, or fails.
The notification appears with the Script Editor icon rather than the Operon logo.

## Desired outcome

1. When running as the bundled application (`/Applications/Operon.app`), Operon sends notifications via macOS's native `UserNotifications` framework (`UNUserNotificationCenter`) using its own bundle identity (`local.operon`).
2. macOS Notification Center attributes the notification to Operon and displays the Operon logo (`Operon.icns`) in the notification banner.
3. When running outside an application bundle (such as in headless CLI mode or in `cargo test`), notification dispatch safely and gracefully falls back to the existing `osascript` implementation so tests and development workflows never crash.
4. Unit tests in `src/tests.rs` cover the notification path and verify the escaping and fallback logic.

## Systems likely affected

- `src/sys.rs`: `send_macos_notification`, adding native `UNUserNotificationCenter` dispatch when inside a macOS app bundle, with fallback to `osascript`.
- `src/app/screens.rs`: triggering notification authorization request when notifications are enabled.
- `src/tests.rs`: tests for notification dispatch and fallback.
