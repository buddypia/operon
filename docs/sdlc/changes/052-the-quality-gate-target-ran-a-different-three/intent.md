# Intent: the quality-gate entry point runs a different three than the gate that refuses commits

- **Status**: approved
- **Opened**: 2026-09-16

## Problem

There is one list of checks this repository runs before anything ships, and it
is copied into every place that has to name it. A recent update added a new place
that names it — a runnable one — and wrote a different list into it: the same
three tools, but with the flag that keeps them off the network dropped and the
set of code they cover widened. The command reports a pass over a build the
gate that actually refuses commits would refuse, and refuses builds that gate
would take.

The list has a test that keeps its copies identical. The test enumerates the
places by name, so the new place was outside it from the moment it existed. The
suite stayed green. Two reviewers read the diff that introduced it and saw
nothing; the pass that found it came in after the change had landed.

## Who feels it, and when

A session or a person who runs the repository's own quality-gate entry point
before committing, takes its pass as the answer, and is then refused by the
commit gate for something the entry point never checked — or, in the direction
that leaves no trace at all, is not refused, because the entry point updated a
stale lockfile in place and reached the network to do it, which is the one
promise this repository states about itself in three languages.

## Desired outcome

Running the repository's quality-gate entry point checks exactly what the commit
gate checks, in the same order, and a future edit that makes the two disagree
fails the suite by name instead of passing quietly. The mechanism that already
keeps the other copies honest covers this one too, so the next copy someone adds
is the only thing that can go wrong again.

## Constraints this change inherits

- Local-first: no telemetry, no accounts, no cloud calls. This is the promise the
  dropped flag was protecting.
- The file the defect lives in is written into by an outside bundle on re-apply,
  so the fix has to be one that survives a re-apply rather than one that is
  overwritten by it.

## Systems likely affected

No application module. The two files are the build entry point and the test
suite that reads this repository's own documents.

## Open questions

None. The cause was named, reproduced by mutation, and the fix watched failing
and then passing.

## Not in scope

- The readiness gate's own disagreement with the route table: it exempts
  `spec.md` for the bugfix route and blocks on `intent.md` for every route,
  which is half of an exemption. Another session is working on that gate, so it
  is written down here and left alone.
- The other four findings the reviewers raised, which are in managed files.
  Those are addressed in direct harness updates.
- Whether the byte bands this repository measures are measuring the right thing.
  Three of them are over their thresholds and none of the three is this change's.
