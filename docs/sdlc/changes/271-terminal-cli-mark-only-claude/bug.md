# Bug: a terminal running Codex or Antigravity keeps the terminal mark

Reported 2026-11-09, in the person's words:

> セッションをターミナルとして起動してからClaude Codeは正常にタブ名にアイコンがターミナルアイコンから切り替わるけど、他AntigravityやCodexは切り替わらない。

A session opened as a plain terminal, in which the person then runs `claude`,
shows the Claude mark on its tab (change 270). Running `codex` or `agy` in the
same kind of terminal leaves the terminal mark on the tab.

## Cause

The only path that changes a terminal's kind is `adopt_native_identity`
(`src/app.rs`), and it runs only when a CLI's status hook posts an event to
Operon's socket. Measured on 2026-11-09 in a scratch tmux server, with the
hook scripts Operon installed pointed at a scratch listener:

- **Codex 0.167** runs its hooks inside the shared `codex app-server daemon`,
  not in the pane. The hook process sees none of the pane's variables
  (`OPERON_SESSION`, `OPERON_HOOK_SOCKET`, `OPERON_LAUNCH_TOKEN` and
  `OPERON_HOOK_ENDPOINT` all unset, recorded by an instrumented copy of
  `codex-hook.sh`), so the script stops at its
  `[ -z "$OPERON_HOOK_SOCKET" ] || [ -z "$OPERON_SESSION" ]` line and nothing
  reaches Operon: not at start, not after a prompt. The same TUI started with
  `codex --no-daemon` posted `SessionStart`, `UserPromptSubmit` and `Stop` with
  the pane's session name. Codex has no environment variable that turns the
  daemon off, and a person typing `codex` into a terminal does not pass the
  flag.
- **Antigravity CLI 1.5.3** posts nothing when it starts. Its first event is
  `PreInvocation`, carrying `conversationId`, when the first prompt is sent.
- **Claude Code** posts `SessionStart` with `session_id` as it starts, which is
  why its mark appears at once.

So the kind of a terminal follows a CLI only when that CLI's hook happens to
carry the pane's identity and fires at launch, which today is Claude alone.

## Fix

Read the CLI from the terminal itself rather than wait for its hook. The
session poll already visits every live session every two seconds on a
background thread. For a session that began as a terminal (the same
`terminal_provenance` change 270 uses), it also reads the pane's terminal device
from tmux and lists that device's foreground process group with `/bin/ps`. A
process in it whose executable is named `claude`, `codex` or `agy`
(`CliProvider::executable`), and whose own parent chain holds no other such
process, is the CLI the person is running. Measured in the scratch server: the
foreground group shows `/Users/…/.local/bin/claude`, `…/bin/codex` (its MCP
servers sit in groups of their own) and `/Users/…/.local/bin/agy` under a
wrapper shell; `pane_current_command` alone does not, since it reads
`2.1.319` for Claude and `bash` for the wrapped `agy`.

When that CLI differs from the session's kind, the session takes it, exactly
as a hook report would: kind, emptied description, provenance kept. The
conversation ID recorded for the previous kind is dropped, because it belongs
to another CLI; the new one arrives with that CLI's first hook event, as now.

Not in this change: Codex sessions launched by Operon also lose their status
hooks to the daemon (same cause); that is a launch-command change with its own
evidence, recorded as a follow-up.

No persisted shape, string, or screen layout changes.
