# Plan: the eval suite measures, and says how to climb it

- **Spec**: `./spec.md`
- **Approved**: 2026-10-02
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `scripts/run-evals.sh` | `--runs`, `--recheck`, `--split`, `--model`, `--effort`; `EVAL_AGENT`, `EVAL_TIMEOUT`; per-run outcome and `peeked` flag; check hiding; transcripts, `results.tsv`, `summary.md`; Wilson interval, headroom and always-fails warnings; a split column in `--list` |
| `evals/00[1-7]-*.md` | a `split:` line in each front matter |
| `evals/README.md` | trust properties, reading a measured run, the hillclimb procedure, and the split's seed |
| `docs/sdlc/README.md` | the eval paragraph points at the measured run and the procedure |
| `src/tests.rs` | the split test, the prompt-pasting test, and the `the_eval_runner_*` tests |
| `docs/sdlc/changes/103-…/mutations.py` | the sweep that watches each new guard fail |

## Order of work

1. Draw the split once: `printf '%s\n' 001 … 007 | shuf --random-source=<(yes 103)`,
   first two to `test`. Record the command and seed in `evals/README.md` so the
   draw can be repeated; write the `split:` lines.
2. Rewrite `scripts/run-evals.sh`. Keep `--list`, `--only`, `--from`, `--keep`
   and the default of one run exactly; the agent-presence check reads
   `${EVAL_AGENT:-claude}`; a timeout from `timeout` or `gtimeout` when either
   exists.
3. Add the tests. The runner tests copy the script into a scratch repository
   under the temp dir, because the script resolves its repository from its own
   path, and spawn `bash` with the inherited git environment scrubbed. Stub
   agents are small shell scripts written by the test: one that does the work,
   one that does not, one that exits with no result line, one that greps
   `evals/`; every stub records its arguments and what the hidden files and
   `git status` looked like. A check that flips on each call gives `unstable`.
4. Run them; mutate each guard and watch it go red; keep the sweep in
   `mutations.py` beside this plan.
5. Write the README sections and the `docs/sdlc/README.md` pointer.
6. Commit; run `bash scripts/run-evals.sh --only 004 --runs 2` against the commit
   and check the summary files.
7. Review (rust-reviewer, route `craft`), then merge.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A hidden eval file is left modified or skip-worktree after the run | the check's `git diff` sees evals/ or misses a real edit | a runner test asserts the bytes and `git status` the check sees |
| `--keep` worktree inspected with checks still hidden | a person reads the placeholder | files are restored before the check, so a kept tree is the real one |
| The runner tests leak a git worktree or an inherited `GIT_DIR` | flaky or polluted test runs | scratch repo under the temp dir, removed at the end; `forget_inherited_repository` on the spawned `bash` |
| CI's `run-evals.sh --from SHA` changes meaning | the harness workflow fails differently | defaults and exit codes unchanged. CI does not keep `target/eval-results`; uploading it is a workflow edit left out of scope |
| macOS has no `timeout` | runs hang forever | detect `timeout`/`gtimeout`; say once that none is applied |
| `peeked` false positives from paths the agent merely prints | a run flagged that did not look | only assistant tool-use lines are scanned; the flag never changes an outcome |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- each new test red under its mutation in `mutations.py`, green after.
- `bash scripts/run-evals.sh --list` — a split column.
- `bash scripts/run-evals.sh --only 004 --runs 2` — a per-eval line, an overall
  rate with an interval, and an existing `summary.md`.

## Departures from the plan

- **Worktrees moved out of the repository.** The plan kept them under
  `target/eval-worktrees/`. Review round 1 showed that put the main checkout's
  own evals one `../` away and loaded its `CLAUDE.md` as an ancestor; they now
  live in a `mktemp -d` directory under `$TMPDIR`, and the `--keep` tests read
  the path from the runner's output.
- **A failed hide is `infra`.** The plan hid the checks without checking the
  result. Review round 1 found a failed hide let the agent run with the answer
  in view; it is now `infra` and the agent does not start. One more runner test
  and two more mutations (20 in all) came with it.
- **Two gate findings on the first full run.** A `grep -q` at the end of a pipe
  under `pipefail` (`every_gate_script_reads_its_pipes_to_the_end`) and a
  backticked spec phrase read as a path. Both fixed; the spec edit was
  re-approved.
- **The measured run in step 6 found a harness fault, not a steering one.**
  `--only 004 --runs 2` against `1a3d234`: `0/2 passed (95% CI 0.0-65.8%)`,
  `infra 0`, cost $0.99, with the always-fails warning. Both transcripts say the
  same thing: the repository's worktree-policy hook refuses every edit in a
  detached-HEAD worktree, so no eval that edits a file can pass. The instruments
  worked as the article intends — the warning pointed at the eval, and the
  transcript named the cause. The fault predates this change and is change 104.
