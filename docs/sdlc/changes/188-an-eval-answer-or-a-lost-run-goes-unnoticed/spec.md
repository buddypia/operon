# Spec: an eval's answer stays out of the steering, and every run is counted once

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `no_eval_prompt_is_pasted_into_the_steering` also refuses a steering
   document that contains a line of any eval's `## Setup` or `## Check` block,
   where a line counts when it is at least `EVAL_ANSWER_LINE_MIN_CHARS` (20)
   characters once trimmed and is not a shell comment. Measured on 2026-10-21:
   145 such lines, none in the steering. Shorter lines (`set -e`, `fi`) are
   shell grammar every script shares.
2. The same test proves the extension is not vacuous: it counts the setup and
   check lines it compared, and a mutant steering text holding one of them is
   refused with that eval's name.
3. `scripts/run-evals.sh`, before the rate, checks that `results.tsv` holds
   one row per (eval id, run number). On a duplicated pair the summary prints
   `refused:` naming it, prints no rate line, and the runner exits 1 even when
   every row passed. A short `results.tsv` is not checked: the loop appends one
   row per eval file per run and an interrupted run dies before the report, so
   that branch could not be reached or watched failing.
4. A runner fixture with two eval files sharing an id, both passing, shows the
   refusal: exit 1, the `refused:` line, and no `evals:` rate line. The
   existing runner fixtures, whose rows are unique, still print their rate.
5. `evals/README.md`'s Reading section names the `refused:` line, and its
   hillclimb step 6 says a setup or check line is caught like a prompt.

## Behaviour

No screen. Messages:

- Suite (Japanese): `{eval} の {section}「{line}」が {document} にあります`;
  `eval の setup と check が一行も読めていません`; a mutant that passes:
  `変異 {what} を見逃しました — 検査が検査していません`.
- Runner (English, as the rest of its report): `refused: eval {id} run {run}
  was recorded {n} times — two eval files share id {id}; no rate is printed`
  only.

## Design

Placement: `EXTEND` for both. The test keeps its document walk and adds two
sections to the lines it compares. The runner's report awk gains a `(id, run)`
count, checked in `END` before the rate; the runner's final exit reads the
summary for a `refused:` line. No persisted shape, no new
external command, nothing on the draw path.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | nothing painted |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | no glyph |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes, in spirit | the lines are read from the eval files, not copied into the test |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | no store |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | no new child; the fixture uses the existing runner helper |
| Documentation — user-facing docs change in all three languages together | no | `evals/README.md` is an English agent document |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | none added |
| Budgets — any new scan or output path states its byte and item ceiling | yes | the test reads the eval files and steering it already reads; the runner reads its own `results.tsv` |

## Flagged concerns

- **False positives** — measured at 20, 30, and 40 characters: none on today's
  tree. A future steering line that legitimately quotes a check is a leak by
  Harness²'s rule; the eval is what changes, not the threshold.

## Acceptance

- `cargo test --locked no_eval_prompt_is_pasted_into_the_steering` and the
  runner fixture test pass; the rest of the suite in CI.
- Each new check watched failing on a mutant.

## Rejected alternatives

- Refusing at commit with a hook: the suite in CI is where this repository's
  validator lives (change 178).
- Deduplicating rows instead of refusing: a merged count is the defect; the fix
  is to say so, as Harness²'s budget identity does.
