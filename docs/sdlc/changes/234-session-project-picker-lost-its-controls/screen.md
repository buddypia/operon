# Restore project navigation without undoing central tabs

1280 logical pixels; the existing left navigator remains approximately280px.

Before: プロジェクト / 新しいセッション / 履歴 / bare project name buttons.

After:
```
┌ 左 280px ───────────┬ 中央 ────────────────────────┬ 検査 ───┐
│ プロジェクト         │ [起動中A][起動中B][file.rs×]   │ ファイル │
│ [新しいセッション][履歴]│                              │ 会話/変更│
│ ▰ Project A  2件  [+]│ 選んだセッション又はファイル    │         │
│ ▱ Project B  0件  [+]│ 必要なら左右/上下へ分割         │         │
└─────────────────────┴──────────────────────────────┴─────────┘
```
Project rows retain the saved drag order and full-path hover identity. Selected
project styling follows focused content. Long names truncate before the count
and launch button. Empty project: central セッションを選択 and existing launch
control; files keep existing 読み込んでいます… and error states. History remains
reachable with its full status/search/row operations. No live session command is
sent during native verification. This restores existing project roles inside
the layout the user explicitly requested and is now correcting.
