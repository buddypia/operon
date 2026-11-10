# Screen: a reply field on the 要対応 row

One control is added at the foot of each waiting row whose terminal is
running. Nothing else in the section moves.

Before:

```
┌ 要対応 2 ────────────────────────────┐
│ ● ✳ ログイン画面を直す          3分 │
│   operon · ⎇ feature/login           │
│   テストを実行してもよいですか？     │
│                                      │
│ ● ◇ README を更新               1分 │
│   operon                             │
│   許可または回答を待っています       │
└──────────────────────────────────────┘
```

After:

```
┌ 要対応 2 ────────────────────────────┐
│ ● ✳ ログイン画面を直す          3分 │
│   operon · ⎇ feature/login           │
│   テストを実行してもよいですか？     │
│   ┌──────────────────────────────┐   │
│   │ 返信を入力して Enter         │   │   hint, muted, until typed in
│   └──────────────────────────────┘   │
│ ● ◇ README を更新               1分 │
│   operon                             │
│   許可または回答を待っています       │
│   ┌──────────────────────────────┐   │
│   │ はい▏                        │   │   being typed; Enter sends
│   └──────────────────────────────┘   │
└──────────────────────────────────────┘
```

- The field is the row's full inner width, one line, 12px text
  (`FontId::proportional(12.0)`, the size of the line above it), in the
  theme's ordinary text-field look. Each running waiting row grows by the
  field's height, about 24px, and what is below moves down by as much.
- **Japanese input**: the Enter that confirms a conversion keeps the caret in
  the field; the next Enter sends.
- **Sent**: the field empties, 「返信を送りました」 shows as a brief notice, the
  screen stays on the session it was on; the row leaves the section when its
  agent starts working.
- **Not running** (the terminal ended): the row is as before, with no field.
- **Empty section**: not drawn, as before.
- **Error**: the existing notice 「ターミナルに入力を送れませんでした: …」.
- **Selected row**: the field keeps its own look on the selected row's fill.
- Clicking the field puts the caret in it and does not open the session;
  clicking anywhere else on the row opens it, as before.
