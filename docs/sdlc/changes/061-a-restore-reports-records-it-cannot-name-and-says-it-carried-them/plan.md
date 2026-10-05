# Plan: a restore reports records it cannot name, and says it carried them

- **Spec**: none — `bugfix` skips intent and spec, and the measurements below are
  the bug report.
- **Approved**: 2026-09-20
- **Status**: done

## The cause, named

Two defects, found together and separable. Both are in the accounting a person
reads after a cross-CLI restore — the restore feature, on the screen
where its result is reported.

**1. `codex/Record/token_usage_record` has no row in `TRANSCRIPT_VOCABULARY`.**

`bash scripts/check-transcript-vocabulary.sh` exits 1 today:

```
vocabulary: 1 of 67 observed kinds have no row in TRANSCRIPT_VOCABULARY:
  codex	Record	token_usage_record
```

It is not hypothetical and it is not rare. Measured against this machine's own
`~/.codex/sessions`: **32 rollouts carry it, 4720 records in total, 2385 of them
in a single thread.** A person restoring that thread is told Operon met 2385
records it cannot name.

What the record actually holds, read from the largest of them:

```json
{"timestamp":"…","ordinal":13,"type":"token_usage_record",
 "payload":{"thread_id":"…","turn_id":"…","session_id":"…","response_id":"…",
            "usage":{…},"turn_token_usage":{…},"thread_token_usage":{…}}}
```

Thread and turn identifiers and three token-count objects. No message, by either
participant. The vocabulary already has this exact thing for another CLI —
`kind("claude", Record, "cost-state", Operational, "accumulated spend for this
session")` — so the classification is `Operational` and the judgement is not a
close one.

**2. The notice says the content crossed. For this kind, nothing crossed.**

`restore_classification_notice` in `src/history.rs:381` appends, whenever
`has_unclassified()`:

> Operon が分類を知らない記録がありました: {kinds}。**内容は復元先へ渡していますが**、
> src/transcript.rs の TRANSCRIPT_VOCABULARY に分類を追加してください。

But `count_unclassified_kind` is called on two different arms of
`codex_transcript_record`, and only one of them carries anything:

```rust
None => {
    summary.count_unclassified_kind(provider, record_type);
    return match record_text(value) {
        Some(text) => unclassified_turn(provider, record_type, &text),  // carried
        None => RecordOutcome::Unclassified,                            // dropped
    };
}
```

`record_text` looks for `message`, `content`, `text`, `summary`, `tools`, or
`output` **at the record's top level**. A `token_usage_record` has `timestamp`,
`ordinal`, `type`, `payload` — none of them. So every one of those 2385 records
takes the second arm, produces no turn, and is then reported under a sentence
that says its content was handed to the destination. The sentence is false for
the only unclassified kind this machine actually has.

This is the durable half. Defect 1 is one row and a new kind will appear again
the next time Codex ships one; defect 2 is wrong for **every** unknown kind with
no readable text, permanently, and it is wrong in the direction that reassures.

**Why it was never seen.** `an_unclassified_record_is_restored_and_reported`
covers the notice, and both of its unknown kinds have readable text:
`telemetry-batch` carries `message.content`, `citation` carries `text`. Both take
the carried arm. The dropped arm has no fixture *in that test*, so the sentence
that is false only on that arm has never been read against it. This is the same
shape as change 060's round-1 finding 3 — the branch every existing fixture
stepped past — one module over.

**3. The printed notice names four of the six buckets.** Narrower than it first
looked, and the narrowing matters: the *struct's* arithmetic does close, and
`restore_counts_every_source_record_into_exactly_one_class` already asserts
`records == restored + operational + unclassified + empty` over a fixture that
includes a `nothing-known` record with no readable text. So the accounting is
right and tested. What is wrong is what reaches the person:
`restore_classification_notice`'s first sentence prints `records`, `restored`,
`reasoning_fragments`, and `operational`, and never `unclassified` or `empty`.
The function's own doc comment says "The point is the arithmetic, not the
reassurance" and that naming the excluded records "is what makes an unexpected
number visible as unexpected" — and then 2385 records go unnamed on screen. The
fix is in the sentence, not in the counters.

## Why this was invisible

`scripts/check-transcript-vocabulary.sh` does its job — it exits 1 and names the
kind. It is not wired to any gate, deliberately and correctly: it reads this
machine's own `~/.codex` and `~/.antigravity`, so a CI caller could only ever
pass, which is lesson 007. `CLAUDE.md` lists it under "what the gates do not
cover" with its healthy output. Nobody ran it. That is the nature of a
by-hand check and not an argument for wiring it; the answer is in the lesson
below, not in a new hook.

## Files that change

| File | Change |
|---|---|
| `src/transcript.rs` | one row: `kind("codex", Record, "token_usage_record", Operational, …)`, in sorted position — `transcript_class` binary-searches the table |
| `src/history.rs` | `RestoreClassification` learns to count an unclassified kind as carried or dropped; `restore_classification_notice` says which, and closes its arithmetic |
| `src/i18n_tables.rs` | the new and changed Japanese message ids |
| `src/tests.rs` | the tests below |

## Order of work

1. The failing test first: a fixture holding a real-shaped `token_usage_record`
   plus two ordinary turns, asserting the notice names no unclassified kind.
   Watch it fail naming `codex/token_usage_record`.
2. Add the vocabulary row. Watch the test pass and
   `bash scripts/check-transcript-vocabulary.sh` reach exit 0.
3. The second failing test: extend the dropped arm into the notice's coverage —
   a fixture with an unknown kind that has readable text beside one that has
   none, asserting the notice reports them as different things. Watch it fail;
   today one sentence covers both and says both crossed.
