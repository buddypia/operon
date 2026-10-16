# Spec: typed launch arguments and a conversation ID, merged with the sheet's options

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. **Typed arguments are appended after the sheet's options.** For codex, claude
   and gemini, the launch command is the sheet's command (model, effort, mode,
   ticked switches, in that order as today) followed by the typed arguments in
   the order typed.
   Test: `typed_launch_arguments_follow_the_ticked_switches`.
2. **Typed arguments are switches only, and never one the sheet owns.** Each
   whitespace-separated token must start with `-` and must use only the
   characters `is_safe_agent_command` already allows. Its name (the part before
   any `=`) must not be an option the sheet sets itself:
   - an option field: every `--`-named entry of `launch_option_shapes` for that
     agent (`--model`, `--effort`, `--permission-mode`, `--sandbox`, `--mode`,
     `--session-id`, `--resume`, `--conversation`, `--new-project`);
   - a checkbox: the first argument of any `agent_flag_options` entry for that
     agent (`--dangerously-skip-permissions`, `--ask-for-approval`, `--search`,
     …). Such a switch is ticked rather than typed, so the catalogue's
     exclusivity (`resolve_agent_flags`: `group`, `conflicts_with_mode`) still
     decides every switch it knows.

   A valued option is written joined: `--add-dir=../shared`. Anything else
   refuses the launch with the message in Behaviour. Test:
   `typed_launch_arguments_refuse_bare_words_unsafe_characters_and_owned_options`.
3. **A typed argument that runs unguarded needs the acknowledgement.**
   `launch_needs_acknowledgement` is true when the typed tokens satisfy
   `arguments_need_acknowledgement` (the same table read for the custom agent),
   and the recorded consent covers the typed arguments: editing them withdraws
   it. Test: `typed_launch_arguments_that_skip_permissions_need_the_acknowledgement`.
4. **Typed arguments are remembered per agent, and pinned with the rest.**
   `AgentLaunchSettings` gains `extra_args` (`#[serde(default)]`), written with
   the other options, restored when the agent is chosen, carried by
   「この設定を毎回使う」, and compared by `launch_matches_pin`. A file written
   before this change loads with it empty; a stored value that fails
   requirement 2 loads empty (and a pin whose value was cleared loses its
   acknowledgement, as any sanitised pin does today). Test:
   `typed_launch_arguments_are_remembered_per_agent_and_old_files_still_load`.
5. **A conversation ID reopens that conversation with every merged option.**
   With a non-empty ID, the command is the CLI's own resume
   (`CliProvider::native_resume_command`) with every token of requirement 1's
   command inserted at the seam `resume_command_with_launch_options` already
   uses: `claude <options> --resume <id>`, `codex resume <options> <id>`,
   `agy <options> --conversation <id>`. Claude's `--session-id` is not added; the
   session record's `native_session_id` is the typed ID for all three CLIs.
   Tests: `a_resume_id_reopens_that_conversation_with_every_merged_option`,
   `a_resume_id_launch_records_the_conversation_it_reopens`.
6. **A conversation ID refuses what it cannot honour.** Refused, with the
   message in Behaviour, and no session record written: an ID that fails
   `is_safe_cli_session_id`; a non-empty request (a resume never sends a prompt,
   `agent_shell_command`); a ticked switch the resume refuses (Antigravity's
   「新しいプロジェクトとして開始する」). The ID is not remembered: it is empty
   when the app starts, cleared after a launch like the request, and cleared by
   `select_agent` when the agent changes — an ID belongs to one CLI, and the
   custom agent, which draws no ID field, therefore never holds one. Test:
   `a_resume_id_refuses_malformed_ids_a_request_and_a_new_project`.
7. **The folded header stays one line.** The typed line and the ID appear in
   the header summary cut to `LAUNCH_DETAIL_SUMMARY_VALUE_MAX` (24) characters
   by `truncate_chars`, which marks the cut with `…`. Test:
   `typed_launch_arguments_are_shortened_in_the_folded_header`.

## Behaviour

Both fields live in 「詳しい設定（任意）」, after the switches, and are drawn only
for codex, claude and gemini (not 「別のコマンドを使う」):

- 「追加の引数（任意）」 — single line, hint 「例: --add-dir=../shared --verbose」,
  with the note 「チェックした項目の後ろに付けて起動します。値は --name=value の形で書きます。」
  Saved when the field loses focus, like the model field.
- 「再開する会話 ID（任意）」 — single line, hint 「空なら新しい会話を始めます」.

The folded header's summary gains, when set, 「追加の引数」 followed by the
typed line and 「再開する会話」 followed by the ID (for example
「追加の引数 --verbose」), each value cut to 24 characters (requirement 7).

Refusals reach the person twice, as today: the footer's reason line (button
disabled) and, if launched anyway through ⌘↩, the notice.

- bare word: 「追加の引数は - で始まるスイッチだけです（値は --name=value の形で書きます）: {arg}」
- unsafe character: 「追加の引数に使えない文字が含まれています。シェル演算子は使えません。」
- owned option or checkbox switch: 「{arg} はこの画面の項目で指定してください。」
- malformed ID: 「会話 ID には英数字とハイフンだけを、64 文字以内で入力してください。」
- request with ID: 「会話を再開するときは依頼を空にしてください。再開後にターミナルで入力できます。」
- refused switch with ID: 「「{label}」は会話の再開と同時に指定できません。」

A dangerous typed argument shows the existing warning and
「理解したうえで起動する」 checkbox. Empty fields change nothing: the command is
byte-for-byte what it is today. A field over the length cap is cut on load
(1024 characters, the cap the model field has).

