# Plan: 「閉じる」 in the session row's and the session tab's right-click menu

- **Spec**: `./spec.md`
- **Approved**: 2026-11-11
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/tests.rs` | four `session_286_*` tests, on the `session_285_*` helpers |
| `src/app.rs` | `remember_closed_tab` taken out of `close_workspace_tab` |
| `src/app/session_order.rs` | `close_session_tab` |
| `src/app/screens.rs` | 「閉じる」 in `row.context_menu` and `label.context_menu` when not pinned |
| `docs/sdlc/changes/286-…/` | `verification.yaml`, `mutations.py`, `state.yaml` |

## Order of work

1. The four tests, watched failing: no 「閉じる」 in either menu, no
   `close_session_tab` (R4 does not compile until step 2; it is added with it).
2. `remember_closed_tab` and `close_session_tab`.
3. The two menu entries and the column's `close` handling.
4. `make q.fast`, `cargo test --locked session_286`, `session_285`.
5. `verification.yaml`, `mutations.py`, verifier, close.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The click that chooses 「閉じる」 also opens or selects the row under it | the closed session comes back to the front | R1 asserts the tab is gone after the click |
| Closing another project's row switches the shown project | the workspace jumps | R4 asserts `workspace_project` and the shown tabs |
| The entry appears on a pinned session | a close that the pin refuses | R3 |
| Another 「閉じる」 already drawn on the page confuses the tests | a test clicks the wrong text | the tests take the entry from the menu frame, below the right-click point |

## Proof of completion

- `make q.fast` — passes.
- `cargo test --locked session_286` — `4 passed; 0 failed`.
- `cargo test --locked session_285` — all pass, as on main.
- `python3 docs/sdlc/changes/286-session-cannot-be-closed-from-its-row-or-tab-menu/mutations.py` — every mutation caught.
- In the app after packaging: right-click a row and a tab name → 「閉じる」
  closes both; a pinned one has no such entry.

## Departures from the plan

None yet.
