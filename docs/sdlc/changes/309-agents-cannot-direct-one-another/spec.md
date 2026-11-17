# Spec: `Operon agents` — an agent lists, reads, instructs, and waits for its project's other agents

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. While Operon runs it keeps `agents.json` beside the local index: every
   session that is not finished or stopped, each with its id, project, name,
   agent, branch, folder, tmux name, and state token, plus every project's id,
   name, and path and the time it was written. It is rewritten when that
   content changes (checked at most once a second) and otherwise every 15
   seconds, never on the frame's thread. —
   `agents_309_snapshot_names_each_live_session_with_its_state`,
   `agents_309_snapshot_is_due_on_change_or_heartbeat_only`
2. One writer thread applies writes and the removal in the order they were
   asked for; quitting Operon removes the file after every earlier write, so a
   write still in flight cannot bring it back. —
   `agents_309_writer_applies_in_order_and_removal_is_last`
3. The caller is the session whose tmux name is in `OPERON_SESSION`. When that
   variable is unset, or names no session in the snapshot, the caller's project
   is the nearest registered project folder at or above the current directory;
   the caller's own terminal is still recognised by that tmux name. Neither →
   exit 2. — `agents_309_caller_is_found_by_session_then_folder`
4. A missing or unreadable snapshot, one larger than 1 MiB, or one older than
   60 seconds means Operon is not running: every verb exits 3 and says so. —
   `agents_309_stale_or_missing_snapshot_says_operon_is_not_running`
5. `list [--json]` shows only the caller's project's sessions, the caller's own
   marked. — `agents_309_list_shows_only_the_callers_project`
6. A target is an id prefix of at least 4 characters naming exactly one session
   of the caller's project; shorter, unknown, ambiguous, or another project's →
   exit 2 naming why. A snapshot tmux name that fails `is_safe_tmux_name` is
   refused before tmux is run. —
   `agents_309_target_prefix_must_name_exactly_one_session`
7. `send <id> <text…>` types the text and Enter into the target only when the
   snapshot says `idle` **and** a capture of the target's visible screen taken
   just before typing does not read as working or as a prompt; otherwise exit
   5 and nothing is typed. Never into the caller itself (exit 2). —
   `agents_309_send_types_only_into_an_idle_agent_of_the_same_project`,
   `agents_309_send_rechecks_the_screen_before_typing`
8. `send` refuses text that is empty, longer than 4 000 characters, or holds a
   control character (newline, tab, escape, …) with exit 64; nothing is typed.
   — `agents_309_send_refuses_empty_long_or_control_text`
9. `read <id> [--lines N]` prints the last N lines (default 40, 1–200) of the
   target's screen and scrollback, without colour codes; a bad N is exit 64. —
   `agents_309_read_returns_the_tail_within_its_ceiling`
10. A tmux failure (missing binary, gone pane) is exit 1 with tmux's own words.
    — `agents_309_tmux_failure_is_reported_with_its_reason`
11. `wait <id> [--until idle|waiting|either] [--timeout S]` returns 0 as soon as
    the target's state is the one asked for (default `either`), and 4 at the
    timeout (default 600 s, 1–3 600 s); a target that leaves the snapshot or
    ends returns 2; a bad option is exit 64. —
    `agents_309_wait_returns_when_the_state_arrives_or_times_out`,
    `agents_309_wait_options_are_checked`
12. No arguments or `help` prints the usage (exit 0); an unknown verb prints it
    with exit 64. — `agents_309_usage_names_every_verb`

## Behaviour

Run from an agent's terminal as `"$OPERON_EXECUTABLE" agents …` (Operon
already puts its own path in every managed session's environment) or
`/Applications/Operon.app/Contents/MacOS/Operon agents …`. It opens no window
and takes no lock.

`list` prints one row per session:
`<id8>  <state>  <agent>  <branch or ->  <name>` with ` (自分)` after the
caller's own row; with nobody: 「このプロジェクトに他のセッションはありません。」.
`--json` prints `{"project":…, "sessions":[{id, name, agent, state, branch,
folder, self}]}`.

State tokens: `working`, `waiting` (blocked on a prompt), `idle` (turn
finished), `running` (a plain terminal), `starting`, `queued`, `blocked`,
`failed`, `lost`, `stopping`, `checking`, `ready`.

Messages (stderr):
- 「Operon が起動していないため、エージェントの状態を取得できません。」 (exit 3)
- 「{path} は Operon に登録されたプロジェクトではありません。」 (exit 2)
- 「{id} に一致するセッションがありません。」 / 「{id} は複数のセッションに一致します: {list}」 / 「セッション ID は 4 文字以上で指定してください。」 (exit 2)
- 「{name} は作業中のため送信しません（状態: {state}）。」 (exit 5) — send to a non-idle target, or one whose screen shows a running turn or a prompt
- 「改行や制御文字を含む内容は送信できません。1 行にまとめてください。」 (exit 64)
- 「自分自身には送信できません。」 (exit 2)
- 「送信する内容が空です。」 / 「送信する内容が長すぎます（最大 {max} 文字）。」 (exit 64)
- 「{name} に送信しました。」 (stdout, exit 0)
- wait: 「{name} は {state} になりました。」 (exit 0); 「{secs} 秒待ちましたが {name} は {state} のままです。」 (exit 4); 「{name} は終了しました。」 (exit 2)
- tmux failure: 「ターミナルを操作できませんでした: {error}」 (exit 1)

