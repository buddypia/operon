# Plan: one vocabulary decides what crosses a restore

- **Spec**: `./spec.md`
- **Approved**: 2026-08-28
- **Status**: done

Accepted in plan mode before any file was edited. Two departures from the
accepted plan are recorded under "Departures" below.

## Files that change

| File | Change |
|---|---|
| `src/transcript.rs` | new — `TRANSCRIPT_VOCABULARY` and its lookups |
| `src/history.rs` | parse / classify / project split; `RestoreClassification`; one read shared by every destination |
| `src/cli.rs` | `ImportedNativeSession.classification`; Antigravity title reads through the shared reader |
| `src/app.rs` | the notice reports the accounting |
| `src/main.rs` | declare and re-export the module |
| `src/tests.rs` | six guards; existing fixtures updated where they encoded the old judgement |
| `CLAUDE.md`, `REVIEW.md` | module map row; review pass row |
| `scripts/check-transcript-vocabulary.sh` | new — local sweep for unmet kinds |
| `docs/sdlc/lessons.md` | entry 007 |

## Order of work

1. Scan the local stores for every kind the three CLIs actually write. Seed the
   table from that, not from documentation.
2. `src/transcript.rs` with the table and lookups. Tree compiles.
3. Split the readers in `src/history.rs`; add `RecordOutcome` and
   `RestoreClassification`. Tree compiles, suite red.
4. Move the transcript read up into `import_full_cli_history` and pass the
   projection to all three destinations and all four verifiers.
5. `ImportedNativeSession`, `manifest.json`, the notice.
6. Update the fixtures that encoded the old judgement; add the six guards.
7. The sweep script; classify whatever it finds; re-run until it is clean.

## Departures

- **A `Role` layer was added.** Not in the accepted plan. Codex addresses the
  model as `developer` and `system` through the same `message` payload as the
  conversation, and the first cut mapped every non-`user` role to `assistant`,
  which restored Codex's own system prompt as a turn. The role is a third
  identifier the readers were judging inline, so it became a fourth layer rather
  than a `match` arm.
- **The sweep found seven kinds a 400-file sample had missed**, and two of them
  changed a classification the plan had assumed:
  `claude/Flag/isAbortedMidStream` sits on assistant records that still hold
  their full text, so excluding it would have lost real turns; and
  `claude/Record/summary` carries the text standing in for the turns a
  compaction replaced, in a `summary` field rather than a message, which forced
  `record_text` to read outside `message.content`. An existing fixture asserted
  `summary` was dropped; it encoded the old judgement and was corrected.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A row is added out of sort order | its kind silently reverts to "never seen", so its content starts arriving labelled `[Unclassified: …]` | `the_transcript_vocabulary_is_well_formed` |
| A reader gains a `match` arm for a kind with no row | the arm is unreachable — classification happens first — so it is dead code, not a loss | by construction; the arm never runs |
| A row is added with no renderer | the kind is classified conversation but produces nothing, counting as `empty` | `restore_counts_every_source_record_into_exactly_one_class` shows the imbalance in `empty` |
| Carrying reasoning overflows the destination's context | the destination compacts sooner than expected | not caught automatically; the notice states the reasoning count so the size is visible before it surprises anyone |
| A CLI ships a new record type | fragments arrive labelled `[Unclassified: …]` and the notice names the kind | the restore's own report, plus `scripts/check-transcript-vocabulary.sh` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. 282 passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — nothing past the compile lines.
- `bash scripts/check-bands.sh` — one **warn** breach:
  `always_loaded_bytes` 11512 against a warn tier of 11000, from the new
  `CLAUDE.md` section. Warn is "recorded, no action"; the band was not moved to
  make it green, because quieting the reading is the failure the band exists to
  catch. `modules` 19 → 20 stays under its warn tier of 24.
- `bash scripts/check-transcript-vocabulary.sh` — `66 kinds observed locally,
  all classified`.
- The reference conversation
  (`~/.claude/projects/-Users-username-dev-playground-operon/2cfa1dd9….jsonl`):
  `records: 1165, restored: 532, operational: 627, unclassified: 0, empty: 6,
  reasoning_fragments: 61`. 532 + 627 + 0 + 6 = 1165, and 471 → 532 is the
  61 reasoning fragments that used to be dropped.
- `cargo test --locked -- --ignored`: 3 of 6 pass —
  `codex_app_server_restores_a_short_transcript_into_a_resumable_terminal`,
  `a_saved_codex_conversation_restores_into_a_resumable_antigravity_terminal`,
  and `antigravity_restore_writes_a_resumable_conversation_holding_the_source_history`,
  which between them cover every path this change rewrote: `inject_turns`, both
  Codex verification passes, and the Antigravity write-and-read-back.
  The other 3 fail before reaching any restored content, on a pre-existing
  defect unrelated to this change: `auto_approve_workspace_trust_prompt` in
  `src/tmux.rs` answers the workspace-trust prompt with a bare `Enter`, and the
  current CLIs open that prompt with the cursor on `No, exit`, so the pane
  exits with status 1 before any restored content is rendered. Needs its own
  intent.
