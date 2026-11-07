# Intent: knowledge a later conversation proved wrong stays as it was

- **Status**: approved
- **Opened**: 2026-11-08

## Problem

The person asked (2026-11-07):

> この https://www.anthropic.com/research/components-of-agent-memory の文書を読み、本文の内容も一緒に読んでから、ステップごとに一つずつ適用して。

This is the article's second component. Memory written as an agent works is
local and incremental, so it collects stale and contradicted notes; the article
shows a wrong note steering an agent off a ledge until a later pass checked the
notes against what actually happened and corrected them. Anthropic's dreaming
pass exists for exactly this: stale or contradicted entries are replaced with
the latest value rather than kept beside it.

Operon's knowledge only ever adds or reinforces. When a later conversation says
the opposite of an entry — the retry limit moved from 3 to 5, the cause turned
out to be something else — the new item is either close enough in meaning to
count as a repeat, in which case the old wording is kept and its count goes up,
or it is added as a second entry beside the old one. Either way the entry that
is now wrong keeps being shown, and the more it is "reinforced" by talk about
it, the longer it lives.

## Who feels it, and when

- An agent searches the knowledge for the retry limit and is told 3, a week
  after a conversation changed it to 5 and said so.
- The person sees two entries on one subject that disagree, and cannot tell
  which is current.

## Desired outcome

- When a conversation shows an existing entry is wrong or out of date, that
  entry now says what the conversation found, in place: one entry on the
  subject, carrying the newer content.
- An entry the person pinned is never rewritten.
- An entry that is only repeated keeps behaving as today.

## Constraints this change inherits

- Local-first: the judge is the person's own agent CLI, inside the existing
  30-calls-a-day budget; no extra call is made for this.
- User-facing text is Japanese; code, comments, and docs are English.
- The knowledge database is private, one per project, on real machines.

## Systems likely affected

`src/knowledge.rs`, `src/config.rs`, `src/tests.rs`, README files.

## Open questions

None. Step 2 of the article applied here; step 1 was change 256.

## Not in scope

- A separate background pass that re-reads old conversations (it would spend
  judge calls the daily budget does not have).
- Showing a history of what an entry used to say.
- Telling agents at session start that the knowledge exists: the next change.
