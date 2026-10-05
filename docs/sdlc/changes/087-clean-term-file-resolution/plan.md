# Plan: Clean Terminal File Resolution & Harness Resilience

## 1. Problem
1. **Conflated Path Resolution in OperonApp (`src/app.rs`)**:
   `resolve_session_terminal_path` coupled resolving a file path on disk with enforcing that the file is inside the project directory (`document_root_for`). When clicking a file path outside the project (e.g. `Read(~/...)`), `open_terminal_target_external` had to work around a `None` return by peeking into internal cache `self.resolved_paths` and clearing `self.notice`. Furthermore, secondary clicks (right-click to copy path or reveal in Finder) on files outside the project failed completely and posted an error notice.
2. **False Readiness Blocking (`scripts/check-readiness.sh`)**:
   `check-readiness.sh` required `intent.md` unconditionally, blocking `bugfix`, `refactor`, `trivial`, and `docs` changes that skip stages 1-2 per `docs/sdlc/routes.yaml`.
3. **Flaky Test Timeout (`src/tests.rs`)**:
   Tests running node subprocesses (`the_trunk_allowlist_and_the_ownership_check_are_switched_on_here`) used 30s timeout, causing spurious timeout failures under 620+ test parallel CPU load.
4. **Agent Merge Editor Lockup (`AGENTS.md`)**:
   `git merge --no-ff` without `--no-edit` hangs agent processes waiting for an editor.

## 2. Solution
1. **Split File vs Document Resolution**:
   - `resolve_session_terminal_file(&mut self, path: &str) -> Option<PathBuf>`: resolves existing files across session worktree, project root, home directory, caching results and blocking `..` traversals.
   - `resolve_session_terminal_path(&mut self, path: &str) -> Option<(Project, PathBuf, PathBuf, PathBuf)>`: uses `resolve_session_terminal_file`, then checks `document_root_for` and sets notice if outside project.
   - Update `open_terminal_target_external` and secondary click in `src/app/screens.rs` to use `resolve_session_terminal_file`.
2. **Fix `check-readiness.sh`**:
   - Skip stage 1-2 requirement for routes that do not have design stages (`bugfix`, `refactor`, `trivial`, `docs`) or when readiness is `pending`/`waived`/`n/a`.
3. **Stabilize Node Test Timeouts**:
   - Standardize node hook test timeouts in `src/tests.rs` to 60s matching line 32676.
4. **Clarify `git merge --no-ff --no-edit` in `AGENTS.md`**:
   - Document non-interactive merge flag.
