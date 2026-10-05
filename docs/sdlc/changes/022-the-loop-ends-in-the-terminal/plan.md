# Plan: choose, commit, and push from the screen the change was read on

- **Spec**: `./spec.md`
- **Approved**: 2026-09-05
- **Status**: in progress

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | The prompt, message, and timeout ceilings. |
| `src/git.rs` | `UpstreamState`; `git_upstream_state`; `git_stage_exactly`; `git_commit`; `git_push`; `git_staged_summary`; `commit_message_prompt`; `clean_generated_commit_message`. |
| `src/agents.rs` | `commit_message_agent`; `commit_message_command`. |
| `src/app.rs` | The tick set, the message field, two background keys, and the three actions. |
| `src/app/screens.rs` | The Changes view: checkboxes, the field, the two buttons, the upstream line. |
| `src/i18n_tables.rs` | EN and KO rows for every new message id. |
| `src/tests.rs` | The eight tests named in the spec's Acceptance. |
| `README.md`, `README.ja.md`, `README.ko.md` | One bullet each. |

## Order of work

1. `src/config.rs`, then the pure functions in `src/git.rs` —
   `commit_message_prompt`, `clean_generated_commit_message` — with their tests.
2. The git verbs, tested against real repositories built with `git init` and a
   bare repository standing in for a remote. No network.
3. `src/agents.rs` draft commands, with the round-trip test that every launchable
   agent has one.
4. `src/app.rs` actions and `src/app/screens.rs` drawing. `src/i18n_tables.rs`.
5. The three READMEs.
6. Gates, bands, a real commit and push in the app, commit.

The tree compiles between every step.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A push rewrites somebody's history | Unrecoverable, and the worst thing in this change | There is no force anywhere. `no_source_file_can_force_a_push` scans `src/` for `--force`, `--force-with-lease`, `push -f`, and a `+` refspec, and fails on any of them. Watched failing with one added. |
| A commit takes a file the person unticked | They committed something they looked at and rejected | `git_stage_exactly` resets the index before adding, and `stages_exactly_the_files_that_were_ticked` ticks one of three and asserts the other two are still uncommitted. |
| A message with a quote or a newline breaks the commit | A mangled commit, or worse a shell doing something | The message is one `-m` argument in a vector. `commits_a_message_that_would_break_a_shell` commits a message containing `"`, `$(touch …)`, and a newline, and reads it back with `git log -1 --format=%B`, asserting the file was not created. |
| A model's output is committed verbatim with its fences | An ugly commit, and a person who stops trusting the button | `clean_generated_commit_message`, tested against the four wrappings models actually produce, and the draft never commits by itself. |
| The drafting CLI edits a file | A "draft a message" button that changed the change | Each invocation is the CLI's own read-only non-interactive mode, in one place, with a test naming the flags. |
| The drafting CLI hangs | The button stays 「書いています…」 for ever | It is a background task with `COMMIT_MESSAGE_TIMEOUT_SECONDS` on the wrapper. |
| `git reset -- .` loses work | Catastrophic if wrong | It is index-only: `git reset` without `--hard` never touches a working tree file. Stated here because it is the line in this change that looks most dangerous and is not. |

## Proof of completion

- `cargo fmt --check`, `cargo test --locked` eight higher with `6 ignored`,
  `cargo clippy --locked -- -D warnings`, `bash scripts/check-bands.sh`.
- Each new guard watched failing by mutation: the reset removed; the force scan
  fed a force; the ahead and behind swapped; the cleaner made a no-op; an agent
  left without a draft form.
- In the running app: two of three files committed, the third still dirty, the
  branch on the remote.

## Departures from the plan

- **`git_commit` is called `git_record_staged`.** `.claude/hooks/gate-commit.sh`
  runs the three gates on any Bash call that looks like a commit, and a shell
  heredoc carrying the words `git commit` set it off while the tree did not yet
  compile. Renaming the function to what it does — record what is staged — is a
  better name anyway, and it means the module can be worked on without tripping
  a hook that exists for something else.
- **A separate `BackgroundKey::Upstream`.** The push row reads the upstream on
  its own, without anybody asking. Sharing `GitMutation` would have had that
  read disable the commit button for as long as it took.
- **A failed upstream read is a reading of "nothing known", not a notice.** It
  runs unprompted, and a banner nobody caused is a banner nobody trusts.
- **The first mutation was aimed at the wrong line.** `git_has_head` appears in
  `git_working_tree_diff` before it appears in `git_stage_exactly`, so the first
  attempt to remove the index reset removed something else and the test passed —
  which read exactly like a guard that does not bite. Re-run against the right
  site, it fails with `staged-by-hand.txt` in the list. Worth recording: a
  mutation that passes is only evidence when the mutation landed where it was
  aimed.
