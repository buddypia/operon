# Intent: a lesson with no guard, or a guard nobody saw fail, can still be recorded

- **Status**: approved
- **Opened**: 2026-10-21

## Problem

Harness² (`~/dev/oss/harness2`, Apache-2.0) carries what it learns from one
task to the next in a library, and admits an entry only when it names the
situation it applies to, the lesson, and the evidence it rests on; an entry
missing any of them is refused before it is stored, and a lesson resting only
on an edit that was adopted, never shown to have helped, is called a guess.

Here the ledger of lessons says the same in prose: every entry names its
mistake, its rule, and the guard that now catches it, an entry with no guard is
unfinished work, and a guard nobody has watched fail is a guess. A script
counts how many entries name a guard the repository holds — today all 82 do —
but it runs only when somebody runs it. Nothing refuses an entry with no rule,
an entry whose guard names a test that does not exist or was renamed away, or
a new entry whose guard was never seen failing.

## Who feels it, and when

The next session that closes a change and appends a lesson in a hurry, or that
renames a test a lesson names as its guard: the ledger reads as finished, the
count drops from 82 of 82 when someone next runs the script, and nobody is
told at the commit that would have been the cheap time to fix it.

## Desired outcome

The suite refuses a lesson that has no rule, no guard, or a guard naming
nothing this repository holds, saying which entry and what is missing; and from
this change on, a new entry whose guard does not say it was watched failing.
The count the script reports and the suite's verdict cannot disagree, because
the suite asks the script.

The person's request, 2026-10-21: 「~/dev/oss/harness2를 참고해서 하네스 설계를
내 프로젝트에 알맞게 최대한 적용시켜줘.」

## Constraints this change inherits

- Lesson 004: a guard reads the value it guards out of the definition; the
  guard-existence rule stays in `scripts/pipeline-indicators.sh`, not copied.
- Lessons 001 to 080 were compressed by earlier changes and are not rewritten.

## Systems likely affected

`src/tests.rs`, `docs/sdlc/lessons.md` (its header), `docs/sdlc/README.md`.

## Open questions

None.

## Not in scope

- Harness²'s confidence field and its `confounded` verdict: a guard here is a
  test that runs, which is stronger than a confidence a reader must weigh.
- Retrieving lessons by situation at the start of a change: the guard delivers
  the lesson by failing, without being looked up.
