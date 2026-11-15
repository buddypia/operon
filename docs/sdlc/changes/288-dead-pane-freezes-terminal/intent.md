# Intent: Dead panes freeze the terminal instead of keeping a shell or restoring killed sessions

- **Status**: approved
- **Opened**: 2026-11-12

## Problem

The person reported:

> Pane is deadが出ているけど、これじゃなくてターミナルのままにしたい。そもそもなんで終了されるんだ？強制終了なら自動でセッションを復元するようにしたい。

Today, every managed terminal runs its CLI or login shell as the top-level process of a pane configured to stay open after exit. When that process exits normally (for example, an agent CLI finishes and exits with status 0, or a shell exits), there is no parent shell underneath it: the pane freezes, prints `Pane is dead (...)` at the bottom, and stops accepting keyboard input even though the tab is still open in front of the person. Conversely, when a running session is force-killed or crashes while Operon is open (`SIGKILL`, an external `kill-session`, or a non-zero exit), Operon only auto-restarts sessions that were found gone on the very first look after Operon itself launched; any forced termination during an active run leaves the session dead on screen until the person notices and manually resumes it.

## Who feels it, and when

Anyone working in Operon's session tabs whenever an agent CLI (`agy`, `claude`, `codex`, or a custom agent) exits cleanly at the end of a run — including tabs already sitting in `Pane is dead (status 0, ...)` from earlier runs — and whenever an external process kill, OOM kill, crash, or lost terminal server terminates an active session while Operon is running.

## Desired outcome

- `Pane is dead (...)` never appears in a managed terminal pane.
- When an interactive session finishes cleanly (exit status 0) without having been stopped by the person, without waiting queued dependents, and outside a setup or run script, the tab stays live as an interactive login shell in the same working folder with the finished output preserved in scrollback above the prompt, ready to run commands or launch another CLI. Open tabs already left in that clean-exited dead state by an older build also become live shells automatically when Operon opens.
- When an active session is force-terminated or crashes while Operon is running (a signal death, a non-zero exit, or a disappeared terminal, not caused by the person stopping it), Operon automatically restores the session in the same tab — reopening the saved CLI conversation when one exists on disk, or reopening the folder's shell for a plain terminal — and briefly tells the person in Japanese, while stopping after one immediate repeat crash so a broken command never loops forever.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.
- Persisted store schema remains unchanged (`STORE_SCHEMA_VERSION` stays 5); no `paused` surface is edited.
- The largest module (`src/app.rs`) sits near its `propose` line ceiling (`10,000` lines), so new behaviour belongs in focused submodules rather than growing `src/app.rs`.

## Systems likely affected

- `src/tmux.rs` — pane exit formatting and respawning a plain shell in a dead pane without erasing scrollback.
- `src/app/relaunch.rs` and `src/app.rs` — transitioning clean-exited interactive sessions into live shells, reopening legacy clean-exited open tabs on startup, and restoring force-terminated sessions mid-run with a cooldown guard.
- `src/config.rs` and `src/i18n_tables.rs` — cooldown constant and Japanese/English/Korean toast strings.
- `README.md`, `README.ja.md`, `README.ko.md` — three-language user documentation of clean exit and forced-termination recovery.

## Open questions

- **What if a restored session crashes again right away?** Answered from the codebase's bounded-retry discipline: guard automatic restoration with a per-session cooldown (`15` seconds) so one forced kill or transient crash recovers automatically, while a command that immediately fails again settles in `Failed` or `Lost` for the person to inspect.
- **What happens to `.operon/setup.sh`, `.operon/run.sh`, explicit Stop/Cancel, and sessions that another queued session depends on?** Answered from existing contracts: setup sessions report their setup banner, run scripts represent a one-shot command run, explicit cancellations honour the cancellation sidecar, and dependency parents must reach `Exited` so `start_ready_queued_sessions` launches their dependents. None of those four turns into an idle shell.

## Not in scope

- Changing the persisted store schema or adding new persisted fields to `Session`.
- Synthesising or guessing a CLI conversation ID when an agent died before writing any transcript to disk (consistent with `.claude/rules/transcripts.md`, a CLI with no written transcript stays unresumed rather than re-sending its initial prompt).
