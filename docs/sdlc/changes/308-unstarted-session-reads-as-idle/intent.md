# Intent: a session that has not been given its first request reads as idle, the same as one that finished a turn

- **Status**: approved
- **Opened**: 2026-11-17

## Problem

The person's request, verbatim:

1. 「まだスタートしてない状態と質問を求められる状況を分けた方が良いと思うけど、ステータスとして」
2. Asked which pair they meant, they chose 「未依頼とターン後が同じ「アイドル」」 — described to
   them as: a session that was only launched and has not been sent its first
   request, and a session that finished a turn and is waiting for the next
   request, both read 「アイドル（ターンを終え、次の依頼を出せます）」.

Today a CLI session that has just started, with nothing asked of it yet, shows
the same mark, the same word (アイドル) and the same explanation (ターンを終え、
次の依頼を出せます) as a session whose agent has finished a piece of work and
is waiting for the next instruction. The explanation is false for the first
one — no turn has ended — and the person cannot tell, from the tab or the
sidebar row, which idle sessions have a result to look at and which are still
blank.

## Who feels it, and when

- Someone who opens several Claude Code or Codex tabs, sends work to some of
  them, and comes back later: every tab that is not working reads アイドル, so
  finding the ones that finished something means opening each one.
- Someone who runs `/clear` in a session: the conversation is new and empty,
  but the row still claims a turn has ended.

## Desired outcome

- A live Claude Code or Codex session whose current conversation has not yet
  received a request shows its own state — its own mark on the tab and in the
  sidebar row, and its own word and explanation on hover — distinct from the
  idle state of a session that has finished a turn.
- The moment the first request is sent (typed into the terminal, sent from the
  launch sheet, or broadcast), the session leaves that state and follows the
  usual working → idle / needs-you course; it never returns to it until the
  conversation is replaced by a new one (`/clear` or a fresh start).
- The new state stays in the アイドル filter group: it is still a session the
  person can hand work to, so counts, filters, and "send to idle agents" keep
  including it.
- A resumed conversation (`--resume`, Operon's own relaunch, a fork, or a
  compaction) is not "not started": it has history, and keeps reading アイドル.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.37 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls, no new `unsafe`.
- User-facing text is Japanese, with English and Korean catalog rows and the
  three-language `README*.md` and `docs/MANUAL*.md` kept in step.
- The persisted store is on real machines; this change must not alter its shape.

## Systems likely affected

`src/tmux/hooks.rs`, `src/app.rs`, `src/app/broadcast.rs`, `src/app/screens.rs`,
`src/ui/widgets.rs`, `src/glyphs.rs`, `src/i18n_tables.rs`, `src/tests.rs`,
`DESIGN.md`, `README*.md`, `docs/MANUAL*.md`.

## Open questions

None — the person chose which pair to separate.

## Not in scope

- Antigravity (`agy`), custom commands, and plain terminals: they report no
  conversation start, so there is no evidence to tell "not started" from
  "finished"; they keep today's behaviour.
- Remembering the state across a restart of Operon itself: sessions that kept
  running while Operon was restarted read アイドル until their next new
  conversation, as today. Persisting it would change the store's shape.
- Separating "the agent asked a question" (要対応) from anything else; that
  state is already its own.
