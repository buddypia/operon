# Plan: Japanese IME text entry at the terminal cursor position

- **Spec**: `./spec.md`
- **Approved**: 2026-09-05
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/tmux.rs` | Add `tmux_cursor_position(name: &str) -> Result<(u16, u16)>` and `tmux_capture_with_cursor(name: &str) -> Result<(String, (u16, u16))>`. Pass `-u` to tmux commands and ensure UTF-8 environment is active. |
| `src/ui/terminal.rs` | Update `terminal_input_events` to accept active preedit status, extract preedit updates, and suppress leaking `Key::Enter` on commits as well as navigation/backspaces during preedit. Add helper to compute cursor rect and draw inline preedit text. |
| `src/app.rs` | Maintain `terminal_cursors: HashMap<Uuid, (u16, u16)>` and `terminal_preedits: HashMap<Uuid, String>`. Send updated active `IMERect` matching the active cursor position, and render the preedit overlay at the cursor. |
| `src/tests.rs` | Add regression tests for terminal cursor coordinate parsing, IME event filtering (commit suppresses enter, preedit suppresses backspace/arrows), and IMERect alignment. |

## Order of work

1. `src/tmux.rs`: Add `tmux_cursor_position` and parse `#{cursor_x},#{cursor_y}` safely. Enforce `-u` flag and UTF-8 environment setup where tmux is spawned.
2. `src/ui/terminal.rs`: Enhance input event filtering to support IME preedit and commit suppression of `Key::Enter`. Add helper to render inline preedit at the cursor.
3. `src/app.rs`: Integrate cursor and preedit tracking per session. Update `IMERect` computation from `terminal_rect.left_bottom()` to the actual cursor coordinate cell within the terminal grid.
4. `src/tests.rs`: Implement unit tests for IME event parsing, key suppression, and cursor coordinate handling.
5. Verify gates: `cargo fmt --check`, `cargo test --locked`, `cargo clippy --locked -- -D warnings`, and `bash scripts/check-bands.sh`.
6. Verify readiness script: `bash scripts/check-readiness.sh docs/sdlc/changes/014-terminal-japanese-ime-support`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Querying cursor coordinates adds excessive command latency | Stuttering terminal refresh | Querying is paired with existing capture-pane execution or uses bounded 5-second timeouts in existing background thread. |
| Preedit styling clashes with terminal themes | Preedit text is unreadable on light or dark mode | Preedit text uses palette.terminal_fg with an accent underline, styled identically across all three palettes. |
| Suppressing Enter causes normal Enter to fail | User cannot submit commands when not using an IME | Enter is suppressed only when an `ImeEvent::Commit` occurs in the exact same event frame. Normal non-IME Enter passes unhindered. |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — all tests pass (315+ passed; 0 failed; 6 ignored).
- `cargo clippy --locked -- -D warnings` — no warnings.
- `bash scripts/check-bands.sh` — no new breaches.
- `bash scripts/check-readiness.sh docs/sdlc/changes/014-terminal-japanese-ime-support` — exits 0 (Go).
