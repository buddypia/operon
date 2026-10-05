# Intent: a development request had two entry points and two records of where it stood

- **Status**: approved
- **Opened**: 2026-10-04

## Problem

A set of development skills was brought in so that a request could be taken
from "add this" to an installed build without a person choosing each step. As
first brought in, it described itself as the single entry point for a
development request, while the pipeline skill already said the same of itself.
It kept its own record of where a change stood beside the pipeline's, its own
readiness and approval rules beside the pipeline's, and it told a session to
write its documents before creating the worktree — writes this repository
refuses on `main`. It also added roughly 450 KB of instructions, which put two
control bands at `propose`.

## Who feels it, and when

Any agent session — Claude Code, Codex CLI, or Antigravity CLI — given a
development request. It picks a skill from its description, follows it to the
letter, and is refused at its first write; or it records progress where no hook
reads it, and stops while the pipeline's own record says nothing happened.

## Desired outcome

One entry point that every session finds, under every CLI. It opens the
worktree before anything is written, and every fact about a change is recorded
once, where the pipeline already keeps it. The bands stay below `propose`, and
a test fails if a second entry point, a write before the worktree, or a skill
no flat catalog names comes back.

## Constraints this change inherits

- The pipeline in `docs/sdlc/README.md` stays the only record of a change.
- `CLAUDE.md` and `AGENTS.md` load on every turn; `always_loaded_bytes` stays
  below its `diagnose` tier.

## Systems likely affected

The skills and the root documents an agent is steered by, and `src/tests.rs`.
No application module.

## Open questions

None. The person chose the shape: a thin entry over the existing pipeline, with
no second record and no band raised (2026-10-04).

## Not in scope

Registering `.claude/hooks/gate-commit.sh` for Codex CLI and Antigravity CLI: that
changes gate configuration, a `paused` surface, and is a change of its own.
