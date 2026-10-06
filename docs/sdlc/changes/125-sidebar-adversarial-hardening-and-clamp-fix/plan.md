# Plan: adversarial hardening for sidebar splitters, drag drop bounds, and ID isolation

- **Spec**: `./spec.md`
- **Approved**: 2026-10-06
- **Status**: approved

This is the plan produced in plan mode and accepted before any file was edited.
If the implementation departs from it, update this file — an abandoned plan is
worse than no plan, because the next reader trusts it.

## Files that change

| File | Change |
|---|---|
| `src/app/screens.rs` | Fix right splitter clamp calculation using `total_row_w`, add Escape cancellation, restrict to Primary button, add deadzone and panel boundary checks |
| `src/ui/session_tree.rs` | Isolate `InspectorTab::Sessions` rendering with `ui.push_id` |
| `src/tests.rs` | Unit tests for right splitter expansion past 180px, drag Escape cancellation, and drop deadzone |

## Order of work

1. Update `src/app/screens.rs` layout and splitter logic.
2. Update `src/ui/session_tree.rs` ID isolation.
3. Add unit tests in `src/tests.rs`.
4. Run `cargo test --locked` and `cargo clippy --locked -- -D warnings`.
5. Run review checks with `scripts/check-review.sh`.
6. Land into `main` and package.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Splitter still collapses under certain window aspect ratios | Sidebar locks to 180px | `test_sidebar_right_splitter_expansion_not_collapsed` verifies resize up to 450px |
| Escape cancellation drops key input intended for terminal | Key event lost | Event only consumed when `dragging_sidebar_tab.is_some()` |

## Proof of completion

- `cargo fmt --check` — clean.
- `cargo clippy --locked -- -D warnings` — clean.
- `cargo test --locked` — all tests pass.
- New tests pass:
  - `test_sidebar_right_splitter_expansion_not_collapsed`
  - `test_sidebar_tab_drag_escape_cancellation`
  - `test_sidebar_tab_drag_deadzone_and_bounds`

## Departures from the plan

None yet.
