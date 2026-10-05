# Intent: add trivial work route to avoid subagent review explosion

- **Status**: approved
- **Opened**: 2026-09-21

## Problem

When AI coding agents (such as Claude Code) perform small, low-risk changes—such as minor internal bugfixes, localized cleanups (<=2 files, <=20 LOC), or configuration adjustments—they frequently default to `feature` or `modify` routes without a mechanism for lightweight handling.

This triggers full subagent review suites (up to 3 dedicated review agents: `rust-reviewer`, `durability-reviewer`, `subprocess-safety-reviewer`). Each nit or minor correction shifts the diff digest, failing the gate and provoking another round of subagent spawning. The agent ends up in an endless ping-pong loop of reviewing and correcting (reaching dozens of review attempts, e.g. review-060), burning tokens and time while blocking delivery of trivial work.

While one-line fixes can bypass the review gate if no SDLC directory exists, changes that legitimately track an SDLC change directory have no option with zero subagents other than the `docs` route.

## Who feels it, and when

Engineers and agent orchestrators working with Claude Code or other autonomous agents on small, deterministic tasks. The session spins out of control spawning subagents repeatedly for minor nits rather than concluding.

## Desired outcome

Introduce a first-class `trivial` route in `docs/sdlc/routes.yaml` that:
1. Requires only deterministic testing (`build,test`) with an attempt ceiling of 1.
2. Closes with `.claude/hooks/gate-commit.sh` and requires no external documents (`-`).
3. Sets `review: none`, waiving LLM review subagents and relying strictly on deterministic gates (`cargo test`, `cargo clippy`, `cargo fmt`).
4. Documents explicit scope boundaries (<=2 files, <=20 LOC, non-high risk surfaces) across `docs/sdlc/README.md` and `.claude/skills/sdlc/SKILL.md`.
5. Passes all structural assertions in `src/tests.rs` (`every_route_the_pipeline_offers_names_stages_that_exist`).

## Constraints this change inherits

- Must strictly adhere to `docs/sdlc/routes.yaml` 5-positional tuple format.
- Must satisfy `every_route_the_pipeline_offers_names_stages_that_exist` in `src/tests.rs`.
- Must not weaken protection for `high` risk surfaces (high-risk surfaces will still be widened by `scripts/check-review.sh` if touched).

## Systems likely affected

- `docs/sdlc/routes.yaml`
- `docs/sdlc/README.md`
- `.claude/skills/sdlc/SKILL.md`
- `src/tests.rs`
