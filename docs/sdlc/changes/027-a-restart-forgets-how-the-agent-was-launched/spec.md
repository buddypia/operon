# Spec: a resume runs the agent the way it was launched

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Reopening a conversation carries every launch option the session record
   holds, in the order the launch wrote them, into the command that reopens it.
2. Four kinds of token are refused rather than carried, because each
   contradicts a resume: Claude's `--session-id <uuid>`, which allocates a new
   conversation; Antigravity's `--new-project`, which says in its own help that
   it starts a new project; any token the command's own validation gate rejects;
   and any bare positional — only a token beginning with `-`, and the value
   immediately following a valued option, may be carried. A positional carried
   into a resume would arrive as a prompt and replay a turn.
3. The reopened command is still recognised as a native resume, so no goal is
   appended to it. A resume that gains a positional prompt replays a user turn.
4. The recorded conversation id is still recoverable from the reopened command
   after the options are added.
5. The verb's hover text names, in Japanese, the options that will be carried.
   With no options to carry it reads exactly as it does today.
6. When one of the carried options is one the launch screen made the person
   acknowledge in writing, the verb takes the same danger treatment `verb_button`
   already gives a destructive verb.
7. Every command this change builds passes `is_safe_agent_command` before it is
   run, as every other launch path does.

## Behaviour

A session whose agent has exited shows the verb it shows today,
`会話 ID から再開`. Hovering it reads:

> 保存された会話 ID からこのセッションを再開します
> 引き継ぐ設定: モデル opus、権限確認をすべてスキップ

The second line is present only when there is something to carry. The options
are named by the same Japanese labels the launch screen used: the `label` field
of the `AgentFlag` that produced them, and `モデル <value>` / `モード <value>` /
`effort <value>` for the three valued options. A model called `opus` reads
`モデル opus` — the value is the CLI's own token and is not translated.

When any carried option is `dangerous`, the button takes the red hover-and-press
wash `verb_button` already draws for `停止`. It is not a solid red button; it is
the same "this one is not ordinary" mark, in the place a person is already
looking.

Nothing changes for a session with no options: same word, same hint, same
colour.

Not-happy states:

- **The record holds no command.** Older records may have an empty
  `agent_command`. Nothing is carried and the hint is today's single line.
- **The record holds a command this build cannot parse.** The first token is not
  the provider's own binary, or a token fails the safety gate. Nothing is
  carried, the resume runs bare, and that is the behaviour of every build before
  this one — a resume that works is better than a resume that refuses.
- **The command holds an option this build does not know.** It is carried
  through as long as it passes the safety gate. The catalogue is what names an
  option in the hint, not what permits it; an option added to a CLI between
  builds still survives a restart, it is just not described.

## Design

`src/agents.rs` gains three functions and changes two:

- `resume_command_with_launch_options(provider, launch_command, session_id) -> String`
  — the whole mechanism. Splits the recorded command on whitespace, drops the
  binary and the per-provider refusals of requirement 2, and re-emits the
  provider's resume spelling with the survivors in place:
  - Codex `codex resume <options> <id>` — `resume` is a subcommand and must
    follow the binary; `codex resume --help` documents `[OPTIONS] [SESSION_ID]`.
  - Claude `claude <options> --resume <id>` — flags are global on
    `claude [options] [command] [prompt]`.
  - Antigravity `agy <options> --conversation <id>` — a flat flag set.
  Returns the bare resume command unchanged when nothing survives, and when the
  result fails `is_safe_agent_command`.
- `carried_launch_options(agent, launch_command) -> Vec<String>` — the same
  survivors, named in Japanese for the hint. Walks `agent_flag_options(agent)`
  to find a flag whose `args` are present, and reads the three valued options
  off the grammar `build_agent_launch_command` writes.
- `carried_launch_is_dangerous(agent, launch_command) -> bool` — whether any
  surviving flag has `dangerous: true`.

`is_native_resume_command` stops matching exactly three tokens and matches the
resume marker instead: `codex` followed by `resume` with a safe id last; a
`--resume <id>` pair for Claude; a `--conversation <id>` pair for Antigravity.
Over-matching here is the safe direction — the only thing the answer decides is
whether a goal is appended, and appending one to a resume is the failure this
guards.

`native_session_id_from_agent_command` learns the same grammar, so requirement 4
holds.

`src/app.rs`: `resume_managed_native_session` builds the new session's
`agent_command` through the new function instead of `provider.native_resume_command`,
and `session_verb` takes the two hint functions. `session_verb` runs in a draw
path; both new functions are a `split_whitespace` over a short string and a walk
of a `&'static` table, which is the same order of work as the `tr()` calls and
the `String` hint the function already builds per frame. No allocation is added
beyond the hint that was already allocated.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | The danger treatment is `verb_button`'s existing `danger: bool` path, which derives its wash from `palette.danger`. No new role, no literal. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | The verb keeps `ICON_RESUME`. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | The resume spellings stay in `CliProvider::native_resume_command` and the new builder reads them from the same place rather than writing `--resume` a second time. The refusal list names Claude's `--session-id`, which `command_with_managed_native_session_id` writes — the test asserts the round trip, not a literal. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | No persisted shape changes. The launch command was already stored. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | No new child. The command built here passes `is_safe_agent_command` before it is stored, and the id still passes `is_safe_cli_session_id`. |
| Documentation — user-facing docs change in all three languages together | Yes | One bullet in `README.md`, `README.ja.md`, `README.ko.md`. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | None of the above. |
| Budgets — any new scan or output path states its byte and item ceiling | No | No new scan. The command being split is one already-validated line. |

## Flagged concerns

- **A dangerous option carried without a second acknowledgement.** Settled by
  the person on 2026-09-06: carry it, and say so on screen. The reasoning is
  that the acknowledgement was given for this conversation and this is the same
  conversation continuing; the failure to avoid is not the carry, it is the
  carry being invisible. Requirements 5 and 6 are that answer.

## Acceptance

- `cargo test --locked` passes, including:
  - `a_resume_carries_every_launch_option_it_was_started_with`
  - `a_resume_refuses_the_options_that_contradict_it`
  - `a_resume_carrying_options_is_still_a_native_resume`
  - `a_conversation_id_survives_the_options_around_it`
  - `the_resume_hint_names_what_it_will_carry`
  - `an_unparsable_launch_command_resumes_bare_rather_than_refusing`
- In the running app: launch Claude with a model and with 権限確認をすべて
  スキップ, let it exit, hover the resume verb. The hint names both, the button
  takes the red wash, and the reopened terminal runs with both.

## Rejected alternatives

- **Store the launch options structurally on `Session`.** Cleanest, and blocked:
  `src/models.rs` is a paused surface and this would be a schema bump for
  something the recorded command already says.
- **Rebuild the command by re-running `build_agent_launch_command`.** Needs the
  model, mode, effort, and flag ids, which is exactly the structure that is not
  stored. Parsing the command it wrote is the same information, one step later.
- **Drop the dangerous flags.** Safe, and it makes the original defect worse in
  the case that matters most: the long session that exits and has to be set up
  again.
- **Ask again before carrying a dangerous flag.** Two clicks every time, to
  re-answer a question already answered about the same conversation.
- **Show the carried options as chips beside the button.** The header's right
  end is already the busiest row on the screen; the hint is where the verb
  already explains itself.
