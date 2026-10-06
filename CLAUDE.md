# Operon — Claude Code notes

Shared agent rules live in `AGENTS.md` and apply here too:

@AGENTS.md

## How work moves through this repository

`.claude/skills/feature-pilot/SKILL.md` is the entry point for anything past a
one-line fix, and `docs/sdlc/README.md` is the pipeline it runs: six stages, one committed
artifact each, automation up to a gate and a person at the gate. Skip it for a
typo — running it on those is how a pipeline gets abandoned.

- **Stopping is a state.** Record `awaiting-user`, `blocked`, or `failed` in
  `state.yaml` rather than just stopping: a session that stops without one is
  indistinguishable from a session that finished.
- **Plan mode is the default.** Interrogate the plan — what breaks, what was
  assumed, what the alternative was — and commit it as `plan.md` before editing a
  source file.
- **A mistake made twice becomes a check.** A convention goes in this file; a
  rule that applies only sometimes goes in `.claude/skills/` or `.claude/rules/`;
  anything checkable becomes a test in `src/tests.rs` or an eval in `evals/`.
  Either way it gets an entry in `docs/sdlc/lessons.md`, and the entry is not
  finished until its **Guard** column names what now catches it — watched
  failing, not assumed.
- **`git commit` runs fmt and clippy; CI runs the suite**, and the merge into
  `main` waits for it. A failing test is evidence; do not quiet it.
- `REVIEW.md` is the review policy. `.claude/skills/ship/SKILL.md` is the release.

## Shape of this codebase

- One crate, one module per concern. `src/main.rs` is only the entry point:
  process startup, `PATH`, fonts, window options. Everything else lives in the
  modules below, and `main.rs` re-exports them all with `pub(crate) use`, so a
  symbol is reachable crate-wide no matter which file defines it. Locate work by
  symbol name *and* confirm the file — a `rg` for a symbol is still the fastest
  route, but edit it where it is defined.

  <!-- Every path below is checked against the repository by
       harness_documents_only_name_paths_that_exist, and this list is checked
       against main.rs by claude_md_maps_every_module_that_exists.
       Add a module, add its line. -->
  - `src/app.rs` — `OperonApp`: all app state, background tasks, and the
    top-level UI wiring. The usual starting point.
    `src/app/screens.rs` beside it holds one run lifted out of it — toolbar
    through terminal workspace — with the helpers only that run calls. It is a
    line range and not a rule: three drawing methods stayed behind. Look in
    both, and `rg` for the symbol rather than guessing which.
  - `src/tests.rs` — the whole inline suite. `#[cfg(test)]`, no other module has
    tests of its own.
  - `src/cli.rs` — discovering and scanning agent-CLI sessions and transcripts;
    the shared-session archive lives here.
  - `src/history.rs` — cross-CLI conversation restore and import.
  - `src/transcript.rs` — `TRANSCRIPT_VOCABULARY`: what a restore does with each
    kind the three CLIs write. The only place that judgement is made.
  - `src/agents.rs` — agent kinds, launch-command construction, and the
    `is_safe_agent_command` / `is_safe_agent_option` validation gates.
    `src/agents/settings.rs` beside it holds the remembered launch options
    (model, mode, effort, flags, custom command) in `recent-agent-settings.json`.
  - `src/tmux.rs` — tmux session lifecycle: create, respawn, stop, retry.
    `src/tmux/hooks.rs` beside it is the status hooks the agent CLIs run: the
    managed scripts, their registration in each CLI's own settings file, the
    socket they post to, and the state machine that reads them.
  - `src/git.rs` — git status, diffs, branches, worktrees.
    `src/git/setup.rs` beside it is what a repository says about preparing a
    fresh checkout of itself: the script at `.operon/setup.sh`, its digest and
    preview, and the per-project approvals that decide whether it may run.
    `src/git/comments.rs` is the notes a person writes against a diff: where
    they are kept, and the message they become when they are handed to an agent.
    `src/git/ports.rs` is which worktree a listening socket belongs to, read
    from `lsof` and joined by the owning process's working directory.
  - `src/theme.rs` — `Palette` and the three `*_PALETTE` tables.
  - `src/store.rs` — the local index: atomic writes, `flock`, migration.
  - `src/models.rs` — the persisted types.
  - `src/glyphs.rs` — the `ICON_*` constants and `ICON_VOCABULARY`.
  - `src/i18n.rs` — `tr`, the `tf!` macro, the active language, and first-launch
    language detection.
  - `src/i18n_tables.rs` — the per-language message tables `tr` looks in.
  - `src/exec.rs` — `run_command_with_output_limit`, `run_command_with_timeout`.
  - `src/files.rs`, `src/markdown.rs`, `src/config.rs`, `src/sys.rs`,
    `src/util.rs`, `src/prelude.rs` — file classification, markdown parsing,
    settings, platform calls, small helpers, shared imports.
  - `src/ui/` — `mod.rs`, `widgets.rs`, `terminal.rs`, `diff.rs`, `files.rs`,
    `markdown_view.rs`, `syntax.rs`: the reusable drawing code `app.rs` calls
    into.
