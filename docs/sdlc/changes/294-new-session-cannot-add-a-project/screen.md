# Screen: the 新しいセッション sheet adds a project in place

Launch sheet, 560px wide, over whatever page is showing (unchanged position).

## 1. Project list opened (header's combo)

```
Before                                       After
┌ 新しいセッション [repo ▾]          ✕ ┐     ┌ 新しいセッション [repo ▾]                ✕ ┐
│   ┌──────────────┐                  │     │   ┌────────────────────────────┐          │
│   │ alpha        │  (added order)   │     │   │ 🔍 プロジェクトを絞り込み    │ ← 8+ only│
│   │ repo       ✓ │                  │     │   │ repo                     ✓ │ ← recent │
│   │ zeta         │                  │     │   │ zeta                       │   first  │
│   └──────────────┘                  │     │   │ alpha                      │          │
                                             │   │ ────────────────────────── │          │
                                             │   │ ＋ 新しいプロジェクト…       │          │
                                             │   └────────────────────────────┘          │
```

- 「＋ 新しいプロジェクト…」 opens the macOS folder picker (title
  「プロジェクトフォルダを選択」). On a choice the sheet's combo shows the new
  project; the request and the session name typed so far stay; the page under
  the sheet does not change. Brief toast: 「プロジェクトを追加しました。」.
- A folder already registered: the sheet switches to it; toast
  「登録済みのプロジェクトに切り替えました。」.
- Not a folder (cannot happen from the picker, possible from a drop):
  notice 「既存のフォルダを選んでください。」; the sheet stays as it was.
- Picker cancelled: nothing changes, no message.
- Filter with no match: the list shows 「一致するプロジェクトはありません。」 above
  the separator; 「＋ 新しいプロジェクト…」 stays.
- Long names: each row is one line, truncated by the popup width (unchanged).

## 2. No project yet (was: notice + jump to プロジェクト)

```
┌ 新しいセッション                                              ✕ ┐
│                                                                  │
│  まだプロジェクトがありません。                                   │
│  セッションを始めるフォルダを選んでください。                      │
│                                                                  │
│  [ 📂 フォルダを選択… ]                                           │
│  Finder からフォルダをこのウィンドウにドロップしても追加できます。  (12px, muted)
└──────────────────────────────────────────────────────────────────┘
```

- No footer (nothing to launch). ✕ and Esc close as today.
- On a choice (button or drop) the sheet becomes the normal form for that project.

## 3. A folder dragged over the window while the sheet is open

The existing full-window overlay; its text reads
「フォルダをドロップして、このセッションのプロジェクトにする」 instead of
「フォルダをドロップしてプロジェクトを追加」. Dropping registers the folder (or
finds it) and the sheet switches to it; the page does not move to プロジェクト.
Several folders: all are registered, the sheet takes the first; toast
「{count} 件のフォルダを追加し、{name} を選びました。」.
