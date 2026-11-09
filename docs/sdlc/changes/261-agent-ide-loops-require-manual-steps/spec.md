# Spec: Hunk-level diff actions, agent context handoff, `@` mentions, external IDE line jump, parallel arena launch, and `⌘K` file/symbol jump

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. **Hunk-level Stage and Discard**: `extract_single_hunk_patch` in `src/git.rs` builds a valid unified diff patch (`--- a/<path>`, `+++ b/<path>`, and the selected `@@ ... @@` hunk lines) for a single hunk in a `DiffFile`. `git_stage_hunk` applies it to the index (`git apply --cached --unidiff-zero`) and `git_discard_hunk` reverses it in the working tree (`git apply --reverse --unidiff-zero`). `diff_pane_with_actions` in `src/ui/diff.rs` draws `ステージ` and `破棄` controls on each `DiffLineKind::Hunk` row and returns the chosen `DiffHunkAction`. Tests: `ide_261_extract_and_apply_single_hunk_stage_and_discard`, `ide_261_diff_pane_hunk_buttons_emit_stage_and_discard`.
2. **Send Editor Context & Terminal Errors to Agent**: `format_code_context_for_agent` in `src/util.rs` formats a file path, 1-based line range, and code snippet into a bounded prompt block (`@<path>:L<start>-L<end>`), and `extract_recent_terminal_error` scans terminal scrollback lines from the bottom for failure lines (`error`, `Error`, `FAILED`, `panicked at`, `ERR!`, `エラー`, `失敗`) and returns a bounded trailing excerpt (at most 25 lines and 4,000 characters). The built-in editor header provides `エージェントに送る` and the terminal session menu provides `直近のエラーをエージェントに送る`, both refusing when no active agent session exists in the project or when the agent is `AgentActivity::Working`. Tests: `ide_261_format_code_context_and_extract_terminal_error`.
3. **`@` Context Mentions in Launch Prompt**: `active_at_mention_query`, `rank_context_mentions`, and `apply_context_mention` in `src/files.rs` detect an active `@<query>` token in a prompt, rank `@diff`, `@knowledge:<title>`, and `@<path>` candidates (capped at 8 items), and replace the active `@<query>` token with the chosen mention. `ui_launch_form` in `src/app/screens.rs` renders an `@ コンテキストを挿入` menu and an inline chip strip when `@` is typed. Tests: `ide_261_at_mention_query_ranking_and_insertion`.
4. **External IDE Expansion and `file:line` Jump**: `ExternalIde` in `src/util.rs` enumerates `VsCode`, `Cursor`, `Zed`, `Windsurf`, `Antigravity`, `Void`, and `Finder` (`ExternalIde::ALL`), and `external_ide_command_args` builds `["-g", "<path>:<line>"]` for VS Code-compatible editors (`code`, `cursor`, `windsurf`, `agy`, `void`), `["<path>:<line>"]` for `zed`, and `["<path>"]` when no line is given or for `Finder`. Worktree, session, and built-in editor menus offer `ExternalIde::ALL`. Tests: `ide_261_external_ide_catalog_and_line_jump_args`.
5. **Parallel Multi-Agent Arena Launch**: `parallel_arena_agents` and `arena_worktree_branch_name` in `src/util.rs` compute the deduplicated list of agents (`codex`, `claude`, `gemini`) and sanitized per-agent branch names (`<base>-<agent>`) when two or more agents are ticked in the launch sheet's `複数エージェントで並走` row. Launching with multiple agents creates an isolated worktree per agent from the mainline and starts one session per agent with the shared goal. Tests: `ide_261_parallel_arena_agents_and_branch_names`.
6. **Command Palette (`⌘K`) File and Symbol Jump**: `extract_file_symbols` in `src/files.rs` scans source text for top-level function and type definitions (Rust, TypeScript/JavaScript, Python, Go; capped at 64 symbols per file) with 1-based line numbers. `PaletteKind::File` in `src/app.rs` and `palette_entries` in `src/app/screens.rs` include matching project files and open-document symbols (capped by `PALETTE_FILE_CANDIDATE_LIMIT = 64`), and activating a `PaletteKind::File` entry opens the document in the built-in editor and sets `editor_jump_line`. Tests: `ide_261_palette_ranks_project_files_and_document_symbols`.

## Behaviour

