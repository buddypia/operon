# Screen: one ＋ per project header in by-project session lists

No new screen. In three existing lists, the framed 「＋ セッションを開始」 under an
empty project goes, and the header line carries the one ＋.

## Session column (terminal workspace, about 300px)

### Before

```
│ ▣ new-app  0 件                   [＋]  │
│   ┌─────────────────────┐               │
│   │ ＋  セッションを開始 │               │
│   └─────────────────────┘               │
```

### After

```
│ ▣ new-app  0 件                   [＋]  │
│                                         │
```

## History → 「プロジェクト別」

### Before

```
┌ ▣ new-app  0 件                                  [▭] ┐
│ /Users/me/dev/new-app                                │
│ ┌─────────────────────┐                              │
│ │ ＋  セッションを開始 │                              │
│ └─────────────────────┘                              │
└──────────────────────────────────────────────────────┘
```

`[▭]` is `ICON_SHOW`, hover 「プロジェクトを開く」, which opened the launch sheet.

### After

```
┌ ▣ new-app  0 件                                  [＋] ┐
│ /Users/me/dev/new-app                                │
└──────────────────────────────────────────────────────┘
```

A project with sessions has the same `[＋]`, then its cards. Under a search with
no match: 「一致するセッションはありません。」 under the path, as today.

## History → 「グリッド」

### Before

```
┌ new-app  /Users/me/dev/new-app  [▭]                  ┐
│ ┌─────────────────────┐                              │
│ │ ＋  セッションを開始 │                              │
│ └─────────────────────┘                              │
└──────────────────────────────────────────────────────┘
```

### After

```
┌ new-app  /Users/me/dev/new-app  [▭]             [＋] ┐
└──────────────────────────────────────────────────────┘
```

`[▭]` still opens the project page; `[＋]` opens the launch sheet.
