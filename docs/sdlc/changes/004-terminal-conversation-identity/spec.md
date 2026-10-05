# Spec: the terminal's own registry names its conversation

- **Intent**: `./intent.md`
- **Status**: approved

## What was found

Claude Code publishes the answer. `~/.claude/sessions/<pid>.json` holds one entry
per CLI process:

```json
{ "pid": 96293, "sessionId": "7ee22502-…", "cwd": "/Users/a/dev/writing",
  "startedAt": 1787552134630, "procStart": "…", "status": "idle", … }
```

`sessionId` is the conversation that process is on **now** — it is rewritten when
the terminal rotates. On the reported failure the entry reads `7ee22502`, the
conversation on screen, against the `f5cb1562` Operon had stored, and its
`startedAt` of 1787552134 sits one second from the terminal's `created_at` of
1787552133.

So the identity does not have to be inferred at all. It is a file read.

## Requirements

1. The conversation of a managed Claude terminal is read from Claude Code's
   session registry, correlated to the terminal by workspace and by when the CLI
   process started.
2. The registry outranks the stored `native_session_id` and everything derived
   from transcript timestamps, including for a terminal launched as a resume —
   `/clear` rotates a resumed terminal too.
3. Correlation uses the allowance already defined for dating a transcript against
   its terminal (`MANAGED_SESSION_CLOCK_SKEW_SECONDS`); no second constant.
4. Two registry entries for the same workspace within that window are told
   apart by the stored ID when one of them still names it — a terminal that has
   not rotated goes on publishing its launch ID, which is evidence. Only when
   both have moved on is it a refusal.
4a. A registry entry that *confirms* the stored identity ends the decision. No
   weaker inference may overrule a positive confirmation.
5. A registry with no entry for this terminal leaves the existing behaviour
   exactly as it is. Claude Code before the registry, or a machine the terminal
   did not start on, must not become a new failure.
6. The three directory names Operon reads inside Claude Code's store are each
   spelled in one place.
7. No new `#[ignore]`d test, and no subprocess in the restore path.

## Behaviour

Restoring or reopening a cleared terminal now reads its current conversation. The
notice is the one that already exists, over the right conversation.

When the registry names a conversation whose transcript is not on disk yet — a
terminal cleared a moment ago and not yet spoken to:

> このターミナルは新しい会話を開始したばかりで、まだ会話ログが書かれていません。
> 1 度発言してから、もう一度お試しください。

When two terminals in one workspace started within the allowance and the registry
names different conversations for them:

> このワークスペースで同時に開始された Claude ターミナルが複数あり、どの会話が
> このターミナルのものか判別できません。誤った会話を復元しないよう、中止します。

Not-the-happy-path: an unreadable or absent registry directory is silence, not an
error — requirement 5. A registry entry naming an ID that is not a safe session
ID is ignored, as anywhere else Operon takes an ID from a file.

## Design

- **`src/config.rs`** — `CLAUDE_STORE_DIRECTORY`, `CLAUDE_PROJECTS_DIRECTORY`,
  `CLAUDE_SESSION_REGISTRY_DIRECTORY`, and the two budgets bounding the registry
  read. Claude Code's projects directory was written inline at five call sites;
  this is the
  `MANAGED_TMUX_PREFIX` lesson applied before the sixth is added.
- **`src/cli.rs`** — `ClaudeRegisteredSession` and
  `read_claude_session_registry_in`; `registered_claude_conversation_in` performs
  the correlation, returning `RegisteredConversation::Unknown` or `Named`;
  `claude_transcript_path_in` extracts the path construction
  `native_session_path_for_project` already did, so the registry and the scan
  build the same path the same way.
  `corrected_managed_claude_conversation_in` holds the whole decision and
  `corrected_managed_claude_conversation` is its `HOME`-rooted wrapper — the
  seam exists so a guard can exercise the ordering, not only the lookup.
- **`Unknown` is not `Named`**, for the reason `TRANSCRIPT_VOCABULARY` separates
  "excluded on purpose" from "never seen". A registry that names the stored
  conversation has confirmed it and the decision ends there; a registry with
  nothing to say lets the older checks run.
