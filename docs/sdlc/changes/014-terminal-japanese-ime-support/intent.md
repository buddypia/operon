# Intent: Japanese IME text entry at the terminal cursor position

- **Status**: draft
- **Opened**: 2026-09-05

## Problem

When running interactive agent CLIs (such as Antigravity CLI / `agy`) in Operon's terminal pane, typing in Japanese does not display the composed text at the cursor position. While in normal macOS terminal emulators (Terminal.app, etc.), typing Japanese displays the preedit/composed text right at the cursor prompt, Operon fails to show it.

There are four distinct technical reasons for this failure:

1. **IMERect mismatch:** `IMERect` is hardcoded to `terminal_rect.left_bottom()` (the bottom-left corner of the terminal window), causing macOS IME candidate and composition windows to appear far from the actual cursor or off-screen.
2. **Missing Preedit rendering:** IME preedit composition events (`ImeEvent::Preedit`) are discarded by `terminal_input_events` without updating any terminal state, and the terminal layout does not draw in-progress composition text at the cursor.
3. **Premature execution via leaky Enter/incompatible keys:** When the user presses `Enter` to commit an IME composition string (e.g. converting "nihon" to "日本"), both `ImeEvent::Commit` and a raw `Key::Enter` event are captured. Forwarding `Key::Enter` to tmux immediately triggers CLI execution/submission before the user intended, erasing the input line. Similarly, editing keys like `Backspace` or arrow keys during IME composition leak to tmux and corrupt the prompt.
4. **Untracked cursor position:** Operon never queries the cursor coordinate `(cursor_x, cursor_y)` from the managed tmux pane, so it cannot position `IMERect` or draw the text cursor and inline preedit correctly.

## Who feels it, and when

Anyone typing Japanese (or other CJK text via an IME) into an interactive agent CLI running inside Operon. For English, text is typed directly, but for Japanese, the user cannot see what they are typing at the cursor until (or unless) it commits cleanly without leaking a newline.

## Desired outcome

- When typing Japanese via IME in the terminal pane, the composition (preedit) text is displayed directly at the cursor position.
- The macOS IME candidate window is anchored to the actual terminal cursor cell (`IMERect` matches the active cursor).
- Committing IME text via `Enter` commits the text into the terminal input without submitting the prompt or executing early.
- Keys used for composition (such as `Backspace` or navigation during preedit) do not leak into the tmux session.
- The terminal cursor position is accurately retrieved from tmux and reflected in both rendering and IME placement.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.
- Must pass `cargo fmt --check`, `cargo test --locked`, and `cargo clippy --locked -- -D warnings`.

## Systems likely affected

- `src/ui/terminal.rs` for terminal input event filtering, preedit handling, and cursor/preedit rendering.
- `src/tmux.rs` for tmux cursor position inspection and UTF-8 environment setup.
- `src/app.rs` for tracking terminal cursor and IME preedit per session, and updating `IMERect` dynamically.
- `src/tests.rs` for unit and regression tests.
