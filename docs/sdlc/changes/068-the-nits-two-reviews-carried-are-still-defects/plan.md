# Plan: the nits two reviews carried are still defects

- **Spec**: none. `bugfix` skips stage 2, and the bug reports are the two
  committed `review.yaml` files' `carried:` blocks — see `./state.yaml`.
- **Approved**: 2026-09-22
- **Status**: done — see "Departures from the plan", which is where the
  interesting half of this change is

Four defects were found by reviewers, judged not worth a round, and written down
under `carried:` rather than fixed. That is `REVIEW.md` working as designed: a
nit taken in the round it is raised voids the approval it arrived with and buys
another round, which produces another nit — change 060 spent nineteen rounds
inside that loop. Carrying them is right. Leaving them carried forever is not,
and nothing in the pipeline schedules them, so this is the change that does it
by hand.

## The four causes, each named before anything was edited

`.claude/skills/root-cause/SKILL.md`'s Iron Law is per requirement here, not per
change. Each cause below was reproduced at a terminal first; the observation
that proves it is beside it.

**R1 — `settle_deferred_kinds`'s `Operational` arm is a mapping nothing tests.**
`src/history.rs:458` maps a record's outcome onto the arm its deferred unknown
kinds are counted against. Three of the four outcomes reach it through existing
fixtures. `RecordOutcome::Operational` does not, because reaching it needs a
record that both (a) defers an unknown `is…` flag and (b) is then excluded by a
*known* operational flag — `src/history.rs:2208`, the only site that returns
`Operational` after `record_is_excluded_by_flag` has already deferred. No
fixture anywhere puts two flags on one record. Observation: 061's reviewer
traced it by hand and said so; `git grep -c 'isMeta'` over `src/tests.rs`
returns nothing outside the vocabulary table.

Why it matters rather than being tidiness: moving that arm to `Carried` passes
the whole suite and all fifteen of 061's mutations, and the result is the exact
sentence 061 exists to stop — a person told a kind's content reached the
restored conversation when the record carrying it was excluded and nothing
crossed.

**R2 — a path that begins with a hyphen is read as an option, twice.**
`scripts/check-installed-build.sh:29` is
`repo_root="$(cd "$(dirname "$0")/.." && pwd)"`, and line 30 is
`bundle="${1:-/Applications/Operon.app}"`. Neither has `--`, and the same
change's `.claude/hooks/gate-stop.sh` does — which is the inconsistency 067's
reviewer objected to. Both are **silent wrong answers**, not errors, which is
what makes them worth a guard rather than a comment. Measured at a terminal:

```
$ dirname "-tree/scripts/x.sh"
dirname: illegal option -- t          # rc=1, and stdout is EMPTY
$ ( cd "$(dirname "-tree/scripts/x.sh")/.." && pwd )
/                                     # rc=0 — repo_root is now the filesystem root
$ plutil -extract CFBundleName raw -o - "-Operon.app/Contents/Info.plist"
unrecognized option: -Operon.app/Contents/Info.plist   # rc=1
```

So `$0` sends the script to read `/scripts/package-macos.sh`, find no key, and
exit 0 with a sentence about a repository it never looked at; and a bundle whose
name starts with `-` is reported as carrying no stamp when it carries one.
`2>/dev/null || true` on line 65 is what converts the second into silence.

**R3 — two different plists get the same sentence.** `plutil -extract` fails
identically for "this plist has no such key" and "this plist cannot be parsed",
and line 67 attributes both to the first: *"carries no OperonSourceCommit (built
before the stamp existed)"*. A person with a damaged bundle is told it is merely
old. Measured: an empty, a truncated, and a missing `Info.plist` all produce
`rc=1` from `-extract`, indistinguishable from a valid plist without the key.
The discriminator that names no key of its own is
`plutil -convert xml1 -o /dev/null`, which returns 0 on a plist that parses and
1 on one that does not.

**R4 — a sweep waits forever on a mutation that hangs.** Seven `subprocess.run`
calls across six `mutations.py` files, none with `timeout=`. A mutation that
makes a test hang therefore hangs the sweep, and the sweep is the evidence that
a change's guards work — so the failure mode is "this change has no evidence",
arriving as a terminal nobody looks at for an hour.

## What this change does NOT do, and why

**It does not edit the five closed sweeps.** The literal nit says 030, 060, 061
and 066 have the same defect. Two reasons not to:

1. `scripts/check-review.sh` refuses a commit whose diff touches more than one
   change directory — *「この diff は複数の変更ディレクトリに触れています」*, exit 2.
   Five directories cannot land together, and splitting into five paper-trail
   commits to satisfy a nit is ceremony, not a fix.
