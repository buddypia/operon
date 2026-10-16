# Plan: a review survives unrelated work landing on main, and change numbers stop colliding

- **Spec**: `./spec.md`
- **Approved**: 2026-10-17
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `scripts/check-review.sh` | normalised `digest=`, `legacy_digest=`, staleness accepts either; a comment saying why |
| `scripts/check-change-number.sh` | new: refuses a change number another branch or change already holds |
| `Makefile` | `wt.new` runs the number check first |
| `src/tests.rs` | `a_review_survives_main_moving_the_lines_above_it`, `a_review_recorded_before_change_162_still_counts`, `a_new_change_cannot_take_a_number_already_in_use`; `the_review_gate_has_no_bypass` pins the `digest=`, `legacy_digest=` and comparison lines, and its `narrowing` filter allows exactly those two assignments |
| `.claude/skills/feature-pilot/SKILL.md` | numbering across branches; the merge-main recipe |
| `.claude/skills/sdlc/SKILL.md` | the numbering line |
| `docs/sdlc/templates/review.yaml` | the digest recipe |
| `docs/sdlc/lessons.md` | lesson 073 with its guards |

## Order of work

1. Write the three new tests against today's scripts and watch them fail for
   their reason: the line-shift digest differs today; the legacy test asserts
   the normalised digest is not the byte-for-byte one, which is false today;
   the number check does not exist.
   The number test is written one arm per rule (an in-use number refused with
   holder and next free number, a same-slug pass across `fix/` and `feature/`,
   a no-number pass, a holder seen only as a remote branch, an unreachable
   `origin` warning yet still refusing), in a temporary repository with a bare
   `origin` and copies of `scripts/check-change-number.sh` and
   `scripts/run-bounded.sh`. `mutations.py` breaks each rule in turn and
   watches its arm fail.
2. Change `scripts/check-review.sh`; the two gate tests pass. The line-shift
   test's invalid-byte arm (a `\xff` line, then a later edit) is watched
   failing with `LC_ALL=C` removed under `LANG=ja_JP.UTF-8`. Amend
   `the_review_gate_has_no_bypass` to pin the `digest=`, `legacy_digest=` and
   comparison lines, and turn its `narrowing` filter's substring allowance
   (`digest=$(run_diff`) into an exact-line allowance for the two pinned
   assignments. Watch it fail on an extra expression in the `sed`, on a third
   digest accepted by the comparison, on a third assignment such as
   `other=$(run_diff | head -n 5 | shasum -a 256)`, and on `LC_ALL=C` removed
   from the `digest=` line. The exact-line pin is what guards `LC_ALL=C`
   everywhere: the invalid-byte arm only fails where BSD `sed` runs under a
   UTF-8 locale, which macOS has and Linux CI may not.
3. Write `scripts/check-change-number.sh` and wire it into `Makefile`; its test
   passes.
4. Docs and the lesson.
5. `make q.fast`; the named tests; `bash scripts/check-bands.sh`; the mutation
   sweep in `mutations.py`.

## 162's own record

Until this lands, main's copy of the gate judges every commit — this branch's
and the landing merge's. So 162's `review.yaml` records the byte-for-byte
digest the old gate computes; the new gate accepts that digest too, by
requirement 3.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The `sed` also rewrites a content line | a digest stable across a real edit | `a_review_survives_main_moving_the_lines_above_it` asserts a different edit and an edit to a context line both move the digest |
| The `sed` stops at a byte that is not UTF-8 | later hunks stop moving the digest | `LC_ALL=C`; the invalid-byte arm of the same test |
| In-flight records in other worktrees are voided when this lands | another session's commit refused with 「判定が古い diff」 | `a_review_recorded_before_change_162_still_counts` |
| The normalisation becomes a place to narrow the digest further | a later edit adds expressions | `the_review_gate_has_no_bypass` pins the exact `digest=` line |
| The number check blocks offline work | `make wt.new` fails without network | the fetch is bounded and its failure only warns; the test runs with an unreachable `origin` too |
| The number check refuses resuming a change | a re-cut branch of the same change refused | same-slug holders pass; asserted in the test |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked a_review_survives_main_moving_the_lines_above_it` — 1 passed.
- `cargo test --locked a_review_recorded_before_change_162_still_counts` — 1 passed.
- `cargo test --locked a_new_change_cannot_take_a_number_already_in_use` — 1 passed.
- `cargo test --locked the_review_gate` — all pass.
- `bash scripts/check-bands.sh` — within bands.
- After landing: change 158 merges main, stages its record at the normalised
  landing digest in that merge commit, and lands.

## Departures from the plan

None yet.
