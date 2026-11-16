# Plan: person approvals recorded without the person's words

- **Spec**: n/a (`bugfix` route — the bug report is in `./state.yaml`)
- **Approved**: 2026-11-16
- **Status**: approved

## Root cause

`.claude/skills/sdlc/references/approval.md` defines the ledger line's fields
but not what a `person` line's reason must hold, and `scripts/check-approvals.sh`
checks only that a reason exists. A `person` line is written by the agent on the
person's behalf, so it is the one line whose truth the ledger cannot show
unless it carries what the person said. Of the 71 ledgers with `person` lines,
some carry no words of the person at all, and approvals read out of the request
the work started from are never shown back to the person.

## The fix

- `approval.md`, "The ledger": a `person` line's reason quotes the person's
  own words in 「…」 (or "…") — the answer they gave, or the request that asks
  for exactly what the artifact does — and says why those words cover the
  artifact. A request counts only for what it asks: one that does not name a
  relaxation does not approve it, and the person is asked instead. When an
  approval is read from a request rather than given as an answer, the session's
  final report says so in the person's words, so they can take it back.
- `scripts/check-approvals.sh` refuses a `person` line without a quote, from
  change 300 on. The 71 earlier ledgers are left as written: the rule did not
  exist, and a ledger line is a record, not something to rewrite.

The edit tightens a check and relaxes none.

## Files that change

| File | Change |
|---|---|
| `scripts/check-approvals.sh` | a `person` line from change 300 on carries a quote |
| `.claude/skills/sdlc/references/approval.md` | what a `person` line holds, and when a request counts |
| `src/tests.rs` | `the_approval_checker_wants_the_persons_words_on_a_person_line`; fixtures given quotes |
| `docs/sdlc/changes/300-…/` | state, plan, approvals, mutations, verification |

## Order of work

1. The test, watched failing on the unedited checker.
2. The checker, and the rule in `approval.md`.
3. The existing approval tests (`approval`), whose fixtures numbered from 300
   gain quotes.
4. `mutations.py`: dropping the check, applying it below 300, accepting a
   single quote mark, and deleting the rule's sentence from `approval.md` each
   turn a test red.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| An old ledger is refused | every stop and commit blocked | the cut at 300; the test's 299 ledger without a quote passes |
| A quote check satisfied by one stray mark | an unquoted line passes | the check wants an opening and a closing mark |
| The rule in prose drifts from the check | sessions write what the checker refuses | the test reads the sentence out of `approval.md` |

## Proof of completion

- `cargo test --locked approval` passes; the new test failed before the edit.
- `python3 docs/sdlc/changes/300-…/mutations.py` — every mutation caught.

## Departures from the plan

- **Three fixtures gained quotes.** The width, escape and paused-surface tests
  wrote `person` lines numbered from 900 with no words of the person; each now
  quotes a fixture reply, which is what the rule asks of a real line.
- **The quote is found with `index()`.** The checker's awk reads bytes, so a
  bracket class built from 」 would also refuse the bytes of the words inside
  the quote; `index()` looks for the two brackets as strings.
- **`steering_bytes` reads 264,424 against a diagnose tier of 264,000.** The
  rule added about 800 bytes to `approval.md`, which loads only when an
  approval is about to be written. The band asks for a read-only look, not a
  cut, and nothing here was moved to make room.
- **The empirical verifier, run in an isolated worktree (change 299), broke
  it once.** A tab before the quote ended the field the checker read, so a
  quoted line was refused (F1). The checker now reads the line past its two
  leading fields. It also noted that an empty 「」 or "" passed; an empty
  quote holds no words, so it is refused too (F2). Both are cases of
  `the_approval_checker_wants_the_persons_words_on_a_person_line`, watched
  failing on the first version of the checker, and three more mutations pin
  them (11 of 11 caught).
