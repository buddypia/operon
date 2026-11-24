# Screen: プロジェクトフォルダを開く

A new sheet, 640px wide, centred over a scrim like 新しいセッション. Opened by
プロジェクト → 「フォルダを選択…」 / 「プロジェクトフォルダを選択…」, ⌘O, the
palette, and 新しいセッション → 「新しいプロジェクト…」 / 「フォルダを選択…」.
Opened from 新しいセッション it takes that sheet's place until it closes.

## 1. Walking and typing (the folder exists)

```
┌─ プロジェクトフォルダを開く ───────────────────────────── 640px ─ ✕ ┐
│ ┌────────────────────────────────────────────────────────────────┐ │
│ │ ~/dev/buddypia/op▏                                             │ │  ← focused on open
│ └────────────────────────────────────────────────────────────────┘ │
│                                                                    │
│ [↑]  ~/dev/buddypia                                    (muted)     │  ← 上のフォルダへ
│ ┌────────────────────────────────────────────────────────────────┐ │
│ │ 📁 operon                                                      │ │  ← click: field becomes
│ │ 📁 operon-site                                                 │ │    ~/dev/buddypia/operon/
│ │ 📁 optics                                                      │ │    and its folders list
│ │                                                         (scroll)│ │  ← about 10 rows tall
│ └────────────────────────────────────────────────────────────────┘ │
│                                                                    │
│ Finder で選ぶ…                              [ キャンセル ] [ 開く ] │
└────────────────────────────────────────────────────────────────────┘
```

- Tab extends `op` to what the matches share; with one match left it becomes
  `~/dev/buddypia/operon/`.
- Enter or 「開く」 opens the typed folder as the project.
- The field ending in `/` lists that folder; no partial, every child folder.

## 2. The path does not exist yet

```
│ │ ~/dev/buddypia/new-idea▏                                       │ │
│ このフォルダはまだありません。作成して開けます。          (muted)     │
│ [↑]  ~/dev/buddypia                                                │
│ ┌────────────────────────────────────────────────────────────────┐ │
│ │ 一致するフォルダはありません。                          (muted)  │ │
│ └────────────────────────────────────────────────────────────────┘ │
│ Finder で選ぶ…                      [ キャンセル ] [ フォルダを作成… ] │
```

After 「フォルダを作成…」 or Enter, the footer becomes the confirmation:

```
│ 「~/dev/buddypia/new-idea」を作成しますか？                          │
│ 途中のフォルダも、まだなければ作成します。                (muted)     │
│                                        [ やめる ] [ 作成して開く ]   │
```

Editing the path, 「やめる」 or Esc brings the footer back. Nothing is created
before 「作成して開く」.

## 3. Other states

| State | Under the field | In the list | Primary |
|---|---|---|---|
| Listing not back yet | — | 読み込み中… | off |
| Folder with no child folder | — | フォルダはありません。 | 開く |
| A file | これはフォルダではありません。 | the parent's matches | off |
| Text that is empty or starts with neither `/` nor `~/` (`dev/x`, `~name`) | / または ~ で始まるパスを入力してください。 | empty | off |
| Folder being made, after 「作成して開く」 | — | unchanged | 作成中… — every button off |
| Parent does not exist either (`~/a/b/c`) | このフォルダはまだありません。作成して開けます。 | この場所はまだありません。 | フォルダを作成… |
| Folder cannot be read | — | このフォルダは読み取れません。 | 開く |
| More than 5 000 entries | — | rows, then 「項目が多いため、先頭の 5000 件だけを読みました。パスを最後まで入力すれば開けます。」 | as above |
| Long name or path | one line, truncated with … | one line per row, truncated | — |

- Names starting with `.` appear only once the typed partial starts with `.`.
- A folder dropped from Finder while the sheet shows fills the field.
- It opens at the parent of the newest project; at `~/` when there is no
  project or that parent is Desktop, Documents, Downloads or a mounted volume.
- 「Finder で選ぶ…」 closes the sheet and opens the macOS folder dialog.
- 「キャンセル」, ✕ and Esc close it with no message; from 新しいセッション the
  sheet underneath returns as it was.

## Unchanged

The projects page keeps its buttons, their labels, and the 「パスを手入力」
group. 新しいセッション keeps 「新しいプロジェクト…」 and 「フォルダを選択…」.
Only what those buttons open changes.
