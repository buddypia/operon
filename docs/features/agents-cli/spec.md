# Spec: エージェント間の連携 (Operon agents)

## Behaviour
1. The binary run with the first argument `agents` (`AGENTS_CLI_SUBCOMMAND`) is `run_agents_cli`; verbs are `list [--json]`, `read <ID> [--lines N]`, `send <ID> <text>`, `wait <ID> [--until idle|waiting|either] [--timeout S]`. No verb prints `agents_usage`. The caller names itself by `OPERON_SESSION` (the tmux name) and its current directory.
2. The caller's project is the project of its own session in the snapshot; failing that, the nearest registered project at or above its directory. Only that project's sessions are visible to every verb.
3. `<ID>` is the 8-character id `list` prints; any prefix of at least `AGENTS_ID_PREFIX_MIN` (4) characters (hyphens ignored, case-insensitive) naming exactly one session works. Shorter is exit 2 「セッション ID は 4 文字以上で指定してください。」; none is 「{id} に一致するセッションがありません。」; several is 「{id} は複数のセッションに一致します: …」 listing their ids.
4. `list` prints one row per session: id, state, agent, branch (`-` if none), name, and 「 (自分)」 on the caller's own row. With only the caller present it adds 「このプロジェクトに他のセッションはありません。」. `--json` prints `{"project", "sessions": [{id, name, agent, state, branch, folder, self}]}`. Control characters are stripped from every field.
5. `read` returns the last N lines (default `AGENTS_READ_LINES_DEFAULT` 40, max `AGENTS_READ_LINES_MAX` 200) of the target's terminal with escape sequences and control characters (except newline and tab) removed. N outside 1..200 is a usage error.
6. `send` joins the words, trims, and sends the text plus Enter. It refuses: empty text (usage); over `AGENTS_SEND_MAX_CHARS` (4,000); any control character, judged before trimming (「改行や制御文字を含む内容は送信できません。1 行にまとめてください。」); the caller itself (「自分自身には送信できません。」); a target whose published state is not `idle` or `not-started` (exit 5); and a target whose screen, re-read just before typing, shows `Working` or `AwaitingInput` (exit 5). The fresh screen can only withhold a send, never grant one.
7. `wait` returns success when the target's state matches `--until` (`either` by default: idle or waiting). If the state already matched at the start, it returns only after the state has been left once, or has held `AGENTS_WAIT_SETTLE_SECONDS` (5) seconds. `--timeout` is 1..`AGENTS_WAIT_SECONDS_MAX` (3600), default 600. If the target becomes `failed`/`lost` or disappears: exit 2 「{name} は終了しました。」. Timeout while not matching: exit 4; timeout while it has matched throughout: success.
8. Exit codes: 0 done, 1 tmux failure (`AGENTS_EXIT_TERMINAL`), 2 target/project/ended (`AGENTS_EXIT_TARGET`), 3 Operon not running (`AGENTS_EXIT_NOT_RUNNING`), 4 wait timeout, 5 busy, 64 usage.
9. The state each verb sees is the one the window shows, published to `agents.json` beside the store: `starting`, `working`, `waiting`, `idle`, `not-started`, `running`, `blocked`, `ready`, `queued`, `failed`, `lost`, `checking`, `stopping`; finished and stopped sessions are not listed. The folder is the session's worktree, else its project path.
10. The window checks every `AGENTS_SNAPSHOT_CHECK_SECONDS` (1) whether the snapshot changed and hands a changed one to a writer thread; an unchanged one is rewritten every `AGENTS_SNAPSHOT_HEARTBEAT_SECONDS` (15). The file is removed on exit.
11. Broadcast: 「アイドル中のエージェントへ送信…」 in the palette opens a sheet for the workspace's project (else the selected one). Targets are that project's sessions in the アイドル group (finished a turn, or not started) with a running terminal, listed with checkboxes, none chosen at open, with 「すべて選択」/「選択を解除」. Enter types the trimmed line and Enter into each chosen target that is still idle, then notices 「{n} 件のエージェントに送信しました」. Empty text, or nobody chosen, sends nothing and says nothing.
12. Sessions that become idle while the sheet is open are added unchosen. Escape closes the sheet; with nobody idle it shows 「アイドル中のエージェントはいません」.

## States
- Snapshot missing, unreadable, over `AGENTS_SNAPSHOT_MAX_BYTES` (1 MiB), or older than `AGENTS_SNAPSHOT_STALE_SECONDS` (60): exit 3 「Operon が起動していないため、エージェントの状態を取得できません。」.
- Caller outside any registered project: refused with the target exit code.
- Target tmux name failing `is_safe_tmux_name`: refused (「{id} のターミナル名が不正です。」).
- tmux failure on read or send: exit 1 with tmux's reason.
- Output is written with `write_all` and a closed pipe is ignored.
- A working, waiting, or ended agent is never a broadcast recipient even if it was chosen earlier.

