# Plan: a re-verdict after main moved can be committed on its own

- **Spec**: none — bugfix route; the bug report in `./state.yaml` is the intent.
- **Approved**: 2026-10-21
- **Status**: done

## The rule

`scripts/check-review.sh` refuses a commit that changes `review.yaml` and no
code, because "a verdict committed apart from the code it judged is a record in
HEAD describing code that is not there". That is true of a verdict about some
other diff. It is not true of a verdict whose digest is the diff the branch now
holds against `main` — the diff the merge into main will judge, and the one the
reviewers re-confirmed. That record describes code that is there.

So a commit of `review.yaml` alone is admitted when the digest it records equals
the digest of the diff from `refs/heads/main` to what the commit lands, taken
the way the gate takes every digest (same paths, same normalisation), and that
diff is not empty. Every other record-only commit is refused as before.

The landing gate is unchanged: the merge into main still judges its own diff
against the record, so nothing lands on a verdict that is not about it.

The stale-verdict refusal gains one sentence naming that exit, so a session
that merged main in before recording the re-verdict reads how to record it
instead of reaching for an amend, a reset or a force push.

## Why

Change 183 merged main into its reviewed branch; the merge forced one edit (a
lesson renumbered because main took the number) and moved the digest. The
re-verdict could no longer ride in the merge commit, which was already made, and
every other way to record it was refused: a record-only commit by this gate, an
amend by the commit guard, a reset by the destructive-git guard, a new branch by
the branch guard. The one way left was rewriting a pushed branch, which goes to
a person. REVIEW.md already says such a re-verdict is not a round and asking a
person about it is asking about a step another review undoes (sdlc 084); the
gate made it one.

## Files that change

| File | Change |
|---|---|
| `scripts/check-review.sh` | the record-only branch admits a record whose digest is the branch's diff against main; the digest pipeline becomes one function both use; the stale refusal names the exit |
| `src/tests.rs` | `the_review_gate_refuses_what_it_says_it_refuses`: on a branch off main, a record-only commit at the diff against main passes, at any other digest is refused, and on main itself (an empty diff) is refused |
| `REVIEW.md` | the re-verdict paragraph says how it is recorded |
| `docs/sdlc/lessons.md` | an entry, with its Guard |

## Order of work

1. Commit this plan, `state.yaml`, and `approvals.log`.
2. Test cases; watch them fail on the unedited script.
3. The script. Green.
4. Mutations: the comparison dropped (any record alone refused again); the
   comparison made against the commit's own base instead of main; the empty-diff
   check dropped; the refusal no longer naming the exit (the claim).
5. Gates, lesson, review (subprocess-safety-reviewer for gate-configuration,
   rust-reviewer for the test).

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A record about other code lands alone | a verdict in HEAD for a diff nobody reviewed | the "any other digest" case; the landing gate re-judges anyway |
| A record of an empty diff is admitted and stays fresh | a branch equal to main carries an approval of nothing | the on-main case |
| `refs/heads/main` is behind the remote | the digest the session reads differs from the one landing computes | landing judges against the same local main the merge is made on; a mismatch is a refusal, never a pass |
| The script's own copy on main judges this branch | the new exit does not exist until this lands | expected: the gate reads main's copy, so the exit works from the first branch after this one |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — clean.
- `cargo test --locked the_review_gate_refuses_what_it_says_it_refuses` — fails
  on the unedited script, passes after.
- The whole suite in CI on the push.
- Change 183 records its re-verdict with a record-only commit once this is on
  main.

## Departures from the plan

1. **Review round 1 found the planned exit skipped every check on the record**
   (subprocess-safety-reviewer, Important): a record-only commit at the right
   digest with no verdicts, a do-not-approve, or rounds past the ceiling would
   have landed, and a `--ff-only` landing makes no merge commit to re-judge it,
   so "landing re-judges" in Risks was wrong. The exit is gone. A commit that
   changes a `review.yaml` and no code now has its base set to `refs/heads/main`
   before the digest is taken, so the digest, the reviewer set and every verdict
   check run on the branch's whole diff through the gate's ordinary path; on
   main that diff is empty and the record is refused as before. The tests gained
   the no-verdict, do-not-approve and other-digest cases; the sweep is three
   mutations (the switch removed, inverted, and the claim).
2. A test outside this change's guard, `the_review_gate_has_no_bypass`, pins
   every line that sets the base; it pins the new one.
3. **CI on the reviewed head failed `a_scrub_may_edit_closed_records_only_to_lose_a_listed_name`**:
   a scrub edits several closed changes' records and no code, so the switch
   took it and judged it against main instead of by its own rules. A re-verdict
   touches one change directory, so the switch now also requires exactly one;
   the scrub test joined the sweep's checks, with a fourth mutation (several
   directories admitted).
4. **CI on round 3's head failed two harness tests**: the count `grep -c .)" -eq 1`
   read to `every_gate_script_reads_its_pipes_to_the_end` as a `grep -q` (its
   flag scan does not stop at `)`, so `-eq` carries a `q`), and `stage: "done"`
   is not a stage. The count is now a `case` on the newline-separated list (no
   pipe), the fourth mutation follows it, and the stage is `test` as every
   closed bugfix's is.
5. **The landing merge was refused as stale**: every recorded digest had been
   taken with `--index` against HEAD, one round's increment each, while the
   merge into main judges the whole branch. Both reviewers re-read the landing
   diff and stood by their verdicts; the record rides a merge made from main's
   tip, as feature-pilot says. Once this lands, the same re-verdict is a commit
   of `review.yaml` alone.
