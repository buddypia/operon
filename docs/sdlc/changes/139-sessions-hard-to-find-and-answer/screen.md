# Screen: waiting sessions first, three columns

- **Status**: approved

The person chose mockup A from `.tmp/session-layout/index.html` (the clickable
HTML set drawn for this change): 「A 返事待ちを先頭に置くで進めて」. This file
is that mockup as it will be built, with every place the build departs from the
mockup listed at the end.

## Before — 1280px, session page

```
┌ ● ● ●  Operon  ⌂ホーム  □プロジェクト  ▣セッション        ⌕検索・操作 ⌘K  [＋新しいセッション ⌘N] ⚙ ┐ 52
├──────────────────────────┬──────────────────────────────────────────────────────────────────────────┤
│ プロジェクト別セッション 1件 履歴 │ (C) operon · Codex CLI ○待機中 · operon / ⎇main · 3ポート稼働中   … □停止 │
│ [すべて1] 要対応 実行中 待機中 終了 │                                                                          │ ← spacer
│──────────────────────────│ ──────────────────────────────────────────────────────────────────────── │ ← rule
│ □ operon 1件           ＋ │                                                                          │ ← spacer
│ ● operon · Codex CLI  … × │ ┌──────────────────────────────────────────────────────────────────────┐ │
│   Codex · 待機中 ⎇main     │ │ terminal                                                             │ │
│══════════ split ═════════│ │                                                                      │ │
│ [ファイル] 会話  変更  ⇆ › │ │                                                                      │ │
│ ⌕ ファイルを絞り込む     ⟳ │ │                                                                      │ │
│ › assets                 │ │                                                                      │ │
│ › docs …                 │ └──────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────┴──────────────────────────────────────────────────────────────────────────┘
```

## After — 1280px, three sessions waiting

```
┌ ● ● ●  Operon  ⌂ホーム  □プロジェクト  ▣セッション   [要対応 3 ⌘J] ⌕検索・操作 ⌘K [＋新しいセッション ⌘N] ⚙ ┐ 52
├────────────────────────┬────────────────────────────────────────────────┬───────────────────────┤
│ プロジェクト別セッション 14件 履歴│ (A) サイドバー再設計 ●要対応 · operon / ⎇feature/… … □停止 │ [ファイル] 会話 変更 › │
│ [すべて] 要対応3 実行中5 待機中3 終了3│ ─────────────────────────────────────────────── │ ⌕ ファイルを絞り込む ⟳│
│ 要対応 3                    │ ┌────────────────────────────────────────────┐ │ › assets              │
│ ● サイドバー再設計       10分 │ │ terminal                                   │ │ › docs                │
│   operon · feature/139-lay… │ │                                            │ │ › src                 │
│   spec.md の Requirements を │ │                                            │ │   AGENTS.md           │
│   5 件にまとめました。この方… │ │                                            │ │   CLAUDE.md           │
│ ● 決済 webhook 修正       6分 │ │                                            │ │                       │
│   trip-jarvis · fix/stripe… │ │                                            │ │                       │
│   許可または回答を待っています │ │                                            │ │                       │
│ ● PDF 書き出し高速化   たった今│ │                                            │ │                       │
│   slide-studio · perf/pdf…  │ │                                            │ │                       │
│ □ operon 2件             ＋ │ │                                            │ │                       │
│ ● tmux 再接続のリトライ    … │ │                                            │ │                       │
│ □ trip-jarvis 2件        ＋ │ │                                            │ │                       │
│ ● 旅程 PDF の多言語化     … │ └────────────────────────────────────────────┘ │                       │
└────────────────────────┴────────────────────────────────────────────────┴───────────────────────┘
   268 (drag-resizable)            terminal (the rest)                          240–300 (⌘⌥B folds)
```

Row anatomy in the pinned section: status dot (warning hue), title 13.5
`text_strong`, wait time 11.5 `warning` right-aligned; second line project ·
branch 12 `text_muted`; optional third line, the agent's words or
「許可または回答を待っています」, 12 `text`, at most two lines with an ellipsis.

## Other frames

**Nothing waiting** — no 要対応 section, no title-bar button; the list starts
with the first project group exactly as today.

```
┌ ● ● ●  Operon  ⌂ホーム  □プロジェクト  ▣セッション             ⌕検索・操作 ⌘K [＋新しいセッション ⌘N] ⚙ ┐
│ プロジェクト別セッション 14件 履歴│ …
│ [すべて] 要対応 実行中5 待機中6 終了3│
│ □ operon 3件             ＋ │
```

**⌘J with nothing waiting** — a brief notice 「返事を待っているセッションはありません」;
selection and page unchanged.

**Panel folded (⌘⌥B)** — the right column disappears and the terminal takes its
width; the existing reopen button appears in the session header.

**Panel docked left** — unchanged from change 127: the panel stacks under the
session list, the pinned section stays at the top of the list part.

**A long title or project name** — truncated with an ellipsis inside the 268px
column; the wait time never wraps.

## Where the build departs from mockup A

- The page navigation (ホーム / プロジェクト / セッション) stays in the title bar;
  the mockup's breadcrumb is not added. The doubled rows the person pointed at
  were the session header and the empty band, and the band is what goes.
- No text filter above the list: `README.md` keeps ⌘K as the only search, and ⌘K
  gains the agent's words and puts waiting sessions first.
- No status bar and no ⌘1–9 session jump in this change.
- The status chips stay.
