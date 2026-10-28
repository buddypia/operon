# Eval: the hypothesis being measured

Evidence for one experiment, kept with its change as `eval.md`. Link its intent
and acceptance criteria rather than copying them. Lifecycle and the next action
belong only in `state.yaml`. Freeze the following before running; record all
outcomes before writing a conclusion.

## Frozen question, inputs and budget

- Hypothesis and criterion that would disprove it:
- Intent/spec acceptance reference, or the bug being investigated:
- Baseline and candidate commit IDs:
- Eval definitions and fixtures: revision/digests, coverage and known gaps:
- Train/held-out case IDs, frozen before seeing results:
- Conditions: machine, agent, model, effort, timeout and recheck setting:
- Parameter grid, adjacent settings, repeats and maximum run/time/call budget:
- Smallest useful gain and the uncertainty rule that decides it:

## All results

Keep failed, flat, infra and unstable cells alongside successful ones. Link
results and their hashes; preserve the compact table here because raw transcripts
under `target/` can be pruned. Never copy keys or sensitive transcripts into git.
Use unknown for unavailable cost/duration, not zero.

| Configuration / split | Passed / judged | Infra / unstable | Cost / duration | Evidence reference + hash | Limits |
|---|---|---|---|---|---|
| Baseline | | | | | |
| Candidate | | | | | |
| Adjacent settings, when parameter-dependent | | | | | |

## Verdict and independent challenge

Choose one and explain it against the frozen criterion:

- **VALIDATED**: required train and held-out improvements are supported by the
  stated uncertainty rule, with no regression or increased missing results.
- **TRADE_OFF**: measured benefits and losses remain; name both and leave the
  acceptance decision to the change's existing approval process.
- **NO_GAIN**: comparable evidence does not support a useful improvement. This
  is a result to preserve, not a reason to keep tweaking the same experiment.
- **INCONCLUSIVE**: missing coverage, mismatched conditions, infra, unstable
  checks or insufficient samples prevent the claimed comparison. Repair the
  measurement before choosing a winner.

State plateau versus isolated winning setting when a grid was relevant. An
isolated peak does not establish robustness. A verdict does not establish
unmeasured production behavior.

- Independent evaluator (not the experiment's author):
- Attacks tried, weakest held-out cell, worst case and unresolved limits:
- Decision and its reference in the change's existing approval/review record:

## Rejected hypotheses and reopening evidence

Read related changes' experiment records before spending the next run. Reopen
only with the new input, corrected measurement or changed assumption stated here;
repeating the same configuration without a new reason is not a new experiment.

| Rejected hypothesis / configuration | Evidence that rejected it | New evidence required to reopen |
|---|---|---|
| | | |
