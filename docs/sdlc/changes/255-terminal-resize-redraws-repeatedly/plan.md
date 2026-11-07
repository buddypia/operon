# Plan: send a terminal one size per resize gesture, and keep its layout across size changes

- **Spec**: `./spec.md`
- **Approved**: 2026-11-07
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | `TERMINAL_RESIZE_SETTLE` |
| `src/app.rs` | `terminal_resize_changed_at`; settle check in `flush_terminal_resize`; both return the wait; `finish_layout_drag` clears the change time |
| `src/app/screens.rs` | `request_repaint_after` with the wait |
| `src/ui/terminal.rs` | narrower `TerminalPaneKey`; caret no longer cached |
| `src/tests.rs` | four `session_255_` tests; existing cache tests keep passing |

## Order of work

1. Write the four tests; watch R1, R3, R5 fail (R2 passes today — it pins
   behaviour the change must keep).
2. Constant and `src/app.rs` settle logic (R1–R4).
3. Repaint wake in the pane (R1's wake).
4. Cache key and caret (R5).
5. `make q.fast`, named tests; mutation sweep; verification record.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Trailing size never sent when nothing repaints | Pane stays at the old wrap until the 2 s idle repaint | `request_repaint_after(wait)` in the pane; R1 asserts the wait is returned |
| A dropped key field was in fact a galley input | Stale colours or highlight after search/theme | Theme stays in the key and a theme switch clears the cache; search highlight is painted outside the cached galley in `terminal_rows` |
| Stale caret after the pane moves | IME box at the old position | R5 |
| A guard on a flag the platform never updates | Passes in tests, dead in the app | No minimize guard (egui-winit 0.31.1 writes `minimized` on macOS only at startup) |
| Completion handler bypasses settle | Intermediate sizes sent back-to-back | R3 |

## Proof of completion

- `make q.fast` — passes.
- `cargo test --locked session_255_` — 4 passed.
- `cargo test --locked terminal_resize_debounces_during_divider_drag` and `cargo test --locked terminal_pane_render_cache` — pass.
- `cargo test --locked resizes_a_managed_tmux_session_to_the_terminal_viewport` — passes (tmux path unchanged).
- Running app: window drag / maximize as in spec Acceptance.

## Departures from the plan

- **A fifth test, `session_255_a_settling_size_wakes_the_frame`.** The mutation
  sweep showed the pane's `request_repaint_after(wait)` was guarded by nothing:
  R1 checks that the wait is returned, not that the frame asks to wake. The test
  draws the pane twice and reads egui's repaint delay. The proof line's count is
  therefore 5, not 4.
- **`TerminalSearchOverlay::query` removed.** Its only reader was the dropped
  `search_query` key field; leaving it is a dead-code error under clippy.
- **Two existing tests rewritten.** CI's full suite failed
  `adversarial_cache_invalidation_matrix` and
  `adversarial_deeply_nested_asymmetric_split_isolation`: both asserted that a
  pane size change rebuilds its rows, the behaviour R5 reverses. Their resize
  steps now assert reuse; every other step (output, theme, font, deletion) is
  unchanged. The plan named only the `terminal_pane_render_cache` tests.
