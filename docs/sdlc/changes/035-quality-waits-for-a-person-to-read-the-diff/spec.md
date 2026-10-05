# Spec: merging is decided by a fresh verdict, not by who is available

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `docs/sdlc/routes.yaml`'s fifth column stops meaning "a person reads the diff"
   and starts naming the reviewer set that closes the route: `full` (all three
   reviewers), `craft` (rust craft only), or `none` (the deterministic checks
   alone). `feature` and `security` take `full`; `modify`, `bugfix`, and
   `refactor` take `craft`; `docs` takes `none`.
2. A change records its verdicts in `review.yaml` beside its `state.yaml`. Each
   line names a reviewer, its verdict, its Important count, its nit count, and the
   digest of the diff it judged.
3. `scripts/check-review.sh` refuses, naming exactly one reason, when any of these
   holds for the change in flight:
   1. there is no `review.yaml`;
   2. a reviewer the route or the touched surfaces require has no line in it;
   3. a line's digest is not the current diff's digest — the verdict is stale;
   4. a verdict is `do-not-approve`;
   5. an Important count is above zero;
   6. a reviewer name is not a file in `.claude/agents/`.
4. The required set is the route's set **widened by what the diff touches**, read
   from `docs/sdlc/risk.yaml`: the `store` or `bundle-swap` surfaces require
   `durability-reviewer`, and the `subprocess` surface requires
   `subprocess-safety-reviewer`, whatever the route said. A route may ask for more
   than the surfaces; it may never ask for less.
5. The digest covers the **index** — `git diff --cached` over every path except
   `docs/sdlc/changes/` — because a commit records the index. With nothing
   staged it falls back to `git diff HEAD`, which is the "run it by hand while
   reviewing" case. `git commit -a` stages after the hook runs, so for that one
   shape `.claude/hooks/gate-commit.sh` passes `--include-unstaged` and the
   digest widens to the working tree. The change's own paper trail is excluded
   throughout: including it would invalidate every verdict the moment its own
   result was written down.
6. `.claude/hooks/gate-commit.sh` runs the review gate ahead of the three gates and
   blocks the commit on its refusal, so the boundary that already refuses an
   unproven tree is the one that refuses an unreviewed one.
7. On success the gate prints the first-view block: a verdict line, a band of
   indicators, and Important findings only when there are any.
8. `steering_bytes` does not rise. `REVIEW.md` is rewritten, not extended.
9. `src/tests.rs` gains guards for: the routes column's new vocabulary, the gate
   being wired into `gate-commit.sh`, and the gate having no bypass.

## Behaviour

There is no UI. What a session sees at `git commit`:

**Everything holds.** The commit proceeds and the gate prints:

```
✅ SHIP  —  sdlc 032 release-gate check
0 Important · 2 Nits

gates   fmt ✓  test 433/0/6 ✓  clippy ✓  bands ⚠ steering_bytes(030)
passes  correctness ✓  subprocess ✓  durability ✓  rust ⚠

Nits (blocking しない)
  · guard-bash.sh:47  root は readonly にできる
  · check-release-preconditions.sh:120  diff 上限に件数を添えられる
```

**Something refuses.** The commit is blocked and the first line says which:

```
❌ BLOCK  —  sdlc 035 · rust-reviewer の判定が古い
判定は diff a1b2c3d4 のもの、現在の diff は 9f8e7d6c。
再レビューしてから commit してください。
```

The refusal names one reason. A gate that lists six problems is a gate whose
first line stops being read, which is the failure this change is about.

The verdict block is the only new user-facing text, and it is read in a terminal
by the person running the commit — so it follows the repository's split: the
labels are the tools' own vocabulary and stay as they are, and the sentences a
person reads are Japanese.

**The change has no directory.** A one-line fix does not open one, and the gate
stays silent — the same rule `guard-stage.sh` already follows. Gating every edit
in a repository where most edits are small is how a gate gets switched off.

## Design

**review.yaml, beside each change's state.yaml** — flat and positional, like every
other data file the pipeline parses without a YAML library:

```yaml
digest: "a1b2c3d4e5f6a7b8"

verdicts:
  rust-reviewer: "approve-with-nits 0 2"
  subprocess-safety-reviewer: "approve 0 0"
  durability-reviewer: "approve 0 1"
```

`<verdict> <important> <nits>`, with the verdicts taken from what the three agent
definitions already tell their reviewers to say: `approve`, `approve-with-nits`,
`do-not-approve`.

**`scripts/check-review.sh`** — computes the digest, reads the route and the
touched surfaces, and compares. It is a script rather than inline hook code for
the reasons `scripts/check-release-preconditions.sh` is: the hook stays a decision
table, a person can run it to see why, and `src/tests.rs` can drive it.

**`docs/sdlc/routes.yaml`** — the fifth column's values change. The header comment
and `every_route_the_pipeline_offers_names_stages_that_exist` move with it.

