# Intent: how much of the window is left is a question with no answer here

- **Status**: approved
- **Opened**: 2026-09-06

## Problem

Claude Code's usage limits are the thing that decides whether a piece of work
gets finished today. A person running three agents in parallel is spending the
five-hour window three times as fast, and the only way to find out how much of
it is gone is to ask the CLI in a terminal, which costs a turn, or to notice
that the answers have stopped.

The number exists and is already on this machine: Claude Code pipes it to its
own status-line command on every turn.

## Who feels it, and when

Anybody on a subscription running more than one agent, which is what Operon is
for. The moment it matters is the moment before starting a long task.

## Desired outcome

The share of each window that is spent is visible without asking for it, and
without Operon making a request of its own. Where it comes from is the CLI's own
reporting, so it costs nothing and cannot be wrong in a way the CLI is not.

## Constraints this change inherits

- Local-first: no telemetry, no accounts, no cloud calls. Reading a number the
  CLI already hands to a local script is not a call; polling an endpoint would
  be, and is out.
- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Another product's settings file is not ours to rearrange. Change 015 set the
  rule and this follows it.
- No new dependency: a dependency is a paused surface, and this crate has no
  date library.
- User-facing text is Japanese; code, comments, and docs are English.

## Systems likely affected

`src/tmux/hooks.rs`, which already owns the managed scripts, the settings-file
edits, and the socket. `src/app.rs` for the reading. The toolbar.

## Open questions

- **Where is it drawn?** Answered before the code: a small chip at the right of
  the toolbar, because the fact is about the account and not about a session.

## Not in scope

- The OAuth usage endpoint, which is a cloud call and rate-limits under polling.
- The hidden-PTY `/usage` scrape, which types into somebody's session.
- Token totals and estimated cost from the transcripts. That takes a whole
  subsystem — dedupe keys, cross-file ownership, an incremental cache at
  schema 6 — and it answers a question that is interesting rather than
  actionable. Its own change, if ever.
- Codex and Antigravity limits. Neither reports them to a local script.
