# Plan: float notices and warnings over the corner on the Sessions page so the terminal never shifts

- **Spec**: `./spec.md`
- **Approved**: 2026-11-12
- **Status**: done

This is the plan produced in plan mode and accepted before any file was edited.
If the implementation departs from it, update this file — an abandoned plan is
worse than no plan, because the next reader trusts it.

## Files that change

| File | Change |
|---|---|
| `src/app.rs` | Extract `ui_central_panel` from `OperonApp::update` without changing call order; skip the in-flow `durability_warning` and non-brief `notice` banners when `self.page == Page::Sessions`; extend `ui_brief_notice_toast` so on `Page::Sessions` both `durability_warning` (no close button, `palette.warning`) and non-brief `notice` (`ICON_NOTICE`, `palette.info`, `✕` button, wrapped within `<= 420px`) render in the bottom-right foreground `Area` without per-frame string clones when idle |
| `src/tests.rs` | Add the four `session_notice_289_*` requirement tests |
| `docs/sdlc/changes/289-session-notice-shakes-terminal/` | `mutations.py`, `verification.yaml`, `approvals.log`, `state.yaml` |

## Order of work

1. Extract `ui_central_panel` as a pure move in `src/app.rs` only to let the
   tests drive the real `CentralPanel` + `ui_brief_notice_toast` frame, OR write
   the four `session_notice_289_*` tests first against `ui_central_panel` +
   `ui_brief_notice_toast` with the old banner condition still intact and watch
   `session_notice_289_terminal_workspace_never_shifts_on_notices_or_warnings`,
   `session_notice_289_persistent_notice_floats_stays_past_toast_timeout_and_closes`,
   and `session_notice_289_durability_warning_floats_without_close_button` fail
   for the stated reasons (workspace `top()` shifts down, non-brief notice not in
   the corner overlay, durability warning shifts workspace).
2. Gate the in-flow banners in `ui_central_panel` on `self.page != Page::Sessions`
   and extend `ui_brief_notice_toast` to render `durability_warning` and
   non-brief `notice` on `Page::Sessions` with bounded text wrapping and `✕`
   dismissal on `notice` only.
3. Run `cargo test --locked session_notice_289_`, `cargo test --locked brief_notice`,
   `cargo test --locked every_theme_stays_readable`, `cargo test --locked every_text_size`,
   `cargo test --locked every_button`, and `make q.fast`.
4. Write `mutations.py` (mutating both layout/state conditions and the visual/close
   claims) and run it; check `bash scripts/check-bands.sh`; run independent
   empirical verification; record `verification.yaml` and settle `state.yaml`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A long error notice pushes the `✕` button outside the 420px card or off the right edge of the window | `✕` cannot be clicked to dismiss a long error | `session_notice_289_persistent_notice_floats_stays_past_toast_timeout_and_closes` uses a long multi-line error string and asserts the close button's rect lies wholly inside `ctx.screen_rect()` and clears `app.notice` when clicked |
| Floating `notice` on `Page::Sessions` accidentally makes errors auto-expire after `NOTICE_TOAST_SECONDS` | failure vanishes unread after 4s | `session_notice_289_persistent_notice_floats_stays_past_toast_timeout_and_closes` calls `expire_brief_notice` past `NOTICE_TOAST_SECONDS` and asserts `app.notice` remains `Some` |
| `durability_warning` gains a close button or disappears on `✕` | unconfirmed disk sync warning can be dismissed while `store_retry_pending` is still true | `session_notice_289_durability_warning_floats_without_close_button` asserts no `この通知を閉じる` widget is drawn when only `durability_warning` is set, and clicking `✕` when both are set clears `notice` while keeping `durability_warning` |
| Non-session pages (`Page::Home`, `Page::Projects`, `Page::Settings`) either lose their top banner or draw a duplicate bottom-right toast | missing or doubled notice on Home | `session_notice_289_other_pages_keep_top_banner_without_duplicate_toast` checks `Page::Home` draws the notice once in the top banner |
| `src/app.rs` crosses the 10 000-line `propose` threshold in `bands.yaml` (currently 9 906 lines) | `check-bands.sh` reports `propose` on `largest_module_lines` | Keep the net addition in `src/app.rs` under ~45 lines and run `bash scripts/check-bands.sh` |

## Proof of completion

- `make q.fast` — `cargo fmt --check` and `cargo clippy --locked -- -D warnings` exit 0 with no warnings.
- `cargo test --locked session_notice_289_` — `test result: ok. 4 passed; 0 failed`.
- `python3 docs/sdlc/changes/289-session-notice-shakes-terminal/mutations.py` — every mutation caught.
- `bash scripts/check-bands.sh` — no metric at `propose` (`largest_module_lines < 10000`).
- In `/Applications/Operon.app` after landing and packaging from `main`: triggering a notice on the Sessions page shows the bottom-right card without moving the terminal workspace.

## Departures from the plan

- None.
