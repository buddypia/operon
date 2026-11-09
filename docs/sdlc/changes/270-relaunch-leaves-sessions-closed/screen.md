# Screen: a relaunch brings the workspace back; a terminal running Claude reads as Claude

- **Status**: approved

No new screen, modal, control, or layout. Existing surfaces show a different
state (a tab's mark and a row's label follow the session's kind), the workspace
comes back the way it was left, and one toast sentence is added.

## Tab strip and left column, a terminal in which `claude` was started (workspace 1100px)

Before — kind decided at creation, never revisited:
```
│ ● [>_] セッション3  ×  │ ● [A] セッション2  × │ + │
│ 左列:  [>_] セッション3   ターミナル                │
```
After — from Claude's first hook event carrying its conversation ID:
```
│ ● [✳] セッション3  ×  │ ● [A] セッション2  × │ + │      ← Claude mark (change 267's asset)
│ 左列:  [✳] セッション3   Claude    作業中           │      ← state read like any agent
```
`[>_]` is the terminal mark, `[✳]` Claude's, `[A]` Antigravity's. Same size,
same position, same hover rules as change 267; the hover reads 「Claude」.

## Workspace after quit and relaunch, CLIs still alive in tmux (window 1400px)

Before the quit:
```
┌ operon ─────────────────────────────────────────────────────────────┐
│ ● LT slide × │ ● セッション4 × │ +  ┊ ● セッション5 × │ +            │
│ ┌───────────────────────────────┐ ┊ ┌──────────────────────────────┐ │
│ │ agy …                          │ ┊ │ claude …                     │ │
│ └───────────────────────────────┘ ┊ └──────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
  (design-lookbook: セッション2 and セッション3 in tabs, セッション1's tab closed)
```
Before this change, after relaunch — one group, every live session of one project:
```
┌ operon ─────────────────────────────────────────────────────────────┐
│ ● LT slide × │ ● セッション4 × │ ● セッション5 × │ +                  │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ agy …                                                           │ │
│ └─────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```
After this change, after relaunch — identical to "before the quit", including
the split, its share, the front tabs, and design-lookbook's tabs when that
project is opened. No notice.

## Relaunch after the tmux server is gone (Mac restarted)

Same tabs as above. For the first poll (≤ 2 s) each relaunched tab shows
「起動中」, then 「実行中」 as its CLI reopens the previous conversation. The
selected tab stays selected. Toast (the existing brief notice, bottom bar):
```
│ 前回開いていたセッションを 3 件再開しました。                          │
```
A session that cannot be resumed (no conversation saved) keeps its row's
stopped state and its existing 「再試行」 verb; no toast counts it. A relaunch
that fails shows the existing 「セッションを開始できませんでした: …」 in the
notice bar and the row goes 「失敗」; no launch dialog opens.

## Empty, error, too long

- No saved layout, or a damaged one: today's relaunch (one group of the first
  project's live sessions).
- Layout cannot be written: once per failure streak, notice bar
  「タブの配置を保存できませんでした: {error}」.
- Many tabs: the strip scrolls as today; nothing new to wrap.