- **Diff hunk controls**: Each `@@ ... @@` hunk row in the diff pane shows `ステージ` and `破棄` buttons on the right. Clicking `ステージ` stages only that hunk and shows `ハンクをステージしました。`. Clicking `破棄` reverses only that hunk in the working tree, refreshes the diff, and shows `ハンクの変更を破棄しました。`. If `git apply` fails (e.g. a newly untracked file with no index entry or a stale hunk), a notice `ハンクを適用できませんでした: {error}` is shown and the working tree is left intact.
- **Editor & terminal handoff**:
  - In the built-in editor header (`Edit` / `Preview` / `Diff` bar), clicking `エージェントに送る` sends `@<path>:L<start>-L<end>` plus the snippet (up to 4,000 characters) to the project's most recently launched active agent session and shows `選択中のコード文脈をエージェントに送りました。`.
  - In the session terminal `⋯` menu, clicking `直近のエラーをエージェントに送る` extracts the latest failure block from the terminal scrollback and sends it to the project's active agent session, showing `直近のエラー出力をエージェントに送りました。`. If no error marker is found in scrollback, it shows `ターミナル出力にエラー行が見つかりませんでした。`.
  - Both actions refuse with `このプロジェクトで動いているエージェントのセッションがありません。` when no active agent session exists, or `エージェントの応答が終わるまで待ってください。` when the target session is mid-turn (`AgentActivity::Working`).
- **`@` context mentions**: In the new-session sheet (`⌘N`), next to `テンプレートを挿入`, an `@ コンテキストを挿入` menu offers `@diff` (when uncommitted changes exist), verified/pinned project knowledge titles (`@knowledge:<title>`), and project files (`@<path>`). While typing `@...` in the request box, up to 8 matching chips appear directly below the box; clicking a chip replaces the `@...` token with the chosen reference.
- **External IDEs**: Worktree, session, and built-in editor menus list `VS Code`, `Cursor`, `Zed`, `Windsurf`, `Antigravity`, `Void`, and `Finder`. Opening from the built-in editor passes the active 1-based line number (`editor_jump_line.unwrap_or(1)`). If an editor binary and macOS `.app` bundle are both absent, the notice `{editor} を起動できませんでした` is shown.
- **Parallel multi-agent launch**: In the launch sheet on a Git repository, a `複数エージェントで並走（別 worktree で比較）` row lets the user tick additional available agents (`Codex`, `Claude Code`, `Antigravity`). When 2 or more agents are selected, pressing Launch creates a separate mainline worktree per agent (`<slug>-codex`, `<slug>-claude`, etc.) and launches each session with the same request, showing `{count} 件のエージェントを別々の worktree で起動しました。`.
- **`⌘K` file and symbol jump**: Typing a query in `⌘K` searches cached project files and symbols in open documents alongside actions, sessions, and projects. File rows show the relative path and project name (`ファイル · <project>`); symbol rows show `<kind> <name>` and `<path>:<line>`. Pressing `Enter` or clicking opens the file in the built-in editor and jumps to that line.

## Design

Placement: `EXTEND` across `src/git.rs`, `src/files.rs`, `src/util.rs`, `src/ui/diff.rs`, `src/app.rs`, and `src/app/screens.rs`. No new top-level module is created, and `src/store.rs` / `src/models.rs` are untouched.

- `src/git.rs`:
  - `extract_single_hunk_patch(file: &DiffFile, hunk_line_index: usize) -> Option<String>`
  - `git_stage_hunk(worktree: &Path, file: &DiffFile, hunk_line_index: usize) -> Result<()>`
  - `git_discard_hunk(worktree: &Path, file: &DiffFile, hunk_line_index: usize) -> Result<()>`
  - Uses a temporary patch file in `std::env::temp_dir()` named `operon-hunk-<uuid>.patch` (removed on return) and runs `git apply --unidiff-zero` (`--cached` or `--reverse`) via `git_command(worktree)` and `run_command_with_timeout(..., Duration::from_secs(5))`.
- `src/ui/diff.rs`:
  - `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub(crate) enum DiffHunkAction { Stage(usize, usize), Discard(usize, usize) }`
  - `diff_pane_with_actions(..., hunk_action: &mut Option<DiffHunkAction>) -> bool` draws `ステージ` and `破棄` buttons at the right end of `DiffLineKind::Hunk` rows; `diff_pane` delegates to `diff_pane_with_actions` with `&mut None` so existing callers and tests stay source-compatible.
