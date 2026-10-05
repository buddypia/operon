# Spec: Enforce worktree isolation and session ownership across all agent CLIs

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Direct execution of build, test, and modification commands on trunk branches (`main`/`master`) by an AI agent must be blocked at PreToolUse. Only worktree management, read-only inspection, and landing/packaging operations are allowed on trunk.
2. An AI agent must not execute any tool (edit, write, or shell command) targeting a worktree owned by another session.
3. Any attempt to modify `core.worktree` via `git config` must be blocked by `destructive-git-guard.mjs`.
4. Git test fixtures in `src/tests.rs` must scrub all inherited repository environment pointers (`GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`) to prevent test side effects on the parent repository configuration.
5. Worktree creation via `worktree-new.mjs` must record `.session-owner` reliably upon creation.
6. The enforcement rules must apply identically across Claude Code, Codex CLI, and Antigravity CLI.

## Behaviour

When an agent running on `main` attempts to execute development commands (e.g. `cargo build`, `cargo test`, `cargo clippy`, `touch`, `mkdir`, etc.), the command is denied with a descriptive error message explaining that development work on trunk is prohibited and directing the agent to `make wt.new BR=feature/<task>`.

When an agent attempts to edit, commit, or run commands targeting a worktree owned by another session (recorded in `.session-owner`), the operation is blocked with:
`[Session Owner Guard] Foreign-session owned worktree command blocked: <worktree-path>`
explaining that the worktree was created by another session and must be worked on by its owner.

When a command attempts `git config core.worktree`, it is denied with:
`[Destructive Git Guard] Mutating git config core.worktree corrupts repository routing and redirects the main checkout to a worktree`.

## Design

1. `.cli/hooks/worktree-policy-guard.mjs`:
   Add handler for `Bash` tool. When the working branch is trunk (`main`/`master`), match against an allowlist of trunk-safe commands:
   - Worktree lifecycle: `make wt.*`, `node .claude/scripts/worktree-new.mjs`, `git worktree`
   - Read-only inspection: `git status`, `git log`, `git diff`, `git branch`, `pwd`, `ls`, `cat`, `head`, `tail`, `grep`, `which`, `find`, `node -v`, `git --version`, `echo`
   - Landing & packaging: `git merge`, `make q.check`, `bash scripts/package-macos.sh`, `bash scripts/check-readiness.sh`, `bash scripts/check-bands.sh`
   Any other development command on trunk returns `HookOutput.deny`.

2. `.cli/hooks/worktree-session-owner-guard.mjs`:
   Extend `Bash` inspection beyond `git commit`. Extract effective worktree target from `cd <path>`, `git -C <path>`, `--manifest-path <path>`, or argument paths pointing to `.worktrees/<branch>`. Verify ownership against `.session-owner` and deny if foreign-owned.

3. `.cli/hooks/destructive-git-guard.mjs`:
   Add `git\s+config\b(?![^\n;&|]*--get)(?![^\n;&|]*--list)[^\n;&|]*core\.worktree\b` to `DESTRUCTIVE_PATTERNS`.

4. `src/tests.rs`:
   Update fixture command runner `fn command(&self, program: &str)` to explicitly call `.env_remove("GIT_DIR")`, `.env_remove("GIT_WORK_TREE")`, and `.env_remove("GIT_INDEX_FILE")`.

5. `.claude/scripts/worktree-new.mjs`:
   In `runWorktreeNew`, after successfully initializing the worktree, if a session ID is available (via environment or hook payload), write `.session-owner`.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | This change does not touch UI drawing or color definitions. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | This change does not add or alter any icons. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | Hook strings and guard matchers are scoped within `.cli/` and test harness. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | Persisted application store schema is not altered. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No new background subprocesses are added to the Operon runtime. |
| Documentation — user-facing docs change in all three languages together | No | This change governs repository developer hooks and AI isolation tooling. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | All guards and checks execute purely locally via node and bash without external calls. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | Guard executions have a 5-second timeout and operate over bounded stdin payloads. |

## Flagged concerns

- **Trunk command allowlist false positives** — A developer or agent might need to run a legitimate read-only diagnostic on trunk that is not in the allowlist. This is resolved by keeping the allowlist focused on read-only patterns (`git *` inspection, `ls`, `cat`, etc.) and providing an explicit bypass instruction via `ALLOW_MAIN_SESSION=1` for intentional human administrative commands.
- **Cross-CLI session ID identification** — Antigravity provides `conversationId` while Claude and Codex pass `session_id`. This is resolved by utilizing the existing `deriveSessionId` helper in `.cli/lib/cli-adapter-utils.mjs` which maps both canonical forms to a unified identifier.

## Acceptance

- `cargo fmt --check`, `cargo test --locked`, and `cargo clippy --locked -- -D warnings` all pass.
- Tests in `src/tests.rs` verify that git fixtures scrub inherited repository environment pointers and do not mutate root `.git/config`.
- Automated test runs of `.cli/hooks/worktree-policy-guard.mjs` confirm that development commands on trunk are denied while worktree commands are allowed.
- Automated test runs of `.cli/hooks/worktree-session-owner-guard.mjs` confirm that commands targeting foreign-owned worktrees are denied.
- Automated test runs of `.cli/hooks/destructive-git-guard.mjs` confirm that `git config core.worktree` is blocked.

## Rejected alternatives

- Block all bash commands on trunk unconditionally: Rejected because agents need to run `make wt.new` and read-only inspection commands like `git status`.
- Allow foreign-session edits with a warning: Rejected because concurrent edits corrupt working tree state and invalidate diff reviews.
