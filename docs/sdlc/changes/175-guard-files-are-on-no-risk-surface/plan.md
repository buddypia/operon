# Plan: classify the enforcement plane as gate configuration

- **Spec**: `./spec.md`
- **Approved**: 2026-10-18
- **Status**: approved

Tightening only: paths are added to the `gate-configuration` row; nothing is
removed, narrowed, or lowered, and what the main-branch judge reads keeps its
shape (four positional values per surface).

## Files that change

| File | Change |
|---|---|
| `src/tests.rs` | `every_hook_an_agent_cli_runs_is_gate_configuration` |
| `docs/sdlc/risk.yaml` | `gate-configuration` gains `.cli`, `.claude/config`, `.claude/hooks.json`, `.codex/hooks.json`, `scripts/ci-verified.sh`, `scripts/run-bounded.sh`, `scripts/check-readiness.sh`, `.github/workflows`; its comment says why |

## Order of work

1. Write the test; run it against today's table and watch it fail naming the
   `.cli/hooks` guards, `.claude/hooks.json`, and `.codex/hooks.json`.
2. Add the paths to the row and its comment; the test and
   `every_risk_surface_names_paths_that_exist` pass.
3. Stage the change and read `bash scripts/check-review.sh --index`: it must
   name `subprocess-safety-reviewer` for this diff (risk.yaml is on the row).

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The row stops parsing for the gate on main | every commit exit 2 | the gate's own run on this commit; `every_risk_surface_names_paths_that_exist` |
| The test passes vacuously (finds no registration file or no command) | a green test that judged nothing | it asserts it found at least three registration files and a guard in each |
| A prefix covers more than intended | a diff to a section template or `scheduled-scan.yml` asks for the safety reviewer | accepted: directory prefixes are the CODEOWNERS granularity, and the spec names the non-enforcing files they also cover |

## Proof of completion

- `cargo test --locked every_hook_an_agent_cli_runs_is_gate_configuration` —
  fails before step 2, passes after.
- `cargo test --locked every_risk_surface_names_paths_that_exist the_review_gate`
  — passes.
- `bash scripts/check-review.sh --index` names `subprocess-safety-reviewer`.

## Departures from the plan
