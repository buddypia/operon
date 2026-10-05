# Spec: agent state from the CLIs' own hooks

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Every agent session Operon launches carries three environment variables in
   its tmux session: the session's tmux name, a per-launch token, and the path
   of an endpoint file. A respawn issues a new token before the pane restarts.
2. Operon listens on a Unix socket, mode 0600, inside its data directory, and
   accepts an HTTP `POST /hook/<source>` whose body is the JSON the CLI piped to
   its hook and whose headers carry the session name, the launch token, and (for
   Antigravity) the event name. It answers 204 to everything well-formed and
   closes; a body over 64 KiB is answered 413 and dropped; a connection idle for
   two seconds is dropped.
3. The endpoint file names the socket path in `KEY=value` form so a hook running
   after Operon restarted reaches the new socket without the CLI being relaunched.
4. One managed POSIX shell script per CLI, written under the data directory with
   mode 0755 and rewritten only when its content changed, reads all of stdin
   first, prints the CLI's expected stdout before anything else, sources the
   endpoint file, posts with `curl --unix-socket` under a 1.5-second ceiling, and
   exits 0 on every path.
5. Installing into Claude Code appends one entry per event to `hooks` in
   `~/.claude/settings.json` for `SessionStart`, `UserPromptSubmit`, `Stop`,
   `StopFailure`, `PreToolUse`, `PostToolUse`, `PostToolUseFailure`,
   `PermissionRequest`, and `PostCompact`, never `PreCompact`. Every entry not
   naming the managed script is preserved byte-for-byte in meaning; entries
   naming it are removed before the fresh set is appended; the file is written
   through a temporary file and rename only when the parsed document changed,
   keeps its mode, and is written at the target of a symlink rather than over it.
6. Installing into Codex appends one entry per event to `~/.codex/hooks.json`
   for `SessionStart`, `UserPromptSubmit`, `PreToolUse`, `PermissionRequest`,
   `PostToolUse`, and `Stop`, with the same preservation rules, ensures
   `hooks = true` is set at the top level of `~/.codex/config.toml`, and writes a
   `[hooks.state."<path>:<event_label>:<group>:<handler>"]` table per entry
   whose `trusted_hash` is `sha256:` plus the hex digest of the canonical JSON
   `{"event_name","hooks":[{"async","command","timeout","type"}]}` with keys
   sorted at every level and no whitespace.
7. Installing into Antigravity sets the `operon-status` bundle in
   `~/.gemini/config/hooks.json` to entries for `PreInvocation`,
   `PostInvocation`, `Stop`, `PreToolUse`, and `PostToolUse`, where the event
   name reaches the script through its environment, the script answers
   `{"decision":""}` to `Stop`, `{"decision":"ask"}` to `PreToolUse`, and `{}`
   otherwise, and other bundles are untouched.
8. Removing undoes exactly what installing added, in all three files, and never
   fails because the script is already gone.
9. Events normalise into one of `working`, `waiting`, `done` per the table in
   **Design**, or are ignored. An event whose launch token differs from the one
   Operon recorded for the session is ignored. An event for an unknown session
   name is ignored.
10. A `done` that arrives from a session boundary (Claude `SessionStart` with
    source `startup`, `resume`, or `clear`; `PostCompact` with trigger
    `manual`) is shown as idle and never notified.
11. A state reported by a hook is authoritative for thirty minutes after its
    last event while the pane is alive; after that, or before any event, the
    screen heuristic decides, exactly as today.
12. A `Stop` or `StopFailure` notifies "finished" and a `waiting` notifies
    "needs input" through the existing notification path, at most once per
    state change, only when notifications are enabled.
13. The `session_id` and `transcript_path` (Antigravity: `conversationId` and
    `transcriptPath`) in a lead event become the session's native resume
    identity when the session has none, and replace it when Claude reports a
    different id after `/clear`.
