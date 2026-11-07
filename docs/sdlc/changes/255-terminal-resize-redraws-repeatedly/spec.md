# Spec: send a terminal one size per resize gesture, and keep its layout across size changes

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Once a session has been sent a size, a changed size is sent only after the
   measured grid has held still for `TERMINAL_RESIZE_SETTLE` (150 ms); every new
   size restarts the wait, and the request reports the time left so the frame
   wakes when it is due. —
   `session_255_terminal_resize_waits_until_the_size_settles`
2. A session that has never been sent a size is sent its first size at once. —
   `session_255_first_terminal_resize_is_sent_at_once`
3. When a resize finishes while a newer, still-changing size is waiting, the
   newer size is not sent until it has settled. —
   `session_255_resize_completion_does_not_send_an_unsettled_size`
4. Ending a divider or tab drag sends the final size at once (the gesture's end
   is its settling). — existing `terminal_resize_debounces_during_divider_drag`
5. A frame whose pane changed size or position, with unchanged output, visible
   rows, and theme, reuses the laid-out rows; the caret box is still placed for
   this frame's position. —
   `session_255_pane_size_change_reuses_rows_and_moves_the_caret`

## Behaviour

Nothing a person reads changes; no string is added. During a resize the pane
keeps showing the last capture (clipped by the pane, or with space to its right)
and reflows once, 150 ms after the size stops changing. A maximize or restore
animation is one gesture. Minimizing to the Dock gets the same rule: a size
that flickers during the animation and returns is never sent.
A failed resize still reports 「ターミナルのサイズを変更できませんでした: {error}」
as today; a vanished tmux server still marks the session lost as today.

## Design

Placement: **EXTEND** — `request_terminal_resize` / `flush_terminal_resize` in
`src/app.rs` already own resize sending, and `TerminalPaneRenderCache` in
`src/ui/terminal.rs` already owns row layout reuse. Route `modify`.

- `src/config.rs`: `TERMINAL_RESIZE_SETTLE: Duration = 150 ms`, replacing the
  inline 150 ms drag throttle.
- `src/app.rs`: `last_terminal_resize_flushed` becomes
  `terminal_resize_changed_at: HashMap<Uuid, Instant>`, set when the requested
  size changes. `flush_terminal_resize` returns `Option<Duration>`: when the
  session already has an applied size and the change is younger than the settle
  time it spawns nothing and returns the remainder. `request_terminal_resize`
  returns that remainder. `finish_layout_drag` clears the change time before
  flushing. The drag-only throttle is removed: the settle rule covers drags.
- `src/app/screens.rs`: call `ctx.request_repaint_after(wait)` with the wait
  `request_terminal_resize` returns, so the settled size is sent even when
  nothing else wakes the frame.
- `src/ui/terminal.rs`: `TerminalPaneKey` keeps `session_id`, `visible_rows`,
  `output_generation`, `theme`; drops `rect_size`, `cursor`, `search_query`,
  `search_current` — none of them is an input to a row's galley (rows are laid
  out with an unbounded wrap width; search highlights and the caret are painted
  per frame). `CachedTerminalPane` drops `caret`; the caret computed this frame
  is returned.
- No new child process, no persisted shape, no background task added; the draw
  path does one `HashMap` lookup and an `Instant` read per visible pane.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | No | No colour painted or added. |
| Icons | No | No glyph added. |
| Identifier SSOT | Yes | The settle delay is one constant in `src/config.rs`, read by the code and by the tests. |
| Durability | No | No store, cancellation, migration, or byte limit touched (tmux resize is not stop/retry). |
| Subprocess safety | Yes | No new child; `tmux_resize` still goes through `run_command_with_timeout`, now fewer times. |
| Documentation | No | No user-facing doc describes resize timing (`README.md:98` mentions divider resizing, not when a size is sent). |
| Local-first | No | No network, telemetry, or `unsafe`. |
| Budgets | No | No scan or output path added. |

## Flagged concerns

- **Minimize detection** — egui-winit 0.31.1 (its lib.rs, line 1008) writes
  `ViewportInfo::minimized` on macOS only at startup (egui issue 3494), so a
  minimized guard would be dead code. Resolved: no minimize rule; the settle
  rule covers transient sizes.
- **Delay length** — 150 ms is the value the drag throttle already used; it is a
  recoverable tuning constant. Resolved: adopt 150 ms.

## Acceptance

- `cargo test --locked session_255_` and
  `cargo test --locked terminal_resize_debounces_during_divider_drag` and
  `cargo test --locked terminal_pane_render_cache` pass.
- `make q.fast` passes.
- In the running app: dragging the window edge leaves the agent unredrawn until
  the drag pauses; maximize/restore redraws once.

## Rejected alternatives

- Keep the leading-edge throttle and extend it to window resizes — still sends
  an intermediate size every 150 ms of a drag, each a full agent redraw.
- Skip resizes while `viewport().minimized` — never updated on macOS after
  startup (egui-winit 0.31.1), so the guard could not fire.
- Re-wrap old capture client-side during the drag — re-wrapping a line tmux
  already wrapped splits lines the terminal considers finished.
- Per-row galley cache keyed by line — scrolling is not the reported cost; egui's
  own galley cache already serves unchanged jobs.
