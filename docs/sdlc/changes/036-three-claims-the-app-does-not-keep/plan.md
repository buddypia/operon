# Plan: the launch screen says what it will run

- **Spec**: `./spec.md`
- **Approved**: 2026-09-06
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/agents.rs` | `dangerous_launch_arguments`, `command_needs_acknowledgement`; `launch_needs_acknowledgement` gains the custom command |
| `src/app.rs` | `LaunchAcknowledgement` gains `custom_command`; `launch_detail_summary`; the two call sites of `launch_needs_acknowledgement` |
| `src/app/screens.rs` | account row into step 1; summarised header; mode, effort and switch controls; the card's resume line |
| `src/i18n_tables.rs` | three ids in all three tables |
| `src/tests.rs` | eight guards |
| `README.md`, `README.ja.md`, `README.ko.md` | one sentence, three languages |

## Order of work

Each step leaves the tree compiling.

1. `src/agents.rs`: the two new functions, and the fourth parameter on
   `launch_needs_acknowledgement`. The three existing call sites gain the
   custom command; nothing on screen changes yet.
2. `src/tests.rs`: `a_dangerous_switch_typed_into_a_custom_command_still_asks`
   and `switches_that_the_cli_refuses_together_cannot_both_be_on`. The first is
   watched failing here, before step 3 can make it pass by accident.
3. `src/app.rs`: `LaunchAcknowledgement.custom_command`, and
   `launch_detail_summary`. `pending_launch` fills the new field.
4. `src/app/screens.rs`: move the account row out of the `CollapsingHeader` and
   into the step-1 frame. Guard first — the frame-running test that reads the
   labels `ui_overview` paints is what says it moved, and it fails while the row
   is still folded away.
5. `src/app/screens.rs`: the summarised header, reading `launch_detail_summary`.
6. `src/app/screens.rs`: the mode combo, the effort combo, and one checkbox per
   flag, with group exclusivity and the mode conflict. This is the largest step
   and the only one whose screen was drawn before its code.
7. `src/app/screens.rs`: the card's `native resume:` line calls
   `resume_command_with_launch_options`. Two guards: the card's line equals what
   the resume builds, and the two displays for a session Operon did not launch
   are unchanged.
8. The three READMEs.
9. Mutation-verify all eight guards, run the three gates and the bands, then the
   review.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The mode combo offers a value the CLI rejects | A window opens, prints usage, and dies with a session record already written | `the_launch_screen_offers_every_mode_and_switch_the_agent_has` compares what is drawn against `agent_mode_options`, and `build_agent_launch_command` already refuses a value outside it |
| Group exclusivity enforced in the drawing code drifts from `resolve_agent_flags` | Two switches from one group both on; the launch fails at the CLI | `switches_that_the_cli_refuses_together_cannot_both_be_on` drives the real checkbox handler, and the handler reads `group` off the table rather than naming pairs |
| The acknowledgement is dropped when the command changes but the tuple does not | A person consents to one command and another runs | `LaunchAcknowledgement` carries the custom command, so the identity moves with it |
| `command_needs_acknowledgement` matches inside a path | A harmless launch demands a checkbox | Matched on whitespace-separated tokens, and a test launches a command whose path contains the text |
| The summary drifts from what step 3 actually draws | The header names a choice that is not there, or misses one | `launch_detail_summary` is one function the header and the guard both call |
| Work added to a per-frame path | Stutter on the launch screen | The added work is a walk of `&'static` tables over one short line; `bands.yaml` measures the draw path and the reviewer reads the call site |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked --all-targets -- -D warnings` — no output past the
  compile lines.
- `bash scripts/check-bands.sh` — `bands: N metrics within their bands`.
- Eight new guards, each watched failing for its own reason before it passes.
- In the running app: a dangerous switch raises the warning from the checkbox
  and from a typed custom command; the account row is visible without unfolding;
  a restarted session's card line matches the command in the hover.

## Departures from the plan

Filled in during implementation.
