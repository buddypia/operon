# Intent: Agent IDE loops require manual steps for hunk control, context handoff, file/symbol jump, and multi-agent runs

- **Status**: approved
- **Opened**: 2026-11-09

## Problem

When supervising coding agents across worktrees and built-in views, six high-frequency IDE and AI-native loop steps still require manual terminal commands or copy-pasting:
1. Hunk-level diff control: The diff review pane only allows staging whole files or leaving notes; discarding an unwanted hunk or staging one hunk at a time requires leaving the app to run interactive Git commands.
2. Sending code context or terminal errors to an agent: While diff comments can be batched to an agent, there is no one-click action to send an open editor file/line snippet or the most recent terminal build/test failure output directly to the project's active agent session.
3. `@` context mentions in prompts: Writing a session request that references project files, uncommitted diffs, or verified project knowledge requires typing paths by hand instead of picking `@` mention candidates inline.
4. External IDE variety and line jump: Opening a project or file in newer AI-native editors (Windsurf, Antigravity, Void) is not offered, and opening from the built-in editor does not jump to the current `file:line`.
5. Multi-agent parallel comparison ("Best-of-N"): Running the same request across two or three agent CLIs in separate isolated worktrees to compare their diffs and land the best result requires repeating the launch workflow manually for each agent.
6. Project file and symbol jump in the command palette: `⌘K` searches actions, sessions, and projects, but cannot jump directly to a repository file or a function/type symbol inside open documents.

## Who feels it, and when

- Developers reviewing an agent's multi-hunk diff who want to discard one stray hunk or stage a clean hunk without reverting the entire file.
- Developers inspecting a file in the built-in editor or watching a test fail in the terminal pane who want to hand that exact snippet or failure block to the idle agent in one click.
- Developers composing a new session request who want to attach `@diff`, `@knowledge:...`, or `@path/to/file` from an inline completion strip, or launch two agents in parallel worktrees to compare solutions.
- Developers pressing `⌘K` to jump straight to a project file or symbol definition, or jumping from the built-in editor into Windsurf, Antigravity, Void, Cursor, VS Code, or Zed at the current line.

## Desired outcome

1. Each hunk header row in the diff pane offers one-click **ステージ** (Stage hunk) and **破棄** (Discard hunk) actions that apply only that hunk and refresh the diff.
2. The built-in editor header offers **エージェントに送る** to send the active file and line snippet to the project's idle agent session, and the session terminal menu offers **直近のエラーをエージェントに送る** to extract and send the latest failure output block.
3. Typing `@` in the launch request field (or using the `@ コンテキストを挿入` menu) surfaces matching `@diff`, `@knowledge:<title>`, and `@<path>` chips that insert into the request on click.
4. External IDE menus include VS Code, Cursor, Zed, Windsurf, Antigravity, Void, and Finder, and opening from the built-in editor passes the current `file:line` jump arguments.
5. The launch sheet allows selecting multiple agents to run the same request in parallel isolated worktrees (`<branch>-<agent>`) in one action.
6. Typing in `⌘K` also matches files in the active project and code symbols in open documents, opening the chosen file at its line in the built-in editor.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese (with English and Korean rows in the translation tables); code, comments, and docs are English.
- The persisted store shape does not change.

## Systems likely affected

- `src/git.rs` (single-hunk patch extraction, hunk stage and discard)
- `src/files.rs` (`@` context mention parsing/ranking, lightweight code symbol extraction)
- `src/util.rs` (external IDE catalog with `file:line` jump arguments, code context and terminal error extractors)
- `src/ui/diff.rs` (hunk-row stage and discard controls)
- `src/app.rs` and `src/app/screens.rs` (command palette file/symbol entries, editor & terminal send-to-agent actions, `@` mention strip, parallel multi-agent worktree launch)
- `src/i18n_tables.rs` and `README.md` / `README.ja.md` / `README.ko.md`

## Open questions

None. The user selected all six capabilities in discovery (`questions: 1 7`).

## Not in scope

- Cloud-hosted agent execution or remote LSP daemons.
- Persisting new fields in `store-v2.json`.
