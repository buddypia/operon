# Intent: Missing terminal panes loop output-read errors instead of recovering sessions or opening a terminal

- **Status**: approved
- **Opened**: 2026-11-17

## Problem

The person reported:

> ターミナル出力を読み取れませんでした: can't find pane: operon-2122260b とターミナル出力を待っています。と出てくるがこれバグっぽい。クラッシュなどならできれば復元できる様にした方が良いが、セッションの復元が難しければターミナルのままにしても良い。

Today, three gaps combine to leave a person staring at a repeating error banner and an indefinitely blank pane when a session's underlying terminal is missing:

1. **Repeated missing-pane error loop**: When an open terminal pane has no output cached yet and its underlying terminal no longer exists (for example, a session already marked lost or failed from a crash, a queued session before startup, or an active pane that disappeared between polls), viewing that tab repeatedly triggers background output reads every redraw frame. Each failed read clears no condition and raises the persistent notice 「ターミナル出力を読み取れませんでした: can't find pane: …」, while the pane continues to display 「ターミナル出力を待っています…」 indefinitely.
2. **Incomplete automatic conversation recovery**: CLI sessions whose conversation ID was not saved yet (such as terminals where the person launched a CLI interactively, or CLI launches tracked only by local conversation transcripts/registries) cannot be automatically resumed after a crash or disappeared terminal, even when the local transcript already exists on disk.
3. **No open-tab fallback or reload recovery for lost/failed tabs**: When an open workspace tab's terminal disappears or crashes and no resumable conversation transcript can be found, or when Operon starts up with an open tab already recorded as lost or failed from a prior crash or tmux server loss, the tab remains without a working terminal instead of reopening as a live shell in the session's folder.

## Who feels it, and when

Anyone opening or viewing a session tab in Operon when:
- A session's terminal pane crashed or disappeared (such as after a tmux server crash, OS restart, or killed pane) and the app tries to draw or re-open the tab.
- A terminal tab had a CLI (`claude`, `codex`, or `agy`) started inside it or started without a saved conversation ID, then crashed or lost its terminal.
- A CLI session failed or lost its terminal before writing any resumable conversation transcript while its tab is open in the workspace.
- A queued or stopped/failed session tab with no captured output is viewed in the terminal workspace.

## Desired outcome

- Viewing any session tab whose underlying terminal is absent never enters a per-frame error-banner loop with `can't find pane`, and an active session whose pane is found missing on output/input checks reconciles cleanly through the session recovery lifecycle instead of spamming read errors.
- Sessions waiting to launch in the queue or already ended without captured output state honestly why there is no terminal output yet rather than claiming 「ターミナル出力を待っています…」 indefinitely, while an explicit user-triggered refresh (「今すぐ同期」) still surfaces read errors if the terminal cannot be read.
- When an interactive session disappears or crashes (either while Operon is running or found already lost or failed in an open workspace tab on startup), Operon recovers and resumes its CLI conversation from local transcripts/registries even when the session record lacked a saved conversation ID; if the conversation still cannot be recovered and the session has an open workspace tab (with no queued dependents and outside setup/run scripts), Operon automatically reopens the tab in place as a live plain terminal shell in that session's folder and briefly tells the person in Japanese, bounded by the existing repeat-crash cooldown.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.37 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.
- The persisted store schema remains unchanged (`STORE_SCHEMA_VERSION` stays 5); no `paused` surface is touched.
- Work in the draw path remains non-blocking, and existing module line-count bounds (`10,000` lines max) are respected.

## Systems likely affected

- `src/app.rs` — transient missing-capture tracking, background output/input/resize result dispatch, startup lost-tab queueing, and adopting foreground CLI launch timing.
- `src/app/terminal_output.rs` — empty-pane placeholder state, initial output request guard, missing-pane reconciliation, and explicit refresh handling.
- `src/app/relaunch.rs` — recovering CLI conversations without a stored conversation ID, falling back unrestorable open tabs to plain terminal shells, and recovering open `Lost`/`Failed` tabs on startup.
- `src/app/screens.rs` — pane empty placeholder display, guarded initial output request, and explicit sync call.
- `src/i18n_tables.rs` — Japanese/English/Korean translations for new user-facing strings.
- `README.md`, `README.ja.md`, `README.ko.md` — three-language user documentation of recovery and plain-terminal fallback.

## Open questions

- None unresolved before `spec.md`: bounded repeat-crash cooldown (`AUTO_RESTORE_COOLDOWN` = 15s), preservation of off-tab closed `Lost`/`Failed` history records, and preservation of queued-dependency/setup/run-script/cancellation invariants follow Operon's established session-lifecycle contracts.

## Not in scope

- Changing the persisted store schema or adding new persisted fields.
- Re-sending the initial agent prompt when a CLI crashed before writing a conversation transcript.
- Automatically reopening closed historical lost or failed sessions that do not have an open workspace tab.
