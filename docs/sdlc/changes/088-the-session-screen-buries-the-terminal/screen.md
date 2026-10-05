# Screen: session workspace, before and after

The owner approved the rendered mock in `.tmp/ui-before-after/index.html`
section ① ("全項目 OK、進めて", 2026-09-27). This file is the part of it that
this change builds, drawn at the width in the owner's screenshot (window
~1720px, sidebar 268px).

## Before

```
┌ ⓘ ファイルを開く: 完了 ────────────────────────────────────────────────── ✕ ┐
├──────────────────────┬──────────────────────────────────────────────────────┤
│プロジェクト別セッション 2件+ 🔍│ design-lookbook · Antigravity ○IDLE   ··· 💬 📂 [■停止]│
│ ⚠要対応0   ○空き2      │ A Antigravity · 📁design-lookbook · ⎇develop · ↗11ポート│
│ ◌実行中0   ✓終了0      │ ┌terminal──────────────┐┌会話履歴 最新へ移動┐┌📂 tree ↻┐│
│📁 design-lookbook 2件 +│ │                      ││プロンプトの入力を ││上限付きの…│
│/Users/…/design-lookbook│ │                      ││待っています…     ││(4 lines)  │
│┌design-lookbook·Antigr…┐│ │                      │└──────────────┘│› $R       │
││○IDLE  A               ││ │                      │                 │› .agents  │
││                       ││ │                      │                 │› …(10 dot)│
││⎇ develop              ││ └──────────────────────┘                 └──────────┘│
│└───────────────────────┘│                                                      │
```

## After

```
├──────────────────────┬──────────────────────────────────────────────────────┤
│プロジェクト別セッション 2件  + 🔍│ A .tmpにテスト生成 ○IDLE · design-lookbook / ⎇develop · ↗11ポート   ···  [■ 停止] │
│[すべて 2][要対応][空き 2][実行中][終了]│ ┌terminal─────────────────────────┐┌ファイル│会話 1│変更 1  ⟩┐│
│📁 design-lookbook 2件   +│ │                                 ││[ファイルを絞り込む   ]│
│● .tmpにテスト生成   3分前│ │                                 ││ターミナルに出たファイル│
│  A Antigravity · IDLE   │ │                                 ││ .tmp/test-output.txt │
│● design-lookbook·Antig… │ │                                 ││› api  › assets …     │
│  A Antigravity · IDLE   │ │                                 ││› 隠しフォルダ 10 件   │
│                        │ │                      [↓ 最新へ]  ││ⓘ 一部のファイルを省略 │
│                        │ └─────────────────────────────────┘└──────────────────┘│
│                        │                         ┌──────────────────────────┐ │
│                        │                         │✓ ファイルを開く: 完了    ✕│ │  ← toast, 4 s
```

- The row's `···` and `✕` replace the relative time while the row is hovered
  or selected; hovering the project name shows its full path.
- `停止` keeps its word and slot; its red hover wash is dropped only while the
  agent is idle. 「停止中…」 and a resume that carries a dangerous flag keep theirs.
- With the panel folded (`⟩`), an icon at the header's right reopens it.

## States that are not the happy one

| State | Shows |
|---|---|
| No sessions | sidebar: 「セッションはまだありません。プロジェクトから起動してください。」 (unchanged) |
| Filter hides all | 「選んだ状態のセッションはありません」 + 「フィルタを解除」 (unchanged) |
| No project for the session | panel opens on 会話; ファイル and 変更 tabs are not drawn |
| Worktree session, 変更 tab | 「worktree の変更はプロジェクトの Git タブで確認できます。」 |
| No changes | 「変更はありません」 |
| Changes not loaded yet | 「変更を読み込んでいます…」 |
| Changes could not be read | 「変更を読み取れませんでした: {error}」 |
| Filter matches nothing | 「一致するファイルはありません」 |
| Terminal mentioned no files | the ターミナルに出たファイル section is not drawn |
| No conversation turns | 「プロンプトの入力を待っています…」 (unchanged) |
| Scan truncated | bottom line 「一部のファイルを省略しています」, full warning on hover |
| Failure notice | stays as today's banner with ✕; only success notices become toasts |
| Width below 640 | the side panel is not drawn; terminal takes the width |
