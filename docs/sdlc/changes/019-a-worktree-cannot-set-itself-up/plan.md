# Plan: a repository says once how a new checkout of it is prepared

- **Spec**: `./spec.md`
- **Approved**: 2026-09-05
- **Status**: in progress

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | `SETUP_SCRIPT_RELATIVE_PATH`, `SETUP_SCRIPT_MAX_BYTES`, `SETUP_SCRIPT_PREVIEW_LINES`. |
| `src/git/setup.rs` (new, child of `src/git.rs`) | Reading the script, its digest and preview; the launch command; the trust file and its four operations. |
| `src/git.rs` | `pub(crate) mod setup;` and its re-export. |
| `src/app.rs` | `pending_setup`, `setup_sessions`, `setup_trust`; `offer_setup`, `start_setup_session`, `ui_setup_prompt`; the finished-setup notice in the session poll. |
| `src/i18n_tables.rs` | EN and KO rows for every new message id, in sorted position. |
| `src/tests.rs` | The eight tests named in the spec's Acceptance. |
| `CLAUDE.md` | The module map gains the child file under the `src/git.rs` entry. |
| `README.md`, `README.ja.md`, `README.ko.md` | One bullet each. |

## Order of work

1. `src/config.rs` constants; `src/git/setup.rs` reading, digest, preview, and
   `setup_command`, with their tests. Compiles; green.
2. The trust file: load, save, approve, check. Tested against a temporary data
   directory, including a corrupt file falling back to "nothing approved".
3. `src/app.rs` wiring: the fields, `offer_setup` from the `WorktreeCreated`
   handler, `start_setup_session`, the exit notice. Compiles; green.
4. `ui_setup_prompt` in the Worktrees tab, matching the agreed screen.
   `src/i18n_tables.rs` rows. Compiles; green; clippy clean.
5. `CLAUDE.md` and the three READMEs.
6. Gates, bands, a real run in the app, commit.

The tree compiles between every step.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The worktree path has a space and the command silently does nothing | Exactly lesson 013, one module over | `setup_command` quotes, and `a_setup_script_runs_end_to_end_in_a_path_with_a_space` runs the built command through a real shell in a real directory with a space and asserts the side effect. |
| A script is approved, then edited, and runs anyway | The person approved something else | The digest is recomputed from a fresh read at the moment of running, not carried from the moment of showing. Test: the file is edited between the two. |
| A corrupt trust file approves everything | Silent escalation | The fallback is an empty approval list, which fails towards asking. Test: a file of garbage. |
| The preview does not match what runs | The person approves eight lines and a hundred run | The preview is a slice of the same bytes the digest is taken over, and the total line count is drawn beside it. Test: a script longer than the preview. |
| A setup session is mistaken for the person's own terminal | They lose the session, or it is stopped as an orphan | It is an ordinary session with its own name; nothing about the existing lifecycle changes. `setup_sessions` only decides whether one notice is worded for setup. |
| `src/app.rs` grows past its warn band again | `scripts/check-bands.sh` warns harder | The reading, the trust, and the command all live in `src/git/setup.rs`; `src/app.rs` gains wiring and one draw function. If the band moves it is recorded. |
| A file that is a symlink out of the tree | Something outside the repository runs | The preview is read through the same path bash will read, so what is shown is what runs; the person is approving the contents they see either way. Stated rather than blocked. |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `ok`, eight higher than before, `6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-bands.sh` — no new breach beyond what is recorded.
- Each new guard watched failing by mutation: the quoting removed; the re-read
  at run time replaced by the shown digest; the corrupt-file fallback made
  permissive; the preview taken from a different read than the digest.
- In the running app: the block appears, 実行する runs it, the checkbox makes
  the next one silent, one edited character brings the block back.

## Departures from the plan

- **`decide_setup_run` was added.** The plan had `accept_pending_setup` in
  `src/app.rs` comparing a fresh digest against the shown one. That put the one
  judgement this feature exists to make — is the file still the file that was
  approved — somewhere it could only be tested through an application. It is
  now a three-way decision in `src/git/setup.rs` with a test of its own, and the
  method reads it rather than makes it.
- **`setup_session_name` lives in `src/agents.rs`** beside `plain_terminal_name`,
  because that is where a session's display name is built and a second place to
  look for one is how the two drift apart.
- **Nine tests, not eight.** The extra one is `rechecks_the_setup_script_at_the_moment_of_running`,
  which the decision above made writable.
- **`largest_module_lines` rose to 9078 from 8837.** `src/app.rs` is 278 lines
  further past its warn band, even with the reading, the trust, and the command
  in the child module. It stays in `recorded, no action` by the band's own rule,
  but this is the second change in a row to say so: splitting `src/app.rs` is
  its own change and it is now overdue.
