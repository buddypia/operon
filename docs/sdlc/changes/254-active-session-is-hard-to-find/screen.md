# Screen: Existing Sessions workspace

- **Status**: approved

Before (1100px workspace, 264px sidebar):
```
project                │ [state       セッション2        ×][state       セッション1       ×]
  state A セッション2  │
  state A セッション1  │  terminal
```
After (same columns; 32px tab strip, existing controls):
```
project                │ [state セッション2 ×][state セッション1 ×]                [+]
  state A セッション2  │                      ━━━━━━━━━━━━━━━━━━
┃ state A セッション1  │  terminal
```
The selected sidebar row uses row_selected and text_strong with a 2px accent rule; the selected tab uses row_selected and an orange bottom rule. Titles are 12px regular, left aligned and content-sized up to 132px. The plus is 28px and fixed at the pane's right edge. Narrow panes scroll the tabs beneath a clipped region ending before the plus.

Empty: a project with no sessions shows [+] above 「セッションを選択」. No project: plus disabled. Loading: new terminal is selected and retains existing starting state. Error: missing tmux or invalid path reports the existing notice and creates no session. Long: 「とても長いセッション名…」 truncates, full name on hover, close remains visible; the rename editor retains a usable width.
