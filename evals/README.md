# Evals

`src/tests.rs` says whether the code is right. These say whether the **steering**
is right: whether an agent handed a real prompt, with nothing but what this
repository tells it, does the thing this repository wants.

Every eval is seeded from a mistake that actually happened. `docs/sdlc/lessons.md`
is the ledger; the `seeded-by` field in each eval points at its entry. Nothing is
invented — an eval for a failure nobody has had is a guess about which failures
matter. Nor is a case chosen because today's model fails it: capability is
jagged, and a suite built from one model's misses measures that model's profile
rather than what this repository needs.

Run them with `scripts/run-evals.sh`. Each one runs in a worktree of a committed
ref, so what is judged is the steering **as committed**, not whatever is in the
working tree. The worktree is on a throwaway `eval/` branch rather than detached:
the agent runs under this repository's own hooks, and the worktree policy
refuses every edit on a branch it cannot name (sdlc 104).

```sh
bash scripts/run-evals.sh --list
bash scripts/run-evals.sh --only 003
bash scripts/run-evals.sh
bash scripts/run-evals.sh --runs 3 --recheck        # a baseline
bash scripts/run-evals.sh --split train --runs 3    # one hillclimb round
```

## When to run them

When the thing being changed is the steering itself: `CLAUDE.md`, `AGENTS.md`,
anything under `.claude/`, or a policy document like `DESIGN.md` or `REVIEW.md`.
That is also when `.github/workflows/harness.yml` runs them, if an API key is
configured.

Not on every code change. They cost an agent turn and a build each.

## Format

Front matter, then three sections, each holding one fenced block:

| Section | Required | Contents |
|---|---|---|
| `## Setup` | no | shell run in the worktree before the prompt — seeds the situation |
| `## Prompt` | yes | what a person would actually type, in the language they would type it |
| `## Check` | yes | shell run in the worktree afterwards; exit 0 is a pass |

The front matter holds `id`, `title`, `split` (`train` or `test`, below),
`seeded-by`, and `guards`. `every_eval_states_a_prompt_and_a_check` in
`src/tests.rs` fails on an eval missing a prompt or a check, because a
half-written eval passes silently, and
`every_eval_declares_a_split_and_both_splits_are_used` fails on one with no split.

Two rules for a check, both learned the hard way:

- **Assert the outcome, not the route.** There is more than one right way to add a
  colour role. The check cares that no literal reached a call site.
- **Do not lean on a single guard test.** Lesson 004 in `docs/sdlc/lessons.md` is
  a guard that went blind on exactly the change it existed to survive. A check
  that only runs that test inherits the blindness, so each check also looks at the
  tree directly.

## What makes a score worth reading

A score is only evidence if four things hold. Each has an instrument here:

| Property | Why it matters | What shows it |
|---|---|---|
| The cases are real | a suite of convenient cases measures convenience | `seeded-by` — every eval names the lesson it came from |
| The check is right | a misjudged run is the commonest way a suite lies | a new check is watched failing and passing (below); `--recheck` runs it twice on the same tree |
| There is headroom | a suite everything passes cannot show a gain | the summary warns at 95% or more over three or more runs |
| The noise is small | a flip inside the noise is not a result | `--runs N` and the 95% interval; `infra` and `unstable` kept out of the rate |

## Reading a measured run

Each run ends in one outcome:

- `pass` / `fail` — the check accepted or rejected the agent's work. Only these
  two are in the rate.
- `infra` — setup failed, the checks could not be hidden or put back, the agent
  hit `EVAL_TIMEOUT`, or the transcript has no successful result. Not a verdict
  on the steering, so not in the rate — but an agent that wanders until the
  timeout can be the steering's doing, so read the `infra` count beside the
  rate, never past it.
- `unstable` — under `--recheck`, the check said two different things about the
  same tree. Fix the check before reading anything else.

