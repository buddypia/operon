# Spec: Japanese IME text entry at the terminal cursor position

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Inspect tmux pane cursor coordinate `(cursor_x, cursor_y)` alongside session output refreshes.
2. Calculate the exact screen pixel rectangle for the active terminal cursor cell based on cell dimensions, margins, and scroll position.
3. Dynamically set `egui::ViewportCommand::IMERect` to the calculated active cursor rectangle when the terminal pane has focus.
4. Capture `egui::Event::Ime(egui::ImeEvent::Preedit(text))` and maintain the active preedit composition string for each active session.
5. Render the preedit composition string inline at the terminal cursor position with distinct visual indication (such as an underline and high-contrast styling).
6. Filter out `Key::Enter` when `ImeEvent::Commit` is received in the same input batch, preventing premature command execution upon IME confirmation.
7. Suppress tmux forwarding of editing keys (`Key::Backspace`, arrow keys) when an active IME preedit composition is underway.
8. Set UTF-8 environment and enforce the UTF-8 flag (`-u`) on tmux commands so multibyte characters are consistently handled by terminal applications.

## Behaviour

When an agent CLI such as Antigravity CLI (`agy`) or Claude Code is running in the terminal workspace and waiting for user input:

- Focusing the terminal pane enables IME input and aligns macOS IME candidates directly below or beside the CLI prompt cursor (e.g. immediately after `> `).
- Typing Japanese romaji (e.g. "nihon") displays the uncommitted composition string ("にほん") inline at the cursor.
- Converting with the space key updates the inline composition (e.g. to "日本") and displays the candidate list at the cursor.
- Pressing `Enter` to commit the Japanese string commits the text ("日本") into the CLI prompt without sending a carriage return / submission to the shell.
- Pressing `Enter` again once composition is finished sends the standard `Enter` key to submit the completed prompt.
- Backspacing during composition edits only the pending preedit string and does not send destructive backspaces to the CLI.

## Design

| Module | What it gains |
|---|---|
| `src/tmux.rs` | `tmux_cursor_position(name: &str) -> Result<(u16, u16)>` or combined inspection of pane cursor. Enforces `-u` argument and UTF-8 locale environment on tmux sessions. |
| `src/ui/terminal.rs` | Enhanced `terminal_input_events` that takes preedit/active IME state, suppresses leaking `Key::Enter` on commits, filters navigation/backspace during preedit, and extracts `TerminalInput::Preedit(String)` updates. Functions to calculate cursor rect and draw inline preedit layout. |
| `src/app.rs` | Stores `terminal_cursors: HashMap<Uuid, (u16, u16)>` and `terminal_preedits: HashMap<Uuid, String>`. Computes the active cursor cell `Rect` and sends it via `IMERect`. Overlays the preedit text at the cursor position in `ui_terminal_panel`. |
| `src/tests.rs` | Tests verifying IME event filtering, cursor coordinate parsing, preedit state transitions, and suppressing enter on commit. |

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | Preedit text uses existing palette colors (`palette.terminal_fg` with `palette.accent_text` underline / highlight) and introduces no new color roles. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No new icons or marks are added. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | No new identifiers or configuration strings are introduced. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | Terminal cursor and preedit states are ephemeral in-memory UI states; neither database schema nor serialized models are modified. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | Tmux commands use existing timeouts and sanitization; no arbitrary shell evaluation is introduced. |
| Documentation — user-facing docs change in all three languages together | No | This is an internal fix to terminal IME behavior; no documentation sections need modification. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Everything executes locally through tmux and egui; no network calls or unsafe blocks are added. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | Cursor inspection is queried only during existing background refresh intervals, bounded by standard 5-second command timeouts. |

## Flagged concerns

- **Preedit text position when the terminal buffer has scrolled up.** When a user has scrolled up in the terminal history to read previous logs, the active prompt may be scrolled partially or fully out of the current viewport. In this situation, anchoring the IME rect to an off-screen row could cause the OS candidate window to render outside the pane or jump erratically.
  *Answer:* If the active cursor row is outside the visible row range of the terminal scroll area, Operon will clamp the reported `IMERect` to the visible bounds of the terminal viewport, and automatically scroll the view to reveal the active cursor line when new keyboard or IME input is received.
- **Race condition between tmux polling and fast typing.** Polling tmux for cursor positions occurs periodically (on output refresh), meaning the tracked cursor position might lag behind very rapid typing by a few tens of milliseconds before the CLI updates its display.
  *Answer:* The IME candidate window and preedit anchor start from the last known cursor position and advance horizontally based on the measured glyph width of the current preedit string, ensuring smooth visual feedback even while waiting for the next tmux capture cycle.

## Acceptance

- `cargo fmt --check` prints nothing; `cargo clippy --locked -- -D warnings` prints nothing past the compile lines.
- `cargo test --locked` passes with 6 ignored tests.
- When Antigravity CLI or any interactive agent CLI is launched in Operon, typing Japanese characters displays the candidate/preedit text directly at the prompt cursor position.
- Pressing `Enter` to commit an IME conversion commits the Japanese text into the prompt without triggering immediate submission of the command.
- Backspacing during Japanese IME composition removes composition characters without sending destructive backspace signals to tmux.
- `bash scripts/check-readiness.sh docs/sdlc/changes/014-terminal-japanese-ime-support` exits with 0 (Go).

## Rejected alternatives

- **Embed a hidden native macOS NSTextView / NSTextInputClient.** This would introduce heavy Objective-C runtime bridging and bypass egui's unified cross-platform rendering model, violating local architecture consistency and adding unneeded complexity.
- **Send raw IME preedit characters to tmux directly on every keystroke.** CLI interactive TUIs expect committed characters rather than provisional composition states; sending uncommitted keystrokes would corrupt shell history and trigger invalid auto-completions in agents.
- **Keep IMERect fixed at the bottom-left corner.** This was the existing buggy behavior that caused macOS to place candidate windows in the corner or off-screen, completely disjoint from the actual typing location.
