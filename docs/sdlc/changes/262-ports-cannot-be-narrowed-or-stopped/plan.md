# Plan: A session's ports, split by folder, and stoppable

- **Spec**: `./spec.md`
- **Approved**: 2026-11-09
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/git/ports.rs` | `PortRoot`, `AttributedPort`, `port_roots`, new `attribute_ports` signature, scan takes projects and sessions and lists git worktrees, `PortStopOutcome`, `stop_listening_port(_with)`. |
| `src/config.rs` | `PORT_STOP_PROGRAM`, `PORT_STOP_CHECK_TIMEOUT_SECONDS`, `PORT_STOP_SIGNAL_TIMEOUT_SECONDS`, `PORT_STOP_POLL_MILLIS`, `PORT_STOP_WAIT_MILLIS`. |
| `src/app.rs` | `listening_ports` type, `request_port_scan` inputs, `SessionPorts`/`session_ports`, badge, `ui_session_ports_menu`/`PortMenuAction`, `pending_port_stop`, `ui_port_stop_modal` and its call, `BackgroundKey::PortStop`, `BackgroundResult::PortStopped` and handler. |
| `src/i18n_tables.rs` | New rows in `EN_TABLE` and `KO_TABLE`; 「稼働中のポート」 removed. |
| `src/tests.rs` | The eleven `ports_262_` tests; existing `ports_` tests and change 077's UI test moved to the new types; `PortStop` in the background-key list. |
| `README.md`, `README.ja.md`, `README.ko.md` | Ports paragraph: own folder, elsewhere in the project, stop with confirmation. |
| `docs/sdlc/changes/262-…/` | `verification.yaml`, `mutations.py`, state. |

## Order of work

1. Contract tests first, against the pure functions: requirements 1 and 2
   (`attribute_ports`, `port_roots`) — they fail to compile, then fail, then
   the types and functions land in `src/git/ports.rs`. The tree does not
   compile between the type change and the `src/app.rs` call sites; step 2
   closes it in the same sitting.
2. Move `listening_ports`, `request_port_scan`, the scan signature and the
   existing tests to `AttributedPort`; write requirement 3's test, then
   `SessionPorts`/`session_ports`.
3. Requirements 6–8: the stop tests — refusal; the sequence with injected
   check and signal (mismatch, unverified, failed signal, every post-signal
   read failing, stopped, still held); one real `python3` listener child with
   real `lsof` and `/bin/kill` — then `listener_still_held`, `send_terminate`,
   `stop_listening_port_with`, the timeout parameter on `lsof_output`, and the
   constants.
4. Requirements 4, 5, 9, 10: badge, menu body, confirmation window,
   background key/result and handler, and the scan generation that drops a
   scan which was in flight when a stop finished — test first, then the
   drawing.
5. Strings in both tables; READMEs in three languages.
6. Tidy; `make q.fast`; `cargo test --locked ports_262_`, the change-077 UI
   test, the existing `ports_` tests and the background-key test.
7. `verification.yaml`, `mutations.py` (mutate: deepest-wins, own vs
   elsewhere matching on `spellings`, the name check before signalling, the
   refusal of PID ≤ 1 / own PID / PID > `i32::MAX`, `Unconfirmed` turned into
   `Stopped` when every post-signal read fails, the stale-generation drop, the
   rescan on outcome, and the claim that the badge counts only this folder by
   crediting a git worktree to the root);
   empirical and formal verifiers; reviewers named by `scripts/check-review.sh`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A different process is signalled (PID reused between scan and confirm) | Someone's unrelated program dies | Requirement 7's mismatched-name case; the re-verification runs immediately before `kill` |
| Git worktree paths and `lsof` directories are canonical and stored session/project paths may not be | Own empty and everything under elsewhere, or a symlinked project credited nothing | Requirement 2 compares canonical paths and keeps stored spellings; requirement 3's test uses a session whose stored path is a non-canonical spelling |
| A scan in flight when a stop finishes lands after it with the pre-stop listing | The stopped server reappears for up to 30 s | Requirement 10's test delivers an old-generation scan after the outcome and asserts it is dropped |
| A failed read after the signal is taken as "port free" | "Stopped" reported for a server still running | Requirement 8's test makes every post-signal read fail and asserts `Unconfirmed` |
| The test listener child outlives a failing test | Stray `python3` processes on CI | Each test kills and waits its child on every path (a drop guard) |
| The menu body draws per frame with new allocations | Stutter while the menu is open | Labels computed in the background (`PortRoot.label`); the frame formats only the row text it already formatted |
| `git worktree list` slows the scan on many projects | Ports appear later | Runs only when listeners exist; background thread; existing 20 s ceiling per call |
| `kill` hangs or is slow | Stop never reports | 2 s timeout, outcome `Failed` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked ports_262_` — `test result: ok. 11 passed; 0 failed`.
- `cargo test --locked ui_session_ports_renders_compact_popover_button_when_active` and the existing `ports_` tests — ok.
- `make q.fast` — passes.
- CI green on the branch head; the `verify` job re-runs `verification.yaml`.
- In the installed app after packaging: a root session's badge names only its
  own server; the worktree server is under 「プロジェクト内のほかの場所」; stopping
  it removes it.