The summary gives `passed/judged` per eval, then the overall rate with its 95%
Wilson interval, and a warning for any eval that failed every judged run —
suspect that eval (an ambiguous prompt, a check that contradicts the policy it
tests) before the steering. Everything a run produced is under
`target/eval-results/<stamp>-<sha>/`: `transcript.jsonl` and `check.log` per run,
`results.tsv`, and a `summary.md` that links them.

While the agent works, every eval file in its worktree is cut down to its id,
title, split, and prompt — the setup, the check, and the `guards:` line naming
the test the check leans on are the answer; a hide that fails makes the run
`infra` rather than letting the agent work with the answer in view. The
worktrees live under `$TMPDIR`, so the main checkout is neither one `../` away
nor an ancestor whose `CLAUDE.md` the agent would load. That hides the answer; it
does not lock it away, since `git show` and the main checkout's absolute path
still reach it. A run whose tool calls mention `evals/` is marked `peeked` so it
can be read with that in mind.

A local run also loads the person's own user-level instructions, which CI does
not have. Compare runs from the same machine.

`--model` and `--effort` pass through to the agent. A suite whose rate does not
rise from a weaker configuration to a stronger one has ambiguous cases or a
misjudging check — run both once in a while.

## The split

Two evals are held out as `test`; the rest are `train`. The draw was made once,
at random, and can be repeated:

```sh
yes 103 | head -c 4096 > /tmp/seed
printf '%s\n' 001 002 003 004 005 006 007 | shuf --random-source=/tmp/seed   # first two are test
```

A new eval goes to `test` while `test` holds fewer than a third of the suite, and
to `train` otherwise — decided when it is written, never after its first score.

## Hillclimbing the steering

Improving the steering against this suite is worth doing and easy to do wrongly:
the steering can be fitted to seven prompts and get worse at everything else.
The procedure:

1. **Edit only cheap, attributable surfaces.** The steering text — `CLAUDE.md`,
   `AGENTS.md`, `.claude/rules/`, `.claude/skills/`, `REVIEW.md` — where an edit
   is one revert away and a score change can be traced to it.
2. **Scope the objective before the first run.** "Eval 007 from 1/3 to 3/3
   without losing another", or "the same rate with fewer `always_loaded_bytes`"
   (`scripts/harness-metrics.sh`). Not "make it better": an open goal stalls
   against the noise.
3. **Measure a baseline** with `--runs 3 --recheck` over both splits. If an
   `unstable` or a run of `infra` appears, fix that first. If the interval is
   wider than the smallest gain worth acting on, add runs before anything else.
4. **One change per round.** Read the **train** transcripts only, name the cause
   of a failure, and make one edit at that cause — not a rewording of the
   symptom. Re-run both splits.
5. **Keep it or revert it.** Revert when train rose and test stayed flat — that
   is the steering learning the train prompts — and revert on any regression.
   Keep it only when both moved the right way.
6. **Never paste a failure into the steering.** No eval's prompt, setup, or check
   text, and no rule that only exists for one eval's situation.
   `no_eval_prompt_is_pasted_into_the_steering` catches the verbatim case; the
   rest is on whoever makes the edit, and review.
7. **After two flat rounds, stop editing and sort the remaining failures** by
   cause: an ambiguous eval, a check bug, an environment fault, or noise. Only a
   real, fixable failure earns another edit.
8. **Do not claim a gain inside the interval.** A final test-split rate that
   overlaps the baseline's interval is no evidence the edit helped, and the
   change says so rather than shipping it as an improvement.

A round is a change of its own, through the pipeline in `docs/sdlc/README.md`;
the route's attempt ceiling is the round ceiling.

## Adding one

An eval earns its place when a mistake happened twice, or once expensively. Write
the lesson first — the eval is the **Guard** column of a `docs/sdlc/lessons.md`
entry, and writing it without the lesson skips the part where the root cause gets
named.

Then judge the check before trusting it: run it by hand once against a tree
where the mistake is made, and once where it is avoided, and confirm it says
`fail` and `pass` for the reasons you expect. `evals/007-mutate-the-claim-not-the-counter.md`
records two first hand-runs where a check passed for the wrong reason.
