# Screen: Launching a CLI opens the terminal workspace directly

- **Status**: approved

Width: full session workspace (`Page::Sessions`), plus the existing 520px
`ui_launch_modal` card only when a launch fails.

Before — launching any CLI or plain terminal covered the window with a modal for up to 8 seconds

```
┌────────────────────────────────────────────────────────────────────────────┐
│                        (scrim over current page)                           │
│        ┌──────────────────────────────────────────────────────────┐        │
│        │ ⠋ Claude Code セッションを起動しています…                │        │
│        │ 作業フォルダ   ~/dev/buddypia/operon                     │        │
│        │ プロジェクト   operon                                    │        │
│        │ エージェント   Claude Code                               │        │
│        │ 起動コマンド   claude --session-id …                     │        │
│        │ ──────────────────────────────────────────────────────── │        │
│        │ 経過: 6秒                         [バックグラウンドで続行] │        │
│        └──────────────────────────────────────────────────────────┘        │
└────────────────────────────────────────────────────────────────────────────┘
```

After — loading state (~50–80 ms while `tmux new-session` runs): switches straight to `Page::Sessions` with no modal

```
┌────────────────────────────────────────────────────────────────────────────┐
│ [Claude] Fix bug in parser  [STARTING] · operon                            │
├────────────────────────────────────────────────────────────────────────────┤
│ ┌────────────────────────────────────────────────────────────────────────┐ │
│ │ ターミナル出力を待っています…                                          │ │
│ │                                                                        │ │
│ │                                                                        │ │
│ └────────────────────────────────────────────────────────────────────────┘ │
│ セッションを起動しています…                                                │
└────────────────────────────────────────────────────────────────────────────┘
```

After — active state (~50–80 ms later once `tmux` starts the CLI): output streams immediately at 33 ms intervals and the pane holds keyboard focus

```
┌────────────────────────────────────────────────────────────────────────────┐
│ [Claude] Fix bug in parser  [ACTIVE] · operon / ⎇ main              [停止] │
├────────────────────────────────────────────────────────────────────────────┤
│ ┌────────────────────────────────────────────────────────────────────────┐ │
│ │ ╭─── Claude Code ────────────────────────────────────────────────────╮ │ │
│ │ │ Welcome to Claude Code!                                            │ │ │
│ │ ╰────────────────────────────────────────────────────────────────────╯ │ │
│ │ > ▊                                                                    │ │
│ └────────────────────────────────────────────────────────────────────────┘ │
│ エージェントセッションを起動し、アプリ内ターミナルで開きました。           │
└────────────────────────────────────────────────────────────────────────────┘
```

After — error state (only when `tmux` or the CLI binary fails to launch): existing 520px diagnostic modal appears

```
┌────────────────────────────────────────────────────────────────────────────┐
│                        (scrim over Page::Sessions)                         │
│        ┌──────────────────────────────────────────────────────────┐        │
│        │ ⚠ セッションの起動に失敗しました                         │        │
│        │ 作業フォルダ   ~/dev/buddypia/operon                     │        │
│        │ プロジェクト   operon                                    │        │
│        │ エージェント   Claude Code                               │        │
│        │ 起動コマンド   claude                                    │        │
│        │ ──────────────────────────────────────────────────────── │        │
│        │ 'claude' が PATH 上に見つかりません。                    │        │
│        │ [閉じる]  [再試行]  [設定を開く]                         │        │
│        └──────────────────────────────────────────────────────────┘        │
└────────────────────────────────────────────────────────────────────────────┘
```

No new colour role, icon, font size, widget, or layout is added: the starting
and active views are the existing `ui_terminal_panel` on `Page::Sessions`, and
the error state is the existing failure branch of `ui_launch_modal`.
