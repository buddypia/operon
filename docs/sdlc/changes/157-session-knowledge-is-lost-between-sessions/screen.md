# Screen: 157 — ナレッジ tab and the model block in Settings › データ

- **Status**: draft
- Structure: one new project tab (a person approves) and a re-drawn block inside
  an existing settings section.

## Project › ナレッジ — enabled, entries present (880px content width)

```
┌─ 概要  Git  ワークツリー  PR  ファイル  エージェント設定  [ナレッジ] ─────────────── 880px ─┐
│                                                                                           │
│  ナレッジ                                              [ 今すぐ更新 ]  [ 収集を止める ▾ ]  │
│  142 件 ・ 最終更新 3分前 ・ 本日の判定 7/30 回                                            │
│  ┌───────────────────────────────────────────────────────────────┐                       │
│  │ 🔍 ナレッジを検索                                              │                       │
│  └───────────────────────────────────────────────────────────────┘                       │
│  並び: (●) 鮮度  ( ) 参照回数  ( ) 新しい順        種類: [すべて ▾]                      │
│ ─────────────────────────────────────────────────────────────────────────────────────── │
│  決定  tmux の再試行は3回で打ち切る                    ████████░░  参照 12  Claude 10/14 │
│  原因  復元で最初のターンが消えるのは header 行の誤分類  ██████░░░░  参照 4   Codex 10/09 │
│  ▾ 手順  リリースは package-macos.sh → replace-macos-bundle.sh  ███████░░░  参照 7  📌   │
│     │ dist/Operon.app を作ってから、起動中のアプリを止めて置き換える。                   │
│     │ 古いバンドルはゴミ箱へ。ロールバックが必要なときだけ残す。                         │
│     │ 出典: Claude Code ・ 2026-10-12 「release の手順を確認」   [ 会話を開く ]          │
│     │                                         [ 固定を外す ]  [ 削除 ]                  │
│  注意  agy の --print は sandbox なしだと書き込む       ███░░░░░░░  参照 0   agy  09/30  │
│  …                                                                                        │
│ ─────────────────────────────────────────────────────────────────────────────────────── │
│  エージェントから参照: /Applications/Operon.app/Contents/MacOS/Operon knowledge search "…" │
│                                                                              [ コピー ]   │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

- Freshness bar: ten cells, filled = freshness × 10, `accent` fill on `muted`
  track; below 0.3 the fill is `warning` (about to fade). 📌 marks a pinned entry.
- Kind badge text: 決定 / 原因 / 手順 / 注意 / 事実.
- Rows are one line, title truncated with 「…」; the opened row shows the body,
  every source (one line each with 「会話を開く」), and the actions.
- A search replaces the order with relevance and shows 「(76%)」 beside each title.

## Disabled (the default for every project)

```
┌─ ナレッジ ────────────────────────────────────────────────────────────── 880px ─┐
│                                                                                 │
│  このプロジェクトの会話からナレッジを集めます                                     │
│                                                                                 │
│  Claude Code・Codex・Antigravity との会話から、決定・原因と対処・手順・注意点を   │
│  短いナレッジにまとめ、会話ごと検索できるようにします。使われないものは徐々に     │
│  薄れて消え、参照されたり繰り返し出てきたりしたものは残ります。                   │
│                                                                                 │
│  ・意味の判定は、この Mac 上の EmbeddingGemma 2 で行います。                      │
│  ・残す候補だけを、その会話を行った CLI に要約させます (1日 30 回まで)。          │
│  ・ナレッジはこの Mac の Operon データフォルダに保存し、リポジトリには書きません。 │
│                                                                                 │
│                              [ このプロジェクトでナレッジを集める ]               │
└─────────────────────────────────────────────────────────────────────────────────┘
```

Without the model, the button reads 「モデルを取得して始める (310MB)」 and shows the
download progress in place.

## Enabled, empty / collecting / degraded

```
│  ナレッジ                                              [ 今すぐ更新 ]  [ 収集を止める ▾ ] │
│  会話を整理しています… (3/12)                                                        │
│                                                                                       │
│  まだナレッジはありません。このプロジェクトで会話を続けると、                         │
│  重要なやり取りがここに追加されます。                                                 │
```

Status line variants (replacing the counts line, `warning` text):
- 「Claude Code で判定できませんでした。次回の更新で再試行します。」
- 「本日の判定回数の上限に達しました。明日続きを処理します。」
- 「モデルがないため停止中」 + [ モデルを取得 ]

「収集を止める ▾」 opens: 「収集を止める (ナレッジは残す)」 / 「ナレッジを削除…」. The
second turns the row into an inline confirmation:
「142 件のナレッジを削除します。元に戻せません。 [ 削除する ] [ キャンセル ]」.

## Settings › データ — model block, before → after

Before (change 142):
```
│  検索エンジン  (●) キーワード検索のみ  ( ) 内蔵 EmbeddingGemma 2      │
│  [ モデルをダウンロード (約 250MB) ]                                   │
│  ● 有効 (Metal GPU / キャッシュ済み: 248MB)   [ キャッシュを削除 ]     │
```

After (520px settings column):
```
┌─ 意味検索（EmbeddingGemma 2） ──────────────────────── 520px ─┐
│  この Mac 上で動くモデルです。会話の内容は外部に送りません。     │
│  検索  (●) キーワードのみ  ( ) 意味検索                          │
│                                                                 │
│  not downloaded:  [ モデルを取得 (310MB) ]                       │
│  downloading:     取得中… 124MB / 310MB   ▓▓▓▓▓▓░░░░  [ 中止 ]   │
│  verifying:       確認中…                                        │
│  ready:           使用可能 (GPU (Metal) / 310MB)  [ モデルを削除 ] │
│  failed:          取得できませんでした: ファイルが壊れていたため   │
│                   破棄しました                       [ 再試行 ]  │
└─────────────────────────────────────────────────────────────────┘
```

「意味検索」 cannot be selected until the model is ready; choosing it with no
model starts 「モデルを取得」.

## Sessions › ローカル履歴 with the model ready

```
│  ローカル履歴  [ 復元で最初のターンが消える            ]  [🔍 履歴を検索 ]   │
│  意味で一致 (ナレッジが有効なプロジェクト)                                   │
│   operon  Codex   復元時の header 行の扱いを直す      (81%)   10/09          │
│   operon  Claude  restore の first turn 欠落の調査   (77%)   10/08          │
│  キーワードで一致                                                            │
│   Codex  「…first turn…」                                                    │
```