**`REVIEW.md`** — "When a human must look" becomes "What the gate requires", and
the section that listed six surfaces for a person becomes the table the gate reads
its required set from. Rewritten to the same size or smaller.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | nothing is drawn; this is a script, a hook, two documents, and tests |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | Yes | the verdict block uses ✅ ❌ ⚠ ✓, which are terminal output from a bash script and never reach an `egui` painter. `ICON_VOCABULARY` governs marks the app draws; a hook's stdout is not one, the same way `guard-bash.sh`'s existing prose is not |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | the reviewer names are not restated: the gate validates them against the filenames in `.claude/agents/`, and the required-set rules are read out of `docs/sdlc/routes.yaml` and `docs/sdlc/risk.yaml` rather than retyped. Lesson 004 is the reason |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes | no persisted shape changes; `review.yaml` is a repository file under git, not app state |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | the gate's children are `git`, `sha256sum`/`shasum`, and `grep`, all fixed literals from a bash hook. Nothing is interpolated from the diff into a command, and `subprocess_sites` does not move because no Rust spawns anything new |
| Documentation — user-facing docs change in all three languages together | No | `README.md` is untouched. `REVIEW.md` and `docs/sdlc/*` are internal English documents with no translations |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | no dependency, no network, no `unsafe`. The reviewers already run locally |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | the verdict block is the output path: at most 5 Important findings and 5 nits are printed, each truncated to 200 characters, and the refusal prints exactly one reason. `REVIEW.md`'s five-nit rule is where the 5 comes from |

## Flagged concerns

- **The gate cannot prove the verdict came from an independent agent.**
  It checks that a verdict exists, is fresh, is clean, and names a reviewer that
  is defined — not that the session ran that reviewer rather than writing the
  line itself. **Answered:** this is the honest ceiling of a bash gate and it is
  stated in `REVIEW.md` rather than hidden. What it replaces is weaker, not
  stronger: today nothing at all records whether a review happened, so the failure
  mode being removed is "no review and no trace", and what remains is "a review
  the author could have falsified" — the same exposure as a person typing LGTM,
  but now with a digest saying which diff it was about. The property that actually
  does the work is freshness: a verdict cannot outlive the code it judged, which
  is the failure a human sign-off has and this does not.
- **A refusal names one reason, and there may be six.**
  Fixing one and re-running to find the next is slower than seeing all six.
  **Answered:** deliberate, and it is the point of the change the owner asked
  for. The six are checked in the order they are cheapest to fix and most likely
  to subsume the rest — a missing `review.yaml` makes the other five unanswerable
  — so the sequence terminates rather than ping-ponging. The full state is one
  command away for anyone who wants it: `bash scripts/check-review.sh --all`.
- **Excluding `docs/sdlc/changes/` from the digest means the paper trail is not
  reviewed by the reviewers.** A `spec.md` could claim something false and no
  verdict would cover it. **Answered:** it is covered by the deterministic half
  instead — `harness_documents_only_name_paths_that_exist` fails on a document
  naming a path that is not there, and `scripts/check-readiness.sh` fails on a
  spec that is not filled in. Those two run in the suite, which the commit gate
  runs before this gate is reached. The alternative, including the change
  directory, makes every verdict stale the instant it is written down, which is a
  mechanism that cannot be used.
- **This removes the last human checkpoint from surfaces where a mistake is
  unrecoverable.** `docs/sdlc/risk.yaml` marks five of them `paused`.
  **Answered:** `paused` is untouched and still means stop-and-ask *before
  editing*, which is a different checkpoint from reviewing afterwards and the one
  that catches the more expensive mistake. What this change removes is the
  after-the-fact reading; what it adds in its place is a required reviewer set
  that those same five surfaces widen. A `store` change that skipped
  `durability-reviewer` used to be caught by a person noticing; now it does not
  commit.

## Acceptance

- `cargo test --locked` passes, including `every_route_the_pipeline_offers_names_stages_that_exist` against the new vocabulary, `the_review_gate_is_wired_into_the_commit_gate`, and `the_review_gate_has_no_bypass`.
- `bash scripts/check-review.sh docs/sdlc/changes/035-quality-waits-for-a-person-to-read-the-diff` exits `0` and prints the verdict block once this change's own `review.yaml` is filled by the three reviewers.
- The same call exits non-zero, naming one reason, in each of five watched
  mutations: no `review.yaml`; a required reviewer removed from it; a digest from
  a different diff; a `do-not-approve`; an Important count of 1.
- `bash scripts/harness-metrics.sh` shows `steering_bytes` no higher than
  152 647, and `gated_commands`, `hooks`, and `routes` unmoved.
- A commit is watched being refused by `gate-commit.sh` for a stale verdict, and
  watched succeeding once the verdict is refreshed.

## Rejected alternatives

- **Run the reviewers from inside the hook.** A bash hook cannot invoke an LLM
  reviewer with any reliability — auth, latency, cost, and a nondeterministic
  answer at a boundary that must be deterministic. The gate checks the verdict; it
  does not produce it.
- **A receipt with no digest.** This is the receipt-versus-re-derivation argument
  change 032 already had, and the digest is what makes this side of it defensible:
  the freshness is re-derived even though the judgement is not.
- **Keep `human: yes` and add the gate beside it.** Then nothing changed: the
  person is still the thing merging waits for.
- **Block on nits as well as Important.** `REVIEW.md` caps nits at five precisely
  because nit volume is how a review stops being read. Blocking on them makes
  every review a negotiation.
- **Put the verdicts in `state.yaml`'s `contract`.** It already carries
  human/machine items and `gate-stop.sh` reads it, so overloading it with per-diff
  digests would make one file answer two questions with different lifetimes.
