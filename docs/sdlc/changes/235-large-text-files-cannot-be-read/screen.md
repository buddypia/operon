# Screen: the editor pane for a text file past 128 KiB

The pane is the existing document pane (Sessions workspace file tab, and the
project Files tab). No new screen, modal, or layout region: the content area
that today holds one muted sentence holds the file instead. Width shown is a
typical 720px document pane.

## Before

```
┌─ report.html ───────────────────────────────────────── 720px ─┐
│ [✎ 編集] [± 差分]                         ⟲  ↗  [💾 保存]     │
│                                                                │
│ 748 KiB のファイルです。エディタで開けるのは 128 KiB までです。  │
│                                                                │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

## After — 128 KiB < size ≤ 8 MiB

```
┌─ report.html ───────────────────────────────────────── 720px ─┐
│ [👁 表示] [± 差分]                               ⟲  ↗         │
│ 748 KiB のファイルのため、読み取り専用で表示しています。          │
│ 編集できるのは 128 KiB までです。                    (text_muted)│
│ ┌────────────────────────────────────────────────────── inset ┐│
│ │   1  <!DOCTYPE html>                                         ││
│ │   2  <html lang="ja">                                        ││
│ │   3  <head><meta charset="utf-8"><style>body{margin:0;fo…    ││
│ │      nt-family:system-ui}.card{…                  (continued)││
│ │   4  <body>                                                  ││
│ │ …    (virtual rows, both scrollbars)                       ▐ ││
│ └──────────────────────────────────────────────────────────────┘│
└────────────────────────────────────────────────────────────────┘
```

- Gutter: line numbers right-aligned, `text_muted`; a continuation row has a
  blank gutter. Text: monospace, syntax colours of the file's language.
- No save button. 「表示」 uses `ICON_PREVIEW`.
- Markdown in this size range: the same, 「表示」 only (no プレビュー tab).

## After — size > 8 MiB

```
┌─ dump.json ─────────────────────────────────────────── 720px ─┐
│ [✎ 編集] [± 差分]                               ⟲  ↗         │
│                                                                │
│ 12 MiB のファイルです。表示できるのは 8 MiB までです。            │
│ 右上の「既定のアプリで開く」から開けます。            (text_muted) │
└────────────────────────────────────────────────────────────────┘
```

The tab keeps 「編集」 here, as every unopenable document does today; only
the message changes.

## Unchanged states

- Loading: 「読み込んでいます…」.
- Binary: 「バイナリファイルはエディタで開けません。」
- Not UTF-8: 「UTF-8 ではないため、エディタで開けません。」
- Files ≤ 128 KiB: the editable pane exactly as today.