## Design

Placement: **NEW_IN_MODULE** — launch-command construction is `src/agents.rs`
(`build_agent_launch_command`), remembered options are `src/agents/settings.rs`,
the sheet is `src/app/screens.rs`; `git grep -n "extra_args\|resume_id"` finds
nothing today. Route `feature`.

- `src/agents.rs`
  - `typed_launch_arguments(agent, typed) -> Result<Vec<&str>, String>`:
    requirement 2.
  - `build_agent_launch_command` gains `extra_args: &str`; appends the validated
    tokens. Custom agent ignores it (its line is the whole
    command; the sheet never fills the field for it).
  - `launch_command_resuming(agent, command, conversation_id) -> Result<String,
    String>`: requirement 5–6 checks, then
    `resume_command_with_launch_options`; refuses rather than falls back if the
    result would lose a token (that function's silent fallback to the bare
    resume would drop a ticked dangerous switch).
  - `launch_needs_acknowledgement` gains `extra_args: &str`.
- `src/agents/settings.rs` — `AgentLaunchSettings.extra_args`, sanitised.
- `src/app.rs` — `OperonApp.agent_extra_args_input`, `resume_conversation_input`;
  `LaunchAcknowledgement.extra_args`; `launch_session` uses both;
  `launch_ready` and the footer read one `launch_input_error()`; summary lines.
- `src/app/screens.rs` — the two fields; record/apply/pin/match/pending/ack;
  `select_agent` clears the ID.
- `src/config.rs` — `LAUNCH_DETAIL_SUMMARY_VALUE_MAX = 24`, beside
  `LAUNCH_DETAIL_SUMMARY_MAX`.

No store shape change: `Session.agent_command` and `native_session_id` hold the
result as they already do. `recent-agent-settings.json` gains one defaulted
field; an older build reading the new file ignores it (serde default, no
`deny_unknown_fields`). No new external command: the command still reaches tmux
through `launch_stored_session` → `agent_shell_command`.

Draw path: `launch_input_error()` runs per frame while the sheet is open — a
split over one line capped at 1024 characters and a table walk over
`&'static` slices; it allocates only when it has an error to show.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | yes, read `.claude/rules/palette-and-glyphs.md` | notes use existing `.weak()` and `palette.accent_soft`; no new role |
| Icons | no | no glyph added |
| Identifier SSOT | yes, read `.claude/rules/identifiers.md` | owned options are read out of `launch_option_shapes`, the resume out of `native_resume_command`; no flag spelled anew |
| Durability | yes, read `durability-invariants` | the store is untouched; `recent-agent-settings.json` keeps its atomic write and its 64 KiB read cap; new field defaults |
| Subprocess safety | yes, read `.claude/rules/rust.md` | the result passes `is_safe_agent_command` (and `is_native_resume_command` for a resume); every token starts with `-`, so no prompt or subcommand can be typed in |
| Documentation | checked `README.md` | it does not describe the launch sheet's options; nothing to change |
| Local-first | yes | no network, no `unsafe` |
| Budgets | yes | field capped at 1024 characters on load; no new scan or output path |

## Flagged concerns

- **Request text with a resume** — sending it would need the resume to carry a
  prompt, which `agent_shell_command` deliberately never does (a retry would
  replay it). Answer: refused with a message (requirement 6), not silently
  dropped.
- **Valued options typed with a space** (`--add-dir ../x`) — the bare value would
  reach the CLI as a prompt, and a resume would carry the switch without its
  value. Answer: refused; the message says to write `--name=value`.
- **Two records, one conversation** — with an ID typed, the record's
  `native_session_id` is set at launch, so `NativeSessionResolved` does not run
  for it, and nothing stops two Operon sessions naming the same conversation
  (the restore path allows this today too). Accepted: the person chose the ID.
- **An ID the CLI does not know** — e.g. a Codex ID from another working
  directory. Requirement 6's refusals are the pre-checks only; past them the
  record is written and the CLI's own error shows in the terminal, as a failed
  launch does today. Accepted: only the CLI can answer whether an ID exists.
- **Unknown typed switches that conflict** — a typed switch the catalogue does
  not know (for example Codex `--full-auto` beside a ticked mode) is not checked
  for exclusivity; the CLI's usage error shows in the terminal. Accepted: the
  catalogue only knows its own switches, and those must be ticked.
- **Typing a dangerous switch** — no protection is weakened: typed tokens go
  through the same acknowledgement as ticked ones, and the pin's carried
  acknowledgement is the person's change-138 decision, extended to one more
  field of the same combination.

## Acceptance

- `cargo test --locked typed_launch_arguments` (5) and
  `cargo test --locked resume_id` (3) pass; the named tests fail with the feature
  removed.
- In the app: claude, tick 「権限確認をすべてスキップする」, type an existing
  conversation ID, acknowledge, launch → the terminal shows that conversation,
  and `ps` shows `claude --dangerously-skip-permissions --resume <id>`.

## Rejected alternatives

- Free-text replacing the checkboxes — loses the exclusivity and danger tables.
- Allowing `--name value` with a space — makes prompts typeable and breaks resume.
- Remembering the conversation ID — the next launch would silently reopen an old
  conversation.
- Accepting a typed checkbox switch and de-duplicating it — a typed sibling of a
  ticked switch (`--allow-dangerously-skip-permissions` beside
  「権限確認をすべてスキップする」) would bypass the catalogue's exclusivity.