Nothing changes in the window.

## Design

Placement: **NEW_IN_MODULE** — managed terminals are `src/tmux.rs`'s concern
(its map line: session lifecycle; `src/tmux/hooks.rs` already lets the CLIs
talk back). `git grep -n "OPERON_SESSION\|send_broadcast\|knowledge_cli"`:
broadcast (282) types into idle agents but only from the person's sheet; the
knowledge CLI (157) is the precedent for a windowless subcommand.

- `src/tmux/control.rs` (new submodule): `AgentSnapshot`, `AgentEntry`,
  `ProjectEntry` (serde); `agents_cli(arguments, caller_env, directory,
  snapshot, now, terminal: &mut dyn AgentTerminal) -> CliOutcome`;
  `agents_wait(…)` with injected loader and sleeper; `AgentTerminal` trait
  (`visible_screen`, `tail`, `send`) with the tmux implementation
  (`tmux_capture_screen` for the recheck, judged by the existing
  `detect_agent_activity`; `capture-pane -p -J -S -N` through
  `run_command_with_output_limit` for `read`; `tmux_send_input`);
  `run_agents_cli` entry. tmux is found on the caller's own `PATH` (the
  dispatch runs before `configure_command_path`, like `knowledge`); when it is
  missing the exit-1 message carries the OS error.
- `src/app/agent_snapshot.rs`: `OperonApp::agent_snapshot()` from
  `self.store` and `session_status_kind_for`; `track_agent_snapshot(now)`
  called from `logic`, at most once a second, comparing the serialised content
  (without the time) to the last written and sending a due write over a
  channel to one lazily started writer thread (`write_file_atomically`);
  `on_exit` sends the removal, drops the sender, and joins the thread, so the
  removal is the last thing done to the file. The file lives in the
  per-user application-support directory beside the store, like the hook
  socket.
- `src/config.rs`: `AGENTS_CLI_SUBCOMMAND`, `AGENTS_SNAPSHOT_FILE`, the
  interval, heartbeat, staleness, line, text, and timeout ceilings.
- `src/main.rs`: dispatch beside `knowledge`.
- `src/i18n_tables.rs`: every string above, every language table.
- No persisted-store change: `agents.json` is derived, rewritten whole, never
  read by the app.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | No | Nothing is drawn. |
| Icons | No | Nothing is drawn. |
| Identifier SSOT | Yes | Subcommand, file name, ceilings live in `src/config.rs`; the round trip (app writes → CLI reads the same file name and tokens) is what `agents_309_snapshot_names_each_live_session_with_its_state` tests. |
| Durability | Yes, lightly | `write_file_atomically`; the file is derived and losing it only makes the CLI say Operon is not running. Not the store; no schema bump. |
| Subprocess safety | Yes | Two tmux calls through `run_command_with_timeout` / `run_command_with_output_limit`; target is a tmux name from the snapshot, checked with `is_safe_tmux_name`; text is passed `-l --` through `tmux_send_input_arguments`. No new launch path. |
| Documentation | Yes | `docs/MANUAL.md`, `.ko.md`, `.ja.md` and the three READMEs gain the section together. |
| Local-first | Yes | Local file and local tmux only; no `unsafe`. |
| Budgets | Yes | Snapshot read capped at 1 MiB; read capped at 200 lines and 256 KiB of capture; send text 4 000 characters; wait 3 600 s, polling once a second. |

## Flagged concerns

- **An agent instructing another agent is a way to reach that agent's
  permissions.** Answered: only sessions of the same project (the person
  already gave both agents that folder), only an agent that has finished its
  turn (the same rule broadcast and diff notes use), and every send lands in
  the target's visible terminal where the person sees it. A sandboxed agent
  that can already run `tmux send-keys` gains nothing new.

## Acceptance

- `cargo test --locked agents_309_` passes; `make q.fast` passes.
- In the running app, from a Claude Code session: `"$OPERON_EXECUTABLE" agents
  list` names the project's other session; `send` to it while it is idle types
  the line; `wait` returns when it finishes.

## Rejected alternatives

- A request socket into the app (Herdr's model): more moving parts in the
  draw-adjacent thread; the snapshot plus direct tmux needs no app round trip.
- The CLI judging state from the screen on its own: the app already merges
  hook reports with the screen. The CLI never overrides the app's state; its
  screen recheck (R7) can only withhold a send.
- Spawning sessions now: needs the app to create records; separate change.