2. A closed change's `mutations.py` is the record of a sweep that was run. Its
   `state.yaml` says "N of N caught" *about that file*. Editing it makes the
   record describe a script that no longer exists.

So the fix goes where the next sweep is written instead. There is no template
for a sweep today — which is why the defect propagated at all: each sweep was
copied from whichever one was nearest, and 065, 066 and 067 carry the same
comments to prove it. `docs/sdlc/templates/mutations.py` is the new one, bounded,
and `.claude/skills/sdlc/SKILL.md` step 10 points at it where a session meets it.

**It does not take 061's `rust/2`** — the two untranslated message ids. Taking
two of 91 leaves 89; the backlog is its own change. `./state.yaml` says so at
length.

**It does not give `scripts/check-installed-build.sh` a risk surface.** It has
none, which is why this change is `low full` while editing a script the Stop
gate runs. `harness`'s own description — "the pipeline's own instruments. Silent
when wrong" — fits it almost word for word, but adding the row changes what
every later change must do on that path, and that is a policy decision rather
than a nit. Recorded in `resume`.

**It does not add a lesson.** Nothing here went wrong twice in a way a reader
needs warning about: the nits were raised, recorded, and taken, which is the
mechanism working. `docs/sdlc/bands.yaml` says the ledger is what put
`steering_bytes` in the diagnose tier, so an entry that only says "we did the
thing we said we would" is a cost with no reader.

## Files that change

| File | Change |
|---|---|
| `scripts/check-installed-build.sh` | `cd --`/`dirname --`; a leading-hyphen `$1` becomes `./$1`; an unparseable `Info.plist` gets its own sentence |
| `docs/sdlc/templates/mutations.py` | **new** — the sweep a change copies, with the bound and the two rules 061 learned |
| `.claude/skills/sdlc/SKILL.md` | one bullet in step 10 naming the template |
| `src/tests.rs` | one case added, one bundle state added, two new tests |
| `docs/sdlc/changes/068-…/` | `state.yaml`, this file, `mutations.py`, `review.yaml` |

## Order of work

The tree compiles between every step; nothing here changes a signature.

1. `src/tests.rs`: add the fifth case to
   `an_unknown_flag_or_role_is_counted_against_what_the_record_did`. **Watch it
   pass** — R1 is a missing test over correct code, so the failure to watch is
   the mutation's, not the fix's. Then mutate `src/history.rs:460` to put
   `RecordOutcome::Operational` on the `Carried` arm and watch the new case go
   red while the other four stay green.
2. `src/tests.rs`: `the_installed_build_check_reads_a_path_that_begins_with_a_hyphen`,
   two arrangements. Watch it fail against today's script, and read the failure:
   it must fail because the script answered about `/`, not because the fixture
   is wrong.
3. `scripts/check-installed-build.sh`: R2. Re-run step 2's test.
4. `src/tests.rs`: a fourth bundle state in
   `the_installed_build_check_reads_its_stamp_key_out_of_the_packager` — an
   `Info.plist` that does not parse. Watch it fail.
5. `scripts/check-installed-build.sh`: R3. Re-run.
6. `docs/sdlc/templates/mutations.py`, and
   `the_mutation_sweep_template_reports_a_run_that_will_not_finish` over it —
   behavioural, because lesson 042 says a guard that a mechanism is *referenced*
   is not a guard that it runs. The test copies the template, points its check
   at a command that never returns, and requires the sweep to finish and name
   the timeout.
7. `.claude/skills/sdlc/SKILL.md`, then 068's own `mutations.py` copied from the
   new template.
8. The three gates, `scripts/check-bands.sh`, the sweep, then review.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The hyphen fixture proves nothing because `bash` itself rejects a `-…` argument before the script sees it | The test fails at the harness, not at the script; stderr names bash | The test asserts on the *script's own* sentence, and invokes it as an executable with a relative `$0` rather than as an argument to `bash` |
| `./$1` breaks a bundle path that legitimately contains `..` or is already relative | A path that worked stops working | `./x` and `x` name the same file; the rewrite happens only when `$1` starts with `-`, which is the one case where they differ |
| The new plist probe rejects a bundle that is fine | The check starts saying "cannot be parsed" about `/Applications/Operon.app` | Measured on the real bundle: `plutil -convert xml1 -o /dev/null` returns 0. The existing three bundle states in the round-trip guard still pass, and they are XML plists written by the test |
| `plutil -convert` writing to `/dev/null` mutates the input | A bundle's `Info.plist` is rewritten by a read-only check | `-o` names the output; the input is untouched. The round-trip guard re-reads the stamp after the probe, so a rewrite would surface as a lost stamp |
| The template test depends on a `python3` that is not there | Test fails on another machine | It invokes `/usr/bin/python3` (3.9.6, part of the OS), not the PATH one |
| A `timeout=` makes a slow-but-healthy sweep report false catches | A sweep says "caught — timed out" for mutations that were fine | The bound is 30 minutes per run, two orders of magnitude above a full `cargo test --locked` here, and a timed-out run prints a reason line saying so rather than a generic catch |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`, N
  four above HEAD's 501.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-bands.sh` — the same two breaches as HEAD and no third.