- `src/util.rs`:
  - `ExternalIde` enum (`VsCode`, `Cursor`, `Zed`, `Windsurf`, `Antigravity`, `Void`, `Finder`) with `ALL: [Self; 7]`, `label()`, `command()`, `app_name()`, `external_ide_command_args(self, path: &Path, line: Option<usize>) -> (&'static str, Vec<String>)`, and `open_in_external_ide(path: &Path, line: Option<usize>, ide: ExternalIde) -> Result<()>`.
  - `format_code_context_for_agent(path: &Path, start_line: usize, end_line: usize, content: &str) -> String` (capped at `AGENT_CONTEXT_SNIPPET_MAX_CHARS = 4_000`).
  - `extract_recent_terminal_error(lines: &[String], max_lines: usize) -> Option<String>` (scans up to 400 trailing lines, returns at most `max_lines` and `AGENT_CONTEXT_SNIPPET_MAX_CHARS` chars).
  - `parallel_arena_agents(primary: &str, extra: [bool; 3]) -> Vec<&'static str>` and `arena_worktree_branch_name(base: &str, agent: &str) -> String`.
- `src/files.rs`:
  - `ContextMentionCandidate { token: String, label: String, detail: String }`
  - `active_at_mention_query(text: &str) -> Option<(usize, &str)>`
  - `rank_context_mentions(query: &str, files: &[PathBuf], knowledge_titles: &[String], has_diff: bool, limit: usize) -> Vec<ContextMentionCandidate>`
  - `apply_context_mention(text: &str, at_byte: usize, token: &str) -> String`
  - `CodeSymbol { name: String, kind: &'static str, path: PathBuf, line: usize }`
  - `extract_file_symbols(path: &Path, content: &str, max_symbols: usize) -> Vec<CodeSymbol>`
- `src/app.rs` & `src/app/screens.rs`:
  - `PaletteKind::File { project: Uuid, path: PathBuf, line: Option<usize> }` (rank `3` after `Project`).
  - `parallel_arena_selection: [bool; 3]` on `OperonApp` (transient UI state, default `[false; 3]`).
  - Wire hunk actions in `ui_project_changes` and `ui_document` (`EditorView::Diff`), `@` mention strip and parallel arena checkboxes in `ui_launch_form`, `エージェントに送る` in `ui_document`, and `直近のエラーをエージェントに送る` in the session terminal menu.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | Uses existing semantic palette roles (`accent`, `accent_soft`, `text`, `text_muted`, `raised`, `card`, `danger`). |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | yes | Reuses existing Phosphor icons (`ICON_FILE`, `ICON_OPEN_EXTERNAL`, `ICON_START`, `ICON_ATTENTION`). |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | no | No cross-process or persisted identifier is introduced. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | `src/store.rs` and `src/models.rs` are untouched; no persisted schema or cancellation path changes. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | yes | `git_stage_hunk`, `git_discard_hunk`, and `open_in_external_ide` invoke child processes exclusively via `git_command` + `run_command_with_timeout` in `src/exec.rs` with a 5-second timeout. Parallel arena sessions use the existing validated `build_agent_launch_command` and `create_worktree_from_mainline` paths. |
| Documentation — user-facing docs change in all three languages together | yes | `README.md`, `README.ja.md`, and `README.ko.md` are updated together, and all new UI strings are added to `src/i18n_tables.rs` in sorted order for English and Korean. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | 100% local operations; zero network calls and zero `unsafe` blocks. |
| Budgets — any new scan or output path states its byte and item ceiling | yes | Context snippets and terminal error excerpts are capped at 4,000 chars (`AGENT_CONTEXT_SNIPPET_MAX_CHARS`) and 25 lines; `@` mention candidates at 8 items; symbol extraction at 64 symbols per open file; palette file candidates at 64 items. |

## Flagged concerns

None.

## Acceptance

- `make q.fast` (`cargo fmt --check` and `cargo clippy --locked -- -D warnings`) passes.
- `cargo test --locked ide_261_` passes all six requirement tests, plus translation table and documentation tests (`every_translation_table_is_sorted_by_message_id`, `every_message_id_has_a_row_in_every_table`, `every_translation_keeps_the_placeholder_names_of_its_message_id`).
- Mutation sweep `python3 docs/sdlc/changes/261-agent-ide-loops-require-manual-steps/mutations.py` catches every mutation.

## Rejected alternatives

- Spawning external LSP servers for symbol search: Rejected because language servers add heavy background processes and per-language setup; a bounded line scanner over open documents and cached file trees is instant and zero-dependency.
- Interactive `git add -p` PTY driving for hunk staging: Rejected because driving interactive stdin is brittle; writing a single-hunk unified diff and running `git apply --cached --unidiff-zero` / `git apply --reverse --unidiff-zero` is deterministic and atomic.