14. Settings shows the toggle and, per CLI, whether its hooks are installed, not
    installed, or failed with the error. The toggle defaults to on, installs on
    the next start for every CLI found on `PATH`, and removes on being turned
    off. The toggle is persisted beside the hook scripts, not in the store.
15. No new dependency, no new `unsafe`, no new `Command::new` site.

## Behaviour

- **Launch.** Nothing visible changes. The tmux session gets the three
  variables; the agent's own hooks start posting.
- **Working.** The chip reads WORKING with hint 「応答を生成中」, or 「{tool} を実行中」
  when the last event named a tool (Claude `tool_name`, Codex `tool_name` or
  `name`, Antigravity `toolCall.name`).
- **Waiting.** WAITING, hint 「ターミナルを開いて回答してください」, notification
  「{agent} が入力を待っています: {session}」.
- **Done.** IDLE, hint 「次の依頼を待っています」, notification
  「{agent} が完了しました: {session}」 — unless the idle came from a session
  boundary, in which case no notification.
- **Restart.** After Operon restarts, every session shows what the screen says
  until its next hook event, then follows the hooks.
- **Hooks not installed** (CLI missing, toggle off, or a settings file that
  could not be parsed): the screen heuristic runs alone; Settings says so per
  CLI with the reason.
- **Settings.** In the 「ローカルデータ」 group, a checkbox 「エージェント CLI
  のフックで状態を検知する（各 CLI の設定ファイルに登録します）」 and three
  rows under it: 「Claude Code: 登録済み」 / 「未登録（CLI が見つかりません）」 /
  「登録に失敗: {error}」, likewise for Codex CLI and Antigravity CLI. The
  checkbox is drawn with the existing checkbox pattern of that group; the rows
  with the existing `tool_status_row` widget.
- **Socket unavailable** (path too long, directory not writable): a notice
  「フック受信ソケットを開けませんでした: {error}」 once; hooks are not installed
  that start.
- **Unparsable CLI settings file**: left untouched; that CLI reports
  「登録に失敗: {error}」.

## Design

One new file, src/tmux/hooks.rs, declared as a child module of `src/tmux.rs`
and re-exported from it, so `src/main.rs` — a paused surface — is not edited.
One concern: the CLIs' status hooks, which the launch in `src/tmux.rs` injects
and the poll consumes. (The path is not backticked above because the file does
not exist until stage 3; see lesson 003.)

