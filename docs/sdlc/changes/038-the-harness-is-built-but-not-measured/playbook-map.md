# What this repository took from the AI-native SDLC playbook, and what it did not

Source: `the-ai-native-sdlc-playbook.md` (Anthropic), 949 lines, six stages and
fifteen named plays. This file is the evidence base for `intent.md` beside it,
which is about the half of each play that was never built.

It lives in a change directory on purpose. `steering_bytes` globs
`docs/sdlc/*.md`, not `docs/sdlc/changes/*/`, and that metric is already past
its `propose` band — a map of the harness should not make the harness more
expensive to read.

Checked against the repository on 2026-09-10, not from memory.

## Every play, and where it is

| Stage | Play | Instantiated as | Measured by |
|---|---|---|---|
| 1 | Capture as `intent.md` | `docs/sdlc/changes/*/intent.md`, 39 changes | `pipeline-indicators.sh` — intent→spec elapsed |
| 2 | Requirements and design | `spec.md`; `scripts/check-readiness.sh` gates stage 3 | same — spec commits after the first plan commit |
| 3 | Plan mode as the default | `CLAUDE.md`; `plan.md` per change | — |
| 3 | Auto mode | `docs/sdlc/risk.yaml` autonomy tiers (`full`/`supervised`/`paused`) | — |
| 3 | The `CLAUDE.md` | present; `always_loaded_bytes` is a banded metric | `check-bands.sh` |
| 3 | Skills as institutional knowledge | `.claude/skills/` — sdlc, ship, root-cause, durability-invariants | — |
| 3 | Hooks as build-time guardrails | `.claude/hooks/gate-commit.sh` | — |
| 3 | Parallel sessions and subagents | `.claude/agents/` — the three reviewers | — |
| 4 | Give Claude a feedback loop | the healthy-output table in `CLAUDE.md` | — |
| 4 | Continuous evals in CI | `evals/` (7) + `.github/workflows/harness.yml` | **cannot run** |
| 5 | AI in the PR review loop | `REVIEW.md`, `.claude/agents/`, change 035's gate | `pipeline-indicators.sh` — plan→review elapsed |
| 5 | Hooks as approval gates | `.claude/settings.json`, `scripts/check-release-preconditions.sh` | — |
| 5 | CI/CD integration | `.github/workflows/ci.yml` | **cannot run** |
| 6 | Closing the loop | `docs/sdlc/bands.yaml`, `check-bands.sh`, `lessons.md` | `--breaches`, `--lessons` |
| 6 | Recurring codebase scans | `.github/workflows/scheduled-scan.yml` | **cannot run** |
| 6 | Claude on call with Claude Tag | — | **not applicable** |

## The four that cannot run

`.github/workflows/` holds `ci.yml`, `claude-review.yml`, `harness.yml` and
`scheduled-scan.yml`, all added 2026-09-02. This repository has no git remote:
`git remote -v` is empty, `main` has no upstream, `gh` says "no git remotes
found". GitHub never receives a push, a pull request or a schedule tick, so none
has ever run and `scheduled-scan.yml`'s Monday cron has never fired. Each file
now carries a `NOT RUNNING` header saying so.

`bash scripts/pipeline-indicators.sh --workflows` reports this and exits 1.

## The one that does not apply

"Claude on call with Claude Tag" is Slack-based incident response. This is a
local macOS GUI tool with no service, no on-call rotation and no Slack
workspace in scope. Not applied, and not a gap.

## What the playbook asks for that this repository structurally cannot have

The playbook's lagging indicators are written for a team on GitHub with an
incident tracker. Two of them have no local equivalent:

- **first-pass CI success rate** — needs CI history. There is none, and the
  reason is above, not the local-first constraint.
- **defects caught before merge vs escaping to production** — needs an incident
  tracker and a production. Neither exists; this ships as a `.app` a person
  installs.

Tried and rejected as a stand-in for the first: a commit the review gate refuses
still leaves its commit object behind, so `git fsck --unreachable` looked like a
way to count refusals. It is not — 60 unreachable commits are there and the
overwhelming majority are `git stash` entries, indistinguishable from refusals
without a record the gate itself wrote, and `git gc` prunes them on its own
schedule. A first-pass rate needs the decision log, which is `intent.md`'s
subject.

## Where this repository went past the playbook

Recorded because a map that only shows gaps is a misleading map.

- **`docs/sdlc/risk.yaml`** — the playbook's auto-mode play says to decide how
  much an agent may do unattended. This turns it into a table of surfaces with
  tiers, and `paused` surfaces stop the agent before it edits them.
- **`docs/sdlc/lessons.md`** — no play asks for this. Every entry names what now
  catches the mistake, and `--lessons` reports 19 of 19 naming a guard the
  repository actually holds.
- **Change 035's review gate** — the playbook's `REVIEW.md` tells a reviewer
  what to look for; nothing in it makes a verdict *exist*. 035 binds verdicts to
  a diff digest and refuses the commit without them. It is the single largest
  departure, and twenty-one rounds of its own reviewers have not closed it.
- **`scripts/check-bands.sh`** — the playbook suggests watching metrics; this
  gives each a baseline and three thresholds, and names the tier of a breach.
