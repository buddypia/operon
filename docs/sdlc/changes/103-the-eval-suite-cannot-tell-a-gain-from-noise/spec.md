# Spec: the eval suite measures, and says how to climb it

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `scripts/run-evals.sh`, given `--runs N`, runs each selected eval N times, each in its
   own fresh worktree. Without `--runs` it runs once, as today.
2. Every run ends in exactly one outcome: `pass`, `fail` (the check rejected the
   agent's work), `infra` (setup failed, the agent timed out, or its transcript
   has no successful result event), or `unstable` (with `--recheck`, the check
   gave two different verdicts on the same tree).
3. The pass rate counts only `pass` and `fail`. The summary prints, per eval and
   overall, `passed/judged`, the overall rate with a 95% Wilson interval, the
   count of each other outcome, and the summed agent cost when the transcript
   reports one.
4. The summary warns when the overall rate is 95% or more over at least three
   runs per eval (the suite has no headroom left to show a gain), and names any
   eval that failed every judged run (suspect the eval before the steering).
5. Every run leaves `transcript.jsonl` (the agent's stream-json output) and
   `check.log` under `target/eval-results/<stamp>-<sha>/<id>/<run>/`, plus a
   `results.tsv` with one row per run and a `summary.md` that links each run's
   files. The summary path is printed at the end.
6. While the agent runs, every `evals/NNN-*.md` in its worktree holds only its
   `id`, `title` and `split` front-matter lines, its `## Prompt` section, and a
   placeholder `## Check` block — no `## Setup`, no check, and no `guards:` or
   `seeded-by:` line, because those name the test a check leans on. `git status`
   in the worktree is unchanged by hiding them, and the files are restored
   before the check runs.
7. A run whose transcript has an assistant tool-use input containing the string
   `evals/` — a read, a `grep`, a `git show HEAD:evals/…` alike — is flagged as
   `peeked` in `results.tsv` and `summary.md`; the flag does not change its
   outcome.
8. Every eval declares `split: train` or `split: test` in its front matter, both
   splits are non-empty, `--split train|test` selects one of them, and `--list`
   prints each eval's split.
9. `--model` and `--effort` pass through to the agent, so the same suite can be
   run on a weaker and a stronger configuration to check the score rises with
   capability.
10. Exit status: 0 when every run passed; 1 when any run that started did not
    pass, `infra` included; 3 when nothing could start — no agent command (the
    one `EVAL_AGENT` names, or `claude`), no evals, or no eval matched the
    selection.
11. No line of any eval's `## Prompt` appears in `CLAUDE.md`, `AGENTS.md`,
    `REVIEW.md`, or any file under `.claude/rules/` or `.claude/skills/`.
12. `evals/README.md` states what makes the suite trustworthy (production-seeded
    cases, a check watched failing and passing, headroom, low variance), how to
    read a measured run, and a hillclimb procedure: edit only cheap,
    attributable surfaces (the steering text); a scoped objective; a baseline
    with `--runs 3 --recheck`; one change per round, read from train
    transcripts only; never paste a failed eval's prompt, setup, or check into
    the steering; revert on a train gain with a flat test score or on any
    regression; root-cause after two flat rounds; and no claimed gain inside
    the interval.

## Behaviour

The runner is a developer tool and prints English, like today. A person reads
the summary at the terminal:

    eval 003   2/3 passed  (1 infra)
    ...
    evals: 14/18 passed (77.8%, 95% CI 54.8–91.0%) against abc1234 · 3 runs · cost $4.12
    infra 1 · unstable 0
    results: target/eval-results/20261002T101500Z-abc1234/summary.md

With no `claude` on `PATH` it still exits 3 with the existing message. A missing
`timeout` command means no timeout is applied, and the summary says so once.

## Design

- `scripts/run-evals.sh` keeps its structure: parse flags, select evals, loop.
  The loop gains an inner loop over runs. Agent: `${EVAL_AGENT:-claude} -p
  "$prompt" --permission-mode acceptEdits --output-format stream-json --verbose`
  plus `--model`/`--effort` when given, wrapped in `timeout ${EVAL_TIMEOUT:-1800}`
  when `timeout` exists. `EVAL_AGENT` exists so the runner itself can be tested
  with a stub; it is documented in the script header.
- Hiding: after setup, `git update-index --skip-worktree` on every tracked
  `evals/[0-9]*.md`, then each is rewritten to its front matter and `## Prompt`
  section with a placeholder `## Check` block. After the agent, `git
  update-index --no-skip-worktree` and `git checkout --` restore them.
- Outcome parsing reads the last `"type":"result"` line of the transcript with
  `grep`/`sed`; the interval and summary are computed with `awk`. No new
  dependency.
- `src/tests.rs` gains: the split is declared and non-empty; no eval prompt line
  is pasted into the steering; and a family of runner tests. Each copies
  `scripts/run-evals.sh` into a scratch repository (the script finds its
  repository from its own location), commits fixture evals, and drives it with
  a stub agent through `EVAL_AGENT`. Between them they assert every outcome
  (`pass`, `fail`, `infra`, `unstable`), the `peeked` flag, the arguments the
  agent receives with and without `--model`/`--effort`, the hidden file's
  exact content and a clean `git status` while the agent runs, `--split`
  selection, the `--list` split column, the Wilson figures for a known count,
  the headroom and always-fails warnings, the results files, and exit 0, 1
  and 3.
- `evals/README.md` gains the trust properties, the measured-run reading, and the
  hillclimb procedure. `docs/sdlc/README.md`'s eval paragraph points at it.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | no | nothing is drawn |
| Icons | no | nothing is drawn |
| Identifier SSOT | yes | the split values and the result-file names are read by the runner and by the tests; the tests read the runner's output rather than retyping a list of outcomes into a second place |
| Durability | no | no store, sidecar, or lock is touched; results go under `target/`, which is already disposable |
| Subprocess safety | no | `scripts/run-evals.sh` is a shell script, not a `Command::new` site in the app; the new test spawns `bash` and `git` the way the existing script tests do |
| Documentation | yes | `evals/README.md` and `docs/sdlc/README.md` are English-only developer documents; no user-facing doc changes |
| Local-first | yes | no new network path; the agent CLI was already the suite's only remote dependency |
| Budgets | yes | each agent run is bounded by `EVAL_TIMEOUT` (default 1800 s); transcripts go under `target/`, which `scripts/prune-target.sh` already prunes |

## Flagged concerns

- **The split is tiny.** Seven evals give a held-out set of two. Answered: the
  procedure says so and asks for more runs, not a bigger claim; the split grows
  with the suite.
- **Hiding is not sequestration.** The agent can still `git show HEAD:evals/…` or
  walk up to the main checkout. Answered: requirement 7 makes a peek visible
  rather than impossible, and the README says so.
- **The agent loads the person's own user-level instructions too.** A local run
  is not only the committed steering. Answered: the README names it as a source
  of variance between machines; CI has no user-level instructions.

## Acceptance

- `cargo test --locked` passes, including
  `every_eval_declares_a_split_and_both_splits_are_used`,
  `no_eval_prompt_is_pasted_into_the_steering`, and the `the_eval_runner_*`
  tests, which between them cover requirements 1–10.
- Each new test is watched failing by mutation.
- `bash scripts/run-evals.sh --list` shows the split column.
- `bash scripts/run-evals.sh --only 004 --runs 2` against this branch's commit
  prints a per-eval line, an overall rate with an interval, and a summary path
  whose files exist.

## Rejected alternatives

- Deleting `evals/` from the worktree — the suite's own tests and path checks
  read it, so an agent running `cargo test` would see red for a reason the eval
  did not set up.
- Committing the hidden files in the worktree — commits fire the repository's
  review hooks, and moving `HEAD` changes what checks that diff against it see.
- A model-graded check — no eval has an open-ended output yet.
- Python for the summary — `awk` already covers a Wilson interval and the
  script has no other dependency.
