# Screen: the broadcast sheet chooses its recipients

Change 282's sheet, re-arranged: the recipients become rows with a checkbox,
none ticked when it opens, and a 「すべて選択」 control sits beside the count.
This is the drawing the person was shown and chose (「何も選ばない」).

Before (change 282):

```
┌ アイドル中のエージェントへ送信 ─────────┐
│ operon · 送信先 3 件                    │
│  ✳ ログイン画面を直す                   │
│  ◇ README を更新                        │
│  ✳ テストを追加する                     │
│ ┌─────────────────────────────────────┐ │
│ │ 全員に送る内容を入力                │ │
│ └─────────────────────────────────────┘ │
│               [キャンセル] [3 件に送信] │
└─────────────────────────────────────────┘
```

After, as it opens (460px wide, over a scrim, where the sheet was):

```
┌ アイドル中のエージェントへ送信 ─────────┐
│ operon · 選択 0 / 3 件      すべて選択  │
│ ☐ ✳ ログイン画面を直す                  │
│ ☐ ◇ README を更新                       │
│ ☐ ✳ テストを追加する                    │
│ ┌─────────────────────────────────────┐ │
│ │ 選んだ相手に送る内容を入力          │ │
│ └─────────────────────────────────────┘ │
│               [キャンセル] [0 件に送信] │  ← disabled
└─────────────────────────────────────────┘
```

Two ticked, a line typed:

```
│ operon · 選択 2 / 3 件      すべて選択  │
│ ☑ ✳ ログイン画面を直す                  │
│ ☐ ◇ README を更新                       │
│ ☑ ✳ テストを追加する                    │
│ │ 作業をコミットして                  │ │
│               [キャンセル] [2 件に送信] │
```

- **All ticked**: the control reads 「選択を解除」 and unticks every row.
- **A row**: the box and the title both toggle it; the agent's mark sits
  between them, as in the 282 list.
- **More than six**: six rows show and the list scrolls inside the sheet;
  nothing is counted away as 「ほか N 件」.
- **Nobody idle**: 「アイドル中のエージェントはいません」 in place of the count
  line, no rows, no 「すべて選択」, field and button disabled (unchanged).
- **A tab falls idle while open**: a new unticked row; the count's second
  number grows, the first does not.
- **Nobody ticked**: 「0 件に送信」 disabled; Enter keeps the line.
- **Sent**: the sheet closes and 「2 件のエージェントに送信しました」 shows as a
  brief notice.
- **Error**: the existing notice for a terminal that refused its input.
