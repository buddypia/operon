# Spec: the review gate stops when normalising the diff fails, and its digests are assigned once

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. In `scripts/check-review.sh`, the `digest=` and `legacy_digest=` command
   substitutions each end in `|| { echo "…"; exit 2; }`. The script runs under
   `set -o pipefail`, so the substitution's status is the pipeline's, and a
   `sed`, `shasum`, or `cut` that exits non-zero — or is not found (127) — stops
   the gate with exit 2 and the message
   `the current diff could not be normalized; is sed working?` (the existing
   git-failure message's form). Test:
   `the_review_gate_stops_when_normalizing_the_diff_fails`, with a `sed` on
   `PATH` that exits 1 when given the normalising expression and defers to
   `/usr/bin/sed` otherwise; a record at the empty-input digest
   `e3b0c44298fc1c14` must not pass.
2. A normaliser that exits 0 and prints nothing is the same failure: when
   `digest` is `e3b0c44298fc1c14` (sha256 of empty input) and `legacy_digest`
   is not, the gate stops with exit 2 and the same message. Test: the same
   function, with a `sed` that prints nothing for that expression.
3. What is hashed does not change: for the same diff, `digest` and
   `legacy_digest` are what they were. Test: the existing
   `a_review_survives_main_moving_the_lines_above_it` and
   `a_review_recorded_before_change_162_still_counts` pass unchanged, and
   `the_review_gate_has_no_bypass` pins the two new lines exactly.
4. `the_review_gate_has_no_bypass` counts assignments: exactly one code line
   of the gate assigns `digest` and exactly one assigns `legacy_digest`. A line
   assigns a name when, anywhere in it (trimmed, comments already dropped by
   `shell_code`), the name followed by `=` or `+=` is not preceded by a letter,
   digit, or `_` — so `[ c ] && digest=…`, `then digest=…`, `export digest=…`
   count, and `recorded_digest=` does not — or when the line names it as the
   target of `read`, `printf -v`, `declare`/`local`/`typeset`, `mapfile`, or
   `for … in`. Test: `the_review_gate_has_no_bypass` (amended).
5. Change 162's `state.yaml` records `lands-158` as `human passed`, citing the
   landing merge 0252b26 after 162's fc3abcc and the record moved to the
   normalised digest without a further review, and its `resume` no longer says
   it is pending. No test; a docs-only commit in this branch.

## Behaviour

- Normal runs: no output change, no digest change.
- Broken normaliser: exit 2,
  `the current diff could not be normalized; is sed working?` on stdout, the
  same channel as the existing message beside it. `.githooks/reference-transaction`
  and `.claude/hooks/gate-commit.sh` already refuse on exit 2.

## Design

Placement verdict: **EXTEND** — the change sits inside the existing digest
lines of `scripts/check-review.sh` and the existing no-bypass test. Route
`security`, risk high, autonomy paused (`gate-configuration`). It tightens only:
a refusal is added, nothing is admitted that was refused.

- `scripts/check-review.sh`: the two assignments gain `|| { …; exit 2; }`; one
  `if` after them compares against `e3b0c44298fc1c14`.
- `src/tests.rs`: `REVIEW_DIGEST_LINE` and `REVIEW_LEGACY_DIGEST_LINE` gain the
  suffix; the no-bypass test gains the assignment count; one new test drives
  the gate in the existing `ReviewGateFixture` with a `PATH` shim.
- No persisted shape, no new child process in the app.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Gate configuration — relaxation goes to a person, tightening to the evaluator (`approval.md`) | Yes | A refusal added; nothing relaxed. Paused surfaces start with a person; the person asked for this follow-up. |
| Review — `REVIEW.md`: gate-configuration needs subprocess-safety-reviewer | Yes | Requested at stage 5. |
| Subprocess safety — new children go through `src/exec.rs` | No | The test runs `bash` through the existing fixture helper, as its neighbours do. |
| Identifier SSOT | Yes | The empty digest literal is the sha256 of empty input, named once in the gate with a comment; the test reads it from there. |
| Durability, Local-first, Budgets | No | Nothing persisted, no network, no scan. |

## Flagged concerns

None open.

## Acceptance

- `cargo test --locked the_review_gate` and `cargo test --locked a_review_`
  pass.
- `bash scripts/check-review.sh` on this change's own diff reads SHIP once
  reviewed.

## Rejected alternatives

- Capturing the normalised diff in a variable first — `$(…)` strips trailing
  newlines, which changes every digest and sends every review in flight back.
- Checking `PIPESTATUS` — inside `$(…)` it is the subshell's, and `pipefail`
  already gives the substitution the pipeline's status.
