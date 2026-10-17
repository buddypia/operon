# Intent: the guards every agent CLI runs are on no risk surface

- **Status**: approved
- **Opened**: 2026-10-17

## Problem

The risk table decides how much a session may do on its own by what a change
touches, and which reviewer it must get. Its "gate configuration" row lists the
Claude Code hooks, their settings, the git hooks, and the review and release
scripts — but not the guards that all three agent CLIs run on every command,
the policy file those guards read, the hook registrations of the other two
CLIs, the scripts the merge and stage gates call, or the CI workflows the merge
gate trusts. A change to any of those matches no row. Change 173 widened the
trunk allowlist — what main permits — and the table had nothing to say: the
session recorded it as unclassified, chose a tier by judgement, and asked the
person whether the table should change. The person did not know what to answer.

## Who feels it, and when

Every session whose change touches a guard, its policy, or CI: it is handed full
autonomy and no extra reviewer by default, on exactly the files that can switch
the other checks off. And the person, who is asked to settle a classification
question in this repository's own terms.

## Desired outcome

A change to anything that decides what a gate permits, or that a gate runs, is
classified as gate configuration: the same tier, autonomy, approval split, and
reviewer as the Claude Code hooks already get. A guard registered for any agent
CLI later is classified the day it is registered, without anyone remembering to
edit the table. The person's instruction, 2026-10-17: 「リスク分類もよくわから
ない。業界標準やベストプラクティスでやって」 — use the industry practice, which is
that a repository's enforcement plane (CI workflows, policy-as-code, and the
hooks that enforce it) is protected like the code it protects: CODEOWNERS and
branch protection on CI configuration, review required to relax a control.

## Constraints this change inherits

- `scripts/check-review.sh` on main reads `docs/sdlc/risk.yaml` as four
  positional values per surface; a row stays that shape.
- docs/sdlc/risk.yaml is itself gate configuration: adding a path is tightening,
  which `.claude/skills/sdlc/references/approval.md` lets the evaluator approve.

## Systems likely affected

`docs/sdlc/risk.yaml`, `src/tests.rs`.

## Open questions

None. Which paths: those the gates execute (from the three hook registration
files), the policy they read, and the checks they delegate to — measured from
the tree, not chosen by taste.

## Not in scope

Splitting `.claude/skills/` by which skill is a procedure and which a policy;
reclassifying any existing row; the review policy's reviewer table, which
already maps gate configuration to `subprocess-safety-reviewer`.
