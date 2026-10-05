# Plan: streamline session list sidebar, card density, and header clutter

- **Spec**: `./spec.md`
- **Approved**: 2026-09-24
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/app/screens.rs` | Remove verbose instruction label from `ui_terminal_session_tabs`; streamline session card layout and move restore-target action into card menu; update pane header metadata row. |
| `src/app.rs` | Refactor `ui_session_ports` to render as a compact menu button with count badge and dropdown popover for port links. |
| `src/tests.rs` | Update existing UI tests that assert session sidebar text and add test coverage for streamlined card and port popover rendering. |
| `docs/sdlc/changes/077-streamline-session-view/state.yaml` | Advance stage and status as work progresses. |

## Order of work

1. **Step 1: Sidebar Instruction Label Removal & Session Card Cleanup**
   - In `src/app/screens.rs`:
     - Remove the static label `RichText::new(tf!("ダブルクリックで名前を変更..."))`.
     - Remove the inline combobox (`復元先 Codex CLI ▼`) and restore button from the session card.
     - Add the restore option ("他の CLI へ復元...") into the session card's `···` action menu, invoking the existing restore modal flow.
     - Expand title width / allow more room for session titles.

2. **Step 2: Consolidate Port Indicator in Pane Header**
   - In `src/app.rs`:
     - Refactor `ui_session_ports`: if `ports.is_empty()`, return early.
     - When ports are present, render a compact menu button (e.g. `🔌 {ports.len()} ポート`) displaying each port in the menu popup with process name and browser open action.

3. **Step 3: Verification & Test Updates**
   - In `src/tests.rs`:
     - Inspect tests that search for the removed instruction string and update them to verify the clean card layout and restore menu action.
     - Add tests verifying the port menu button rendering and session card action menu contents.

4. **Step 4: Quality Gates & Build Verification**
   - Run `cargo fmt --check`.
   - Run `cargo test --locked`.
   - Run `cargo clippy --locked -- -D warnings`.
   - Run `bash scripts/check-readiness.sh docs/sdlc/changes/077-streamline-session-view`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A user cannot find how to restore a session to another CLI | Session handoff feature seems missing | The `···` menu clearly provides "他の CLI へ復元..." and session card tooltip explains it |
| Multiple ports cannot all be clicked | Only the first port in the list is accessible | Dropdown menu enumerates every port with full clickable browser links |
| Tests asserting old sidebar instruction fail | `cargo test` failures on string match | Updated test suite verifying new layout |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. 584 passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-readiness.sh docs/sdlc/changes/077-streamline-session-view` — exit 0 (Go).
- Running the application shows a clean sidebar, full session titles, and a neat port indicator.

## Departures from the plan

None yet.
