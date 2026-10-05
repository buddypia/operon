# Plan: a finding left unfixed reaches a person in words they can act on

- **Spec**: none — bugfix route; the bug report in `./state.yaml` is the intent.
- **Approved**: 2026-09-26
- **Status**: done — departed from (see below)

## The rule

Every finding carried in `review.yaml` has, beside the reviewer's verbatim
line, a `person:` line in plain Japanese: what goes wrong, in what situation,
and whether the person has to do anything. The review gate refuses a carried
finding without one, and prints those lines — not the reviewer's — when a
change ships. A report to a person is written from them, not in the
harness's vocabulary.

## Shape

```yaml
carried:
  - rust-reviewer/1 · <the finding, as they wrote it>
    person: "<何が・どんな時に起きるか。対応が要るか>"
```

Four-space indent, so every existing reader of `  - ` lines (the carried
block, the SHIP findings list) keeps reading exactly what it read.

## Files that change

| File | Change |
|---|---|
| `scripts/check-review.sh` | refuse a carried item with no following `person:` line, or one with no Japanese in it, or one written in harness words (nit, digest, carried, round, Important); SHIP prints the `person:` lines under a Japanese heading |
| `docs/sdlc/templates/review.yaml` | the `person:` line in the carried example, and why |
| `REVIEW.md` | the carried bullet asks for it |
| `AGENTS.md` | one line: write to a person in their words; a leftover finding reaches them through its `person:` line |
| `.claude/skills/sdlc/SKILL.md` | Report: leftovers from the `person:` lines |
| `src/tests.rs` | the round-ceiling test's carried fixture gains a `person:` line; a new test for the refusal, the jargon refusal, and the SHIP text; a test holding the AGENTS.md sentence |
| `docs/sdlc/lessons.md` | an entry, with its Guard |

## Order of work

1. Commit this plan and `state.yaml`.
2. Tests: a carried item with no `person:` line is refused; one whose line is
   ASCII only is refused; one using "nit" is refused; with a good line SHIP
   prints it and not the reviewer's text. Watch them fail on the unedited
   gate.
3. The gate. Green.
4. The documents and the sentence test.
5. Mutations: drop each refusal; print the verbatim line again; weaken the
   AGENTS.md sentence.
6. Gates, bands, lesson, review.

## What was considered and not done

- **Rewriting past changes' `review.yaml`.** Their carried items stay as they
  are: the gate judges only the change being committed, and a commit touching
  two change directories is refused. The person gets the 083 and 084 leftovers
  explained in the reply that closes this change.
- **An eval for the report's wording.** The mechanical half — the text a
  report is written from exists and is plain — is a test. Whether a session
  then uses it is steering, held by the AGENTS.md line and its test.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The `person:` line breaks the verdict parser | a 4-space `person: "…"` read as a verdict | verdicts are read from `^  [a-z-]+: "` (two spaces); the new test ships a change with one |
| The jargon list refuses honest prose | a real explanation refused | the list is five harness words; the refusal names the word |
| A reviewer's line is dropped from the record | follow-up loses the finding | the verbatim line is still required and still stored; only SHIP's display changes |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- The new test fails on the unedited gate and passes after.

## Departures from the plan

- The Japanese check was first "any non-ASCII byte"; review round 1 showed the ` · ` in every carried id satisfies that. It now looks for kana and kanji lead bytes, and the test pastes a reviewer line to prove it.
