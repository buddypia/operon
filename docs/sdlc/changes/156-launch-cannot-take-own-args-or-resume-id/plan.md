# Plan: typed launch arguments and a conversation ID, merged with the sheet's options

- **Spec**: `./spec.md`
- **Approved**: 2026-10-16
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/agents.rs` | `typed_launch_arguments`, `launch_command_resuming`; `build_agent_launch_command` and `launch_needs_acknowledgement` take `extra_args` |
| `src/agents/settings.rs` | `AgentLaunchSettings.extra_args`, sanitised (custom clears it; invalid clears it; 1024-char cap) |
| `src/app.rs` | `agent_extra_args_input`, `resume_conversation_input` state; `LaunchAcknowledgement.extra_args`; `launch_session` resume branch; `launch_input_error`; `launch_ready`; summary lines |
| `src/app/screens.rs` | two rows in 「詳しい設定（任意）」; record/apply/pin/match/pending/ack read `extra_args`; footer reason reads `launch_input_error`; `select_agent` clears the conversation ID |
| `src/config.rs` | `LAUNCH_DETAIL_SUMMARY_VALUE_MAX = 24` |
| `src/i18n_tables.rs` | rows for every new string in EN and KO |
| `src/tests.rs` | the eight tests named in the spec; existing callers of the two widened functions |

## Order of work

1. Tests for requirements 1, 2, 5, 6 (pure functions; 2 includes a typed
   catalogue switch such as `--allow-dangerously-skip-permissions`) against the new
   signatures; watch them fail to compile, then fail.
2. `src/agents.rs`: `typed_launch_arguments`, the `extra_args` parameter, and
   `launch_command_resuming`; update existing callers (app and tests) with `""`.
   Tree compiles here.
3. Test for requirement 4, then `settings.rs` field and sanitiser.
4. Tests for requirement 3 and the app-level part of 5/6
   (`a_resume_id_launch_records_the_conversation_it_reopens`), then the app
   state, `LaunchAcknowledgement`, record/apply/pin/match, `launch_session`,
   `launch_input_error`, `launch_ready`.
5. Test for requirement 7, then the two rows and the truncated summary lines;
   i18n rows.
6. `make q.fast`; `cargo test --locked typed_launch_arguments`,
   `cargo test --locked resume_id`, plus the existing launch/pin/ack tests by
   filter (`launch`, `pin`, `acknowledg`).
7. Mutations: remove the `-` check, the owned-option check, the
   checkbox-switch check, the ID clear in `select_agent`, the summary cut, the
   extra-args term of `launch_needs_acknowledgement`, the ack comparison, the
   goal refusal, and the no-fallback check — each must turn a named test red.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A ticked dangerous switch silently dropped on resume (fallback to bare resume) | resume runs with prompts despite the tick | `launch_command_resuming` refuses on token loss; `a_resume_id_reopens_that_conversation_with_every_merged_option` asserts the flag is present |
| Typed dangerous switch launched without consent | no warning shown | `typed_launch_arguments_that_skip_permissions_need_the_acknowledgement` |
| Consent survives an edit of the typed line | launch enabled over a changed dangerous line | same test, edit then assert `!launch_acknowledged()` |
| Claude resume still allocates `--session-id` | two conversations named in one command | resume test asserts no `--session-id`; app test asserts `native_session_id` |
| Old `recent-agent-settings.json` fails to parse | every remembered setting lost | remembered test loads a file without the field |
| Per-frame cost of validation | stutter while typing | bounded string; reviewed by rust-reviewer |
| A typed catalogue switch bypasses `group` exclusivity | two contradictory switches reach the CLI | refusal test types `--allow-dangerously-skip-permissions` beside the ticked skip |
| A hidden ID blocks a custom launch | disabled button, no visible cause | `a_resume_id_refuses_…` switches agent and asserts the ID is empty |
| Header wraps on a long typed line | header pushes the section off screen | `typed_launch_arguments_are_shortened_in_the_folded_header` |
| Missing translation row | i18n table test fails | existing i18n drift tests |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked typed_launch_arguments` — 5 passed.
- `cargo test --locked resume_id` — 3 passed.
- CI on the branch: `test result: ok. N passed; 0 failed; 6 ignored`.
- `bash scripts/check-bands.sh` — within bands.
- Packaged app: launching claude with the skip-permissions tick and a
  conversation ID opens that conversation.

## Departures from the plan

Review round 1 (digest 05ffebc0ddf41657) returned three Important findings,
fixed together:

- **durability-reviewer/1** — an over-long typed line was saved whole and could
  push `recent-agent-settings.json` past its 64 KiB read cap, losing every
  remembered option; a line between 1 024 characters and the cap was cut
  mid-token on load. Now `LAUNCH_EXTRA_ARGS_MAX_CHARS` (1 024, `src/config.rs`)
  bounds the field (`char_limit`), `typed_launch_arguments` refuses a longer
  line whole, the save writes only a line the gate accepts, and the load no
  longer cuts. Test: `typed_launch_arguments_too_long_are_refused_and_never_saved`.
- **rust-reviewer/1** and **subprocess-safety-reviewer/1** — options Operon
  decides that no shape or switch names (`--continue`, `-r`, `-p`, Codex's `-m`
  `-s` `-a`, Antigravity's `-i`) and other spellings of owned ones
  (`-rOTHER`, `-anever`, Antigravity's single-dash `-new-project`) passed.
  Now `operon_owned_launch_options` lists the first kind, and a single-dash
  token must be one short option (`-x` or `-x=value`), which refuses bundles
  and single-dash long names. Requirement 2 is wider than the spec says by
  these two rules. Tests: additions to
  `typed_launch_arguments_refuse_bare_words_unsafe_characters_and_owned_options`.

The sweep grew to 18 mutations, all caught. `cargo test --locked
typed_launch_arguments` now runs 6 tests; `resume_id` runs 4 (the fourth is the
existing `gives_new_managed_claude_sessions_a_persistent_resume_id`).
