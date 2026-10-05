# Plan: approve by evaluation unless significant, and earn that trust from escapes

- **Spec**: `./spec.md`
- **Approved**: 2026-09-26, by the owner, after the build (see below)
- **Status**: done

Built on the branch without the owner's answer to the spec question (no reply in
600s). Silence is not approval, so nothing merges to `main` until the owner
approves; everything below is reversible until then.

## Files that change

| File | Change |
|---|---|
| `.claude/skills/sdlc/references/approval.md` | new: the significance rule, the evaluator brief, the ledger format, escapes and demotion |
| `scripts/check-approvals.sh` | new: the checker; modes `<change-dir>`, `--list`, `--summary`, `--metrics` |
| `docs/sdlc/risk.yaml` | `gate-configuration` row gains the two new paths; still four values |
| `src/tests.rs` | six guards named in the spec's Acceptance |
| `.claude/hooks/gate-stop.sh` | runs the checker on changes in play; refusal and one-line summary |
| `scripts/harness-metrics.sh` | three keys; `the_approval_` joins the invariant regex |
| `docs/sdlc/bands.yaml` | `approval_escapes`, `demoted_categories` |
| `.claude/skills/sdlc/SKILL.md` | §4, §6, §7 and the reference table point at the rule |
| `.claude/skills/sdlc/references/screen-approval.md` | the structural split; small changes go to the evaluator; closing section trimmed |
| `.claude/skills/sdlc/references/state-and-resume.md` | `human` items backed by a file go to the evaluator |
| `.claude/hooks/guard-stage.sh` | the screen message's wording only |
| `docs/sdlc/README.md`, `REVIEW.md` | the sentences saying only a person proves an artifact |
| `docs/sdlc/lessons.md` | entry for this change's guard |

## Order of work

1. `approval.md` and `check-approvals.sh`, then the checker's four fixture tests
   (1–4), each watched failing against its mutation.
2. `risk.yaml` row widened; test 5 against approval.md, watched failing.
3. `gate-stop.sh` wiring; test 6, watched failing.
4. harness-metrics and bands; `check-bands.sh` shows no new breach.
5. The prose rewrites, net non-growth of `steering_bytes` where possible.
6. Three gates, lesson entry, review, and this change's own `approvals.log`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A `risk.yaml` row loses its fourth value | the main-branch judge skips a high surface silently | `every_risk_surface_names_paths_that_exist` |
| The checker refuses every legacy change | every Stop in every tree refuses | numbered-above-082 cut, and test 2's legacy case |
| gate-stop refuses twice on one position | a loop the session cannot leave | the checker output folded into the state hash; test 6's second stop |
| bash 3.2 syntax in the new script | the gate is unreachable on the stock shell | `every_gate_script_parses_under_the_stock_shell` |
| The hook runs from the main checkout | this branch's gate-stop edit is inert until merged | stated; test 6 runs the branch's hook directly |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- The six new tests, each watched red under its mutation.
- `the_stop_gate_reports_automatic_approvals_in_one_line` shows the summary on a
  fixture change. (This change is 082 and below the checker's cut, so
  `--summary` on it prints nothing — by design, not a failure.)

## Departures from the plan

- **The digest skips the `- **Status**:` line.** The spec said "the bytes
  before `## Departures from the plan`". Flipping a draft to approved would then
  move the digest the approval was recorded at, so every approval would void
  itself. Guarded by `the_approval_checker_refuses_a_claim_the_ledger_does_not_back`.
- **Only `approval_escapes` is banded.** The spec banded `demoted_categories`
  too; it reads 5 on day one because the paused surfaces start demoted by
  design, so a `max 0` band would breach at birth and a band at 5 says nothing.
- **An automatic verdict in a demoted category is a shadow**, not counted in
  `auto_approvals` or the summary's 自動承認 — the person decided it.
- **`root-cause` names escapes** in its Phase 4, beside `SKILL.md` §10: a defect
  arrives through that skill more often than through the pipeline.
- **Review round 1 (two Importants, one per reviewer, plus one Important):** a
  commented-out approval still backed a claim; a person's later `revise` did not
  outrank the evaluator's `approve`; and the ledger's category was the writer's
  word, so a plan editing a paused surface could be logged as `plan`. Now
  comments are skipped, the last person verdict at a digest decides, and a plan
  on a branch touching a still-demoted paused surface needs a person's approve
  whatever its category. `the_approval_checker_holds_a_paused_edit_to_a_person_whatever_its_category`
  is new. The checker also unsets `GIT_DIR` and friends (sdlc 080).
- **`screen-approval.md` was trimmed further** than its closing section, and
  `approval.md` tightened, to keep `reference_bytes` under its 22000 warn
  (21988).

