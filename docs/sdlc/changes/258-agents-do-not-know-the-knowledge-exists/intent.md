# Intent: an agent starting in a project does not know its knowledge exists

- **Status**: approved
- **Opened**: 2026-11-08

## Problem

The person asked (2026-11-07):

> この https://www.anthropic.com/research/components-of-agent-memory の文書を読み、本文の内容も一緒に読んでから、ステップごとに一つずつ適用して。

This is the article's third group of components: steering an agent to use its
memory, and promoting a small top layer of it into the context every session
starts with. The article found that telling an agent what memory is for works
better than prescribing how to use it; that an agent which did not read its
memory before answering gained little until it was prompted to; and that the
injected top layer must stay small, because long instruction files lower
adherence and wrong notes spread from there.

Operon distils each opted-in project's knowledge and offers it to agents through
`Operon knowledge search`. Nothing tells an agent that. A Claude Code or Codex
session started in the project begins with no hint that the knowledge exists,
what it is for, or how to search it; the only mention is one line in the
README. In practice the agent never searches, re-derives what an earlier
session found, and the knowledge serves only the person browsing the tab.

## Who feels it, and when

- A new Claude Code session in a project whose knowledge already says why a
  test is flaky; the agent investigates from scratch.
- The person who turned knowledge on expecting their agents to benefit, and
  sees reference counts stay at zero.

## Desired outcome

- A Claude Code or Codex session started (or resumed, cleared, or compacted) in a
  project with knowledge turned on begins knowing, in a few short lines: that
  the project has this knowledge and what it is for, the exact command to search
  it, what 検証済み and 未確認 mean, and the titles of the few entries most worth
  knowing now.
- A session in a project without knowledge, or with none yet, starts exactly as
  today.
- Starting a session never fails or stalls because of this.
- The person can read, before turning knowledge on, that agents are given this.

## Constraints this change inherits

- Local-first: nothing leaves the machine that the agent's own CLI would not
  already send; titles only, never bodies, and only for opted-in projects.
- The status hooks Operon registers must never break or block the agent.
- User-facing text is Japanese; code, comments, and docs are English.

## Systems likely affected

`src/knowledge.rs`, `src/tmux/hooks.rs`, `src/app.rs`, `src/app/knowledge.rs`,
`src/config.rs`, `src/i18n_tables.rs`, `src/tests.rs`, README files.

## Open questions

None. Step 3 of the article applied here; steps 1 and 2 were changes 256 and 257.

## Not in scope

- Antigravity: its hooks have no session-start event to carry context.
- Sessions when Operon's agent CLI hooks are turned off.
- Putting knowledge into files in the project's tree (it stays private).
