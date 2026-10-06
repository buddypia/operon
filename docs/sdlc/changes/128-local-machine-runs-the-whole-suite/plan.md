# Plan: CI runs the whole suite; the local machine waits for its verdict

- **Spec**: `./spec.md`
- **Approved**: 2026-10-06
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `scripts/ci-verified.sh` | new: did `ci.yml` pass on this commit (or an identical-tree parent) |
| `.claude/hooks/gate-merge.sh` | new: refuse a merge into `main` without CI's pass; local-suite fallback offline |
| `.claude/settings.json` | register `gate-merge.sh` on `Bash` |
| `.claude/hooks/gate-commit.sh` | drop the `cargo test --locked` step |
| `.claude/hooks/guard-bash.sh` | plain `git push [-u] origin <branch>` not asked |
| `.claude/config/worktree-policy.json` | trunk allows `git push origin main`, `gh run list/view/watch`, `bash scripts/ci-verified.sh` |
| `scripts/package-macos.sh` | default fmt/clippy/build; `--full` adds test; `--fast` removed |
| `scripts/check-release-preconditions.sh` | skip local test when tree clean and CI passed on HEAD |
| `.github/workflows/ci.yml` | push triggers for `feature/**`, `fix/**`; plain packager call |
| `Makefile` | `q.fast` |
| `src/tests.rs` | tests in spec R1–R7 |
| `AGENTS.md`, `CLAUDE.md`, `CONTRIBUTING.md`, feature-pilot and ship skills | the new flow |

## Order of work

1. Tests for R1–R7 written first and watched failing.
2. `scripts/ci-verified.sh`, then `gate-merge.sh` + settings registration.
3. `gate-commit.sh`, `package-macos.sh`, `check-release-preconditions.sh`.
4. `guard-bash.sh`, worktree-policy allowlist, `ci.yml`, `Makefile`.
5. Documents.
6. Local verify: `make q.fast` + the named tests by filter; `bash scripts/check-bands.sh`.
7. Review (route `modify`, reviewers per `scripts/check-review.sh`), commit.
8. Push the branch, `gh run watch` until CI is green, merge through the new gate,
   push `main`, clean up the worktree.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A green run on a different tree vouches for this one | broken code merges | identical-tree rule; test case with a parent whose tree differs |
| `gh` hangs | merge hook hangs to 600 s | 20 s alarm in `ci-verified.sh`; test with a sleeping fake `gh` |
| Offline machine can never merge | merges refused forever | exit 2 → local suite fallback; test case |
| The merge hook misreads a non-merge command | unrelated commands blocked | hook acts only on `git merge` without `--abort` on `main`; test cases |
| Docs still say three local gates | agent runs the suite locally anyway | documents updated; `every_document_that_names_the_gates_names_the_same_three` |
| Bands (`always_loaded_bytes`) exceeded by AGENTS/CLAUDE edits | `check-bands.sh` fails | run it; keep edits net-neutral |

## Proof of completion

- `cargo fmt --check` — no output; `cargo clippy --locked -- -D warnings` — clean.
- New tests pass by filter locally; the full `cargo test --locked` passes in CI on
  the branch push (`0 failed; 6 ignored`).
- `bash scripts/check-bands.sh` — `bands: N metrics within their bands`.
- This change's own merge is admitted by `gate-merge.sh` on CI's pass.

## Departures from the plan

- **Renumbered 127 → 128.** Another session landed `127-two-sidebars-on-one-side`
  on `main` while this was in flight. The review gate refuses a commit spanning
  two change directories, so the unpushed planning commit was folded with
  `git reset --soft` and recommitted under 128; `main` was then merged into the
  branch (the step this change documents), since `rebase` is refused.
- **128 collided too.** `fix/128-landed-without-review` reached `main` during
  review. This one kept 128: renumbering again would mean a commit spanning two
  change directories, which the review gate refuses, and the slug tells the two
  apart. `main` was merged into the branch again before the last review round.
- **Review rounds 1–3 reshaped the merge gate.** It no longer reads arbitrary
  merge spellings: on `main` only `git merge --no-ff|--ff-only [--no-edit]
  BRANCH` as the whole command is judged and every other spelling is refused;
  merges are recognised by git's subcommand, not by the word. Both timeouts go
  through `scripts/run-bounded.sh`, and `ci-verified.sh --worktree` became the
  release check's single condition so untracked files count.
- **The timeout kills a process group, not a process.** The first
  `perl -e 'alarm …; exec …'` ended `gh` but not a child of it, which held the
  pipe open; `ci_verified_tells_passed_failed_absent_and_unreachable_apart`
  failed at 10 s against a 1 s limit. Now perl forks, `setpgrp`s the child and
  signals the group.
- **`always_loaded_bytes`.** The first wording of the AGENTS.md / CLAUDE.md
  edits took the reading from 12970 to 13613, past the 13000 diagnose tier; the
  landing steps were left to the feature-pilot skill and the two files pointed
  there instead (12993).
- **Guards watched failing.** The timeout guard failed for real (above). The
  others are argued from `main`'s text, where each fails: `gate-commit.sh` there
  still runs `$(cargo test`, `ci.yml` has no `feature/**`, `guard-bash.sh` asks
  for every push, and the release check names no `ci-verified.sh`.