| Piece | Shape |
|---|---|
| Identifiers | `src/config.rs` gains `HOOK_DIRECTORY` (`agent-hooks`), `HOOK_ENDPOINT_FILE` (`endpoint.env`), `HOOK_SOCKET_FILE` (`hook.sock`), `HOOK_SETTINGS_FILE` (`settings.json`), `HOOK_SESSION_ENV`, `HOOK_LAUNCH_ENV`, `HOOK_ENDPOINT_ENV`, `HOOK_SOCKET_ENV`, `HOOK_EVENT_ENV`, `HOOK_STALE_AFTER_SECONDS` (1800), `HOOK_BODY_MAX_BYTES` (65 536), the Antigravity bundle name, and the per-CLI script file names, each used through one helper. |
| Scripts | `hook_script(provider) -> String`; `managed_command(provider, script_path, event) -> String`; the install recognises its own entries by `HOOK_DIRECTORY/<script file>` appearing in the command. |
| Install | `install_hooks(data_dir, tools) -> Vec<HookInstall>` and `remove_hooks(...)`, where `HookInstall { provider: CliProvider, state: Installed / NotInstalled(reason) / Failed(error) }`. JSON edits go through `serde_json::Value`; TOML edits are line-based (find or insert `hooks = true` before the first table header; replace or append the `[hooks.state."…"]` blocks by exact key). Writes go through the same temporary-file-and-rename helper the store uses, then `chmod` back to the original mode. |
| Digest | `sha256(bytes) -> [u8; 32]` implemented in `src/util.rs` from the FIPS 180-4 description (no crate), with the standard test vectors. |
| Listener | `HookListener::bind(socket_path, ctx) -> Result<HookListener>` spawns a thread: accept, parse request line + headers + `Content-Length`, read body, reply 204/400/413, send `HookEvent { source, session: String, launch_token: String, event_name: String, payload: serde_json::Value }` on an `mpsc` channel, `ctx.request_repaint()`. Sequential per connection with a two-second read timeout. The socket file is removed before bind and on drop. |
| Normalisation | `normalise(source, event_name, payload) -> Option<HookReading { state, boundary, tool_name, last_message, session_id, transcript_path }>`; ignores events carrying `agent_id`. |
| State | `HookStatus { state, boundary, state_started_at: Instant, received_at: Instant, tool_name, last_message }`; `apply_reading(previous: Option<&HookStatus>, reading, now) -> (HookStatus, Option<ActivityNotice>)`; `activity(&HookStatus) -> AgentActivity`; `is_fresh(&HookStatus, now) -> bool`. |
| App | `OperonApp` gains `hook_listener: Option<HookListener>`, `hook_events: Receiver<HookEvent>`, `hook_status: HashMap<Uuid, HookStatus>`, `hook_launch_tokens: HashMap<Uuid, String>`, `hook_installs: Vec<HookInstall>`, `hooks_enabled: bool`. `update` drains `hook_events` each frame. `start_session` mints the token and passes a `HookEnvironment` into `launch_stored_session`. `apply_session_poll` substitutes a fresh hook activity for the screen activity and pre-settles the tracker so the existing debounce neither delays nor duplicates. Native identity is written through the existing persist path. |
| tmux | `start_tmux_agent_session` and `start_empty_tmux_session` take `&[(&str, &str)]` and pass `-e KEY=VALUE` to `new-session`; `respawn_tmux_pane` is preceded by `tmux set-environment -t <name> KEY VALUE` for the fresh token. |

Normalisation table:

| Source | working | waiting | done | ignored |
|---|---|---|---|---|
| claude | `UserPromptSubmit`, `PreToolUse` (tool not `AskUserQuestion`), `PostToolUse`, `PostToolUseFailure` | `PermissionRequest`, `PreToolUse` with tool `AskUserQuestion` | `Stop`, `StopFailure`; boundary: `SessionStart` with source in {startup, resume, clear}, `PostCompact` with trigger `manual` | any event with `agent_id`; `SessionStart` with other sources; `PostCompact` otherwise; `UserPromptSubmit` whose prompt starts with "This session is being continued from a previous conversation" |
| codex | `UserPromptSubmit`, `PreToolUse` (tool not `request_user_input`), `PostToolUse` | `PermissionRequest`, `PreToolUse` with tool `request_user_input` | `Stop`; boundary: `SessionStart` | events with `agent_id`, `SubagentStart`, `SubagentStop` |
| antigravity | `PreInvocation`, `PostInvocation`, `PreToolUse` (tool not `ask_question` / `ask_permission`), `PostToolUse`, `Stop` with `fullyIdle` false | `PreToolUse` with tool `ask_question` or `ask_permission` | `Stop` otherwise | `PostToolUse` after a `Stop` on the same transcript path |

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | Nothing new is painted in colour; the settings rows reuse `tool_status_row`, the chip reuses its existing tones. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No new mark. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | The environment variable names, the endpoint key, the file names, and the Antigravity bundle name are constants in `src/config.rs`; the script text, the tmux launch, the endpoint file, and the listener all read them there. The test writes the endpoint file with the constant, sources it with `/bin/sh`, and checks the script's `curl` line names the variable the file sets. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes | The store's shape is unchanged. The hook settings file and every CLI settings file are written through temporary file, rename, and directory sync; a CLI file is never truncated in place; the previous content is kept as `.bak`. Hook state is in memory only, so a crash loses nothing that was not already re-derivable from the screen. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | No new `Command::new`. The tmux launch gains `-e` arguments built from constants and a UUID; the launch command itself still passes `is_safe_agent_command`. The hook script is data written to disk, run by the CLI, not by Operon. |
| Documentation — user-facing docs change in all three languages together | Yes | `README.md`, `README.ko.md`, and `README.ja.md` each gain one bullet under Supervision saying state comes from the CLIs' hooks where installed. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | The socket is a local file; `curl` talks only to it; nothing leaves the machine; no `unsafe`. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | Request bodies are capped at `HOOK_BODY_MAX_BYTES`; headers at 8 KiB; `last_message` kept to 200 characters; one connection is served at a time under a two-second timeout; the installer reads each settings file at most 1 MiB. |

