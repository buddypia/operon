# Spec: an eval summary names its conditions and says when little of it was judged

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `scripts/run-evals.sh` prints, on every summary — a rate, `nothing was
   judged`, or `refused:` — one line after the rate's place:
   `conditions: agent A · model M · effort E · split S · recheck R · timeout T`,
   where `A` is the agent command's base name, `M` and `E` the `--model` and
   `--effort` given or `default`, `S` the `--split` given or `both`, `R` `on` or
   `off`, and `T` `EVAL_TIMEOUT` in seconds as `Ns`, or `none` when no timeout
   command bounds the runs. The line reaches `summary.md` with the rest.
   `the_eval_runner_passes_the_model_and_effort_through` asserts it for a run
   with `--model model-x --effort effort-y`, for a run with no flags, and for a
   run with `--split train --recheck`.
2. When some runs were judged and fewer than half of the recorded runs were,
   the summary prints `warning: only J of S runs were judged — fix the infra and
   unstable runs before reading the rate`, and prints no headroom warning.
   Exactly half judged prints no such warning. A new test,
   `the_eval_runner_warns_when_most_runs_were_not_judged`, runs one passing
   eval beside two that crash, three runs each (3 of 9 judged, all passing):
   the warning with `3 of 9` and no `no headroom`; and one passing eval beside
   one crashing (3 of 6): no warning, and the headroom warning as today.
3. `evals/README.md` names both lines in its Reading section, its headroom row
   says the warning gives way to the coverage one, and hillclimb steps 3 and 5
   compare only summaries whose `conditions:` lines match.

## Behaviour

No screen. Runner messages (English, as the rest of its report):

- `conditions: agent claude · model default · effort default · split both · recheck off · timeout 1800s`
- `warning: only 3 of 9 runs were judged — fix the infra and unstable runs before reading the rate`

The runner's exit code does not change: a summary with this warning already
holds a run that did not pass, and exits 1 for it.

## Design

Placement: `EXTEND`. The runner builds the conditions string once, beside
`agent_options`, and hands it to the report awk with `-v`; the awk prints it
after the rate, `nothing was judged`, or `refused:` line, and counts the rows it
read as the scheduled runs. Test messages are Japanese, as the suite's are. No
persisted shape, no new external command, nothing on the draw path.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | nothing painted |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | no glyph |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes, in spirit | the conditions are read from the parsed options the runner already uses, not restated |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | no store |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | no new child; `basename` is a string operation in the runner |
| Documentation — user-facing docs change in all three languages together | no | `evals/README.md` is an English agent document |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | none added |
| Budgets — any new scan or output path states its byte and item ceiling | yes | one line and one count over the `results.tsv` the awk already reads |

## Flagged concerns

- **The timeout field in the tests** — the fixtures set `EVAL_TIMEOUT=60`, and
  the runner prints `none` on a machine with neither `timeout` nor `gtimeout`.
  The test asserts `timeout 60s` when either command is on its `PATH`, and
  `timeout none` otherwise, so it reads the same fact the runner does.
- **Under `refused:`** — the coverage warning prints there too when its counts
  hold; the refusal line comes first and is what the reader acts on.

## Acceptance

- `cargo test --locked eval_runner` passes; the rest of the suite in CI.
- Each new line watched failing on a mutant, including one that keeps the
  counts and weakens the words.

## Rejected alternatives

- Putting the conditions on the rate line: change 184 keeps that line's shape,
  and a `refused:` summary has no rate line to carry them.
- A coverage threshold other than half: below half, the judged runs are the
  minority of the suite and the rate describes less than it omits; the existing
  outcome fixture (6 of 10) stays quiet.
- Refusing the rate instead of warning: a thin rate is still a measurement; the
  person decides whether the infra runs matter, as the README asks.
