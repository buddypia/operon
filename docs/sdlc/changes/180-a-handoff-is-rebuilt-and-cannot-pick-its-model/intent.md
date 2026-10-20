# Intent: A handoff cannot choose the model it starts, and a mistaken one has to be built again

- **Status**: draft
- **Opened**: 2026-10-20

## Problem

> 「Handoffのモデルも調整できるようになった方が利便性やコスト削減になると思う。
> 特に問題ない？Handoffを一度作成したら使い回しができるようにしたい。キャッシュみたいに無駄にならないようにしておきたい。なぜならCodex CLIへHandoffするつもりで間違えてClaude CodeにHandoffしてしまってまたHandoffを再度作ったりはもったいないからそのまま利用したいね。TTLみたいに自動的に消えて欲しい。」

A conversation handed to another CLI always starts that CLI on whatever model
and reasoning depth the CLI's own configuration names. A person who wants the
continuation to run on a cheaper or a stronger model than their everyday one
has no way to say so in Operon; the only lever is editing the other CLI's
configuration file, which changes every other use of that CLI too.

Every handoff also copies the whole source transcript into a fresh archive
entry, even when the same, unchanged conversation was handed off a minute
earlier. The handoff note typed for the first attempt is cleared once it is
used, so after picking the wrong destination the person types it again for the
second attempt. And the archive entries are never removed: each one holds a
whole transcript, prompts, tool output and source code, and they accumulate
for as long as the app is used.

## Who feels it, and when

- A person handing a Claude Code conversation to Codex CLI who wants the
  continuation on a lighter model to save usage, or on a heavier one for a hard
  step, without changing the model every Codex session on the machine uses.
- A person who picks Claude Code in the destination picker when they meant
  Codex CLI, presses the button, notices, and hands the same conversation off
  again: today that rebuilds the archive and loses the note they wrote.
- Anyone who uses handoff regularly: the archive directory keeps every copied
  transcript indefinitely.

## Desired outcome

- Before handing a conversation off, the person can set the model and the
  reasoning depth the destination CLI starts with, per destination CLI; the
  choice is remembered across restarts, applies to handoffs started from the
  history tab and from the session list menu alike, and leaving it empty keeps
  today's behaviour (the CLI's own default).
- Handing off the same conversation again while it has not changed since the
  previous handoff reuses the earlier snapshot and its note rather than
  copying the transcript again, and the notice says the earlier bundle was
  reused. If the conversation has moved on since, a fresh snapshot is taken so
  the destination never receives an out-of-date excerpt.
- Handoff archives that have not been used for seven days are deleted
  automatically, at startup and whenever a handoff runs; archives written by the
  earlier full-history restore are left alone.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.
- The persisted store is on real machines — this change must not alter its
  shape. The handoff records already stored keep loading and displaying.
- The archive holds whole transcripts: it stays owner-only, nothing is copied
  outside the app data directory, and nothing is deleted outside it.

## Systems likely affected

- `src/history.rs` — the handoff import: reuse, expiry, launch options.
- `src/cli.rs` — the imported-session carrier and the archive helpers.
- `src/agents.rs`, `src/agents/settings.rs` — launch-command construction and
  the remembered per-destination handoff options.
- `src/app.rs`, `src/app/screens.rs` — the history card's controls, the session
  menu, the notice, and the startup sweep.
- `src/config.rs`, `src/i18n_tables.rs`, `README*.md`.

## Open questions

- None that need the person. The expiry period (seven days since last use) and
  what counts as "unchanged" (the source transcript's size and modification
  time) are decided in the spec; both are constants a later change can move.

## Not in scope

- Choosing the destination's sandbox, permission mode, or extra switches for a
  handoff.
- Stopping or removing the session a mistaken handoff already started.
- Reusing one archive across different source conversations, or summarising a
  conversation with a model.
