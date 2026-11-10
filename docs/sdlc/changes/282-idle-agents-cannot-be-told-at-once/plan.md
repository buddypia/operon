# Plan: one instruction to every idle agent of the project

- **Spec**: `./spec.md`
- **Approved**: 2026-11-10
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/app/broadcast.rs` | new: the sheet, its recipients, the send |
| `src/app.rs` | `mod broadcast`; field `broadcast_sheet`; `update` draws the sheet |
| `src/app/screens.rs` | the palette row and its arm |
| `src/app/keymap.rs` | app-wide chords held back while the sheet is open |
| `src/i18n_tables.rs` | seven strings, English and Korean |
| `src/tests.rs` | seven `broadcast_282_` tests |
| `CLAUDE.md` | module map line; five neighbouring lines shortened |
| `README.md`, `README.ja.md`, `README.ko.md`, `docs/MANUAL.md`, `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md` | a sentence or paragraph each |
| `docs/sdlc/changes/282-idle-agents-cannot-be-told-at-once/` | `mutations.py`, `verification.yaml` |

## Order of work

1. The person's choice of the sheet, recorded in `approvals.log`.
2. Recipients and send, with their tests.
3. The sheet and the palette row, with tests that draw the sheet and drive it
   with injected events, as change 279's do.
4. The documents.
5. `make q.fast`; `cargo test --locked broadcast_282_`, `palette`, `table`,
   `claude_md`, `harness`, `readme`, `manual`; `bash scripts/check-bands.sh`.
6. `mutations.py`, `verification.yaml`, the verifier.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A working or prompting agent is typed into | Text lands in the middle of a turn or answers a prompt | `broadcast_282_reaches_only_the_projects_idle_running_agents` |
| Another project's agents are reached | An instruction arrives where it was not meant | The same test |
| The palette's fixed list and its tests disagree | `an_empty_palette_query_lists_only_the_actions` fails | `cargo test --locked palette` |
| The Enter of a Japanese conversion sends | Half a sentence to every agent | `broadcast_282_sheet_names_who_it_reaches_and_sends_on_enter` |
| The palette or the launch sheet opens over the sheet | Escape closes both and the typed line is lost; ⌘J hands a terminal the keyboard | `broadcast_282_f1_page_shortcuts_do_not_reach_past_the_sheet`: the chords are not live while the sheet is open, and the palette closes before the sheet opens |

## Proof of completion

- `make q.fast` — passes.
- `cargo test --locked broadcast_282_` — `7 passed; 0 failed`.
- `python3 docs/sdlc/changes/282-idle-agents-cannot-be-told-at-once/mutations.py`
  — every mutation caught.
- `bash scripts/check-bands.sh` — no tier newly reached.

## Departures from the plan

- The recipients were first filtered by `SessionStatus::Active` as well as
  by the idle state; the mutation sweep showed the first condition decided
  nothing, since a session whose terminal has ended is never idle, and it
  was removed rather than left untested.
- The verifier and the evaluator both found ⌘K, ⌘N and the page chords live
  under the open sheet. `handle_app_shortcuts` now holds them back as it
  does for the launch sheet; the verifier's test is kept as
  `broadcast_282_f1_…` and the spec gained requirement 7.
- The module-map lines for `working_time.rs` and this file were shortened to
  keep `always_loaded_bytes` under its diagnose tier, as were the lines for
  `terminal_output.rs`, `waiting_reply.rs` and `dock_badge.rs`.
