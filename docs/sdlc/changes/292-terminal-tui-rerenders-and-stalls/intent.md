# Intent: Working in a CLI TUI inside a session re-renders needlessly, polls blindly, and stalls on incremental updates

- **Status**: approved
- **Opened**: 2026-11-13

## Problem

The person's verbatim request:

> セッションで各CLIをターミナルで起動してユーザーがTUIで作業しているが、ここのパフォーマンスが一番重要です。再レンダリングをしないようにし、効率よく無駄なくパフォーマンスを出す必要性があります。業界標準の動きや実装にしてください。

Today, interactive CLI TUIs (Claude Code, Codex, Gemini CLI, editors, status spinners, prompts) running inside a session pane suffer from four compounding inefficiencies:

1. **Blind timer polling and two-second output lag**: After each keystroke, Operon wakes the UI and spawns a `tmux` child process every 33ms for a full second (~30 wakeups and ~30 child processes per keypress) even after the echoed character has already landed. Conversely, once one second passes without typing, autonomous TUI output (streaming tokens, progress spinners, tool execution logs) updates only once every two seconds (0.5 FPS).
2. **Whole-buffer ANSI re-parsing on every changed byte**: Whenever a single character changes in a session with 5,000 lines of scrollback, Operon copies the entire multi-hundred-kilobyte output buffer to strip carriage returns and re-parses ANSI escape sequences across all 5,000 lines.
3. **Whole-viewport glyph re-shaping on one-row edits or scrolling**: When one TUI status line changes or the user scrolls by a single row, every visible row in the pane discards its shaped text and re-runs font layout across all visible rows.
4. **Spurious UI repaints and hover disk re-probes while idle or streaming**: Unchanged captures and resting the pointer over dotted non-file tokens on static lines during streaming trigger extra UI frames and repeated main-thread path checks, while drawing the session workspace clones project metadata on every frame.

## Who feels it, and when

Every person working interactively inside a session terminal pane with any TUI-based CLI:

- While typing prompts or navigating TUI menus, where unnecessary 30Hz timer polling wastes CPU and adds latency.
- While watching an agent stream code or animate a one-line spinner over a deep scrollback buffer, where output either stutters at 0.5 FPS or re-parses and re-shapes thousands of untouched lines on every tick.
- While leaving one or up to four split session panes open and idle, or resting the mouse pointer over terminal text while output streams.

## Desired outcome

- **Event-driven, frame-coalesced terminal updates**: When a visible session's terminal produces output, the pane updates smoothly within a 60Hz frame budget (~16ms when focused) without waiting for a two-second poll, and without blind 30Hz timer loops after keystrokes.
- **Zero re-rendering when nothing changed**: When a visible session's screen text and cursor are unchanged, Operon schedules zero terminal repaints, performs zero ANSI parsing, and shapes zero text galleys.
- **Dirty-line and dirty-row proportional work**: Changing one line in a 5,000-line session or scrolling by one row parses and shapes only the changed or newly visible line, reusing all untouched lines and shaped rows in place.
- **Lean steady-state frame execution**: Resting the pointer over non-file text on static lines during streaming causes no repeated disk probes or repaint cascades, and all watcher child processes exit cleanly as soon as a pane leaves the screen or Operon closes.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls, no new external crates, and no new `unsafe`.
- User-facing text is Japanese; code, comments, and docs are English.
- The persisted store schema (`STORE_SCHEMA_VERSION`) and paused surfaces (`src/store.rs`, `src/models.rs`, `src/main.rs`, `src/sys.rs`, `Cargo.toml`) remain untouched.

## Systems likely affected

- `exec` — bounded child spawning and clean lifecycle management for long-lived read-only `tmux` control-mode watchers.
- `tmux` — read-only control-mode stream spawning and bounded activity scanning paired with Operon's existing single-invocation cursor-plus-pane capture.
- `ui` (`terminal`) — zero-copy incremental line-level ANSI layout and row-level dirty `Galley` cache with single-pass hover hit-testing.
- `app` (`terminal_output`, `screens`) — per-visible-pane event-driven stream coalescer with respawn cooldown, unchanged-snapshot repaint suppression, dirty-line-aware negative path-cache TTL, and allocation-lean workspace rendering.
- `config` — frame-budget, cache-capacity, and negative-path-TTL constants.

## Open questions

None. Empirical probes against `tmux 3.8` confirmed that `tmux -C attach-session -t <target> -f read-only,ignore-size` preserves pane geometry (`ignore-size`), blocks mutations (`read-only`), emits `%output` / `%layout-change` / `%exit` immediately on PTY activity, and detaches with exit status `0` in ~2ms when `stdin` closes, while Operon's existing `tmux_capture_with_cursor` already captures ANSI text and cursor coordinates in a single `tmux` invocation without `-J`.

## Not in scope

- Replacing `tmux` as Operon's session multiplexer or embedding a custom kernel PTY/VT100 emulator crate (`Cargo.toml` is a paused surface, and Operon's session persistence across app restarts depends on `tmux`).
- Changing the visual appearance, colours, fonts, or layout chrome of the session workspace.
- Altering persisted session/store JSON formats.
