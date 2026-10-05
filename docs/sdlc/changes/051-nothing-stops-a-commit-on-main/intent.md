# Intent: every rule about where work happens is a sentence, and sentences are not refusals

- **Status**: approved
- **Opened**: 2026-09-16

## Problem

This repository has a pipeline, three reviewers, a commit gate and a review gate
that git itself runs — and none of them care *where* the commit came from. All
fifty-six commits in this history landed on `main`, in one linear line, from the
working tree the session happened to be sitting in. There are no branches and no
merges, so there has never been a moment where two sessions' work existed
separately and had to be brought together on purpose.

That held while one session worked at a time. It stopped holding the moment a
second and third CLI were in use on the same checkout, which is the situation
now: change 050 is recorded as `blocked`, and its own note says why — *"Another
session is editing src/app/screens.rs in this same working tree … Nothing in the
repository decides whose tree this is."* A gate that judges the diff cannot help
here, because both sessions produce diffs that are individually fine. What is
missing is the isolation, not the judgement.

The second half is that the destructive git commands are also only prose. A
session that runs `git reset --hard` to clear a conflict destroys another
session's uncommitted work in the same tree, and nothing refuses it.

## Who feels it, and when

Whenever more than one agent session is open on this checkout at the same time —
Claude Code, Codex CLI and Antigravity CLI are all in use here. It shows up as
one session's `cargo test` failing to compile because of another session's
half-finished edit, which is exactly how change 050 stalled. It also shows up
after the fact, as a commit containing two unrelated pieces of work because the
tree held both.

## Desired outcome

A commit on `main` is refused, by a mechanism, in all three CLIs. Opening an
isolated tree is one command that works the same way from each of them. The
destructive git commands that can take another session's work with them are
refused rather than discouraged. None of this is new policy — it is the policy
`AGENTS.md` already states, moved from prose into something that returns non-zero.

## Constraints this change inherits

- macOS only.
- Local-first: no telemetry, no accounts, no cloud calls. A guard that phones
  anywhere is not a guard this repository can take.
- This repository has **no remote**. Anything that assumes `origin`, a push, or
  a pull request is inert here and must be marked inert rather than left to be
  discovered by whoever runs it.
- Three CLIs, three different registration mechanisms. A guard registered in one
  of them is a guard that does not fire in the other two.
- The existing gates keep working. `gate-commit.sh`, `gate-stop.sh`,
  `guard-write.sh` and `.githooks/reference-transaction` are the reason this
  repository can be trusted at all, and a change to `.claude/settings.json` is
  the one change that can switch all of them off at once.

## Systems likely affected

None of the Rust modules. This is entirely harness: `.claude/settings.json`,
`.codex/hooks.json`, `.claude/hooks.json`, the `Makefile`, `.gitignore`,
`AGENTS.md`, and new files under `.cli/` and `.claude/scripts/`.

## Open questions

- Is there a quality-gate command for a Rust repository to give the harness
  tooling? — answered by the user: the cargo three, as `make q.check`.
- Does the `create-pr` skill work here? — answered by the
  repository: no, it needs an SSH `origin` and `gh`, and there is no remote.

## Not in scope

- Any change to the Rust source, the store, or the app.
- Making `create-pr` work. It arrives with the guards because it is one bundle;
  it is documented as inert rather than adapted.
- Adopting a second development pipeline. `.claude/skills/sdlc/SKILL.md`
  remains the single entry point for a development request in this repository.
  An earlier draft of this change configured five further components — including
  a second, competing entry point — and was rolled back for that reason before
  this change was opened. See `plan.md`.
