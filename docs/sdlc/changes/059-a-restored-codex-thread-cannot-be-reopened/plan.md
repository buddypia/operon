# Plan: the replay records a Codex restore appends carry no rollout ordinal

- **Spec**: none — `bugfix` route skips it; `./intent.md` is the bug report
- **Approved**: 2026-09-18
- **Status**: done

## The named cause

`codex_replay_event` (`src/history.rs:1461`) writes
`{"timestamp": …, "type": "event_msg", "payload": …}` and no `ordinal` field.
Codex 0.155.0 numbers every record it writes into a rollout — `session_meta`,
`response_item`, `event_msg` alike — and its thread store reads the **final**
record to continue that sequence. `add_codex_replayable_history` appends
Operon's un-numbered replay records after Codex's numbered ones, so the final
record has no ordinal, and `thread/resume` aborts:

```
thread-store internal error: failed to resume local thread recorder:
final paginated rollout record at …/rollout-….jsonl is missing an ordinal
(code -32603)
```

Measured in both directions through the same `thread/resume` the TUI calls, so
the diagnosis does not depend on a pane drawing in time
(`~/.operon-hook-backups/probe-codex-ordinal.py`, against the rollout the
failing end-to-end test left behind):

| The rollout | `thread/resume` |
|---|---|
| as Operon wrote it — 15 records, ordinals on 0-10, none on 11-14 | refused, with the error above |
| the same file with 11, 12, 13, 14 filled in | resumed |

And the boundary, because "add an ordinal everywhere" would have been the wrong
lesson (`probe-codex-resume-only.py`):

| The rollout | `thread/resume` |
|---|---|
| a 2026-09-05 session, 15 records, **no** ordinal anywhere | resumed |
| a 2026-09-10 session, 895 records, ordinal on **every** one | resumed |

So Codex accepts both conventions and refuses the mixture. The fix is
therefore "continue the file's own convention", not "always write an ordinal".

## Why no test caught it

The CI test over this writer, `codex_restore_appends_history_the_terminal_can_
replay`, builds its rollout by hand:

```
{"type":"session_meta","payload":{"id":"thread"}}
```

That is a legacy-shaped record — no ordinal — so the file the test appends to
has nothing to be inconsistent with, and the appended records are correct for
it. The end-to-end test that does use a real Codex is `#[ignore]`d, needs an
authenticated CLI, and is not run by CI: it has been failing since Codex
started numbering records, which the local session files date to before
2026-09-10. Eight days, minimum, with a green suite.

## Files that change

| File | Change |
|---|---|
| `src/history.rs` | `codex_rollout_highest_ordinal`, a new reader; `codex_replay_event` takes the ordinal to write; `add_codex_replayable_history` numbers the appended records from the file's own highest; `verify_codex_replayable_history` refuses a rollout whose final record lost its ordinal |
| `src/tests.rs` | three CI-runnable tests: the modern convention, the legacy convention, and the verification that refuses the mixture |
| `scripts/check-codex-rollout-shape.sh` | new, `not-wired`: which envelope fields Codex writes on this machine that Operon's replay record does not |
| `docs/sdlc/lessons.md` | entry 024 |

## Order of work

1. Write the three tests and watch them fail, with the messages predicted:
   the modern one for a missing `ordinal`, the legacy one — which must pass
   before and after, so it is the guard that the fix does not over-apply — and
   the mixture one for a verification that accepts what Codex refuses.
2. `codex_rollout_highest_ordinal`, then thread it through `codex_replay_event`
   and `add_codex_replayable_history`. The tree compiles between the two only
   if the signature change and its three call sites move together, so they are
   one step.
3. `verify_codex_replayable_history`'s refusal, so that reverting step 2 fails
   the restore in front of the person rather than in the pane.
4. `scripts/check-codex-rollout-shape.sh`, run against the local rollouts.
5. The three gates, then `cargo test --locked -- --ignored`.
6. Entry 024, with each new guard watched failing.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The ordinal must be contiguous, not merely increasing | Codex resumes but writes a duplicate ordinal on its next record | `highest + 1 + index` is contiguous in a well-formed file; the measured resume used exactly these values |
| A rollout with a non-monotonic ordinal sequence | the appended record is not the maximum, and the resume fails as before | the highest is taken over every record, not the last one, so the appended records exceed all of them |
| The legacy convention gets an ordinal it should not have | a pre-numbering session stops resuming after a restore | the legacy test, which asserts the absence of the field rather than its value |
| A huge rollout is read twice | a slow restore | it is already read twice — `verify_codex_replayable_history` reads the whole file — and a restore is not a frame |
| The next format change is silent again | another eight green days over a dead pane | step 3's refusal, and step 4's script |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- The three new tests fail before step 2-3 and pass after, each for the
  message predicted in step 1.
- `cargo test --locked -- --ignored codex_app_server` — passes, which is the
  real Codex resuming a real restored thread.
- `bash scripts/check-codex-rollout-shape.sh` — reports no field Codex writes
  that the replay record omits.

## Observed and not changed

The thread Operon creates records `"originator": "xirp-copy"`, from the
`clientInfo.name` and `serviceName` the app server is given. It is the
pre-rename name, and it is written into session files that already exist on
disk, so it is a persisted identity rather than a label — the same situation as
the store value `"gemini"` for Antigravity. Changing it is a change of its own,
with the question of what happens to sessions already carrying it.

## Departures from the plan

**One test became four arrangements, and the fourth was written because a
mutation had nothing to redden.** The plan named three tests. The writer's
choice to take the *highest* ordinal rather than the last was unguarded — every
fixture was monotonic, so "last" and "highest" agreed — and the mutation that
swaps them stayed green. A non-monotonic arrangement was added to the first
test, and the same mutation then reddened it. The verification's non-maximal arm
got the same treatment, as a second arrangement of the mixture test: unreachable
through the writer, which is exactly why a branch nobody has watched fail is a
guess about what it catches.

**The two new messages have no rows in the translation tables, and that is the
module's state rather than a shortcut.** All 89 message ids in `src/history.rs`
are absent from both `EN_TABLE` and `KO_TABLE` — measured, not assumed — so a
restore failure is reported in Japanese whatever language is selected. Adding
rows for these two alone would have implied the rest were covered. The gap is
worth its own change on the `text` surface; it is recorded here because this
change is what measured it.

That follow-up needs a **guard**, not just rows, and its intent has to say so.
Review established why: `every_message_id_has_a_row_in_every_table` compares
the two tables against `catalog_keys()`, which is the union of those same two
tables — so it can only catch a language added halfway, never a message id the
code uses and no table knows. Nothing in the suite reads the `tr`/`tf!` call
sites. Fill the 89 rows without adding that check and the next change makes it
91.

**The end-to-end test still fails, for a second cause this change found and did
not fix.** `codex_app_server_restores_a_short_transcript_into_a_resumable_
terminal` no longer dies: the pane stays alive, the resume error is gone, and
Codex appends its own `thread_settings_applied` record at ordinal 15 — it
continued Operon's numbering, which is the fix confirmed from Codex's side. What
remains is that the restored conversation is not rendered. Measured through
`thread/resume`: the restored thread returns **0 turns**, a thread Codex wrote
itself returns **4**, each turn holding `userMessage` and `agentMessage` items.
A turn in Codex 0.155.0 is delimited by `task_started`, `turn_context`, and
`item_completed` records carrying a `turn_id`, and Operon writes none of them —
its `event_msg` replay records are the track an older Codex TUI replayed from.
That is a separate cause with its own change, and this change's contract records
the end-to-end test as failing for it rather than claiming a pass.
