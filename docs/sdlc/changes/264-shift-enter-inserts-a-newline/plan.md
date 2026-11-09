# Plan: Shift+Enter is a newline in the session terminal

- **Spec**: `./bug.md` (bugfix route: the bug report is the intent)
- **Approved**: 2026-11-09, independent evaluator
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| src/ui/terminal.rs | `terminal_key_binding`: Enter with exactly Shift (`modifiers.matches_exact(egui::Modifiers::SHIFT)`) returns what Ctrl+J returns (`C-j`), with a comment saying why tmux cannot carry the Shift. |
| src/tests.rs | `shift_enter_puts_a_newline_in_the_agents_input_instead_of_sending`. |

## Order of work

1. Write the test; watch it fail on the current code (Shift+Enter yields `Enter`).
2. Add the Shift+Enter arm to `terminal_key_binding`, built from the Ctrl+J path rather than a second `C-j` literal.
3. `make q.fast`, the new test and the neighbouring key and IME tests (`maps_tui_control_keys_without_intercepting_macos_commands`, `terminal_input_events_handles_ime_preedit_and_commit_enter_suppression`, `a_second_enter_in_the_frame_that_committed_a_conversion_still_sends`); verification and review; CI on the branch, land, package.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A CLI reads LF as submit | Shift+Enter still sends there | Measured: Claude Code, Codex, Antigravity CLI insert a newline; a shell runs the line, as in cmux. |
| The IME's confirming Enter becomes a newline | A stray line break after a conversion | The ownership count runs before the key binding; the test feeds a commit with a Shift+Enter in the same frame. |
| Ctrl+Shift+Enter or Option+Shift+Enter change | Different bytes for those chords | `matches_exact(SHIFT)` requires alt, ctrl and command off (`shift_only()` would admit Ctrl, egui 0.31.1 `data/input.rs:739`); the test pins Ctrl+Shift+Enter as `C-Enter` and Option+Shift+Enter as `M-Enter`. |

## Proof of completion

- The new test fails before the change with `Key("Enter")` where `Key("C-j")` is expected, and passes after.
- `make q.fast` passes; CI on the branch passes.
- In the installed app, Shift+Enter in an Antigravity session leaves two lines in the input; Enter sends.

## Departures from the plan
