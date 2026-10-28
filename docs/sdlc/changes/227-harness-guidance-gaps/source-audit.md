# Folder 1: 0xwhrrari_post

Read first on 2026-10-28: post.md, image_text.md and all four JPEGs in images/.
The post's linked summary.md and video/ are absent; their contents are not assumed.
.DS_Store is filesystem metadata. No later folder has been read.

| Source recommendation | Operon evidence / application |
|---|---|
| Request contract and success before execution | spec.md Acceptance and state.yaml contract; check-readiness.sh and gate-stop.sh |
| Small project map and selective context | CLAUDE.md module map, path-scoped rules, skill references; harness-metrics.sh and bands.yaml |
| Tools with boundaries and explicit failures | src/exec.rs bounded wrappers; src/agents.rs launch validators; .cli/hooks worktree isolation |
| Durable state beyond conversation | change state.yaml, plan.md departures, approvals.log, review.yaml; local store/history |
| Sensors before autonomy | make q.fast, named tests, CI, check-bands.sh, screen approval, scripts/run-evals.sh |
| Permissions outside the model | git reference-transaction hook, risk.yaml and approval.md; no new bypass |
| Traces and local recovery | git, review digests, mutations, root-cause skill, resume and atomic bundle rollback |
| Instructions as infrastructure | src/tests.rs invariant checks, readiness/review/CI gates and lessons with required guards |
| Bounded retries and escalation | routes.yaml attempts, review-rounds.yaml, state questions, handoff.md |
| Failure improves the system | sdlc Close the loop, root-cause and lessons.md guard requirement |
| Separate reasoning, tools, history | external agent CLIs, Operon launch/exec, archived sessions and git |
| Per-change receipt | GAP: read-only committed-state JSON joining existing records to artifact/policy hashes and rollback parent |
| Comparable model upgrades | evals repeated runs, costs/durations and conditions; unavailable receipt telemetry remains null |
| Start small / scale to risk | routes and risk surfaces; on-demand report rather than workflow runtime |

The twelve checklist items reduce to these rows. Execution isolation here means
worktrees and existing CLI permissions, not an OS sandbox guarantee. The sample
enterprise/network/deploy policy and parallel vendor agents are examples, not
requirements for a single-user terminal manager. No vendor service is added.

## Sequence still to do

After this folder's application is verified and landed, read hanakoxbt_article,
apply its relevant gaps and verify; then do the same for kyr0stack_video.
