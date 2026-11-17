# Screen: change 307 — side-by-side diff, compare with revision, annotate

Width: the editor pane at a typical 900px. Strings are the Japanese the app shows.

## Before — editor, 差分 view

```
┌──────────────────────────────────────────────────────────────── 900px ─┐
│ [✎ 編集] [⇄ 差分]                            ⟳  外部エディタで開く ▾  … │
│ src/git.rs                                              +12  −4        │
│ @@ -40,6 +40,9 @@ fn git_command                                        │
│  40  40   let mut command = Command::new("git");                       │
│  41     - command.arg("-C");                                           │
│      41 + command.args(["-C", path]);                                  │
└────────────────────────────────────────────────────────────────────────┘
```

## After — 差分 view, side by side (default)

A compare bar under the view switch. Left column = older text, right = newer.
Removed lines tint the left, added lines tint the right, a rewritten line keeps
its word highlights on both sides, and the empty side of a pure addition or
removal is a hatched filler so the two columns stay row-aligned and scroll as one.

```
┌──────────────────────────────────────────────────────────────── 900px ─┐
│ [✎ 編集] [⇄ 差分] [≡ 注釈]                   ⟳  外部エディタで開く ▾  … │
│ 比較: [ HEAD（最新コミット） ▾ ] ↔ ローカル   [左右|統合]  ↑ ↓  2 / 5 変更 │
│ src/git.rs                                              +12  −4        │
│ ─ HEAD ──────────────────────────┬─ ローカル ──────────────────────────│
│ @@ -40,6 +40,9 @@ fn git_command                                        │
│  40  let mut command = Comm…     │  40  let mut command = Comm…        │
│  41  command.arg("-C");          │  41  command.args(["-C", path]);    │  ← rewrite pair
│ ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │  42  command.env_remove("GIT_DIR"); │  ← addition
│  42  command                     │  43  command                        │
└──────────────────────────────────┴─────────────────────────────────────┘
```

- `↑` / `↓` (also F7 / ⇧F7, IntelliJ's keys) scroll to the previous / next
  change block and update `2 / 5 変更`. Hover: 「前の変更 (⇧F7)」「次の変更 (F7)」.
- `[左右|統合]` switches layout; 統合 is today's interleaved column. The choice
  is remembered across files and relaunches.
- Staging / discarding a hunk and writing a note stay available, on the hunk
  header row and the right-hand line, only while the comparison is
  「… ↔ ローカル」 against HEAD (the only case where they mean something).

### The revision picker (`比較: [ … ▾ ]`)

```
┌─ 比較するリビジョン ──────────────────────────────── 520px ─┐
│ ● HEAD（最新コミット）                                       │
│ ─────────────────────────────────────────────────────────── │
│   a1b2c3d  2026-11-15  buddypia  fix(git): pass -C once      │
│   9f8e7d6  2026-11-02  buddypia  feat: worktree ports        │
│   …（このファイルの履歴、最大 200 件）                        │
└─────────────────────────────────────────────────────────────┘
```

Picking a commit makes the bar read `比較: [ a1b2c3d ▾ ] ↔ ローカル`.

### Showing what one commit changed (opened from 注釈)

```
│ 比較: a1b2c3d の変更（親 ↔ a1b2c3d）  [HEAD と比較に戻る]  [左右|統合] ↑ ↓ │
│ ─ 1e2d3c4（親） ─────────────────┬─ a1b2c3d ──────────────────────────│
```

## After — 注釈 view (blame), new

Read-only file text with a gutter: short hash, date, author, shown on the first
line of each run of lines from the same commit; later lines of the run leave the
gutter blank. Lines not yet committed show 「未コミット」. Hovering a gutter cell
shows the full commit line: `a1b2c3d · buddypia · 2026-11-15 14:02 · fix(git): pass -C once`.

```
┌──────────────────────────────────────────────────────────────── 900px ─┐
│ [✎ 編集] [⇄ 差分] [≡ 注釈]                   ⟳  外部エディタで開く ▾  … │
│ a1b2c3d 2026-11-15 buddypia │  40 │ let mut command = Command::new(…   │
│                             │  41 │ command.args(["-C", path]);        │
│ 未コミット                   │  42 │ command.env_remove("GIT_DIR");     │
│ 9f8e7d6 2026-11-02 buddypia │  43 │ command                            │
└────────────────────────────────────────────────────────────────────────┘
```

Clicking a gutter cell opens a menu:

```
┌──────────────────────────────────────┐
│ このコミットの変更を表示              │  → 差分 view, 「a1b2c3d の変更」
│ このリビジョンとローカルを比較        │  → 差分 view, 「a1b2c3d ↔ ローカル」
│ コミットハッシュをコピー              │
└──────────────────────────────────────┘
```

(「未コミット」 offers only 「HEAD とローカルを比較」.)

## States

| State | What shows |
|---|---|
| loading blame / history / diff | 「注釈を読み込んでいます…」 / 「履歴を読み込んでいます…」 / 「差分を読み込んでいます…」 |
| file not tracked yet | 注釈: 「このファイルはまだコミットされていません。」 picker: HEAD only |
| no difference | 「このリビジョンとローカルに違いはありません。」 / today's 「このファイルは、最後のコミットから変更されていません。」 for HEAD |
| git error | 「注釈を取得できません: {error}」 / 「差分を取得できません: {error}」 |
| too large | blame of a file past the read ceiling: 「大きすぎるため注釈を表示できません（{limit} KiB まで）。」; diff keeps today's 「… 差分を {p0} KiB で切り詰めました」 |
| narrow pane (< 560px) | side by side falls back to 統合 for that frame; the toggle stays where it was |
