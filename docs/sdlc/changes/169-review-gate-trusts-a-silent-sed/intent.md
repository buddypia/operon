# Intent: the review gate trusts a normalising step whose failure it cannot see

- **Status**: approved
- **Opened**: 2026-10-17

## Problem

The person's words, verbatim: 「残タスクもやって」 — said of the work change 162
left for a follow-up, two points of which its review named:

- When the step that normalises a diff before hashing fails — the program is
  missing, crashes, or prints nothing — the review gate does not notice. It
  hashes empty input and goes on judging as if that were the change. A review
  record carrying the digest of empty input would then be fresh for every diff.
- Nothing stops a later edit from assigning the gate's digest a second time,
  after the pinned line, from something that is not the diff. The pinned line
  stays exactly as it is, every guard stays green, and the gate compares a
  record against whatever that second assignment says.

Separately, change 162's record still lists "158 lands with no further review"
as pending, although 158 has landed that way. The record is wrong about the
past.

## Who feels it, and when

- Anyone whose machine runs the gate with a broken or shadowed `sed` on `PATH`:
  the refusal that should say "the diff could not be read" does not come.
- Whoever edits `scripts/check-review.sh` next: a reassignment of the digest
  passes every test.
- A session reading change 162's record cold, which is told an outcome is still
  open.

## Desired outcome

- When normalising the diff fails or yields nothing for a diff that is not
  empty, the gate stops with exit 2 and says the diff could not be read, as it
  already does when `git diff` itself fails. No verdict is judged.
- A second assignment of either digest anywhere in the gate fails the suite.
- Change 162's record shows its last open item settled.
- Every digest already recorded stays valid: what is hashed does not change.

## Constraints this change inherits

- `scripts/check-review.sh` is a `gate-configuration` surface: this change only
  tightens it.
- The digest must stay byte-for-byte what it is today, or every review in
  flight in another worktree is sent back.

## Not in scope

- The other twelve points change 162 left, which its record says need nothing.
