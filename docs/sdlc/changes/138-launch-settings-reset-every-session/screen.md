# Screen: 「この設定を毎回使う」 on the launch sheet

- **Status**: approved

Width: the launch sheet, 560px. Only the footer changes.

Before

```
├──────────────────────────────────────────────┤
│ 選んだ AI は外部ツールとして実行され、…        │
│ 警告: 確認もサンドボックスもなしに…            │  (danger only)
│ ☐ 理解したうえで起動する                      │  (danger only)
│ [ターミナルだけ開く]         [Claude を起動 ⌘↩] │
└──────────────────────────────────────────────┘
```

After — not pinned

```
│ 警告: 確認もサンドボックスもなしに…            │
│ ☑ 理解したうえで起動する                      │
│ ☐ この設定を毎回使う                          │
│ [ターミナルだけ開く]         [Claude を起動 ⌘↩] │
```

After — pinned, form equals the pin (also how the next sheet opens)

```
│ 警告: 確認もサンドボックスもなしに…            │
│ ☑ 理解したうえで起動する                      │
│ ☑ この設定を毎回使う                          │
│ [ターミナルだけ開く]         [Claude を起動 ⌘↩] │
```

After — pinned, something changed for this launch

```
│ ☑ この設定を毎回使う                          │
│ 固定した設定から変更しています。次回は固定した │
│ 設定で開きます。  [今の設定で固定し直す]       │
│ [ターミナルだけ開く]         [Claude を起動 ⌘↩] │
```

The checkbox is the existing `ui.checkbox` look used by the acknowledgement; the
note is `.small().weak()` like the sheet's other footer notes; the button is
`quiet_button`. No new colour role, icon, size, screen, or layout. The person
saw the pinned frame as a preview and chose it.