- `bash scripts/check-installed-build.sh` — still `CURRENT` on this machine.
- `python3 docs/sdlc/changes/068-…/mutations.py` — every mutation caught,
  including the claim mutation `REVIEW.md` requires.
- `bash scripts/check-review.sh` — ✅ SHIP.

## Departures from the plan

**The hyphen test could not run the script the way the plan said, and the reason
is the interesting part.** Step 2 said to invoke it "as an executable rather than
as an argument to `bash`, because `bash -tree/… ` is rejected by bash". Measured:
running it as an executable is rejected *identically*, because the shebang is
`#!/usr/bin/env bash` and the kernel hands the path to bash as an argument
anyway — `bash: -/: 無効なオプション`, and the script never starts. So the first
version of this guard proved a fact about bash's option parsing and nothing about
this repository. It now invokes `bash -- <script> <bundle>`, which is also the
shape `.claude/hooks/gate-stop.sh` uses, and the `--` is called out in the test
as not being part of what is under test. The `chmod` the plan implied is gone
with it.

Worth keeping because it is the same mistake in a new place: the first
arrangement *failed*, which looked like the guard working, and it failed one
layer above the code it was aimed at. A test that goes red for the wrong reason
is a test that will go green for the wrong reason later.

**The suite is 503, not 505.** The plan said "four above HEAD's 501" by counting
requirements. Two of the four are new tests and two are new cases inside existing
ones, which the plan should have said.

**Two guards the plan did not have.** The SKILL.md bullet naming the template had
nothing checking it, so `the_pipeline_documents_ask_their_questions_before_review_does`
gained a fourth row — the same guard change 067 added for the same reason, one
document over. And the template test asserts a **green baseline**: a sweep whose
check returns 0 must report `NOT CAUGHT`. Without it, "the sweep reported a
catch" would be satisfied by a template that reports a catch for everything,
which is change 067's eval mistake in a new costume.

**The damaged-plist assertion reads its claim out of the other run.** Rather than
retyping the sentence reserved for an unstamped bundle, the test takes it from
the unstamped run's own output and requires the damaged run not to contain it.
`.claude/rules/identifiers.md`, applied to a sentence instead of a constant: a
guard that restates what it guards goes blind on the rewording it exists to
survive.

**The sweep is 9 mutations, not 5.** One per requirement is five; the two claim
mutations `REVIEW.md` requires, the one over the skill bullet, and the one round
1 of review added make nine. All nine caught, each by a different assertion
except the two halves of the damaged-plist claim, which meet at the same line
from opposite directions.

**R4 was half a fix, and review found the other half.** The plan said "bound
every run" and the guard asserted the sweep finishes. Both were true and neither
was enough: `subprocess.run(timeout=…)` kills the process Python forked and
nothing under it, and the check a sweep actually runs is `cargo test`, which runs
the test binary as a **grandchild**. So a mutation that hangs a test left that
binary running on the machine after the sweep had reported and exited — which is
the exact situation R4 exists for, surviving the fix for it.

The guard could not see it, and the reason is the part to keep: the fixtures were
`sleep 30` and `true`, both childless. A test whose fixture is simpler than the
thing it stands for measures the fixture. Reproduced twice before it was taken —
once by the reviewer with a real hanging `cargo test`, once here with
`(sleep 2 && touch marker) & sleep 30`, which left both the marker and an orphan
`sleep 30` behind.

Taken rather than carried, which is what `REVIEW.md` reserves for an Important.
`run_checks` is now `Popen(..., start_new_session=True)` with the bound killing
the **group**, in the template and in this change's own sweep; the guard gained a
third arrangement with something under it and was watched failing on it; and the
test is renamed from `…_reports_a_run_that_will_not_finish` to
`…_ends_a_run_that_will_not_finish`, because "the sweep stopped waiting" and "the
work stopped" are two claims and the old name promised the second while checking
the first.

One line deliberately has **no** mutation: `start_new_session=True`. Removing it
while keeping the group kill makes `child.pid` a number that is not a group id,
and `killpg` would then name whatever group on this machine happens to hold it —
a SIGKILL at a stranger, once per sweep. It is covered from the other side
instead: `bound-reaches-only-the-direct-child` shows the group kill is what stops
the grandchild, and a group kill needs the group. Recorded in the sweep beside
the mutation that is not there.