## Departures from the plan

- Tests and code for each step were written in the same sitting rather than
  each test watched failing first. The mutation sweep (`mutations.py`, 17
  mutations, one per guard plus three claims) is the evidence that every new
  guard goes red when its rule is removed.
- The stop outcome handler also drops a listener that was seen stopped from
  the list at once, instead of leaving it until the rescan lands a moment
  later; requirement 10's test asserts it.
- Requirement 10's test checks the stop outcome and the stale scan as two
  separate deliveries: delivered together, the stale scan's own rescan request
  hid a handler that forgot to ask for one.
- The real-listener test for requirement 7 is recorded as `I6`: the
  verification record allows one line per id.
- The background-key partition test counted 55 variants by number; it now
  counts 56. Its name still says 55, because renaming a test reads as losing
  one at the commit gate.
- The READMEs said a right-click copies the URL; the menu has always had a copy
  control instead, and the rewritten paragraph says so.
- Two findings were reproduced against the first commit (94f2252) and fixed,
  each kept as a regression test: a session with no worktree runs in the
  project checkout but owned nothing (`F1`,
  `ports_262_session_without_worktree_owns_the_project_root`); and a PID that
  kept the port under a new name after SIGTERM was reported stopped (`F2`,
  `ports_262_stop_does_not_take_a_renamed_holder_for_gone`), so a read now
  says `Same`, `Renamed` or `Free` instead of yes or no. Requirements 3 and 7
  in `spec.md` say so and were approved again. With them the proof above
  reads `13 passed` for `cargo test --locked ports_262_`, not eleven.
- CI's `Control bands` step failed on the first branch run: `src/app.rs`
  reached 10 013 lines, past the `largest_module_lines` propose tier. The
  badge, menu and confirmation drawing moved to `src/app/screens.rs` beside
  the other modals, and `SessionPorts` and the outcome words to
  `src/git/ports.rs`; `src/app.rs` is now 9 729 lines (9 722 on `main`).
- The sweep grew to 23 mutations for the findings and for the guards the
  verifier read as unwatched (lsof complaining, the confirmation's wording,
  the stale scan's re-read, the list drop). All 23 were caught locally, run
  as two halves; the record keeps one `mutations.py`, which is the only sweep
  file the gate admits, and at the 24 s per mutation CI measured on the first
  branch run it fits inside CI's 900 s per obligation.
- Renumbered from 261 to 262 before landing. A parallel session opened
  `261-agent-ide-loops-require-manual-steps` minutes apart and was about to
  land first; `change_directory_numbers_are_unique` makes the second lander
  move. The directory, the branch, the `ports_262_` test names and the
  comments moved together; commits 2441799, 2354b74, 94f2252 and 9f0be9f
  still say 261 in their messages.
