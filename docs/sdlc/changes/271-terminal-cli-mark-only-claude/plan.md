# Plan: a terminal takes the mark of the CLI running in it, for every CLI

- **Spec**: `./bug.md` (bugfix route: the bug report is the intent)
- **Approved**: 2026-11-09, independent evaluator
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| src/tmux.rs | `tmux_foreground_cli(name) -> Option<CliProvider>`: `tmux list-panes -t <name> -F '#{pane_tty}'` (5 s timeout), the first pane's device with `/dev/` stripped and refused unless ASCII alphanumeric, then `/bin/ps -t <device> -o pid=,ppid=,pgid=,tpgid=,comm=` (5 s timeout, the existing output ceiling of `run_command_with_timeout`). `parse_foreground_cli(&str) -> Option<CliProvider>`: keep rows whose `pgid == tpgid`, name each by the last path component of `comm` against `CliProvider::executable()`, drop a named process that has a named ancestor among those rows, and answer only when what is left names one CLI. |
| src/app.rs | `request_session_poll` carries `terminal_provenance(session).is_some()` per session; the worker calls `tmux_foreground_cli` for such a session when its observation is `Alive`, and `BackgroundResult::SessionPoll` gains `clis: Vec<(Uuid, CliProvider)>`. Its handler calls a new `adopt_terminal_cli(id, provider)` per entry next to `apply_session_poll` (whose signature stays). The kind switch in `adopt_native_identity` moves into a shared `take_cli_kind(&mut Session, CliProvider) -> bool`; `adopt_terminal_cli` uses it, clears `native_session_id` and `native_session_path` when the kind moved, and persists with the existing rollback. It checks before cloning the store, so a poll that changes nothing costs no clone. |
| src/tests.rs | `the_cli_in_a_terminals_foreground_is_read_from_its_processes` (parser, fixtures from the measured `ps` output), `a_terminal_takes_the_kind_of_the_cli_seen_running_in_it` (app), `the_foreground_cli_of_a_live_pane_is_read_through_tmux_and_ps` (live tmux on the suite server, a symlink named `codex` to `/bin/sleep`). |
| docs/sdlc/changes/271-terminal-cli-mark-only-claude/ | `bug.md`, `state.yaml`, this plan, `mutations.py`, `verification.yaml`, `approvals.log`. |

## Order of work

1. The three tests, against stub `tmux_foreground_cli` / `parse_foreground_cli` returning `None` and an `adopt_terminal_cli` that does nothing; watch each fail.
2. `parse_foreground_cli`, then `tmux_foreground_cli`.
3. `take_cli_kind`, `adopt_terminal_cli`, and the poll wiring.
4. `make q.fast`; the three new tests and the change-270 neighbours (`a_terminal_takes_the_kind_of_the_cli_running_in_it`, `a_terminal_running_claude_receives_review_notes`, `sessions_running_at_the_quit_come_back_when_their_terminal_is_gone`) by name; `mutations.py`.
5. Verification record; CI on the branch; land; package from main and swap the installed app.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A CLI run by another CLI (Claude's tool runs `codex exec`) is taken for the person's | The mark flips to Codex while Claude is in front | The outermost-match rule; the parser test feeds Claude with a `codex` child in the foreground group and expects Claude. |
| Two CLIs in one pipeline | The mark guesses | Two outermost CLIs answer `None`; parser test. |
| A tty name reaching `ps` as an option or a path | Wrong process list, or an argument injection | Only an ASCII-alphanumeric device name is passed, after `/dev/`; parser of the tmux line tested through the live test, the refusal by a mutation. |
| A stale poll result after the person switched CLIs | The kind flips back for one poll and the new CLI's ID is dropped | Self-heals on the next poll (two seconds) and the next hook event; accepted, written here. |
| Extra processes per poll | CPU every two seconds | Only sessions that began as terminals and are alive: two short processes each, on the existing background thread; nothing in the draw path. |
| A terminal that ran Codex is no longer started again as a shell after tmux is lost | No automatic shell after a reboot; its 「再試行」 stays | The same rule change 270 gave a CLI session without a conversation ID (R6). Accepted and written here rather than special-cased. |
| The conversation ID of the previous CLI kept under the new kind | A resume opens another CLI's conversation | `adopt_terminal_cli` clears ID and path when the kind moves; the app test checks it. |

## Proof of completion

- The three new tests fail before the change (the stubs answer `None` / do nothing) and pass after.
- `make q.fast` passes; the change-270 neighbours pass; `mutations.py` catches every mutation; CI on the branch passes.
- In the installed app: a new terminal session, `codex` typed in it, shows the Codex mark on its tab within about two seconds; the same for `agy` and Antigravity's mark; `claude` still shows Claude's.

## Departures from the plan