4. Split `unclassified_kinds` into carried and dropped, and rewrite the notice
   around it. The tree does not compile between 3 and 4 — the test names a
   counter that does not exist yet — which is the one step in this order where
   it cannot, and is why it is one step.
5. Print the rest of the arithmetic: the notice names `unclassified` and `empty`
   too, so the printed counts account for every record read. The struct already
   balances and is already tested; only the sentence changes.
6. `mutations.py`, one mutation per claim, each watched failing.
7. `docs/sdlc/lessons.md`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The vocabulary table is binary-searched; a row out of sorted order silently answers `None` for kinds near it | Restores start reporting kinds that have rows | Already covered: `the_transcript_vocabulary_is_well_formed` asserts strict ordering across every layer AND looks every row up through `transcript_class` itself, so a misplaced row fails on its own line. Checked, not assumed |
| Classifying `token_usage_record` as `Operational` is a judgement about a format Operon does not own | If Codex ever puts a message in it, a real turn is dropped silently | The reason string in the row, and the fact that `Operational` is what the identical `claude/cost-state` gets. A restore that loses a turn shows as a count that does not match |
| Splitting the counter touches the one struct every restore destination reads | A destination reads the old field and reports zero | The compiler, for the field rename; the tests for the meaning |
| The notice is user-facing Japanese and every id needs a row in every table | `src/i18n_tables.rs` falls out of step | The existing test that every `tr`/`tf!` id has a row in every language table |
| 89 message ids in `src/history.rs` are already untranslated (open, unrelated) | Not this change's; do not silently absorb it | Left alone and still recorded as open |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-bands.sh` — exit 0.
- `bash scripts/check-transcript-vocabulary.sh` — **exit 0**, `67 kinds observed
  locally, all classified`. It exits 1 before this change; that transition is the
  proof, and it is the one measurement here taken against real local data rather
  than a fixture.
- `cargo test --locked -- --ignored` — AGENTS.md asks for it when `src/history.rs`
  is touched. Three of the six are known to fail for causes outside this change
  (the two Antigravity fixtures assert `imported_records == 4`; a restored Codex
  thread opens empty). Record the before and after so this change is not credited
  with them and not blamed for them.
- The new tests, each watched failing before the fix.

## Departures from the plan

**Step 3 was written so that it compiles.** The plan said the tree would not
compile between steps 3 and 4 because the test would name a counter that did not
exist yet, and called that the one unavoidable place. It was avoidable: the
claim is about the *notice*, which is a `String` either way, so
`a_kind_that_crossed_and_a_kind_that_was_dropped_are_reported_as_different_things`
was written against the notice alone, compiled against the unfixed tree, and was
watched failing with the defect printed in the assertion message — one sentence
covering both kinds and saying `内容は復元先へ渡していますが`. The struct-level
assertions were added after step 4. Strictly better: the plan's version could
only have been watched not-compiling.

**A fourth test, because a mutation was not caught.** `mutations.py` started at
nine. `carried-role-called-dropped` — an unknown *role*, which does not stop the
message, so its turn crosses and it is `Carried` — passed every existing test,
because no fixture anywhere had an unknown role. That is the same finding as the
one this change is about, one arm over, so it became
`an_unknown_role_is_reported_as_carried_because_its_turn_crosses` rather than a
dropped claim. `met-forgets-the-dropped` was uncaught for the same reason until
the carried/dropped/`met` triple was asserted on a kind that took the dropped
arm.

**The manifest's `schema_version` went to 5**, which the plan did not mention.
`unclassified_kinds` stopped being `{kind, count}`; `count` is kept and still
means "met", with `carried` and `dropped` beside it. Nothing in this crate reads
a manifest back, so this is a paper-trail shape and not a migration — but a
reader of an old manifest has to be able to tell that its counts cannot answer
which arm a record took.

**`src/i18n_tables.rs` was not touched.** The plan listed it. The notice's
message ids are among the 89 `src/history.rs` ids that have no row in either
table, so `catalog_keys()` — the union of `EN_TABLE` and `KO_TABLE` — never held
the old ids and `every_message_id_has_a_row_in_every_table` has nothing to say
about the new ones. Adding rows here would have been the start of that separate
backlog, which the plan's own risk table says to leave alone. Still open.

**The three `--ignored` failures were measured on both sides**, rather than
carried over from the plan's note. `docs/sdlc/changes/061-*/state.yaml` records
the numbers: the same three fail before and after, with the same messages.

**Three review rounds, against a ceiling of two, and the two extra rounds are
the most useful thing in this change.** Rounds 1 and 2 each returned an
Important, and both were the same defect this change exists to fix, written
again by the fix for it, at the two `Carried` sites `unclassified_record` could
not absorb: first a sentence promising the `[Unclassified]` heading that an
unknown flag and an unknown role never attach, then — after that was corrected —
the weaker claim still being made before the record had been read and could
still come back `Empty`. The guard set went from four tests to six and the
mutations from ten to fifteen because of them.

Mutation testing caught neither, and the reason is worth the line: every
mutation asked whether a *count* moved, and neither defect was in a count. The
plan's step 6 treats `mutations.py` as the thing that verifies a guard, which it
is — but a claim nobody wrote a guard for has no mutation either, and that is
what review is for. Recorded in lesson 041.

The round ceiling is in `docs/sdlc/review-rounds.yaml` and `check-review.sh`
refuses `rounds > ceiling` only when `carried:` is empty. Its rationale, in its
own comment, is the loop a *nit* buys — change 060's nineteen rounds. Neither
extra round here was bought by a nit; both were forced by an Important, which
the gate's own refusal text says to fix rather than carry. The ceiling still did
its job: it is why each round was counted out loud rather than drifting.