- The correlation key is `encode_claude_project_path`, not string equality on
  `cwd`: it is the same normalization the projects directory itself is keyed by,
  so the registry and the transcript directory cannot disagree about which
  workspace an entry belongs to.
- Nothing is persisted differently. `STORE_SCHEMA_VERSION` is untouched; the
  corrected identity is written into the existing
  `native_session_id`/`native_session_path` fields by the call site that already
  writes them.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | no | draws nothing |
| Icons | no | adds no mark |
| Identifier SSOT | **yes** | three directory names move into `src/config.rs`; the guard is a sweep for the literals, derived from the constants |
| Transcript vocabulary | no | classifies no record; the registry is not a transcript |
| Durability | partial | reads only; the one write is through the store path that already existed |
| Subprocess safety | yes | spawns nothing. The PID-tree route was rejected for this reason |
| Documentation | partial | `CLAUDE.md`, `docs/sdlc/lessons.md`. No `README.*` change |
| Local-first | yes | reads one local directory |
| Budgets | yes | the registry read is bounded by an entry count and a per-file byte ceiling, both named constants |

## Flagged concerns

- **Is `startedAt` reliably within 5 s of Operon's `launched_at`?** — Measured at
  1 s. If a cold start ever exceeds it the correlation finds nothing and the
  existing behaviour returns, which is today's behaviour: a miss is safe, a wide
  window is not. The allowance is deliberately not widened to buy recall.
- **PID reuse.** Not exposed: the PID is only a filename here. Correlation is by
  workspace and start time, so a recycled PID that overwrote an old entry either
  matches this terminal or does not.
- **A stale entry for a terminal that has exited.** Its last `sessionId` is still
  the right answer for that terminal, which is what a reopen needs.
- **A refusal blocks reopening, not only restoring.** The first caller is
  `resume_managed_native_session`, so an error means the terminal does not open
  at all. That is why the ambiguous case is narrowed by the stored ID first and
  the refusal is the last resort: two terminals launched together in one
  workspace would otherwise both become unresumable, which is a worse failure
  than the one being fixed.
- **The registry read truncates silently at
  `CLAUDE_SESSION_REGISTRY_VISIT_LIMIT`.** Measured at 34 entries against a
  limit of 4 096, and truncation degrades to `Unknown`, which is the previous
  behaviour. Named here rather than reported at runtime because the reporting
  would cost more than the case is worth; revisit if a store is ever seen near
  the limit.

## Acceptance

- `cargo test --locked` → `0 failed; 6 ignored`, including
  `a_cleared_claude_terminal_restores_the_conversation_it_moved_to`,
  `a_resumed_claude_terminal_still_follows_its_registry`,
  `a_registry_confirmation_is_not_overruled_by_the_start_time_check`,
  `a_registry_entry_belonging_to_another_terminal_is_not_read_as_this_one`,
  `two_claude_terminals_started_together_are_told_apart_or_refused`, and
  `the_claude_store_directories_are_spelled_in_exactly_one_place`.
- Every guard goes through the whole decision rather than the lookup, and each
  of four mutations of that decision fails the suite.
- The reported terminal restores `7ee22502` rather than `f5cb1562`.

## Rejected alternatives

- **Follow the `/clear` chain through the transcripts.** The rotated files do
  record their own provenance — the first user record is
  `<command-name>/clear</command-name>`, on 2 081 of 5 528 local transcripts — so
  a chain could be walked. It is still inference, it breaks the moment Claude
  Code renames the command or adds another way to rotate, and it is strictly more
  code than reading the answer.
- **Find the terminal's `claude` PID through the tmux pane's process tree.** Ties
  the restore path to `ps`, adds subprocess work where there was none, and buys
  nothing: the registry is already unique per workspace and start time.
- **Re-resolve the identity on every poll and keep the store current.** Larger
  blast radius for the same outcome, and it writes to the store on a timer.
- **Widen `MANAGED_SESSION_CLOCK_SKEW_SECONDS` so the correlation never misses.**
  Trades a safe miss for an unsafe match, in the one place where matching the
  wrong conversation is the bug being fixed.
