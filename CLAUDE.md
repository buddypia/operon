# Operon — Claude Code notes

Shared agent rules live in `AGENTS.md` and apply here too:

@AGENTS.md

**Plan mode is the default.** Interrogate what breaks, what was assumed, and
alternatives; commit `plan.md` before editing source files.

## Shape of this codebase

- One crate, one module per concern. `src/main.rs` is the entry point (startup,
  `PATH`, fonts, window options) and re-exports all modules below with
  `pub(crate) use`. Locate symbols with `rg` and edit where defined.

  <!-- Checked by harness_documents_only_name_paths_that_exist and
       claude_md_maps_every_module_that_exists. Add a module, add its line. -->
  - `src/app.rs` — `OperonApp`: app state, background tasks, top-level UI wiring.
    `src/app/screens.rs` holds toolbar through terminal workspace drawing.
    `src/app/knowledge.rs` holds the knowledge tab, model block, and run wiring.
  - `src/tests.rs` — inline `#[cfg(test)]` suite; no other module has tests.
  - `src/cli.rs` — discovering/scanning agent-CLI sessions and shared archive.
  - `src/history.rs` — cross-CLI conversation restore and import.
  - `src/transcript.rs` — `TRANSCRIPT_VOCABULARY`: restore classification table.
  - `src/agents.rs` — agent kinds, launch-command construction, and
    `is_safe_agent_command` / `is_safe_agent_option` gates.
    `src/agents/settings.rs` holds remembered launch options in
    `recent-agent-settings.json`.
  - `src/tmux.rs` — tmux session lifecycle (create, respawn, stop, retry).
    `src/tmux/hooks.rs` manages CLI status hook scripts, settings registration,
    socket listener, and state machine.
  - `src/git.rs` — git status, diffs, branches, worktrees.
    `src/git/setup.rs` handles `.operon/setup.sh` digests, previews, and
    approvals. `src/git/comments.rs` handles diff review notes.
    `src/git/ports.rs` maps listening sockets via `lsof` to worktrees.
  - `src/knowledge.rs` — per-project knowledge base: SQLite store, ranking,
    judge, freshness, search, and the `Operon knowledge search` CLI.
    `src/knowledge/embed.rs` holds the EmbeddingGemma 2 model files, download,
    and in-process Metal runtime. `src/knowledge/map.rs` computes the 2D map:
    neighbours, groups, layout, hulls.
  - `src/theme.rs` — `Palette` and the three `*_PALETTE` tables.
  - `src/store.rs` — local index: atomic writes, `flock`, migration.
  - `src/models.rs` — persisted types.
  - `src/glyphs.rs` — `ICON_*` constants and `ICON_VOCABULARY`.
  - `src/i18n.rs` — `tr`, `tf!` macro, active language, first-launch detection.
  - `src/i18n_tables.rs` — per-language message tables for `tr`.
  - `src/exec.rs` — `run_command_with_output_limit`, `run_command_with_timeout`:
    every child process goes through one of them.
  - `src/files.rs`, `src/markdown.rs`, `src/config.rs`, `src/sys.rs`,
    `src/util.rs`, `src/prelude.rs` — file classification, markdown parsing,
    settings, platform calls, helpers, shared imports.
  - `src/ui/` — `mod.rs`, `widgets.rs`, `terminal.rs`, `diff.rs`, `files.rs`,
    `markdown_view.rs`, `syntax.rs`, `knowledge_map.rs`: reusable drawing modules.
- **User-facing text is Japanese; code, comments, and docs stay English.**
  Tool-output match patterns (`can't find session`, `do you trust this folder`)
  stay as emitted. Reach users via `tr("…")` or `tf!("…{name}", name = value)` —
  never `format!` — with rows in every table in `src/i18n_tables.rs`.
- `eframe`/`egui` 0.31 immediate-mode GUI on macOS.
- Persisted agent kind `"gemini"` launches **Antigravity's `agy`** binary; UI
  labels and `README.md` say "Antigravity".

## Policies that load when you open the file they govern

Path-scoped rules in `.claude/rules/` load when a matching file is read, and
each is enforced by a test in `src/tests.rs`:

| Rule | Arrives with | Forbids |
|---|---|---|
| `.claude/rules/palette-and-glyphs.md` | `src/theme.rs`, `src/glyphs.rs`, `src/app.rs`, `src/app/`, `src/ui/` | colour, glyph, size literals; raw buttons |
| `.claude/rules/transcripts.md` | `src/history.rs`, `src/transcript.rs`, `src/cli.rs` | judging another CLI's record kind in a `match`; inferring a terminal's conversation |
| `.claude/rules/identifiers.md` | `src/config.rs`, `src/tmux.rs`, `src/agents.rs` | one string in two places; a guard restating what it guards |
| `.claude/rules/rust.md` | any `.rs`, `Cargo.toml` | work in a per-frame draw path; an unargued `unwrap`; a raw `.output()` |
