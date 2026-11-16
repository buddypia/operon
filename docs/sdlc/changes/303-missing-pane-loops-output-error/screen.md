# Screen: Missing-pane empty states and plain-terminal fallback toast

No new modals, panels, or layout changes are introduced. Existing session tabs, pane headers (`···` -> `今すぐ同期`), terminal viewport placeholder text, and the bottom-right corner overlay (`src/app/toast.rs`, clamped to `200px..=420px`, using `ICON_STATUS_DONE` for brief toasts and `ICON_NOTICE` + close `✕` for persistent notices on `Page::Sessions`, never a top banner) carry all state transitions within existing `Palette` roles, `CONTROL_HEIGHT` controls, and `DESIGN.md` typography levels.

## 1. Open tab with missing tmux pane (`can't find pane: operon-2122260b`) — Before vs After

### Before (760px terminal workspace pane on `Page::Sessions`)

Every redraw frame re-requests tmux capture and re-triggers the persistent bottom-right notice card (`<= 420px`) while the pane stays stuck on 「ターミナル出力を待っています…」:

```
┌─ Sessions workspace ──────────────────────────────────────────────── 760px ─┐
│ [● Claude broken-tab ×]  [>_ other ×]                                 [ + ] │
├─────────────────────────────────────────────────────────────────────────────┤
│ operon / broken-tab                                                         │
│ ● Claude  broken-tab  消失  ⎇ main                         [ 再試行 ] [ ··· ] │
├─────────────────────────────────────────────────────────────────────────────┤
│ ターミナル出力を待っています…                                                 │
│                                                                             │
│                                      ┌────────────────────────────────────┐ │
│                                      │ ℹ ターミナル出力を読み取れませんでした: ✕ │ │
│                                      │   can't find pane: operon-2122260b │ │
│                                      └────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

### After — Case A: Resumable CLI conversation recovered (760px workspace + brief toast)

```
┌─ Sessions workspace ──────────────────────────────────────────────── 760px ─┐
│ [● Claude broken-tab ×]  [>_ other ×]                                 [ + ] │
├─────────────────────────────────────────────────────────────────────────────┤
│ operon / broken-tab                                                         │
│ ● Claude  broken-tab  実行中  ⎇ main                         [ 停止 ] [ ··· ] │
├─────────────────────────────────────────────────────────────────────────────┤
│ Resuming conversation 2122260b…                                             │
│ > █                                                                         │
│                                      ┌────────────────────────────────────┐ │
│                                      │ ✓ 強制終了されたセッションを自動で  ✕ │ │
│                                      │   復元しました: broken-tab         │ │
│                                      └────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

- When two or more lost/failed open tabs recover on startup in the same poll, the corner toast reads: `「前回開いていたセッションを 2 件再開しました。」`.

### After — Case B: Conversation unrecoverable, open tab reopened as plain terminal

```
┌─ Sessions workspace ──────────────────────────────────────────────── 760px ─┐
│ [>_ broken-tab ×]  [>_ other ×]                                       [ + ] │
├─────────────────────────────────────────────────────────────────────────────┤
│ operon / broken-tab                                                         │
│ >_ ターミナル  broken-tab  実行中  ⎇ main                    [ 停止 ] [ ··· ] │
├─────────────────────────────────────────────────────────────────────────────┤
│ a13973@mac broken-tab % █                                                   │
│                                                                             │
│                                      ┌────────────────────────────────────┐ │
│                                      │ ✓ 会話を復元できなかったため、       ✕ │ │
│                                      │   ターミナルとして開き直しました:  │ │
│                                      │   broken-tab                       │ │
│                                      └────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

- When two or more unrecoverable open tabs reopen as shells in the same poll, the `<= 420px` corner toast reads: `「会話を復元できなかったため、2 件をターミナルとして開き直しました。」`.

## 2. Loading, empty, error, and long-title states (760px)

### Loading (`Starting` or `Active` before first capture arrives)

```
┌─ Sessions workspace ──────────────────────────────────────────────── 760px ─┐
│ [◌ Claude starting-tab ×]                                             [ + ] │
├─────────────────────────────────────────────────────────────────────────────┤
│ operon / starting-tab                                                       │
│ ● Claude  starting-tab  起動中  ⎇ main                                [ ··· ] │
├─────────────────────────────────────────────────────────────────────────────┤
│ ターミナル出力を待っています…                                                 │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Queued session before start (`SessionStatus::Queued`)

