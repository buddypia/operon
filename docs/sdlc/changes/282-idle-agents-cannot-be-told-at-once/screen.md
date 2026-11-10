# Screen: the sheet that sends to every idle agent

A new sheet, opened from the command palette's new row
「アイドル中のエージェントへ送信…」. It is the shape the person was shown and
chose.

```
┌ アイドル中のエージェントへ送信 ─────────┐
│ operon · 送信先 3 件                    │
│  ✳ ログイン画面を直す                   │
│  ◇ README を更新                        │
│  ✳ テストを追加する                     │
│ ┌─────────────────────────────────────┐ │
│ │ 作業をコミットして                  │ │
│ └─────────────────────────────────────┘ │
│               [キャンセル] [3 件に送信] │
└─────────────────────────────────────────┘
```

- 460px wide, over a scrim, placed as the 新しいセッション sheet is; the title
  in the sheet's heading size, the project line muted.
- **More than six recipients**: six titles, then 「ほか N 件」.
- **Nobody idle**: 「アイドル中のエージェントはいません」 in place of the project
  line, no list, the field and 「0 件に送信」 disabled.
- **Empty field**: the send button disabled.
- **Enter** in the field sends; **Escape** or キャンセル closes; a click on the
  scrim does nothing.
- **Sent**: the sheet closes and 「3 件のエージェントに送信しました」 shows as a
  brief notice.
- **Error**: the existing notice for a terminal that refused its input.
