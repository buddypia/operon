# Screen: Terminal pane after clean exit and forced-termination toast

No new modals, panels, or layout changes are introduced. Existing session tabs, left-column session rows, terminal viewport, and the bottom-right `notice_briefly` toast surface carry all state transitions within existing palette roles and glyph marks.

## 1. Clean exit (`exit 0`) — Before vs After

### Before (760px terminal workspace pane)

```
┌─ Sessions workspace ──────────────────────────────────────────────── 760px ─┐
│ [Agy design-lookbook ×]  [Claude api-refactor ×]                      [ + ] │
├─────────────────────────────────────────────────────────────────────────────┤
│ ✓ Done. Updated 3 files in src/components/.                                 │
│                                                                             │
│ Pane is dead (status 0, Thu Nov 12 00:14:09 2026)                           │
│                                                                             │
│ (pane is frozen in SessionStatus::Exited; typing does nothing)              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### After (760px terminal workspace pane)

```
┌─ Sessions workspace ──────────────────────────────────────────────── 760px ─┐
│ [>_ design-lookbook ×]  [Claude api-refactor ×]                       [ + ] │
├─────────────────────────────────────────────────────────────────────────────┤
│ ✓ Done. Updated 3 files in src/components/.                                 │
│ a13973@mac design-lookbook % █                                              │
│                                                                             │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

- No `Pane is dead (...)` line is ever printed (`remain-on-exit-format ""`), and legacy dead-pane tabs from older builds have their `Pane is dead (...)` overlay cleared on respawn.
- Prior CLI output remains in scrollback directly above the login shell prompt.
- The tab and matching left-column row keep their title and position; the mark switches from the finished CLI mark to the existing plain-terminal mark (`TERMINAL_MARK`) because the session is now a live shell (`SessionStatus::Active`, `agent == "terminal"`). Typing `claude`, `codex`, or `agy` switches the mark back to that CLI as today.

## 2. Forced termination mid-run (`SIGKILL`, crash, external `kill-session`)

### Happy state — single session restored (760px workspace + existing toast)

```
┌─ Sessions workspace ──────────────────────────────────────────────── 760px ─┐
│ [● Claude api-refactor ×]  [>_ design-lookbook ×]                     [ + ] │
├─────────────────────────────────────────────────────────────────────────────┤
│ Resuming conversation 7f3c9a12…                                             │
│ > █                                                                         │
│                                                                             │
│               ┌───────────────────────────────────────────────────────────┐ │
│               │ 強制終了されたセッションを自動で復元しました: api-refactor │ │
│               └───────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Multiple sessions restored in the same poll (e.g. `tmux kill-server`)

```
                ┌─────────────────────────────────────────────────────────┐
                │ 強制終了されたセッションを 3 件自動で復元しました。      │
                └─────────────────────────────────────────────────────────┘
```

## 3. Non-happy states

- **Loading / transitioning (`SessionStatus::Starting`)**: Between poll detection and `SessionStarted` (~50–150 ms), `LaunchFocus::Keep` suppresses the modal launch dialog (`launch_progress` stays `None`), keeps current focus and page unchanged, and refreshes terminal output as soon as `SessionStarted` lands if the pane is on screen.
- **Repeat crash within 15s cooldown (`AUTO_RESTORE_COOLDOWN`)**: Automatic restoration does not fire a second time; the session settles in its existing `失敗` (`SessionStatus::Failed`) or `不明` / `消失` (`SessionStatus::Lost`) state with its error output preserved in the pane (no `Pane is dead` text obscuring the stderr), and existing retry/resume controls remain available.
- **CLI killed before any transcript was written (`NoTranscript`)**: No toast is shown; the session settles in `Lost` / `Failed` so its initial prompt is never re-sent behind the person's back.
- **Too-long session title**: Single-session toast uses existing wrapping/clipping inside Operon's standard notice bar; tabs and sidebar rows continue truncating with ellipsis as today.
- **Empty workspace**: Unchanged (`開いているセッションはありません`).
