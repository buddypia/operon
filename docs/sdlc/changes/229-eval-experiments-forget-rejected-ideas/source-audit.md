# Folder 2: hanakoxbt_article

Read only after first-folder changes 227 and 228 passed CI, landed on main and
were cleaned up (main 4fe8d2f). Read post.md, image_text.md and all six images.
No third-folder contents have been opened. This is an investing-research example;
its product's dataset/pricing/performance/model claims are supplied article
content, not independently verified facts or recommendations for Operon.

| Source idea | Application and evidence |
|---|---|
| Ask a concrete question before implementing | intent/spec/plan pipeline; evals/README.md objective-before-first-run rule |
| Validate actual data coverage and point-in-time inputs | eval split/seeded-by validation; immutable --from commits; hidden checks; add frozen input/definition/coverage fields to experiment record |
| Model proposes, real tools compute | run-evals.sh, named Rust tests, harness-metrics.sh; no model-invented scores |
| Lead/scout/sweeper/skeptic/librarian roles | Main session/feature-pilot, readiness and placement check, repeated evals/mutations, independent reviewers, module map+lessons+knowledge search; reuse roles rather than five extra agents |
| Idea author never grades itself | REVIEW.md, approval evaluator and two independent code reviewers |
| Full grid and nearby settings, never hide failed cells | Existing --model/--effort/--runs controls; add predeclared grids and all-row requirement to experiment guidance/template |
| Held-out data and no look-ahead | Existing fixed train/test split, blind hidden checks, immutable git refs; financial chronological splits do not replace software's fixed held-out case split |
| Fixed evidence-based verdicts | Existing pass/fail/infra/unstable per run; add experiment-level VALIDATED/TRADE_OFF/NO_GAIN/INCONCLUSIVE definitions without treating a nonsignificant score as a gain |
| Remember rejected ideas and require a new reason to retest | GAP: handoff.md preserves dead hypotheses only at failure ceilings; eval.md224 is a precedent but not a reusable experiment record; add template with rejection evidence and explicit reopen condition |
| Skeptic lists weakest point and everything tried | Existing independent review and mutations; experiment template names independent evaluator, worst cell, held-out weakness and limitations |
| Inspect one result by hand before scaling | Existing eval Adding one requires hand pass/fail checks; guidance keeps result inspection before parameter expansion |
| Bound calls/cost and give each agent one job | Existing routing/attempt ceilings and now portable eval deadlines; experiment grid declares run budget before launching |
| Paper before execution; honest limits | Existing worktree tests, CI-before-main, atomic bundle swap; record limitations and independent acceptance before changing runtime |
| Vendor MCP, paid quant warehouse, brokerage/trading runner | Not applicable to a local-first coding-agent desktop manager; no account, trading tool, keys, vendor model IDs or financial-data service added |

## Change

Documentation route: add docs/sdlc/templates/eval.md and link it from the existing
hillclimb guide. It records experiment evidence only; state.yaml remains the
single lifecycle/resume record and no research.md ledger is introduced. Do not
run paid model experiments merely to fill a template or invent measured gains.

Intent/spec/plan/readiness design stages are skipped by the docs route. Existing
document-path checks, fmt/clippy, bands and CI verify the change; independent
source coverage and a concrete template-application evaluation assess guidance.
After this change is verified and landed, read kyr0stack_video as folder 3.
