# Screen: A session's ports, split by folder, and stoppable

- **Status**: approved

Session header, 1100px workspace (unsplit). Only the ports badge at the end of
the row and the menu it opens change; the agent chip, title, status chip and
branch are as today.

## Header badge

Before (root session; `:3000` runs in `.worktrees/feature-y`, which has no session):
```
│ [Claude] Operon を改善   ● 稼働中 · ⎇ main · ↗ 2 ポート稼働中                       │
```
After — the badge counts only this session's own folder:
```
│ [Claude] Operon を改善   ● 稼働中 · ⎇ main · ↗ :5173 vite                           │
```
After — own folder serves nothing, the project does (badge in `text-muted`):
```
│ [Claude] Operon を改善   ● 稼働中 · ⎇ main · ↗ プロジェクトで 1 ポート稼働中         │
```
Nothing listening in the project: no badge, no separator (as today).

## Menu (opens under the badge, ~360px, scrolls past 240px)

Before:
```
┌──────────────────────────────┐
│ 稼働中のポート                │
│ ──────────────────────────── │
│ [:3000 node]  ⧉              │
│ [:5173 vite]  ⧉              │
└──────────────────────────────┘
```
After:
```
┌────────────────────────────────────────────┐
│ この作業フォルダ                            │
│ ────────────────────────────────────────── │
│ [:5173 vite]                       ⧉   ■   │
│ プロジェクト内のほかの場所                  │
│ ────────────────────────────────────────── │
│ [:3000 node]   .worktrees/feature-y  ⧉   ■ │
│ [:6006 node]   プロジェクトのルート  ⧉   ■ │
└────────────────────────────────────────────┘
```
- `[:5173 vite]` opens the URL in the browser (hover 「ブラウザで開く: http://localhost:5173/」).
- `⧉` copies the URL (hover 「URL をコピー」, notice 「URL をコピーしました。」).
- `■` (ICON_STOP, hover 「このプロセスを停止…」) opens the confirmation below.
  While a stop is in flight every `■` is disabled, hover 「停止しています…」.
- The folder label (muted, truncated, full path on hover) is the folder the
  server's worktree lives in: relative to the project root when inside it,
  the folder name when the worktree lives elsewhere, 「プロジェクトのルート」
  for the root itself.
- An empty section is not drawn: a worktree session with nothing elsewhere
  shows only 「この作業フォルダ」; a session whose own folder serves nothing
  shows only 「プロジェクト内のほかの場所」.

## Confirmation window (centred, 440px, like 「worktree を main にマージ」)

```
┌─ ポートを停止 ─────────────────────────────── × ┐
│ :3000 node を停止しますか？                      │
│                                                  │
│ PID      48213                                   │
│ フォルダ ~/dev/operon/.worktrees/feature-y/web   │
│                                                  │
│ 終了シグナル (SIGTERM) を送ります。Operon の外で │
│ 起動したプロセスも含まれます。                   │
│                                                  │
│                      [キャンセル]  [停止する]    │
└──────────────────────────────────────────────────┘
```
`[停止する]` is in `danger`. `×` or 「キャンセル」 closes it and nothing is sent.
「フォルダ」 is the process's own working directory; when it could not be read
the row says 「不明」.

## After confirming (notice banner)

- Stopped: 「node を停止しました（:3000）。」 — the row is gone on the rescan
  that follows immediately.
- Still listening after 3 s: 「node に停止を求めましたが、まだ :3000 で動いています。」
- Already gone, or the port now belongs to another process:
  「:3000 はもう node が使っていないため、何もしませんでした。」
- Refused (PID 0/1 or Operon itself): 「このプロセスは停止できません。」
- `kill` failed (e.g. another user's process):
  「node を停止できませんでした: {error}」
