# Plan: Terminal Prompt Timeline Navigation

- **Spec**: `./spec.md`
- **Approved**: 2026-09-15
- **Status**: approved

This is the plan produced in plan mode and accepted before any file was edited.
If the implementation departs from it, update this file — an abandoned plan is
worse than no plan, because the next reader trusts it.

## Files that change

| File | Change |
|---|---|
| `src/app.rs` | Define `PromptTurn`, add `session_prompt_turns` and `show_prompt_timeline` fields to `OperonApp`, implement `record_prompt_turn` and hook turn completion |
| `src/app/screens.rs` | Add right-side prompt timeline layout (`ui_prompt_timeline`), toggle control in toolbar, and wire click events to `search.scroll_to` |
| `src/i18n_tables.rs` | Add Japanese, English, and Korean message table entries for all timeline UI strings |
| `src/tests.rs` | Unit tests for turn tracking, excerpt truncation, bounded capacities, and scroll-to assignment |

## Order of work

1. Register internationalized strings in `src/i18n_tables.rs`.
2. Define `PromptTurn` and add session prompt tracking in `src/app.rs`.
3. Hook prompt capture on `UserPromptSubmit` and completion on `Stop` / `Idle` in `src/app.rs`.
4. Implement `ui_prompt_timeline` in `src/app/screens.rs` with split layout, cards, scroll jump, and toggle action.
5. Add unit tests in `src/tests.rs` verifying prompt turn recording, line mapping, and bounds.
6. Verify gates with `cargo fmt --check`, `cargo test --locked`, and `cargo clippy --locked -- -D warnings`.
7. Package and install to `/Applications/Operon.app` using `bash scripts/package-macos.sh`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Excess space usage in narrow windows | Terminal text wraps awkwardly | Timeline panel is collapsible via toolbar button; width bounded to min 260px / max 320px |
| Unbounded memory growth in long sessions | Memory usage increases with 10k+ prompts | `PromptTurn` list per session is capped at 500 items; old entries truncated safely |
| Line drift if terminal output exceeds scrollback | Jump positions viewport past top of buffer | Clamped with `.min(layouts.len().saturating_sub(1))` before assigning `scroll_to` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — all passed; 0 failed; 6 ignored.
- `cargo clippy --locked -- -D warnings` — no warnings past normal compilation.
- New unit test `prompt_timeline_records_turns_and_sets_scroll_target` passes in `src/tests.rs`.
- In the running app, opening an active session shows the prompt timeline panel with accurate prompts and smooth jump on click.

## Departures from the plan

None so far.
