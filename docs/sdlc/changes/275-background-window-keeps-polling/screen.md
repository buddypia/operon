# Screen: session list and terminal while Operon is behind another application

No layout, string, colour, or control changes. Only *how often* what is shown
is read again. Window 1280px, sessions page, one terminal open.

## Before — the same rate in front and behind

```
Operon in front     ┌ セッション ──────┬ claude ───────────────────┐  read every 2 s
                    │ ● api  作業中    │ > テストを実行しています… │
                    │ ◐ web  入力待ち  │                           │
                    └──────────────────┴───────────────────────────┘
Operon behind       (the same window, unfocused)                      read every 2 s
```

## After — a third as often behind, at once on return

```
Operon in front     unchanged                                         read every 2 s
Operon behind       same rows, same text, refreshed less often        read every 6 s
click into Operon   the first frame reads again                       read at once
```

- **Empty** (no session): nothing is read in either state; 「セッションを選択」 unchanged.
- **Error** (tmux missing): nothing is read; the existing notice is unchanged.
- **Too long** (many sessions): same rows; the saving grows with their number.
- **Agent with status hooks**: 「入力待ち」 still appears when the hook reports it,
  in front or behind.
