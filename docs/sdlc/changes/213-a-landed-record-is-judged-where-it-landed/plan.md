# Plan: a landed record is judged against the main it landed on

- **Spec**: none — bugfix route; found while landing changes 209 and 210.
- **Approved**: 2026-10-22
- **Status**: approved

## The defect

`scripts/check-review.sh` judges a commit that changes one change directory's
`review.yaml` and no code against `refs/heads/main` (change 190): the
re-verdict a branch commits after taking main in is checked at the branch's
whole diff against main. That holds while the branch is unlanded. Once it is
merged, `refs/heads/main` keeps moving, and the stop gate
(`.claude/hooks/gate-stop.sh`) re-judges every commit since a branch's audit
mark with `--commit`. On a later branch, 208's record-only commit `0b32a27`
was judged against a main that already held 209 and 210, so its diff carried
their code, its digest was stale, and the stop gate refused it. The commit was
right; the base was wrong. It was cleared by moving the audit marks by hand.

A second, smaller defect showed in the same refusal: git 2.56 deprecates
grafts and prints `hint: Support for <GIT_DIR>/info/grafts is deprecated` on
every git call made with `GIT_GRAFT_FILE` set. Seven scripts export
`GIT_GRAFT_FILE=/dev/null` (change 008's guard against a forged parent), so
every gate message is buried under repeated hints.

## The fix

1. In `--commit` mode, when the record-only commit is already an ancestor of
   `refs/heads/main`, the base is the newest commit on main's first-parent
   line that the record already contains: the oldest first-parent commit `X`
   not reachable from the record, and its parent `X^1`. When the branch was
   up to date at its merge, `X` is the merge itself and `X^1` is the main the
   merge gate and the git hook judged it against; when main moved after the
   branch last took it in, `X^1` is the main the branch took in, which is the
   diff its reviewers read. Either way the record is checked at the diff it
   was reviewed at. An unlanded commit, `--index`, and `--against` keep
   `refs/heads/main`. A record committed on main's own line gets `X^1` equal
   to itself; one at main's tip has no `X` and keeps `refs/heads/main`; both
   give an empty diff and the refusal they get today.
2. `export GIT_ADVICE=0` beside `export GIT_GRAFT_FILE=/dev/null` in the seven
   scripts. It silences git's advice hints only; errors and the graft guard are
   unchanged.

## Files that change

| File | Change |
|---|---|
| `scripts/check-review.sh` | the landed base, and `GIT_ADVICE=0` |
| `.claude/hooks/gate-merge.sh`, `gate-stop.sh`, `guard-stage.sh`, `.githooks/reference-transaction`, `.githooks/installed/reference-transaction`, `scripts/pipeline-indicators.sh` | `GIT_ADVICE=0` |
| `src/tests.rs` | the landing test merges the re-verdict into main with `--no-ff`, lands another change on main, and expects `--commit` on the re-verdict to still ship; the graft-file test requires `GIT_ADVICE=0` before the first git call, and covers `pipeline-indicators.sh` |
| `docs/sdlc/lessons.md` | entry 098 |

## Order of work

1. This plan and `state.yaml`; approvals recorded.
2. Tests, watched failing on the unedited scripts.
3. The fix. Green.
4. Mutations: base back to `refs/heads/main`; base `X` instead of `X^1`;
   drop the ancestor check (an unlanded record would be judged against the
   wrong main); drop the fallback when there is no `X`; drop `GIT_ADVICE=0`
   from one script. The test also commits a record alone at main's tip and
   expects `--commit` to refuse it.
5. Gates, review.

## What this is not

- **A relaxation of what lands.** Nothing reaches main differently: the merge
  gate and the git hook judge a branch before it lands exactly as before. Only
  the re-judgement of a commit main already holds changes, and it now asks
  the question it was asked at landing instead of one nobody could answer.
- **A way to claim a landing.** The landed base applies only when the commit
  is already an ancestor of `refs/heads/main`, and a commit gets there only
  through the merge gate and the git hook, which judged it against
  `refs/heads/main` as before. A record not yet on main, on any branch, keeps
  `refs/heads/main`. `--index` and `--against` are untouched: the new branch
  sits inside `[ "$mode" = commit ]`.
- **A different question when main moved before the merge.** If main gained
  commits between the branch's last merge of main and the merge, `X` is the
  first of them, not the merge, and the base is the main the branch took in:
  the branch's own diff, with nothing of main's later work in it, reversed or
  otherwise.
- **Moving the audit marks.** The hand fix stays; with this change the next
  landed record-only commit does not need it.

## Departures from the plan

- A record committed alone at main's tip is not refused under `--commit`
  today: its own diff names no change directory, so it reads as a commit with
  none and passes on an empty diff. The test asserts that unchanged answer,
  which is what the fallback keeps; it is not this change's to alter.
- No separate fallback when there is no `X`: `git rev-parse --verify` refuses
  a bare `^1`, so the assignment does not happen and main stays the base. A
  guard for it changed nothing a test could see, so it is not there, and the
  mutation that would drop it has nothing to drop.
- The test also covers a record not yet landed while main moved: judged
  against today's main and refused as stale, which is what the ancestor check
  keeps.
