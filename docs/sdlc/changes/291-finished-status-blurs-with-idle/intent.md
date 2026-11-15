# Intent: the Finished status group blurs with Idle and leaves killed sessions dead in their tabs

- **Status**: approved
- **Opened**: 2026-11-12

## Problem

The person's requests, verbatim:
1. 「ステータスの中で終了とアイドルの違いについて教えて。AIの入力待ち（起動したばっかり）がアイドルで終了がAIが何かしら作業後の認識だったけど、今どうなっている？」
2. 「終了とか要る？強制終了とかはセッションIDから自動復元して起動するようにしてほしいけど」
3. 「おすすめは？」 → 「進めて」（以下の構成に明示合意：『終了』グループを廃止して『要対応』『実行中』『アイドル』の3分類にする／強制終了・異常終了は保存済み会話 ID またはシェルから同じタブで自動復元する／自動復元できない異常終了は『要対応』に出す／通常の終了・停止はピン留め中でなければタブを閉じて履歴へ移す）

Today the app puts four status groups in front of the person — 要対応, 実行中,
アイドル, and 終了. To someone running several AI terminals at once, "finished a
turn and waiting for the next instruction" (アイドル) and "finished" (終了) sound
like two words for the same moment, even though one means a live CLI and the
other means a dead terminal. Worse, when a session's process or terminal dies
unexpectedly while Operon is open, Operon only restarts lost sessions once at
app startup; a crash or forced kill during a run leaves the tab sitting dead
under 終了 instead of bringing the conversation back, while sessions the person
deliberately exited or stopped linger beside live work under a fourth filter
bucket nobody asked to manage.

## Who feels it, and when

- Anyone glancing at Home's summary tiles, the session sidebar's filter tiles,
  or the session history filter bar and trying to tell whether 終了 means "turn
  done" or "process exited".
- Anyone whose AI CLI or tmux server is killed or crashes mid-run while Operon
  is already open: the tab goes dead instead of reopening from its saved
  conversation ID.
- Anyone who exits a CLI (`/exit`) or stops a session and still sees an unpinned
  dead tab and a fourth status tile cluttering live workspace navigation.

## Desired outcome

- Status group tiles and filter chips across Home, the session sidebar, and the
  session library show three actionable groups only: 要対応, 実行中, and アイドル.
  終了 no longer appears as a status group tile or filter chip.
- When a running session ends unexpectedly (`Lost` or `Failed`) while Operon is
  running — not only at startup — Operon automatically restores it in the same
  tab from its saved conversation ID (or reopens a shell in its folder for a
  plain terminal), subject to a short per-session cooldown so a session that
  keeps crashing immediately does not loop forever.
- When an unexpected exit cannot be restored automatically (for example, no
  conversation transcript was written yet, a setup run failed, or the session
  crashed again inside the cooldown window), its tab stays open so the output
  can be read and the session is counted and shown under 要対応 until its tab is
  closed and it is read.
- When a session exits cleanly or is cancelled by the person, an unpinned
  workspace tab for it closes automatically so the session moves to History
  (and Recent Sessions on Home), while a pinned tab stays open in place.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls, and no `unsafe`.
- User-facing text is Japanese, with matching English and Korean catalog rows
  and three-language documentation (`README*.md` and `docs/MANUAL*.md`).
- The persisted store is on real machines — `SessionStatus` variants on disk do
  not change, and `STORE_SCHEMA_VERSION` stays `5`.

## Systems likely affected

`ui/widgets.rs`, `app.rs`, `app/relaunch.rs`, `app/session_order.rs`,
`app/screens.rs`, `config.rs`, `i18n_tables.rs`, `tests.rs`, `README*.md`, and
`docs/MANUAL*.md`.

## Open questions

None — the person chose the recommended option B ("進めて").

## Not in scope

- Removing or renaming `SessionStatus::Exited`, `Failed`, `Cancelled`, or `Lost`
  in the persisted JSON store (`store.rs` / `models.rs`).
- Automatically starting a brand-new conversation when a CLI crashed before any
  transcript existed on disk (doing so would silently resend the initial prompt).
- Closing tabs that the person explicitly pinned, or closing tabs when the
  person manually opens a past session from History to read its scrollback.
