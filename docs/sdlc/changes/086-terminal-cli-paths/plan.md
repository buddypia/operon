# Plan: Fix Terminal CLI Tool Calls and Tilde Path Opening

## 1. Problem
When clicking file paths in terminal output like `Read(~/dev/buddypia/example-webshop/.tmp/test-output.txt)`:
- `~` is not expanded, so path resolution fails completely.
- Tool call wrappers `ToolName(path)` across CLIs (Antigravity, Claude Code, Codex) are not recognized as unified clickable targets.
- `file:///` URLs are ignored.
- External opening via macOS `open` should open valid files on disk.

## 2. Solution
1. In `src/ui/terminal.rs`:
   - Add home directory expansion for paths starting with `~/` or equal to `~`.
   - Add support for `file:///` URLs (stripping prefix and resolving as path).
   - In `terminal_targets`, recognize tool invocations `ToolName(path)` (e.g. `Read`, `Write`, `Edit`, `View`, etc.) so clicking anywhere on `ToolName(...)` targets the path.
   - Resolve absolute and home-expanded paths properly.
2. In `src/app.rs`:
   - In `open_terminal_target_external`, if the path resolves to an existing file on disk, launch it via `open_path(&resolved)` without refusing files outside project boundary.
3. In `src/tests.rs`:
   - Add comprehensive tests for `~/...` tilde path resolution.
   - Add tests for `Read(...)`, `Edit(...)`, `Write(...)`, `View(...)` tool call recognition.
   - Add tests for `file:///` path detection.
   - Verify external opening logic.
