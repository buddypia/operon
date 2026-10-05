# Spec: one vocabulary decides what crosses a restore

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Every record, payload, role, content block, record flag, and message wrapper
   the three CLIs write has exactly one row in a single table, giving its class
   (`Conversation`, `Reasoning`, `Operational`) and the reason for that class.
2. The readers in `src/history.rs` hold no judgement of their own: what to keep,
   what to strip, and what to exclude are all answered by that table.
3. A kind with no row is a distinct outcome from a kind excluded on purpose. It
   is carried across under a label naming it, and reported.
4. Reasoning crosses to every destination as attributed text, and never as
   native reasoning.
5. Every source record is counted into exactly one of restored, operational,
   unclassified, or empty, and the four sum to the number of records read.
6. The accounting reaches the person as a notice and is written beside the
   archive in `manifest.json`.
7. The notice no longer claims a fidelity nothing checks.
8. No new `#[ignore]`d test.

## Behaviour

After a successful restore the notice reads, in Japanese:

> Claude Code の会話 3c33bce8 から 532 件の会話ターンを復元し、Codex CLI の会話
> 9f1c… を作成しました。（元の記録 1165 件から会話 471 件・思考 61 件を取り、
> 運用レコード 627 件は除外）復元先セッションを起動しています…

When the source held a kind the table does not know, a second sentence follows:

> Operon が分類を知らない記録がありました: claude/telemetry-batch (5) ·
> claude/citation (2)。内容は復元先へ渡していますが、src/transcript.rs の
> TRANSCRIPT_VOCABULARY に分類を追加してください。

Inside the restored conversation, a carried-over reasoning fragment is prefixed
`[Thinking]` and an unclassified one `[Unclassified: claude/citation]`. These are
English, matching the `[Tool call: …]` and `[Tool result]` markers already
written beside them: they annotate carried data rather than address the person.

Not-the-happy-path: an empty projection still fails the restore with the existing
message. The per-file byte and line budgets still fail closed. A source line that
is not JSON is counted and reported as `<provider>/unparsable-line` rather than
skipped.

## Design

- **`src/transcript.rs`** (new) — `TranscriptClass`, `TranscriptLayer`,
  `TranscriptKind`, `TRANSCRIPT_VOCABULARY`, `transcript_class`,
  `transcript_wrappers`, `RESTORED_REASONING_LABEL`, `unclassified_label`. The
  table is sorted by `(provider, layer, kind)` and looked up by binary search.
- **`src/history.rs`** — parsing, classification, and projection split.
  `RecordOutcome` has four variants. `RestoreClassification` carries the
  accounting. `read_transcript_conversation` returns both, and
  `read_transcript_conversation_turns` becomes its projection.
  `classified_content_text` replaces the `.text`-only block renderer for source
  reading; `transcript_message_content_text` stays for reading back what Operon
  itself wrote.
- **The read happens once**, in `import_full_cli_history`. All three
  destinations and all four verification passes receive the same projected list,
  so a destination cannot disagree with the reader about what the conversation
  was. `CodexAppServer::inject_transcript` becomes `inject_turns` and the second
  parse of the source disappears with it.
- **`src/cli.rs`** — `ImportedNativeSession` gains `classification`.
- **`manifest.json`** — `schema_version` 3 → 4, gaining a `classification`
  object. This file is a local archive artifact, not the persisted store.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | no | draws nothing |
| Icons | no | adds no mark |
| Identifier SSOT | **yes** | this is the policy the change is an instance of. The kinds were literals in `src/history.rs` in three readers; they become rows in one table, and the guards are round trips through the readers that consume them, not comparisons against literals the tests typed |
| Durability | partial | `STORE_SCHEMA_VERSION` is untouched: `ImportedNativeSession` is in-memory and `manifest.json` is an archive artifact with its own version, bumped 3 → 4. Writes still go through `write_file_atomically` and the existing temp-file-plus-rename paths |
| Subprocess safety | no | spawns nothing new; the Codex app-server lifecycle is unchanged |
| Documentation | partial | `CLAUDE.md`, `REVIEW.md`, `docs/sdlc/lessons.md`. No `README.*` change, so the three-language rule does not bind |
| Local-first | yes | reads only local files; `scripts/check-transcript-vocabulary.sh` reads the developer's own stores and reports to stdout |
| Budgets | yes | `TRANSCRIPT_FILE_MAX_BYTES` and `TRANSCRIPT_FILE_LINE_LIMIT` still bound the read and still fail closed. Reasoning and unclassified fragments are counted against the same retained-byte budget as everything else |

## Flagged concerns

- **Does carrying reasoning inflate the destination past its context window?**
  — Answered: it may, and that is the accepted cost. Measured at +13 % turns on
  the reference conversation (471 → 532). The class is one column in the table,
  so reverting it is a one-row edit rather than a code change.
- **Should an unknown kind fail the restore closed instead?** — Answered no. The
  industry rule for readers of formats they do not own is to pass unknown fields
  through rather than strip them; failing closed would break every restore on
  the day any of the three CLIs ships a new record type, for a loss the label
  and the report already make visible.
- **Should Claude→Claude write native `thinking` blocks back with their
  signatures?** — Answered no. A stale or mis-serialized signature returns
  HTTP 400 on every subsequent turn and the conversation cannot be recovered
  without deleting the file. Attributed text degrades; a forged block does not.

## Acceptance

- `cargo test --locked` → `test result: ok. 282 passed; 0 failed; 6 ignored`,
  including `the_transcript_vocabulary_is_well_formed`,
  `restore_counts_every_source_record_into_exactly_one_class`,
  `an_unclassified_record_is_restored_and_reported`,
  `every_operational_wrapper_is_stripped_by_the_reader_that_declares_it`,
  `reasoning_reaches_every_destination_as_attributed_text`, and
  `every_content_block_crosses_as_the_kind_of_thing_it_is`.
- `bash scripts/check-transcript-vocabulary.sh` → `all classified`.
- A restore of a real long conversation shows the breakdown in the notice, and
  the destination CLI's own resume replays the reasoning turns.

## Rejected alternatives

- **Add a `thinking` arm to the Claude reader.** Fixes one instance; the next
  record type Claude Code ships is lost the same way.
- **Show "471 of 1165" in the notice.** A raw record ratio is mostly noise —
  about half the records are legitimately not conversation — so it would read as
  alarming when nothing is wrong and unremarkable when something is.
- **Make the real-transcript sweep an `#[ignore]`d test.** Breaches the
  `tests_ignored` band, and hides a developer tool inside the suite.
- **Make the sweep a harness metric with a control band.** CI has no
  conversation files, so it would read 0 for ever: a mechanism that can only
  pass.
- **Adopt AAIF as the interchange format.** The only real candidate is an IETF
  draft that deliberately leaves message-level semantics undefined, which is
  precisely the part this change needs.
