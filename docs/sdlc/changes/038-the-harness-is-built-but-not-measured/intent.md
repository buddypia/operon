# Intent: the harness has every mechanism the playbook asks for and measures none of them

- **Status**: draft
- **Opened**: 2026-09-09

## Problem

The AI-native SDLC playbook this repository was shaped from gives each play two
halves: a mechanism, and a pair of indicators that say whether the mechanism is
working. Nineteen changes have built the mechanisms. Not one indicator is
computed.

Concretely, the playbook asks for the eval pass rate over time, the trend in
findings per scan, the time from a band breach to an `intent.md`, and — for the
approval-gate play — every hook decision written down with a timestamp and an
allow-or-block verdict. This repository can answer none of those questions. It
produces the numbers for a single moment: `scripts/harness-metrics.sh` prints one
JSON object, `scripts/check-bands.sh` compares that object against thresholds,
and both are thrown away when the terminal scrolls. A band is therefore a
tripwire that can only say "now", never "since when" or "how fast".

The gap is not theoretical. `steering_bytes` is over its `propose` threshold as
this is written, at 164948 against 150000. It crossed before the change that is
currently in flight, is attributed in that change's `state.yaml` to change 030,
and no artifact anywhere records when it crossed or that anyone was told. The
play whose whole subject is "band breach becomes an `intent.md`" is the play this
breach has been sitting outside of.

The same is true one stage earlier. The playbook's Stage 1 and Stage 2
indicators — elapsed time from `intent.md` to `spec.md`, `spec.md` commits dated
after the first `plan.md` commit, whether the merged diff still matches the
committed `plan.md` — are all derivable from what this repository already keeps:
thirty-six change directories and their git history. Nothing derives them. A
pipeline that cannot say whether it is getting faster or slower is a pipeline
being followed on faith.

## And four of the mechanisms do not run at all

Written after the first draft, from checking rather than assuming.

`.github/workflows/` holds `ci.yml`, `claude-review.yml`, `harness.yml` and
`scheduled-scan.yml`. All four were added on 2026-09-02. **This repository has
no git remote** — `git remote -v` is empty, `main` has no upstream, and `gh`
answers "no git remotes found". A GitHub Actions workflow runs when GitHub
receives a push, a pull request, or a schedule tick for a repository it hosts.
None of those can happen. `scheduled-scan.yml` carries a Monday 03:00 cron that
has never fired, and never will from here.

So four playbook plays are instantiated as files that describe an intention:
Stage 4's continuous evals in CI, Stage 5's AI-in-the-PR-review-loop, Stage 5's
CI/CD integration, and Stage 6's recurring scans. This also explains why the
lagging indicators looked structurally unavailable — there is no CI history and
no PR history because CI and PRs have never happened, not merely because the
tool is local-first.

This is the repository's own recurring failure turned on the harness itself:
entry 003 in `docs/sdlc/lessons.md` is a doc comment naming a script that was
never committed, and entry 017 is a gate that went vacuous when its inputs were
deleted. Four workflows that cannot run are the same shape, and nothing in the
suite notices, because every check reads whether a file exists and none asks
whether the thing it describes can happen.

The decision this needs is not technical. Either the local gates *are* the CI
for a single-machine tool — in which case the workflows should say so or go, and
the playbook's CI plays are honestly marked not-applicable — or this repository
is meant to have a remote and does not have one yet. Both are defensible. What
is not defensible is four files that look like a working CI to anybody reading
the repository, including the next agent.

### One thing already done about it, and one thing tried and rejected

Each workflow now carries a `NOT RUNNING` header saying it has never run, why,
what does run instead, and where the open question lives. That is honest and
reversible and decides nothing — the files stay correct for a repository that
has a remote, and `git remote add` would make them live.

Tried and rejected, so nobody spends the hour again: a commit the review gate
refuses still leaves its commit object in the repository, because the refusal
happens at the ref update after the object is written. `git fsck --unreachable`
therefore looked like a way to count refusals — a local stand-in for the
playbook's first-pass CI success rate. It is not one. Sixty unreachable commits
are there and the overwhelming majority are `git stash` entries (`On main:`,
`index on main:`), which are indistinguishable from refusals without a record
that the gate itself wrote, and `git gc` prunes the lot on its own schedule. A
first-pass rate needs the decision log, which is the item waiting on an answer.

### Two more, found by trying to record what had just been measured

**A contract item cannot say it failed.** `docs/sdlc/templates/state.yaml`
declares the closed set `verdict: "pending passed waived"`, and
`every_state_file_names_a_route_and_a_stage_that_exist` enforces it. So when
035's three-gates item was checked and did not pass, there was no way to write
that down: `passed` is false, `waived` is a decision nobody made, and `pending`
reads as *not yet checked* rather than *checked and not satisfied*. The
difference matters exactly here — a reader cannot tell an item nobody has run
from one that ran and came back red. A machine contract whose vocabulary cannot
express failure is a contract that quietly rounds failure toward "not yet".

