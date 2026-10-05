# Rust build chain — stage 3 in this crate

Read with `CLAUDE.md` open: it holds the module map and the conventions; this
file holds only the order and the traps a change meets while building.

## 1. Contracts first

**A persisted shape** (`Store` and the records in `src/models.rs`, written by
`src/store.rs`). Touching it is a `paused` surface in `docs/sdlc/risk.yaml`, so
approval comes before the edit.

- `parse_store_contents` deserialises and refuses a file whose `schema_version`
  is newer than `STORE_SCHEMA_VERSION` (`src/config.rs`). There is no per-version
  conversion code: an older file loads because every added field is
  `#[serde(default)]`.
- Bump `STORE_SCHEMA_VERSION` only when the meaning of data already on disk
  changes, and re-interpret it where the store is loaded — the model is the
  re-judge in `src/app.rs` gated on `EVIDENCE_BASED_FAILURE_SCHEMA_VERSION`.
- Writes go through the atomic writer and its `WriteOutcome`; the ordering rules
  are `.claude/skills/durability-invariants/SKILL.md`.
- Tests: a round trip; a file written before the field existed still loads; a
  newer schema is refused.

**Another program's output** (git, tmux, an agent CLI, a transcript file).
Which path a child takes depends on how long it lives:

- **A short command whose output is read.** Build the `Command` with the
  program's own constructor where one exists — `git_command(path)` in
  `src/git.rs`, `tmux_command()` in `src/tmux.rs` — and run it through
  `run_command_with_timeout` or `run_command_with_output_limit(command,
  timeout, COMMAND_OUTPUT_MAX_BYTES, COMMAND_ERROR_MAX_BYTES)` in `src/exec.rs`;
  never a bare `.output()`. The second returns `Ok` with `stdout_truncated` set
  when the ceiling cut the output, and the caller checks that flag before it
  treats the output as complete; the first makes truncation an error.
  `no_git_command_in_the_crate_can_inherit_the_session_repository` and
  `every_tmux_the_crate_runs_goes_through_tmux_command` refuse a bare `git` or
  `tmux`.
- **An agent CLI a person works in.** Never through the timeout helpers, which
  kill the child at the deadline. Two functions own the launch, and a new path
  calls them rather than copying their parts:
  `build_agent_launch_command` in `src/agents.rs` (the agent allowlist, the
  per-CLI mode and effort lists, the flag-conflict check, and
  `is_safe_agent_command` / `is_safe_agent_option`), and `launch_stored_session`
  in `src/tmux.rs`, which checks `tool_available` for tmux and for the agent's
  own program, quotes the person's goal through `agent_shell_command` and
  `shell_quote`, and only then calls `start_tmux_agent_session`. That last
  function quotes nothing, so every other caller hands it a string it has
  already quoted — today the approved setup and run scripts (`src/app.rs`,
  `src/app/screens.rs`), each `bash` plus a `shell_quote`d path. The list grows; read the current one
  with `git grep -n 'start_tmux_agent_session('` rather than trusting this
  sentence for it. A resume id passes `is_safe_cli_session_id`, a tmux name
  `is_safe_tmux_name`. tmux runs the command through a shell, so every value
  spliced into that string goes through `shell_quote` — a goal holding `$(…)`
  or `;` otherwise runs.
- **A long-lived child the app talks to** (the Codex app-server in
  `src/history.rs`). It cannot use `src/exec.rs`: remove
  `INHERITED_REPOSITORY_POINTERS` (`src/config.rs`) from its environment, and
  give it an explicit end — a wait or a kill on every path, including errors.

Parse in a function that takes `&str` and returns a value, so the test feeds it
fixture text: the normal case, empty, truncated at the byte ceiling, and
garbage. State the item ceiling for anything that scans. SQLite lives only in
`src/history.rs`: Antigravity's database is read with `SQLITE_OPEN_READ_ONLY`
and written only by the restore transaction.

`src/exec.rs`, `src/agents.rs`, `src/tmux.rs`, and `src/git.rs` are the
`subprocess` surface in `docs/sdlc/risk.yaml`, reviewed by
`.claude/agents/subprocess-safety-reviewer.md`. `src/history.rs` is
`foreign-store`, which no reviewer owns: a change to its app-server child pulls
that reviewer in only when the diff adds a `Command::new`, a spawn, a raw output
call, or `unsafe` (`scripts/check-review.sh`). Ask for it by hand when the change
alters how that child ends.

Each contract test is watched failing before the code that satisfies it exists.

## 2. Tests

- Every test lives in `src/tests.rs` as `#[test] pub(crate) fn
  <behaviour_in_words>()`; do not add a `mod tests` to a module (`CLAUDE.md`).
- Run the new one alone first and watch it fail for the reason it names:
  `cargo test --locked <name>`.
- A test that starts `git` drops the inherited repository pointers (lesson 048).
- Never add `#[ignore]` and never delete a test; the commit gate refuses both.
  A test that needs a live agent CLI is the exception `AGENTS.md` describes.

## 3. Wiring

- **New module**: `mod <name>;` and `pub(crate) use <name>::*;` in `src/main.rs`
  (`i18n`, `i18n_tables`, and `prelude` are the `mod`-only exceptions), and a line
  in the `CLAUDE.md` map — `claude_md_maps_every_module_that_exists` fails
  without it. A submodule is re-exported by its parent.
- **Constants and identifiers** two places must agree on: `src/config.rs`.
- **Strings**: the Japanese text is the message id — `tr("…")`, or
  `tf!("…{name}…", name = value)` with named arguments only. Add the row to both
  `EN_TABLE` and `KO_TABLE` in `src/i18n_tables.rs`, in message-id order
  (`every_translation_table_is_sorted_by_message_id`,
  `every_message_id_has_a_row_in_every_table`).
- **Keys**: `src/app/keymap.rs`. **Colour**: a role from `DESIGN.md` through
  `src/theme.rs`, never a literal. **Icons**: an `ICON_*` constant and an
  `ICON_VOCABULARY` entry in `src/glyphs.rs`.
- **Background work**: a `BackgroundKey` and a `BackgroundResult` variant in
  `src/app.rs`; start it with `spawn_background(key, job)`; handle the result in
  `process_background_results`, removing the key and setting the state. The
  draw path never reads a file, starts a process, or waits.
- **Errors**: `anyhow::Result` comes in through `src/prelude.rs`. What a person
  reads goes through `tr` into `self.notice` or `notice_briefly`; no `unwrap()`
  or `panic!` on a path input can reach.

## 4. Tidy

Over the diff: no `dbg!`, stray `println!`, or `eprintln!`; no unused import; no
commented-out code or test; no magic number that belongs in `src/config.rs`.
`make q.fix` handles formatting and the mechanical clippy fixes.

## 5. Verify

- `make q.check` is the three gates in order: `cargo fmt --check`,
  `cargo test --locked`, `cargo clippy --locked -- -D warnings`. The commit gate
  runs the same three itself.
- `cargo test --locked -- --ignored` when restore or the shared-session archive
  moved, or say that you could not.
- A change that paints is looked at in the built app at a realistic width, in
  Japanese, in its empty and error states — the screen approved at stage 2 is
  what it is compared with.
- `bash scripts/check-bands.sh` when the change adds steering or reference text.