Zero tmux capture attempts are issued and no notice card appears:

```
┌─ Sessions workspace ──────────────────────────────────────────────── 760px ─┐
│ [◌ Codex queued-task ×]                                               [ + ] │
├─────────────────────────────────────────────────────────────────────────────┤
│ operon / queued-task                                                        │
│ ● Codex  queued-task  待機中  ⎇ main                    [ 今すぐ開始 ] [ ··· ] │
├─────────────────────────────────────────────────────────────────────────────┤
│ 開始待機中のため、まだターミナル出力はありません。                                │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Ended / cooldown-stopped session without captured output (`Lost`/`Failed`/`Exited`/`Cancelled`, `capture_missing == true`)

After repeat crash within the 15-second cooldown (or queued-dependency hold), no capture loop runs and no error notice appears:

```
┌─ Sessions workspace ──────────────────────────────────────────────── 760px ─┐
│ [✕ Claude crashed-twice ×]                                            [ + ] │
├─────────────────────────────────────────────────────────────────────────────┤
│ operon / crashed-twice                                                      │
│ ● Claude  crashed-twice  失敗  ⎇ main                      [ 再試行 ] [ ··· ] │
├─────────────────────────────────────────────────────────────────────────────┤
│ このセッションのターミナルは終了しているため、表示できる出力はありません。            │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Explicit manual refresh failure (`···` -> 「今すぐ同期」 on missing pane)

Clicking `···` -> 「今すぐ同期」 issues one explicit capture request and shows the persistent error once in the bottom-right `<= 420px` corner card without looping on subsequent frames:

```
┌─ Sessions workspace ──────────────────────────────────────────────── 760px ─┐
│ [✕ Claude ended-tab ×]                                                [ + ] │
├─────────────────────────────────────────────────────────────────────────────┤
│ operon / ended-tab              ┌─────────────────────────────────────────┐ │
│ ● Claude  ended-tab  消失       │ Terminal.app で開く（マウス操作・tmux 操作用） │ │
│                                 │ 表示中のログをすべてコピー                 │ │
│                                 │ 今すぐ同期                               │ │
│                                 └─────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────────────────┤
│ このセッションのターミナルは終了しているため、表示できる出力はありません。            │
│                                                                             │
│                                      ┌────────────────────────────────────┐ │
│                                      │ ℹ ターミナル出力を読み取れませんでした: ✕ │ │
│                                      │   can't find pane: operon-2122260b │ │
│                                      └────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Too-long session title (truncated tab/header ellipsis + wrapped corner toast within `<= 420px`)

```
┌─ Sessions workspace ──────────────────────────────────────────────── 760px ─┐
│ [>_ very-long-session-title-that-keeps-going… ×]                      [ + ] │
├─────────────────────────────────────────────────────────────────────────────┤
│ operon / very-long-session-title-that-keeps-going-and-gets-truncated…       │
│ >_ ターミナル  very-long-session-title-that-ke…  実行中      [ 停止 ] [ ··· ] │
├─────────────────────────────────────────────────────────────────────────────┤
│ a13973@mac very-long-session % █                                            │
│                                                                             │
│                                      ┌────────────────────────────────────┐ │
│                                      │ ✓ 会話を復元できなかったため、       ✕ │ │
│                                      │   ターミナルとして開き直しました:  │ │
│                                      │   very-long-session-title-that-kee │ │
│                                      │   ps-going-and-gets-truncated      │ │
│                                      └────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

- **Empty workspace**: unchanged (`「開いているセッションはありません」`).
