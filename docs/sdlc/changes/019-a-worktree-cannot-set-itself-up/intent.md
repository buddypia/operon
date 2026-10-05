# Intent: a fresh worktree cannot do anything until somebody sets it up by hand

- **Status**: approved
- **Opened**: 2026-09-05

## Problem

A worktree is a fresh checkout, and a fresh checkout of a real project does not
run. Dependencies are not installed, generated files are not generated, the
local database is not seeded. Every project already knows how to fix that — the
instructions are in its README, and in the heads of the people who work on it —
but Operon does not, so the first thing that happens in a brand new worktree is
that a person types the same three commands they typed the last time, from
memory, in the right order.

The same problem lands on the agent. An agent started in a worktree that has not
been set up spends its first turns discovering that the build is broken for a
reason that has nothing to do with its task, and sometimes fixes the symptom.

## Who feels it, and when

Every worktree, every time, on any project with a build step. It is worst on the
projects Operon is most useful for: the ones big enough that running several
agents in parallel is worth doing.

## Desired outcome

A project can say once, in the repository, how a new checkout of it is prepared.
Operon then does that after making a worktree, in a place the person can watch
and interrupt, and the person is told when it finishes and when it fails. A
project that says nothing behaves exactly as it does today.

Because the instruction comes out of the repository, running it is a decision
the person makes and not one the repository makes for them: they see what it is
before it runs, and they are asked again whenever it changes.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.
- No new dependency: a dependency is a paused surface, so the repository's
  instruction cannot be in a format that needs a parser crate.
- Every child process goes through the wrappers in `src/exec.rs`, or is a tmux
  window whose lifetime tmux owns.
- The persisted store is a paused surface.

## Systems likely affected

`src/tmux.rs` for the extra window. A new module for reading the script and
deciding whether it may run. `src/app.rs` for the confirmation and for running
it after a creation. `src/config.rs` for the path and the ceilings.

## Open questions

- **What format does the repository speak?** Answered in `spec.md`: a shell
  script at a fixed path, not a configuration file. A YAML file with a schema
  and five keys was the alternative; the only key that earns its place here is the one that holds a
  shell script, and a file that is already a shell script needs no parser and
  therefore no dependency.
- **Who decides it may run?** Answered in `spec.md`: the person, once per
  content. The screen was agreed before this was written.

## Not in scope

- The rest of a YAML project configuration: default tabs, an archive hook, shared
  directories symlinked into each worktree, environment recipes. Each is its own
  change and none is needed to make the first one useful.
- Making the agent wait for setup to finish before it starts. There is a policy
  for this; it needs a marker file and a wrapper around the launch command, and
  it is worth doing after the plain case is in use.
- Per-project overrides of the script from Operon's own settings.
