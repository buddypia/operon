# Plan: pin the launch settings with 「この設定を毎回使う」

- **Spec**: `./spec.md`
- **Approved**: 2026-10-09
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/agents/settings.rs` | `PinnedLaunch`; `RecentAgentSettings.pinned`; sanitise it in `load_recent_agent_settings` |
| `src/app.rs` | `pin_launch_settings`, `unpin_launch_settings`, `apply_pinned_launch`, `launch_matches_pin`; call `apply_pinned_launch` from `open_project_session_setup` |
| `src/app/screens.rs` | checkbox, note, re-pin button in `ui_launch_footer` |
| `src/i18n_tables.rs` | rows for the three new message ids in every table |
| `src/tests.rs` | the three tests the spec names; two existing `RecentAgentSettings` literals gain `..Default::default()` |

## Order of work

1. Tests first (`pinned_launch_*`); watch them fail to compile/fail.
2. `PinnedLaunch` + load sanitising — tests 1–2 and 5 (sanitising part) pass.
3. App methods + `open_project_session_setup` wiring — remaining tests pass.
   `launch_matches_pin` compares agent, model, mode, effort, flags, custom
   command, account, and whether the acknowledgement is given — unticking the
   acknowledgement on a pinned dangerous combination counts as "differs".
   `apply_pinned_launch` only fills the form; it does not write the recents
   (they are written, as today, by edits and by a launch).
4. Footer UI and i18n rows.
5. `make q.fast`, then one `cargo test --locked <filter>` per filter:
   `pinned_launch`, `recent_agent_settings`, `custom_command`,
   `every_button_keeps_a_documented_height`, and the i18n catalog tests.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Acknowledgement restored for a combination the person did not acknowledge | dangerous launch without consent | ack set via `pending_launch()` after applying, cleared when sanitising changed the pin — `pinned_launch_drops_what_no_longer_holds` |
| Pin overwritten by per-launch edits | next sheet opens with the edit | `pinned_launch_fills_the_sheet_each_time_it_opens` |
| Old settings file fails to parse | all remembered settings lost | `serde(default)`; round-trip test loads a file without `pinned` |
| Per-frame allocation in the footer | stutter | `launch_matches_pin` compares by borrow; rust-reviewer |
| Missing translation row | catalog test fails | existing i18n catalog tests |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked pinned_launch` — 3 passed.
- Mutation: drop the `acknowledged = false` on sanitise change, and drop the
  `apply_pinned_launch` call; each turns a test red.
- In the app: pin, reopen the sheet, ⌘↩ launches with the pinned combination.

## Departures from the plan
