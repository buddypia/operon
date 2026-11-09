# Screen: a terminal pane while its agent prints

No layout, string, colour, or control changes. The same rows are drawn; they
are prepared on the thread that read them instead of on the frame.

## Before and After — identical

```
┌ claude ─────────────────────────────────────── 900px ─┐
│ > テストを実行しています…                              │
│   running 1287 tests                                   │
│   test poll_275_front_window … ok                      │
│ ▌                                                      │
└────────────────────────────────────────────────────────┘
```

- **Empty / waiting** (「ターミナル出力を待っています…」): unchanged.
- **Error** (「ターミナル出力を読み取れませんでした: {error}」): unchanged.
- **Too long** (5 000 lines of scrollback): same rows; the frame no longer
  spends about 3 ms preparing them each time the text changes.
- **Theme switched**: the pane keeps its rows in the old colours until the
  next capture arrives in the new ones, as today.
