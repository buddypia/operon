# Intent: the stage-3 readiness gate prints its findings and then says Go

- **Status**: approved
- **Opened**: 2026-09-15

## Problem

`scripts/check-readiness.sh` is the gate that decides whether a change may begin
stage 3. It finds problems correctly — it printed, on a fixture built to fail:

```
blocking  …/spec.md: 中身のない flagged concern があります:
      - **A concern nobody answered.** unresolved. (12 chars)
readiness: Go — 1 change(s), 0 blocking, 0 warning
```

Both lines are from the same run. It lists a blocking finding and then reports
zero of them, exits 0, and lets stage 3 begin.

The verdict is counted by a function that takes the array by name:

```sh
count() { local -n a=$1; … }
```

`local -n` is a bash 4 nameref. The script's shebang is `#!/usr/bin/env bash`,
and the bash a stock macOS puts first on `PATH` is 3.2, which has no namerefs:
it prints `local: -n: invalid option`, leaves the loop iterating over nothing,
and echoes `0`. The error goes to stderr, which nobody reads, and `0` is a
perfectly good number, so the gate is silent about having stopped working.

The failure is invisible in a second way. Whether it happens at all depends on
which bash is first on `PATH`, so on a machine with a newer bash installed the
gate works and its own test passes. The repository has no way to notice from the
inside that the gate has been dead everywhere else.

This blocks everything else, not just stage 3. `.claude/hooks/gate-commit.sh`
runs `cargo test --locked` whenever a Rust file has changed — staged or not — so
while this test is red no commit of any kind is possible in this repository.

**Correction, 2026-09-17.** The paragraph above was true when it was written and
is not true now, and the difference is itself the evidence. `/opt/homebrew/bin/bash`
is a symlink to 5.3.20 dated 16 Sep 02:36 — it was installed *after* the failure
was observed. Before then the `bash` this test found was 3.2 and the test was red
for everyone; since then it is 5.3 and the test is green while the gate stays dead
under the `/bin/bash` the script's own `env` lookup finds on a machine without
that install. Nobody changed the gate or the test. Installing an unrelated shell
moved this test from red to green, which is the whole defect stated as an
experiment: a guard that reports on the machine it runs on rather than on the
machine the code runs on.

## Who feels it, and when

Every change that reaches stage 3 on a machine without a newer bash on `PATH`:
its spec is waved through with unanswered flagged concerns, which is the one
thing this gate exists to stop. And, right now, anyone trying to commit
anything.

## Desired outcome

1. The gate's verdict agrees with the findings it printed, under the bash a
   stock macOS provides.
2. The guard over it fails on any machine when the gate stops refusing, rather
   than only on machines that happen to lack a newer bash.
3. `cargo test --locked` is green, so commits are possible again.

## Systems likely affected

- `scripts/check-readiness.sh` — the verdict counting.
- `src/tests.rs` — `the_readiness_gate_refuses_a_spec_that_is_not_filled_in`,
  which must stop depending on the ambient `PATH` to see the defect.

## Not in scope

- Pinning the gate scripts to `/bin/bash`. The requirement is that they *work*
  on 3.2, not that they never see a newer shell — so the guard runs the gate
  under `/bin/bash` **and** under a newer `bash` from `PATH` when the machine has
  one, and fails if the two disagree.
- Fixing `.claude/hooks/gate-stop.sh`, which the sweep this change added found
  does not parse under 3.2 at all: a `case` inside a `$( )` that bash 3.2's
  paren-counting scanner ends early, so on a stock macOS the Stop hook exits 2
  with a shell syntax error and its checks never run. The one-character fix is
  known and written down at `KNOWN_UNPARSED_UNDER_BASH_3_2` in `src/tests.rs`.
  `.claude/hooks` is the `gate-configuration` surface that `docs/sdlc/risk.yaml`
  marks `paused`, so a person decides it.
- The other findings in this repository's harness that are not this gate. The
  breached metrics, read from `scripts/check-bands.sh` rather than recalled —
  the first draft of this line named `unsafe_blocks`, which is at 2 against a
  band of 3 — are `steering_bytes` at its `propose` tier and
  `always_loaded_bytes` at `warn`. Also
  `scripts/check-readiness.sh` is absent from the `gate-configuration` surface in
  `docs/sdlc/risk.yaml`, so a change to the stage-3 gate is currently classified
  as touching nothing. Both are reported rather than fixed here — `risk.yaml` is
  a paused surface.
