# Rust build chain — stage 3 in this crate

Read with `CLAUDE.md` open for the module map and conventions.

## 1. Contracts first

**A persisted shape** (`Store` and records in `src/models.rs`, written by
`src/store.rs`) is a `paused` surface in `docs/sdlc/risk.yaml`; get approval first.

- `parse_store_contents` refuses `schema_version` newer than
  `STORE_SCHEMA_VERSION` (`src/config.rs`); older files load via
  `#[serde(default)]`.
- Bump `STORE_SCHEMA_VERSION` only when on-disk meaning changes (like
  `EVIDENCE_BASED_FAILURE_SCHEMA_VERSION` in `src/app.rs`).
- Writes use the atomic writer and `WriteOutcome` per
  `.claude/skills/durability-invariants/SKILL.md`.
- Tests: round trip, older-file load, and newer-schema refusal.

**Another program's output** (git, tmux, agent CLIs, transcripts):

- **Short commands**: use `git_command(path)` in `src/git.rs` or
  `tmux_command()` in `src/tmux.rs`, routed through `run_command_with_timeout`
  or `run_command_with_output_limit(command, timeout, COMMAND_OUTPUT_MAX_BYTES,
  COMMAND_ERROR_MAX_BYTES)` in `src/exec.rs` — never a bare `.output()`. Check
  `stdout_truncated` on the latter.
- **Agent CLIs**: never use timeout helpers. Use `build_agent_launch_command` in
  `src/agents.rs` (`is_safe_agent_command` / `is_safe_agent_option`) and
  `launch_stored_session` in `src/tmux.rs` (`tool_available`, `agent_shell_command`,
  `shell_quote`, `start_tmux_agent_session`). Validate resume IDs with
  `is_safe_cli_session_id` and tmux names with `is_safe_tmux_name`. Every shell-spliced
  value in `src/app.rs` or `src/app/screens.rs` must pass `shell_quote`.
- **Long-lived children** (Codex app-server in `src/history.rs`): strip
  `INHERITED_REPOSITORY_POINTERS` (`src/config.rs`) and ensure wait/kill on all
  paths.
- Parse in pure `&str` functions tested on normal, empty, byte-ceiling-truncated,
  and garbage inputs. SQLite in `src/history.rs` opens Antigravity DBs with
  `SQLITE_OPEN_READ_ONLY` except inside the restore transaction.
- `src/exec.rs`, `src/agents.rs`, `src/tmux.rs`, and `src/git.rs` pull in
  `.claude/agents/subprocess-safety-reviewer.md` (`scripts/check-review.sh`).

## 2. Tests

- Add tests to `src/tests.rs` as `#[test] pub(crate) fn <name>()` and watch each
  fail first with `cargo test --locked <name>`.
- Drop inherited git pointers in tests that spawn `git`.
- Never add `#[ignore]` or delete a test (`AGENTS.md`).

## 3. Wiring

- **New module**: `mod <name>;` and `pub(crate) use <name>::*;` in `src/main.rs`,
  plus a `CLAUDE.md` module-map line.
- **Constants/identifiers**: `src/config.rs`.
- **Strings**: Japanese message id via `tr("…")` or `tf!("…{name}…", name = value)`,
  with sorted rows in `EN_TABLE` and `KO_TABLE` in `src/i18n_tables.rs`.
- **Keys**: `src/app/keymap.rs`. **Colour**: `DESIGN.md` role via `src/theme.rs`.
  **Icons**: `ICON_*` and `ICON_VOCABULARY` in `src/glyphs.rs`.
- **Background work**: `BackgroundKey` and `BackgroundResult` in `src/app.rs` via
  `spawn_background(key, job)` and `process_background_results`. Never do I/O or
  spawns on the draw path.
- **Errors**: `anyhow::Result` (`src/prelude.rs`); surface via `tr` into
  `self.notice` or `notice_briefly`.

## 4. Tidy & Verify

- No `dbg!`, stray `println!`/`eprintln!`, unused imports, or magic numbers;
  run `make q.fix` / `make q.check`.
- Run `cargo test --locked -- --ignored` when restore or archive paths change,
  and `bash scripts/check-bands.sh` when steering/reference text changes.