## Invariants
- The command never asks the window to do anything; it reads the snapshot and reaches tmux directly.
- Sends only reach agents that have finished their turn (or not yet started) in the caller's own project; the screen is re-read right before typing.
- The guardrails are not a security boundary: caller identity comes from its environment and directory, and anything running as the same user can already reach tmux.
- The snapshot is written off the frame thread; nothing in the draw path touches the disk.
- The broadcast line and the CLI `send` both type into the terminal as a single line followed by Enter.
- One state word per kind, shared with the window; the CLI does not judge states a second time.

## Code and tests
- Symbols:
  - `src/tmux/control.rs::agents_cli`, `run_agents_cli`, `agents_usage`: dispatch and entry.
  - `src/tmux/control.rs::agents_list`, `agents_read`, `agents_send`, `agents_wait`: the four verbs.
  - `src/tmux/control.rs::find_target`, `caller_project`, `fresh_snapshot`: scoping and staleness.
  - `src/tmux/control.rs::AgentSnapshot`, `AgentEntry`, `agent_state_token`, `read_agent_snapshot`, `agents_snapshot_path`: the published state.
  - `src/tmux/control.rs::AgentHost`, `LiveAgentHost`: the seam for tmux and the clock.
  - `src/app/agent_snapshot.rs::AgentSnapshotWriter`, `track_agent_snapshot`, `finish_agent_snapshot`, `agent_snapshot`: the writer thread and heartbeat.
  - `src/app/broadcast.rs::open_broadcast_sheet`, `broadcast_targets`, `broadcast_recipients`, `send_broadcast`, `ui_broadcast_sheet`: the sheet and the send.
  - `src/config.rs::AGENTS_SNAPSHOT_FILE`, `AGENTS_SEND_MAX_CHARS`, `AGENTS_WAIT_SECONDS_MAX`: limits.
- Tests:
  - `agents_309_list_shows_only_the_callers_project`: project scoping.
  - `agents_309_target_prefix_must_name_exactly_one_session`: id prefixes.
  - `agents_309_send_types_only_into_an_idle_agent_of_the_same_project`: send gate.
  - `agents_309_send_rechecks_the_screen_before_typing`: fresh screen check.
  - `agents_309_send_refuses_empty_long_or_control_text`: text limits.
  - `agents_309_f1_control_characters_at_the_edges_are_refused`: control characters before trimming.
  - `agents_309_read_returns_the_tail_within_its_ceiling`: read limits.
  - `agents_309_wait_returns_when_the_state_arrives_or_times_out`: wait rules.
  - `agents_309_wait_options_are_checked`: wait options.
  - `agents_309_stale_or_missing_snapshot_says_operon_is_not_running`: exit 3.
  - `agents_309_tmux_failure_is_reported_with_its_reason`: exit 1.
  - `agents_309_usage_names_every_verb`: usage text.
  - `agents_309_snapshot_names_each_live_session_with_its_state`: snapshot content.
  - `agents_309_snapshot_is_due_on_change_or_heartbeat_only`: snapshot cadence.
  - `agents_309_writer_applies_in_order_and_removal_is_last`: writer order and removal.
  - `agents_309_caller_is_found_by_session_then_folder`: caller identity.
  - `broadcast_306_sheet_opens_with_nobody_chosen_and_a_row_ticks`: nobody chosen at open.
  - `broadcast_306_send_reaches_only_the_chosen_idle_agents`: chosen recipients only.
  - `broadcast_306_an_agent_idle_after_opening_is_not_chosen`: late idle joins unchosen.
  - `broadcast_306_nobody_chosen_sends_nothing_and_keeps_the_line`: empty choice.
  - `broadcast_282_reaches_only_the_projects_idle_running_agents`: target set.
  - `unstarted_308_counts_as_idle_and_receives_a_broadcast`: not-started is a target.

## Decisions inherited
- Change 309: the window publishes its own state judgement to `agents.json` and the command reads it, so the CLI never disagrees with the window; scope is the caller's project, sends only to finished agents, with a screen re-check; stated as guardrails and not a security boundary.
- Change 282: one line can be sent to the idle agents of a project at once from the command palette.
- Change 306: broadcast reaches only sessions the person ticks, none preselected, after it reached tabs nobody chose.
- Change 308: a not-started agent counts as idle for send, wait and broadcast, and is listed as `not-started`.
- Change 281: the Dock badge counts waiting agents, using the same state reading.
- Change 279: a waiting agent can be answered from its row, with the same typed-line mechanism.
- Change 241: a custom command may not contain a line break; control characters are refused.
- Change 258: an agent starting in a project is told its knowledge exists, through the session-start hook.
