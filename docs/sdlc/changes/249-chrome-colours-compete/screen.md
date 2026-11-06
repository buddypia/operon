# Screen: quiet dark chrome

- **Status**: approved

Existing 1440px desktop structure, no moved controls.

Before: orange app name and [新しいセッション]; blue [セッション] and [ファイル].
After:
```
┌ Operon  ホーム  プロジェクト  [セッション] ───── [検索・操作] [新しいセッション] ┐
│ プロジェクト │ session tabs                         │ [ファイル] 会話 変更 │
│ selected row │ session heading / status             │ file tree          │
│              │ terminal CLI output                  │                    │
└───────────────────────────────────────────────────────────────────────────┘
```

Black/grey surfaces; selected tabs and rows #383838 with light text; primary action #FF9438 with dark text; app label light grey. Selected navigation tabs have a 2px orange bottom rule. Focus uses a light outline. Terminal and semantic status retain colours.
Empty: current empty text on dark panel. Loading: current indicator/status. Error/interrupted: existing warning/error colour. Long names: current truncation. No geometry or content changes.
