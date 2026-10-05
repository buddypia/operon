# Bugfix: tmux socket connect error prevents session deletion and triggers resize errors

- **Route**: `bugfix` — no intent and no spec by design. The reproduction below
  is the intent, and inventing a spec for it is the ceremony that gets pipelines
  abandoned.
- **Skill**: `.claude/skills/root-cause/SKILL.md`

## The cause, in one sentence

When a tmux server is dead or its UNIX socket file does not exist, tmux outputs
`error connecting to <socket> (<reason>)` (e.g. `error connecting to /private/tmp/tmux-306374814/default (No such file or directory)`),
which `tmux_error_state` did not recognize as `TmuxState::Gone`, leaving the session
in `Active` state where background terminal resize continuously failed with an error banner
and session deletion failed because both `stop_session` and `close_completed_terminal`
treated the gone server as an `Unknown` error.

## The observation that proves it

`tmux_error_state("error connecting to /private/tmp/tmux-306374814/default (No such file or directory)")`
returned `TmuxState::Unknown` instead of `TmuxState::Gone`.

## The test, and the message it printed

`recognizes_gone_tmux_server_errors`, watched failing before the fix:

```
thread 'tests::recognizes_gone_tmux_server_errors' panicked at src/tests.rs:11051:5:
assertion `left == right` failed
  left: Unknown
 right: Gone
```

And `terminal_resize_failure_from_missing_tmux_socket_transitions_session_to_lost_without_error_banner`, watched failing before the fix:

```
thread 'tests::terminal_resize_failure_from_missing_tmux_socket_transitions_session_to_lost_without_error_banner' panicked at src/tests.rs:11094:5:
assertion `left == right` failed
  left: Some("ターミナルのサイズを変更できませんでした: error connecting to /private/tmp/tmux-306374814/default (No such file or directory)")
 right: None
```

And `deleting_session_with_missing_tmux_socket_removes_session_record`, watched failing before the fix:

```
thread 'tests::deleting_session_with_missing_tmux_socket_removes_session_record' panicked at src/tests.rs:11129:5:
assertion failed: app.store.sessions.is_empty()
```

## What changed

1. In `src/tmux.rs` (`tmux_error_state`): Added matching for `"error connecting to"`, `"failed to connect to"`, and `"connection refused"` to classify socket connection failures as `TmuxState::Gone`.
2. In `src/app.rs` (`TerminalResized`): When terminal resize fails with a gone tmux state, suppress the `notice` error banner and transition an active/starting session to `SessionStatus::Lost`.
3. In `src/app.rs` (`SessionStopped`): When stopping a session fails but the error indicates the tmux server is `Gone` and `remove_after_close` is set, mark status as `Cancelled` and proceed to remove the session record.
4. In `src/app.rs` (`TerminalClosed`): When closing a completed terminal fails but the error indicates the tmux server is `Gone` and `remove_after_close` is set, remove the session record.

## Guard, and whether it was watched failing

- `recognizes_gone_tmux_server_errors`
- `terminal_resize_failure_from_missing_tmux_socket_transitions_session_to_lost_without_error_banner`
- `deleting_session_with_missing_tmux_socket_removes_session_record`
- `closing_completed_terminal_with_missing_tmux_socket_removes_session_record`

All watched failing on the unedited tree and passing after the fix.
Recorded as lesson 062 in `docs/sdlc/lessons.md`.

## Ruled out, and not done

- Forcing session removal in `remove_session_record` regardless of state without checking tmux: ruled out because removing records for genuinely running active sessions would create untracked orphan daemon processes. The fix properly reconciles `Gone` states so removal operates on confirmed inactive sessions.
