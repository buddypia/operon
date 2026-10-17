# Plan: the review gate stops when normalising the diff fails, and its digests are assigned once

- **Spec**: `./spec.md`
- **Approved**: 2026-10-17
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `scripts/check-review.sh` | `|| { echo "the current diff could not be normalized; is sed working?"; exit 2; }` after each digest substitution; one `if` refusing the empty-input digest when the byte-for-byte one is not |
| `src/tests.rs` | the two pinned lines gain the suffix; the no-bypass test counts assignments; one new test with a `sed` shim |
| `docs/sdlc/changes/162-…/state.yaml` | `lands-158` settled, `resume` rewritten — its own commit, so the gate sees one change directory per commit |

The shim is a `bash` script written into the fixture's own temp directory as
`shim/sed`, made executable, and put first on `PATH` for the gate run. It
looks for `@@` in its arguments — only the normalising call has it — and in
mode `fail` exits 1, in mode `silent` prints nothing and exits 0; any other
call `exec`s `/usr/bin/sed "$@"`. The fixture stages a store edit and records
`e3b0c44298fc1c14` with all three reviewers approving, so a gate that hashed
nothing would read SHIP. Both modes must give exit 2 and the message, and must
not print SHIP.

## Order of work

1. Write the new test and the amended pins and counts; run them against
   today's gate. The new test fails in both modes (today the gate passes the
   empty-digest record, or names the empty digest as current); the pins fail
   because the lines do not have the suffix yet. Watch each fail.
2. Change the two lines and add the empty-digest `if`; the tests pass.
3. Confirm digests did not move: `a_review_survives_main_moving_the_lines_above_it`
   and `a_review_recorded_before_change_162_still_counts` pass, and this
   branch's gate on change 166's landed diff names `d8d2d10447b4e4fe` as before
   (measured with `--commit bef1b7d`, whose first-parent diff is 166's).
4. The 162 record edit, committed alone.
5. `make q.fast`; named tests; mutation sweep.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The digest moves | every review in flight elsewhere is refused | step 3's measurement and the two existing digest tests |
| The `|| exit 2` fires on a healthy run | every commit refused | the whole gate suite in CI; `make q.fast` commits run the gate |
| The shim leaks into the real gate | — | the shim lives in the fixture's temp tree and is on `PATH` only for that one command |
| A second digest assignment slips in | gate compares against something else | the assignment count in `the_review_gate_has_no_bypass` |

## Proof of completion

- `cargo test --locked the_review_gate` and `cargo test --locked a_review_` pass.
- Mutation sweep: drop each `|| exit 2`, drop the empty-digest `if`, add a
  second `digest=` line, add `[ -n "${X:-}" ] && digest=x`, add
  `export legacy_digest=x`, add a `read -r digest` line — each turns a test red.
- `make q.fast` passes.

## Departures from the plan

- **The byte-for-byte digest's refusal says "could not be hashed".** No `sed`
  runs in that pipeline, so the spec's "is sed working?" would point at the
  wrong program. The normalising line and the empty-digest check keep the
  spec's wording.
- **The empty digest is named once in the gate as `empty_digest`.** The test
  writes the same sha256-of-empty literal into its record, since it is a fact
  about sha256 and not a setting either side could change.
- **A new `the_digest_assignment_count_sees_every_form`** runs the counting
  rule against each form the spec names, both ways, so the rule is watched on
  every form rather than only on the four the sweep adds.
- Measured: `bash scripts/check-review.sh --commit bef1b7d` on this branch reads
  SHIP at `d8d2d10447b4e4fe`, as on main. `mutations.py`: 8 of 8 caught.
