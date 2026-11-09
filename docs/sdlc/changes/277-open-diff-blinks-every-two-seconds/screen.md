# Screen: the editor's 差分 view while Operon re-reads the diff

No layout, string, colour, or control changes. The Files page, a modified file
open, 差分 chosen; pane about 760px. Operon re-reads the diff every two seconds
so that an agent's next write shows up.

## Before — every two seconds the diff is replaced by the loading line

```
t = 0.0 s  ┌ agent.rs ─ 編集 │ 差分 ──────────────────────────────┐
           │ @@ -1 +1 @@                                          │
           │ - fn main() {}                                       │
           │ + fn main() { first_agent_write(); }                 │
           └──────────────────────────────────────────────────────┘
t = 2.0 s  ┌ agent.rs ─ 編集 │ 差分 ──────────────────────────────┐
           │ 差分を読み込んでいます…                              │  until git answers;
           └──────────────────────────────────────────────────────┘  scroll position lost
t = 2.1 s  the diff again, from the top
```

## After — the diff stays until its replacement has arrived

```
t = 2.0 s  the same rows, same scroll position; the diff is read behind them
t = 2.1 s  unchanged diff: nothing moves.  changed diff: the new rows replace the old
```

- **Empty** (「このファイルは、最後のコミットから変更されていません。」): unchanged; it also
  no longer blinks.
- **Loading** (「差分を読み込んでいます…」): still shown the first time a file's diff is
  read, and after choosing 差分, staging or reverting a hunk, or saving — the
  moments the person asked for a fresh one.
- **Error** (「差分を取得できません: {error}」): replaces the diff when the re-read
  fails, as today.
- **Too long**: a large diff keeps its rows and position across re-reads.