- **User-facing text is Japanese; code, comments, and docs stay English.** Every
  notice, button, label, and error string a person can read is written in
  Japanese. Strings matched against tool output — tmux errors like
  `can't find session`, CLI prompts like `do you trust this folder` — are
  matching patterns, not copy, and must stay exactly as the tool emits them.
  The Japanese string at the call site is also the message id: reach a person
  through `tr("…")` for a plain string or `tf!("…{name}", name = value)` for one
  with holes — never `format!` — and give the id a row in every table in
  `src/i18n_tables.rs`. A `const` cannot call `tr`, so a table like
  `CODEX_FLAGS` stores message ids and the drawing code translates them.
- `eframe`/`egui` 0.31 immediate-mode GUI. macOS only.
- Many `Command::new` call sites drive external tools (`tmux`, `git`, `gh`,
  `bash`, `codex`, `claude`, `agy`). Route new subprocess work through
  `run_command_with_output_limit` or `run_command_with_timeout` so the
  output-byte budgets and timeouts still apply.
- The agent kind `"gemini"` launches **Antigravity's `agy`** binary, not Gemini
  CLI. It is a persisted store value, so the name cannot be changed without a
  schema migration. UI labels and `README.md` correctly say "Antigravity".

## Policies that load when you open the file they govern

Four policies live in `.claude/rules/`, each declaring its paths, so the body
arrives when a matching file is read and costs nothing otherwise. The pointers
are here because a rule you do not know exists cannot be consulted:

| Rule | Arrives with | Forbids |
|---|---|---|
| `.claude/rules/palette-and-glyphs.md` | `src/theme.rs`, `src/glyphs.rs`, `src/app.rs`, `src/app/`, `src/ui/` | a colour or glyph literal at a call site |
| `.claude/rules/transcripts.md` | `src/history.rs`, `src/transcript.rs`, `src/cli.rs` | judging another CLI's record kind in a `match`; inferring a terminal's conversation |
| `.claude/rules/identifiers.md` | `src/config.rs`, `src/tmux.rs`, `src/agents.rs` | one string in two places; a guard restating what it guards |
| `.claude/rules/rust.md` | any `.rs`, `Cargo.toml` | work in a per-frame draw path; an unargued `unwrap`; a raw `.output()` |

A path rule fires when Claude **reads** a matching file, not when it writes one:
an edit is covered, a brand-new file is not. So every policy above keeps its test
in `src/tests.rs` — the rule reminds, the test enforces. Do not re-inline one
here; this file is what `always_loaded_bytes` measures.

## What the gates do not cover, and how to see the harness move

Verification is part of "done", so know what a healthy run prints:

| Command | Healthy output |
|---|---|
| `cargo fmt --check` | nothing |
| `cargo test --locked` | `test result: ok. N passed; 0 failed; 6 ignored` |
| `cargo clippy --locked -- -D warnings` | nothing past the compile lines |
| `bash scripts/check-bands.sh` | `bands: N metrics within their bands` |
| `bash scripts/check-transcript-vocabulary.sh` | `N kinds observed locally, all classified` |

The `6 ignored` is the number that matters: those six need a live agent CLI, and a
seventh means a test was moved out of CI rather than fixed.

`cargo test --locked` skips six `#[ignore]`d tests that need a live `codex`,
`claude`, or `agy`. They are the only end-to-end coverage of the cross-CLI
restore. Touching `src/history.rs` or the archive path in `src/cli.rs` means
running `cargo test --locked -- --ignored` too, or saying plainly that you could
not.

`bash scripts/harness-metrics.sh` prints one JSON object describing this
repository as a place to work: module sizes, how many tests CI actually runs, how
many bytes of instructions load before any code is read. Diff it across a change
when the change is *to* the harness.
