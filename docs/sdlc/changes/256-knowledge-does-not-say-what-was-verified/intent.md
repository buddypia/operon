# Intent: a knowledge entry does not say whether anyone checked it

- **Status**: approved
- **Opened**: 2026-11-07

## Problem

The person asked (2026-11-07):

> この https://www.anthropic.com/research/components-of-agent-memory の文書を読み、本文の内容も一緒に読んでから、ステップごとに一つずつ適用して。

The article's first component treats a memory note as a prediction: writing it
down bets that a later session will rely on it. In its benchmark the agents whose
memory helped most closed the loop — each note carried the rule, how it was
verified, and the traps around it — while an agent that wrote untested theories
gained almost nothing. Marking notes as checked or unconfirmed is what let a
later session know which ones to trust without deriving them again.

Operon's project knowledge keeps a kind, a title, and a body for each entry, and
nothing about whether the conversation it came from ever confirmed it. A cause
someone guessed at and a cause a passing test proved look the same in the
knowledge tab and in what `Operon knowledge search` hands an agent. A later
agent either trusts a guess or checks a fact again; it cannot tell which is which.

## Who feels it, and when

- An agent in a new session runs `Operon knowledge search "flaky test"` and gets
  "the cause is the shared temp directory". It cannot tell whether that was
  proved last week or was a theory dropped an hour later.
- The person browsing プロジェクト › ナレッジ before relying on an entry, wanting
  to know whether it was ever confirmed.

## Desired outcome

- Every entry distilled from now on says whether its conversation confirmed it
  (a test or command output showed it, or the person said so) or not, and a
  confirmed entry's text says how it was confirmed.
- An entry first kept unconfirmed becomes confirmed when a later conversation
  confirms the same knowledge; a confirmed entry never goes back.
- The knowledge tab and both outputs of `Operon knowledge search` show which
  entries are confirmed.
- A knowledge database written before this change opens with its entries shown
  as unconfirmed, and an older Operon still opens a database this change wrote.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: the judge is the person's own agent CLI, as today.
- User-facing text is Japanese; code, comments, and docs are English.
- The knowledge database is private, one per project, on real machines.

## Systems likely affected

`src/knowledge.rs`, `src/app/knowledge.rs`, `src/i18n_tables.rs`, `src/tests.rs`.

## Open questions

None. This is the article's first component applied to this repository; the
later components are separate changes.

## Not in scope

- Replacing an entry a later conversation contradicts (the article's
  "dreaming corrects memory"): the next change.
- Telling agents at session start that the knowledge exists: a later change.
- Judging again the entries kept before this change.
