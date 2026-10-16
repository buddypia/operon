# Screen: empty project groups in by-project session lists

No new screen, no re-arrangement: in three existing lists, the weak empty line
of a project with no sessions becomes a hairline button in the same slot.

## Session column (terminal workspace, about 300px)

### Before

```
┌─ セッション 2 件                 [履歴] ┐
│ [すべて 2] [要対応] [実行中 2] …        │
│─────────────────────────────────────────│
│ ▣ operon  2 件                    [＋]  │
│   ● ログイン不具合の修正                │
│   ● README の整理                       │
│                                         │
│ ▣ new-app  0 件                   [＋]  │
│   セッションはまだありません   (weak)   │
│                                         │
└─────────────────────────────────────────┘
```

### After

```
┌─ セッション 2 件                 [履歴] ┐
│ [すべて 2] [要対応] [実行中 2] …        │
│─────────────────────────────────────────│
│ ▣ operon  2 件                    [＋]  │
│   ● ログイン不具合の修正                │
│   ● README の整理                       │
│                                         │
│ ▣ new-app  0 件                   [＋]  │
│   ┌─────────────────────┐               │
│   │ ＋  セッションを開始 │  (hairline)   │
│   └─────────────────────┘               │
│                                         │
└─────────────────────────────────────────┘
```

Clicking 「＋ セッションを開始」 opens the launch sheet for new-app over the
workspace — the same sheet the header ＋ opens.

States in this column:

- Status chips on, project has nothing matching: the group is not drawn (as
  today).
- Every session of the project pinned in 要対応: header only, no button (as
  today).

## History → 「プロジェクト別」 (about 900px)

### Before

```
┌ ▣ new-app  0 件                                  [◉] ┐
│ /Users/me/dev/new-app                                │
│ セッションはまだありません。 [＋]                     │
└──────────────────────────────────────────────────────┘
```

### After

```
┌ ▣ new-app  0 件                                  [◉] ┐
│ /Users/me/dev/new-app                                │
│ ┌─────────────────────┐                              │
│ │ ＋  セッションを開始 │                              │
│ └─────────────────────┘                              │
└──────────────────────────────────────────────────────┘
```

With a search typed and nothing in this project matching:

```
┌ ▣ new-app  0 件                                  [◉] ┐
│ /Users/me/dev/new-app                                │
│ 一致するセッションはありません。  (weak)              │
└──────────────────────────────────────────────────────┘
```

## History → 「グリッド」

### Before

```
┌ new-app  /Users/me/dev/new-app  [◉]                  ┐
│ セッションはまだありません。  (weak)                  │
└──────────────────────────────────────────────────────┘
```

### After

```
┌ new-app  /Users/me/dev/new-app  [◉]                  ┐
│ ┌─────────────────────┐                              │
│ │ ＋  セッションを開始 │                              │
│ └─────────────────────┘                              │
└──────────────────────────────────────────────────────┘
```

Search with no match keeps 「一致するセッションはありません。」 (unchanged).

## Too long

The label is fixed and short; in the narrowest session column (the dock's
minimum) the button is narrower than the project name row above it.
