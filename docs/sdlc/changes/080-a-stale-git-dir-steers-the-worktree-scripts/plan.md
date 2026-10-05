# Plan: the worktree scripts find their repository from the directory they are given

- **Spec**: none — bugfix route; the bug report in `./state.yaml` is the intent.
- **Approved**: 2026-09-25
- **Status**: done — departed from (see below)

## Named cause

The worktree tooling spawns git with the parent's environment, and git reads
`GIT_DIR` / `GIT_WORK_TREE` before the working directory it was handed. A
session that carries them — EnterWorktree exports both — therefore steers
every git these scripts start at the session's repository:

- `.claude/scripts/create-pr/ops.mjs` — `execOnce` (every `git(...)`), and
  `resolveProjectRoot`. `cmdCleanupWorktree` reads the target's branch with
  `git rev-parse --abbrev-ref HEAD` in the target's directory, and gets the
  session's branch instead: `main`.
- `.claude/scripts/worktree-new.mjs` — `defaultGitFn`, and `defaultNodeFn`,
  whose child (`worktree-init.mjs`) spawns git too. With a pointer at a
  removed worktree, `localBranchExists('main')` is false and it refuses.
- `.cli/hooks/worktree-policy-guard.mjs` — `queryBranch` → `safeGit` →
  `safeExec` in `.cli/lib/utils.mjs`. A dangling pointer makes the branch
  `null`, which fails closed, so every non-allowlisted command is denied.

## Files that change

| File | Change |
|---|---|
| `.cli/lib/utils.mjs` | `INHERITED_REPOSITORY_POINTERS` and `withoutInheritedRepository(env)`; `safeExec` spawns with it |
| `.claude/scripts/create-pr/ops.mjs` | `execOnce`, `resolveProjectRoot`, `runFollowupDebtRegister` spawn with it |
| `.claude/scripts/worktree-new.mjs` | `defaultGitFn`, `defaultNodeFn` spawn with it |
| `src/tests.rs` | two tests, below, and a check that the JS list equals `INHERITED_REPOSITORY_POINTERS` in `src/config.rs` |
| `docs/sdlc/lessons.md` | entry, with its Guard |

The helper lives in `utils.mjs` because the guard and `ops.mjs` already import
it and `worktree-new.mjs` already imports from `.cli/lib/`: one list, not three.
It names the same two variables as the Rust list — the ones a session actually
exports — and the test pins the two lists together so they cannot drift.

## Order of work

1. Commit this plan and `state.yaml`.
2. Tests in `src/tests.rs`, each driving the real scripts with the pointers
   set (the existing `node_in` scrubs them, which is why no test saw this):
   - `cleanup_removes_the_worktree_it_was_given_when_the_session_names_another`
     — pointers at the fixture's main checkout; `cleanup-worktree` on a merged
     worktree must delete *its* branch and leave `main`.
   - `a_git_dir_that_no_longer_exists_does_not_stop_the_worktree_tools` —
     pointers at a removed worktree's gitdir; `runWorktreeNew` (dry run) must
     find `main`, and the policy guard on a feature-branch checkout must let
     `cargo build` through rather than failing closed.
   Run them against the current scripts and watch both fail for the named cause.
3. The scrub, in the three places above. Tests go green.
4. Mutation: remove each call site's scrub in turn; a test must go red each time.
5. Gates, bands, lesson, review (`craft` = rust-reviewer, widened by
   `scripts/check-review.sh`).

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A caller relied on an inherited `GIT_DIR` to reach a repository its cwd is not in | a script that worked from outside a repo now says "not a git repository" | every caller in the three files passes a `cwd` inside the repository; the existing worktree tests run them |
| `safeExec` runs non-git commands too | a hook's child loses `GIT_DIR` | intended: the same pointers would mislead any git that child starts; git's own hooks do not run this code (`.githooks/` has no Node) |
| Other scripts (`wt-run.mjs`, `worktree-init.mjs`, …) still inherit | the same failure from another entry point | out of scope, named in the lesson |
| `worktree-init.mjs` writes `GIT_DIR` into a new worktree's `settings.local.json` | a session opened in that worktree exports it | not changed here: that pointer is correct for that session; it only goes wrong once the tree is removed, which the scrub now survives |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- Both new tests fail on the unmodified scripts with the messages that name
  the cause, and pass after.

## Departures from the plan

- **The first draft of the tests passed on the broken scripts.** They set the
  pointers with `Command::env`, and `run_command_with_output_limit` in
  `src/exec.rs` removes both from every command it spawns, so node never saw
  them. `node_steered_at` now writes them to a file under the fixture's `.git`
  and hands it to node as `--env-file`, which node loads into its own
  environment — the place its children inherit from. Watched failing only
  after that.
- **Not guarded by a test**, as the plan's risk table implied they would be:
  `resolveProjectRoot` (the tests set `CLAUDE_PROJECT_DIR`, which it prefers),
  `defaultNodeFn`, and `runFollowupDebtRegister`. The scrub is there; nothing
  turns red without it. Named in lesson 049.
