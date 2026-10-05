# Plan: a session says what its worktree is listening on

- **Spec**: `./spec.md`
- **Approved**: 2026-09-06
- **Status**: in progress

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | The interval, the timeout, the entry and byte ceilings, and the program name. |
| `src/git/ports.rs` (new) | The two parsers, the attribution, the URL, and the scan. |
| `src/git.rs` | `pub(crate) mod ports;` and its re-export. |
| `src/app.rs` | `listening_ports`, `ports_scanned_at`, `BackgroundKey::PortScan`, the interval check in the frame loop, and the header line. |
| `src/app/screens.rs` | One call, under the row that names the worktree. |
| `src/i18n_tables.rs` | EN and KO rows. |
| `src/tests.rs` | The seven tests named in the spec's Acceptance. |
| `CLAUDE.md` | The module map gains the child file. |
| `README.md`, `README.ja.md`, `README.ko.md` | One bullet each. |

## Order of work

1. Capture the real `lsof` output from this machine **before** writing a parser
   for it, and paste it into the test as the fixture.
2. `src/config.rs` and the pure half of `src/git/ports.rs`, with their tests.
3. The scan, the app state, the interval, and the header line.
4. `CLAUDE.md` and the three READMEs.
5. Gates, bands, a real server in a real worktree, commit.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The parser is written against imagined output | It works on nothing real | The fixture was captured from this machine first, and `a_missing_lsof_is_a_reading_of_nothing_rather_than_an_error` also runs the real program once. Lesson 013 is the rule this follows. |
| A port is given to the wrong worktree | Somebody reviews the wrong branch, which is the defect this exists to fix | Deepest match, with its own test, including the sibling whose path merely starts with the same characters. |
| A dual-stack bind is counted twice | Two rows for one server | Identity is process and port; its own test. |
| The scan blocks a frame | Stutter proportional to the process table | It is a background task on an interval, requested from the frame loop beside the session poll, never from a draw path. |
| `lsof` is missing or slow | A standing complaint about a reading nobody asked for | Every failure is a reading of nothing, and nothing is drawn. |

## Proof of completion

- `cargo fmt --check`, `cargo test --locked` seven higher with `6 ignored`,
  `cargo clippy --locked -- -D warnings`, `bash scripts/check-bands.sh`.
- Each guard watched failing by mutation.
- In the running app: a server in a worktree, named in that session's header
  within the interval, clickable.

## Departures from the plan

- **A test emptied `PATH` and broke another test.** The failure-path test set
  `PATH` to an empty directory so the scan would find no `lsof`. `set_var` is
  process-wide and the suite runs in parallel, so it took out
  `a_managed_launch_carries_the_hook_environment`, which needs to find `tmux` —
  a test that fails somewhere else, for a reason that is nowhere near it. The
  program name is now a parameter, `scan_listening_ports_with`, and the test
  names something that does not exist. `PORT_SCAN_PROGRAM` in `src/config.rs`
  is what the real call passes.
- **The first attempt at the command-name mutation passed**, because every block
  in the fixture had a `c` line and clearing the name changed nothing. `-F`
  omits a field it has no value for, so the fixture gained a block without one
  and the test now asserts such a block is nameless rather than named after
  whoever came before it. Second time recording this: a mutation that passes is
  evidence only once you have checked it landed where it was aimed.
- **`scan_listening_ports` returns a reading, not a `Result`.** Every failure
  means the same thing to every caller, and a `Result` would be a decision each
  caller made separately and differently.
