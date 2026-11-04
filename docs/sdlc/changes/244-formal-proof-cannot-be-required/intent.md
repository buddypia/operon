# Intent: a proof can be asked for nowhere, because nobody measured whether one runs

- **Status**: approved
- **Opened**: 2026-11-04

## Problem

Since change 242 a change lands on evidence that is run again, and the
evidence table names two formal methods: an exhaustive walk of a finite model,
and a machine-checked proof. The second could not be required of any change,
because nobody knew whether the proof tool builds this application at all —
it carries a native C++ library and a GUI toolkit — or how long it takes. So
the launch gates that stand between a typed command and a shell are checked
by examples and a bounded enumeration, and never proved.

## Who feels it, and when

Whoever next changes the launch gates: the newline hole change 241 fixed was
found by the first enumeration, not by any review, and a proof over every
input would have found it without choosing an alphabet first.

## Desired outcome

One measured answer — does it build, does it prove, how long — and, if it
does, a change to the launch gates cannot land without the proof passing
again in CI.

## Constraints this change inherits

- The proof tool is never a dependency of the application: CI only.
- CI already runs five macOS jobs for each branch update, the account's ceiling.

## Systems likely affected

The evidence table, the risk table, the commit gate's record check, the CI
`verify` job, and one harness in the test suite.

## Open questions

- Does it build and prove in the time an obligation is allowed (900 s)?
  Answered by the probe: yes, 419 s.

## Not in scope

Proofs of anything other than the launch gates; a proof obligation for this
change itself (the table that judges it is main's).
