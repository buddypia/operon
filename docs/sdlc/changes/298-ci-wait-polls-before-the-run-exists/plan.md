# Plan: the CI wait polls before the run exists

- **Spec**: n/a (`bugfix` route — the bug report is in `./state.yaml`)
- **Approved**: 2026-11-16
- **Status**: approved

## Root cause

`.claude/skills/feature-pilot/SKILL.md` §4, `.claude/config/worktree-policy.json`
`landing_steps` and two refusals in `.claude/hooks/gate-merge.sh` all say "wait
for CI with `gh run watch`". Run straight after `git push`, that finds no run:
GitHub lists the push's workflow run a few seconds after the push. With no run
to watch, sessions polled `gh run list` by hand, a tool call per ask; change
297's landing spent 219 turns there. The missing step is "wait until the run
exists", and it is deterministic, so it belongs in a script (AGENTS.md: a script
for a deterministic step).

## The fix

`scripts/wait-ci.sh [commit]`: ask `gh run list --workflow ci.yml --commit <sha>`
every few seconds until a run appears (bounded, default 180 s), then
`gh run watch <id> --exit-status` (bounded, default an hour). Exit 0 on success,
1 on failure or no run, 2 when gh cannot be asked — the codes
`scripts/ci-verified.sh` already uses. Every `gh` call goes through
`scripts/run-bounded.sh`. The landing steps and the merge gate's two refusals
name the script instead of `gh run watch`; the gate's verdict does not change.

## Files that change

| File | Change |
|---|---|
| `scripts/wait-ci.sh` | new |
| `.claude/skills/feature-pilot/SKILL.md` | §4 waits with the script |
| `.claude/config/worktree-policy.json` | `landing_steps` step 1 names the script |
| `.claude/hooks/gate-merge.sh` | two refusal messages name the script; no logic change |
| `src/tests.rs` | `the_ci_wait_waits_for_the_run_to_appear_then_watches_it`, `the_ci_wait_gives_up_when_no_run_appears` |
| `docs/sdlc/changes/298-…/` | state, plan, approvals, mutations, verification |

## Order of work

1. The tests against a fake `gh` that lists no run for its first asks, watched
   failing with no script.
2. The script.
3. The documents and messages.
4. `mutations.py`: watching without waiting, a failed run reported as passed,
   and no bound on the wait each turn a test red.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The run never appears (branch not pushed) | the wait never ends | the appear bound; `the_ci_wait_gives_up_when_no_run_appears` |
| A failed run reported as passed | a red branch looks green to the session | the watch's exit status decides; the merge gate still asks `ci-verified.sh` itself |
| gh hangs on a dead network | the wait never ends | every gh call bounded by `run-bounded.sh` |

## Proof of completion

- `cargo test --locked the_ci_wait_` passes; it failed before the script existed.
- `python3 docs/sdlc/changes/298-…/mutations.py` — every mutation caught.
- This change's own landing waits with `bash scripts/wait-ci.sh` in one call.

## Departures from the plan
