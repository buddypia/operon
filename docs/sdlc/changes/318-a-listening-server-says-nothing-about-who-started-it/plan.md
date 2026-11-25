# Plan: Who started each listening server, when, and with what

- **Spec**: `./spec.md`
- **Approved**: 2026-11-25
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/git/ports.rs` | `ListenerProcess`, `SessionTag`, `AttributedPort.process`; `parse_ps_details`, `parse_elapsed`, `parse_ps_environment`, `redact_command_line`; `scan_listening_ports_with` takes the `ps` program and runs the two reads. |
| `src/config.rs` | `PORT_PROCESS_PROGRAM`, `PORT_ENVIRONMENT_MAX_BYTES`, `PORT_COMMAND_MAX_CHARS`. |
| `src/app.rs` | `PortOrigin` and `is_leftover`, `OperonApp::port_origin`, `SessionPorts.origins` and `leftovers`, filled in `session_ports`. |
| `src/app/screens.rs` | Badge segment, two detail lines per menu row, modal rows. |
| `src/i18n_tables.rs` | Rows for every string in spec **Behaviour**, in every table. |
| `src/tests.rs` | The eight `ports_318_` tests; `process: None` / `origins` added to existing literals. |
| `README.md`, `README.ja.md`, `README.ko.md` | Ports paragraph: origin, start, command, 終了忘れ?, the new ceilings. |
| `docs/features/worktrees/spec.md`, `docs/features/worktrees/brief.md` | Items 12–14 extended; pinned tests named. |
| `docs/features/glossary.yaml` | 終了忘れ under ポート. |
| `docs/sdlc/changes/318-…/` | `verification.yaml`, `mutations.py`, state. |

## Order of work

1. Contract tests for the parsers (requirements 1–3): fixtures taken from the
   measured `ps` output shapes — a cut last line in both reads, an unparsable
   `lstart` and `etime`, a PID missing from the environment output, and an
   argument that is itself `OPERON_SESSION=…` before the environment; one
   assertion per redaction rule; watch them fail; write the parsers and
   `redact_command_line`.
2. Requirement 8 and the scan: `scan_listening_ports_with` gains the `ps`
   program; test with a missing program, and with a real `python3` listener
   child (drop guard kills it) whose environment carries a tag built from
   `HOOK_SESSION_ENV` and `managed_tmux_name`. Same-user `ps -E` reads the
   environment on macOS (measured on this machine; CI is `macos-latest`), so
   the test asserts `Found` with that value and fails otherwise rather than
   skipping. Then wire both reads; the existing caller passes
   `PORT_PROCESS_PROGRAM`.
3. Requirement 4: `PortOrigin`, `port_origin`, `SessionPorts.origins` —
   test against sessions in `Active`, `Exited`, removed, untagged, unread.
4. Requirements 5–7: badge, rows, modal — tests through `ctx.run_ui` as change
   262's do; then the drawing.
5. Strings in every table; READMEs; feature docs; glossary.
6. Tidy; `make q.fast`; `cargo test --locked ports_318`, `ports_262`,
   `attributes_a_port`; the i18n and type-scale tests.
7. `verification.yaml`, `mutations.py` (mutate: last-token tag search turned to
   first; cut environment line read as `Absent`; a missing PID read as
   `Absent`; the cut details line kept; an `etime` form misparsed; each
   redaction rule removed; the scan storing the raw command and redaction
   moved to the frame; leftover for `Unknown`, for `Unread`, and not for
   `RemovedSession`; running set widened to every status; badge counting own
   folder only; parent-gone text without the `== 1` test; the claim
   「終了忘れ?」 shown for an untagged orphan); empirical verifier; reviewers
   named by `scripts/check-review.sh`; run in the built app.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A secret in a command line is drawn | A token visible on screen or in a screenshot | Requirement 3's test, one case per rule; the redaction runs in the scan, so the frame never holds the raw line |
| A cut environment line names a different session | A live server marked 終了忘れ? | Requirement 2's cut-line case; `Unread` is never leftover |
| `ps` with many PIDs or large environments is slow | Ports appear later | Background task only, one call each, 8 s timeout, 4 MiB ceiling |
| `lstart` localized (e.g. `水`) breaks the parse | No start time | `LC_ALL=C` set on the command; the test feeds the `C` form, and an unparsable one yields `None` not a panic |
| Per-frame work grows with the rows | Stutter while the header or menu is drawn | Accepted and bounded in spec Design: parsing, redaction and the start string are scan-time; per frame, `session_ports` adds one session lookup and name clone per listed PID, and the open menu one join and one `relative_time` per row. No test measures frame time; `rust-reviewer` reads the draw path |
| One `ps` per PID creeps in | A scan spawns hundreds of children | The requirement 8 test's fake `ps` (a script in a temporary directory that appends one line per invocation) is called exactly twice for three listeners |
| Existing tests and literals miss the new fields | Compile errors | The compiler |

## Proof of completion

- `make q.fast` — passes.
- `cargo test --locked ports_318` — `8 passed`.
- `cargo test --locked ports_262` — all pass, unchanged in substance.
- In the built app: an agent session starts `python3 -m http.server`, the
  session is stopped, and the row reads 「…」(停止中) with the start, command,
  and 終了忘れ?; the header reads 終了忘れ? 1.

## Departures from the plan

