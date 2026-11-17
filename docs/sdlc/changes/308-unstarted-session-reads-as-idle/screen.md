# Screen: 未開始 beside アイドル

No layout moves. One state gains its own badge glyph, word, and hover; the
アイドル group's hover sentence widens. Widths are today's.

## 1. Workspace tab strip (28px strip, 14px marks)

```
Before — two Claude Code tabs, one just launched, one that finished a turn
┌──────────────────────────────────────────────────────────────────────┐
│ [Ⓒ‖ 新しいセッション  ×] [Ⓒ‖ ログイン画面の修正  ×] [Ⓒ⟳ API の型 ×] │
└──────────────────────────────────────────────────────────────────────┘
   hover on either of the first two:
   ┌──────────────────────────────────┐
   │ Claude Code                      │
   │ アイドル                         │
   │ ターンを終え、次の依頼を出せます │
   └──────────────────────────────────┘

After
┌──────────────────────────────────────────────────────────────────────┐
│ [Ⓒ• 新しいセッション  ×] [Ⓒ‖ ログイン画面の修正  ×] [Ⓒ⟳ API の型 ×] │
└──────────────────────────────────────────────────────────────────────┘
   hover on the first:                 hover on the second (unchanged):
   ┌────────────────────────────────┐  ┌──────────────────────────────────┐
   │ Claude Code                    │  │ Claude Code                      │
   │ 未開始                         │  │ アイドル                         │
   │ まだ最初の依頼を受けていません │  │ ターンを終え、次の依頼を出せます │
   └────────────────────────────────┘  └──────────────────────────────────┘
```

- `Ⓒ` is the CLI's mark; the character after it is the 9px state badge over
  its lower right corner, in the state's tone. `•` is the new `StatusMark::Dot`
  (a dot in the badge's surface colour on the アイドル `calm` ink), `‖` is
  today's pause. Same size, same colour; the shape tells them apart.
- No working time is appended: no turn has run.

## 2. Session sidebar (default 268px) — rows and tiles

```
Before                                   After
┌──────────────┐ ┌──────────────┐        ┌──────────────┐ ┌──────────────┐
│ 要対応     0 │ │ 実行中     1 │        │ 要対応     0 │ │ 実行中     1 │
└──────────────┘ └──────────────┘        └──────────────┘ └──────────────┘
┌─────────────────────────────┐          ┌─────────────────────────────┐
│ アイドル                  2 │          │ アイドル                  2 │
└─────────────────────────────┘          └─────────────────────────────┘
 operon                                   operon
  Ⓒ‖ 新しいセッション                      Ⓒ• 新しいセッション
  Ⓒ‖ ログイン画面の修正                    Ⓒ‖ ログイン画面の修正
  Ⓒ⟳ API の型                              Ⓒ⟳ API の型
```

- The アイドル count still includes the not-started session (2).
- Hover on the アイドル tile:
  - Before: 「ターンを終え、次の依頼を出せるセッション」
  - After: 「未開始、またはターンを終えて次の依頼を出せるセッション」

## 3. Status chip (History rows, Home rows)

```
Before                         After
[ ⏸ アイドル ]                 [ ◉ 未開始 ]      (just launched)
[ ⏸ アイドル ]                 [ ⏸ アイドル ]    (finished a turn)
```

`◉` is Phosphor `RECORD` (a ring holding a dot), the chip form of the same
shape the tab badge draws.

## 4. Non-happy states

- **Empty**: no sessions — no row, no tab; nothing new is drawn.
- **Loading**: between launch and the CLI's `SessionStart`, the tab reads
  `実行中` with the power badge (`Starting`, hover 「tmux と CLI を準備中」),
  as today; then it turns to `•` 未開始.
- **Error / no evidence**: Antigravity, a custom command, a plain terminal, or
  a CLI whose hooks are not installed reads `‖` アイドル when idle, as today.
  After Operon itself restarts, sessions that kept running read `‖` アイドル.
- **Too long**: the word 未開始 (3 characters) is shorter than アイドル; the
  chip and hover reflow as they do today. Long session titles truncate with
  `…` before the close button, unchanged.
