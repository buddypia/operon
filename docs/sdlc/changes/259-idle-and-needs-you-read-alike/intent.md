# Intent: the idle group reads like the needs-you group

- **Status**: approved
- **Opened**: 2026-11-09

## Problem

The person asked: 「要対応とかいる？指示待ちと要対応は一緒じゃない？」 Told that
the two are different — 要対応 is a session stopped mid-turn on a question or a
failed prerequisite, 指示待ち is one that finished its turn and is free — they
answered 「それで進めて」 to the recommendation: keep 要対応 and give the idle
group a word without 「待ち」.

Both words read as "something is waiting on you". 「指示待ち」 literally says the
agent awaits your instruction, which is also what 要対応 means, so the four
filter chips do not tell a person which sessions they must act on and which
they may leave alone. The idle state's hover explanation 「次の依頼を待っています」
repeats the same reading.

## Who feels it, and when

Someone scanning the session list or the tab bar to decide where to go next:
a session reading 「指示待ち」 looks as urgent as one reading 「要対応」, though
only the second stays stuck until they act.

## Desired outcome

The idle group's word and its hover explanation say the agent is done and free,
not that it is waiting on the person, so 要対応 is the only status that reads as
a request for action. Korean, which used 「지시 대기」, gets the same distinction.

## Constraints this change inherits

- User-facing text is Japanese; code, comments, and docs are English.
- The idle word must not be 「待機中」, which names a queued session (change 221).

## Systems likely affected

The status group word and hint in `src/ui/widgets.rs`, the message tables in
`src/i18n_tables.rs`, the READMEs and manuals in three languages.

## Open questions

None: the person chose the direction; the exact word is a reversible wording
choice recorded in the spec.

## Not in scope

- Which states belong to which group, and the home banner that counts unread
  completions as needing you — unchanged.
- The English word "Idle", which already reads as free.
- Re-shooting `docs/images/01-home.png`.
