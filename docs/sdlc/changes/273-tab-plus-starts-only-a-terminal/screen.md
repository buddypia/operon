# Screen: The tab-strip + menu and compact session tabs

- **Status**: draft

Shown to the person as previews on 2026-11-09 (list menu vs icon row; margins
plus 14pt marks vs margins only); no answer in ten minutes, recommended options
adopted. Widths are points in a 1100px workspace with a 264px sidebar.

Tabs before (each ≈172pt with its gap, 32pt tall; change 267):
```
│ [ (Ⅱ)  [✳]  セッション5    ×  ]  [ (Ⅱ)  [A]  セッション2    ×  ]  [+]
│                                  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```
Tabs after (≈140pt with its gap, 28pt tall; state and CLI marks 14pt, 4pt gaps, 6/2 margins):
```
│ [(Ⅱ)[✳] セッション5  ×] [(Ⅱ)[A] セッション2  ×] [+]
│                         ━━━━━━━━━━━━━━━━━━━━━━━
```
After pressing + (menu directly under it, left edges aligned, popup frame):
```
│ [(Ⅱ)[✳] セッション5  ×] [(Ⅱ)[A] セッション2  ×] [+]
│                                                ┌──────────────────────┐
│                                                │ [>_]  ターミナル       │
│                                                │ [✳]   Claude Code     │
│                                                │ [◎]   Codex CLI       │
│                                                │ [A]   Antigravity     │
│                                                └──────────────────────┘
```
Choosing Claude Code: the menu closes and a new tab opens after the last one,
selected, with the starting state; the notice reads 「セッションを起動しています…」.
```
│ [(Ⅱ)[✳] セッション5  ×] [(Ⅱ)[A] セッション2  ×] [(◌)[✳] セッション7  ×] [+]
│                                                ━━━━━━━━━━━━━━━━━━━━━━━
```
A CLI not on this Mac (here Codex CLI): its row is dimmed and does nothing;
hover says 「未準備（設定で確認）」.
```
│                                                │ [◎]   Codex CLI       │  (dimmed)
```
Many tabs (640px pane): tabs clip before the +, which keeps the right edge; the
menu opens below it and is kept inside the window by egui's area constraint.
```
│ [(●)[✳] long-session-na… ×][(●)[✳] long-se│ [+]
│                                 ┌──────────────────────┐
│                                 │ [>_]  ターミナル       │ …
```
Empty workspace: the + at the left opens the same menu; without a project the +
is disabled. Hover on the + (every pane): 「ターミナルや AI を起動」.
```
│ [+]
│ ┌──────────────────────┐
│ │ [>_]  ターミナル       │ …
│ セッションを選択
```
Remembered options that need the danger acknowledgement (for example a mode
that skips permissions): choosing the CLI opens the existing launch sheet on it,
with the pane's folder, and starts nothing until the person launches there.
