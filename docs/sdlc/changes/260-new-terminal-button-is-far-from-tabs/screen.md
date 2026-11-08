# Screen: The new-terminal button follows the last tab

- **Status**: approved

Before (1100px workspace, 264px sidebar, two sessions; change 254):
```
project                │ [● セッション2 ×][● セッション1 ×]                              [+]
  ● セッション2        │                  ━━━━━━━━━━━━━━━━
┃ ● セッション1        │  terminal
```
After (same columns, same strip height, same controls):
```
project                │ [● セッション2 ×][● セッション1 ×][+]
  ● セッション2        │                  ━━━━━━━━━━━━━━━━
┃ ● セッション1        │  terminal
```
After pressing + (a third tab starts and is selected):
```
project                │ [● セッション2 ×][● セッション1 ×][◌ ターミナル ×][+]
                       │                                   ━━━━━━━━━━━━━━━
```
Many tabs (640px pane): tabs scroll under a clip that ends before the +, which keeps the right edge:
```
│ [● long-session-na… ×][● long-session-na… ×][● long-se│ [+]
```
Split panes: each pane's + follows its own last tab:
```
│ [● セッション2 ×][+]                │ [● セッション1 ×][+]                │
```
Empty workspace — before: + at the right; after: + at the left, where the first tab appears:
```
│ [+]
│ セッションを選択
│ 一覧からセッションを選んでください。
```
No project: the + is drawn disabled. Missing tmux: 「tmux が必要です。「設定」を開いてください。」 and no tab. Long names truncate at 132px with the full name on hover; the + still follows the truncated tab. Hover on + shows 「空のターミナルを開く」. Icons, colours, sizes and strings are the existing ones.