And it cannot be fixed by adding the word. `.claude/hooks/gate-stop.sh` reports
open contract items with an awk that matches `/"machine pending /` and nothing
else, so a `machine failed` item would be *less* visible than a pending one —
the stop gate would stop mentioning it at all. The fix is two files at once,
`docs/sdlc/templates/state.yaml` for the vocabulary and `gate-stop.sh` for the
reader, and `gate-stop.sh` is on the `gate-configuration` surface that 035
holds. So this waits on 035 landing or failing, not on a decision. Adding the
word alone is the shape this repository keeps relearning: a mechanism shipped
without the reader that makes it mean anything.

**A gate whose verdict depends on the machine.**
`ranking_the_palette_over_a_full_store_stays_inside_a_frame` asserts a search
stays under 5ms. It already takes the fastest of twenty passes to resist load,
and it still failed at 6.37ms and again at 19.5ms while this machine sat at load
average 157 with a background `cargo` run killed for memory pressure — four of
five isolated runs passed. `cargo test --locked` is one of the three gates every
commit must pass, so a busy machine can refuse a commit that a quiet one would
allow, and can pass one it would refuse. That is the same class as a band with
no history: a threshold with nothing recording the conditions it was read under.
Raising the ceiling would be quieting a test, and the number is a promise to a
person about how the palette feels, so it is not this change's to move.

## Who feels it, and when

Whoever has to decide whether the pipeline is worth its cost, at the moment they
have to decide it. Today that decision has no evidence behind it in either
direction: the six stages might be halving rework or doubling it, and the
repository holds no number that distinguishes those.

Also whoever inherits a breached band. A person reading `check-bands.sh` output
sees a threshold crossed and is told to open an `intent.md`; they cannot see
whether the crossing is a week old or a minute old, whether it has been
crossed-and-recovered five times, or whether someone already decided to accept
it. The current `steering_bytes` breach is being carried in prose in one
change's `state.yaml`, which is where a fact goes to be forgotten.

## Desired outcome

The questions the playbook asks can be answered from the repository, by a
command, without reconstructing history by hand:

- how each harness metric has moved, per commit that changed it, not just now
- when a band was breached, and whether the breach became an `intent.md`
- what each gate decided, when, and whether it allowed or blocked
- how long a change took between its stage artifacts, and whether its merged
  diff matched the plan it committed

And a breach stops depending on a person noticing a line of terminal output.

## Constraints this change inherits

- Local-first: no telemetry, no accounts, no cloud calls. The playbook's
  indicators are written for an OpenTelemetry export and PR metadata; neither
  exists here and neither may be added. Whatever answers these questions has to
  come out of git history, the change directories, and files this repository
  writes itself.
- User-facing text is Japanese; code, comments, and docs are English.
- A gate decision log is written by hooks that run on every commit, so its cost
  is paid on every commit — it cannot be slow, and it cannot fail a commit by
  failing itself.
- `docs/sdlc/changes` is excluded from the review gate's digest. Anything stored
  there is invisible to that gate, which is right for a record and wrong for a
  mechanism.

## Systems likely affected

`scripts/harness-metrics.sh`, `scripts/check-bands.sh`, `docs/sdlc/bands.yaml`,
`.claude/hooks/gate-commit.sh` and `.claude/hooks/gate-stop.sh` (a decision log
is written from where the decision is made), `docs/sdlc/README.md` for stage 6,
and `src/tests.rs` for whatever becomes checkable. No Rust production module is
expected to change; if one does, this intent was wrong about its own scope.

## Open questions

- Does a metrics series belong in the repository as a committed file, or is it
  derived on demand from git history? A committed file is a merge conflict on
  every branch; a derived one cannot record anything git does not already hold —
  such as the moment a band was breached. **Answered by**: `spec.md`, against
  what the Stage 6 indicators actually require.
- A gate decision log records that a gate blocked. Does it also record what it
  blocked — a command, a digest, a branch — and does that make it a privacy
  surface on a local-first tool? **Answered by**: the repository's owner.
- Is the current `steering_bytes` breach this change's to settle, or change
  029's? Measured since this was written, by the
  `--breaches` mode of `scripts/pipeline-indicators.sh`: the `propose` crossing
  is at `30a879f` on 2026-09-06, which is
  change 029 — not 030, which `035`'s `state.yaml` names. Two changes each
  believing the other owns it is how it stayed open, and neither of them was
  the one that crossed it. **Answered by**: the repository's owner, now with a
  date and a commit.
- Do the four GitHub Actions workflows stay, go, or get a remote? They have
  never run and cannot. **Answered by**: the repository's owner.
- The playbook's lagging indicators mostly need an incident tracker and PR
  history. With neither, is a lagging indicator honestly available at all here,
  or should this change claim only the leading half and say so? **Answered by**:
  `spec.md`.