## Flagged concerns

- **Codex trust entries for a user-level `hooks.json`.** Unknown whether Codex
  0.153 requires them for `~/.codex/hooks.json`. Resolved: write them; the
  digest algorithm was checked against three real entries in the developer's
  `config.toml` and matched, so a redundant entry costs nothing and a missing one
  costs a `/hooks` prompt.
- **The Antigravity hooks file is mode 0444 on the developer's machine.**
  Resolved: the write goes through rename, which needs the directory, not the
  file; the mode is restored afterwards so the file stays as its owner left it.
- **Socket path length.** `sun_path` allows 104 bytes; the data directory path is
  about 75. Resolved: bind fails with a clear notice if the path exceeds 100
  bytes; nothing else changes.
- **One checkbox and three rows in Settings without a screen approval.**
  Resolved by recording it: the group, the checkbox pattern, and the row widget
  all exist; no new layout is introduced. The state file says so and a reader can
  reopen it.

## Acceptance

- `cargo test --locked` passes, including
  `sha256_matches_the_fips_test_vectors`,
  `codex_trust_hash_matches_a_hash_codex_wrote`,
  `claude_hook_install_is_idempotent_and_keeps_every_foreign_entry`,
  `codex_hook_install_writes_the_feature_flag_and_trust_entries_once`,
  `antigravity_hook_install_owns_only_its_bundle`,
  `removing_hooks_restores_each_settings_file_to_its_foreign_entries`,
  `hook_events_normalise_per_provider`,
  `a_session_boundary_never_notifies`,
  `a_stale_launch_token_is_ignored`,
  `the_hook_socket_round_trips_a_posted_event`,
  `the_endpoint_file_sources_into_the_variable_the_script_posts_with`,
  `a_fresh_hook_state_outranks_the_screen_and_a_stale_one_yields`, and
  `a_managed_launch_carries_the_hook_environment` (live tmux).
- `bash scripts/check-bands.sh` reports no breach past warn; `modules` rises by
  one, `subprocess_sites` does not rise.
- In the running app with Claude Code: launch a session, ask something; the chip
  goes WORKING within a second, IDLE when the answer ends, one notification;
  `/clear` produces no notification; Settings shows 「Claude Code: 登録済み」.
- Turning the toggle off removes every entry naming `agent-hooks/` from the
  three files and leaves the rest byte-equivalent as JSON.

## Rejected alternatives

- **A loopback TCP port with a token.** It is needed for Windows and WSL; a Unix
  socket with 0600 is simpler and needs no token to be safe, while the launch
  token still fences stale processes.
- **Writing events to files instead of a socket.** No ordering within a second
  without a lock, and two hooks appending at once can interleave.
- **Persisting hook state across restarts.** That would take a seven-day TTL and
  an "unconfirmed" flag; here the screen heuristic already covers the gap and
  the store is a paused surface.
- **A managed `CODEX_HOME`.** It would isolate Operon's hooks from `~/.codex`;
  Operon has one instance and one user, and the isolation would hide the user's
  own hooks and skills.
- **Registering `Subagent*` and `TeammateIdle`.** They only matter for a roster
  Operon does not draw; every child event is ignored by `agent_id` anyway.
