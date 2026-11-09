# Screen: the tab of a terminal running Codex or Antigravity

- **Status**: approved

No layout moves, no new control, no new string. The tab draws the mark it
already draws for a session of that kind (change 267's `session_mark`); what
changes is when a terminal's kind becomes the CLI's.

## Session tab strip, a plain terminal in which the person typed `codex` (tab ~180px)

Before — the terminal mark stays for as long as Codex runs:
```
│ [>_] セッション6  × │
```
After — within one poll (about two seconds) of Codex starting:
```
│ [◎ Codex] セッション6  × │
```
The same for `agy`, with Antigravity's mark; `claude` shows Claude's mark as
it did. The name is unchanged.

States:
- The person leaves the CLI and is back at the shell: the mark stays the CLI's,
  as change 270 left it for Claude.
- The person starts a different CLI in the same terminal: the mark moves to it
  within one poll.
- The terminal is gone (tmux lost): no poll reads it; the mark is the last
  one seen.
