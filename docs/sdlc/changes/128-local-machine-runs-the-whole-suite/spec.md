# Spec: CI runs the whole suite; the local machine waits for its verdict

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. **One question, one script.** `scripts/ci-verified.sh`, given a commit, answers
   whether the `CI` workflow (`.github/workflows/ci.yml`) completed with
   `success` on that commit, or on a parent of it whose tree is identical (a
   `--no-ff` merge of a branch that `main` had not moved past). Exit `0` verified,
   `1` not verified (failed, still running, or never ran — the reason on stdout),
   `2` CI could not be asked (`gh` missing, not logged in, network down, no
   GitHub remote). `gh` runs under a timeout.
   Test: `ci_verified_tells_passed_failed_absent_and_unreachable_apart`.
2. **The merge waits for CI.** `.claude/hooks/gate-merge.sh`, a PreToolUse(Bash)
   hook, refuses `git merge` of a branch into `main` unless `ci-verified.sh`
   exits `0` for the branch head. On `1` it blocks with the reason and what to do
   (push the branch, `gh run watch`). On `2` it falls back to running
   `cargo test --locked` in the branch's worktree and allows the merge only if it
   passes. `git merge --abort` and merges on other branches pass through.
   Test: `the_merge_gate_waits_for_ci_and_falls_back_to_the_local_suite`.
3. **The commit gate stops running the suite.** `.claude/hooks/gate-commit.sh`
   runs `cargo fmt --check` and `cargo clippy --locked -- -D warnings`, and keeps
   the test-erosion count; `cargo test --locked` moves to CI and the merge gate.
   Test: `the_commit_gate_leaves_the_suite_to_ci_and_the_merge_gate`, and
   `the_commit_gate_runs_the_gates_in_the_tree_the_commit_lands_in` updated to
   tell the trees apart by clippy's refusal.
4. **Packaging stops running the suite by default.** `scripts/package-macos.sh`
   runs fmt, clippy, build; `--full` or `OPERON_FULL_PACKAGE=1` adds
   `cargo test --locked` between fmt and clippy. `--fast`/`OPERON_FAST_PACKAGE`
   (change 126) is removed — it is the default now.
   Tests: `the_packager_runs_the_three_gates_before_it_builds` (with `--full`),
   `the_packager_leaves_the_suite_to_ci_by_default` (replacing
   `the_packager_fast_mode_skips_tests_when_requested`).
5. **The release check accepts CI's verdict.** In
   `scripts/check-release-preconditions.sh`, the `cargo test --locked` gate is
   skipped when the working tree is clean and `ci-verified.sh HEAD` exits `0`;
   otherwise it runs locally as before. Its final message says which.
   Test: `the_release_check_takes_cis_verdict_only_for_a_clean_tree`.
6. **CI runs on branch pushes.** `ci.yml` triggers on pushes to `main`,
   `feature/**`, `fix/**`, and pull requests; the package job calls the packager
   plainly. Test: `ci_runs_on_the_branches_the_merge_gate_asks_about`.
7. **Pushing is part of landing.** `.claude/hooks/guard-bash.sh` lets a plain
   `git push [-u] origin BRANCH` (no force, no refspec tricks, one simple
   command) through without asking; any other push still asks. The trunk
   allowlist admits `git push origin main`, `gh run list|view|watch`, and
   `bash scripts/ci-verified.sh`. Test: `a_plain_push_to_origin_is_not_asked_about`.
8. **Local verify is the fast set.** `make q.fast` runs fmt and clippy;
   feature-pilot's Verify step runs `make q.fast` plus `cargo test --locked
   FILTER` for the tests the change names. `make q.check` (the three) stays.
9. **Documents say the new flow.** `AGENTS.md`, `CLAUDE.md`,
   `.claude/skills/feature-pilot/SKILL.md`, `.claude/skills/ship/SKILL.md`,
   `CONTRIBUTING.md` describe: commit (fmt+clippy) → push branch → CI → merge
   (gate) → push `main` → package. Held by
   `every_document_that_names_the_gates_names_the_same_three` (the three still
   named) and the existing path checks.

## Behaviour

Developer tooling only; nothing in the app changes. Messages are English, as
the other hooks' are.

- Branch CI green → merge proceeds, hook note: `Merge gate: CI passed on SHA.`
- CI failed / running / absent → merge blocked: `Merge gate: CI has not passed on
  BRANCH (SHA): REASON. Push the branch (git push -u origin BRANCH) and
  wait for it (gh run watch), then merge.`
- CI unreachable → note `CI could not be asked (WHY); running cargo test
  --locked in WORKTREE` then allow or block on the local result. No worktree
  for the branch → blocked, saying so.
- Release check, clean tree + CI green → `cargo test: passed in CI on SHA`.

## Design

Verdict: `EXTEND` — the gates in `.claude/hooks/` and `scripts/` grow; no Rust
module changes. `src/tests.rs` only.

- `scripts/ci-verified.sh`: `gh run list --workflow ci.yml --commit SHA
  --json status,conclusion --limit 20`, under `timeout`/`perl -e alarm` 20 s;
  `gh` resolved from `PATH` so the tests can put a fake one first. Candidates:
  the commit, then each parent with `git rev-parse P^{tree}` equal to the
  commit's tree. Any candidate with a `completed/success` run → 0. Any `gh`
  failure → 2.
- `.claude/hooks/gate-merge.sh`: reads `tool_input.command` and `cwd`; acts only
  when the command contains `git merge` without `--abort`, and the repository at
  `cwd` has `main` checked out. Branch = last non-option word of the merge.
  Worktree for the fallback from `git worktree list --porcelain`. Registered in
  `.claude/settings.json` on `Bash`, timeout 600.
- No persisted shape, no dependency, no `unsafe`.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | no | no UI change |
| Icons | no | no UI change |
| Identifier SSOT | yes | the workflow name `ci.yml` is named once in `scripts/ci-verified.sh`; the hook and release check call the script rather than repeating the query |
| Durability | no | store, migration, cancellation untouched; packaging lock (fd 9) unchanged |
| Subprocess safety | yes | shell gates, not app code; `gh` bounded by a 20 s timeout, the fallback suite by the hook's 600 s timeout, output tailed |
| Documentation | no | developer documents only (English); no user-facing docs |
| Local-first | yes | the app makes no new network call; the network is used by developer tooling only, and offline the gates fall back to the local suite |
| Budgets | yes | `gh` output limited by `--limit 20`; refusal output tailed to 60 lines as gate-commit does |

## Flagged concerns

- **Gate relaxation (paused surface `gate-configuration`)**: the commit gate and
  packager no longer run the suite. Approved by the person 2026-10-06 (landing
  flow question); the protection moves to the merge, where it covers every
  commit that reaches `main`.
- **Trust in CI's verdict**: a green run on another tree must not vouch for this
  one — hence the identical-tree rule, not "branch name had a green run".

## Acceptance

- `cargo fmt --check` — no output; `cargo clippy --locked -- -D warnings` — clean.
- `cargo test --locked` — `0 failed; 6 ignored`, run in CI on the branch push.
- The new and updated tests named above pass locally by filter.
- `bash scripts/check-bands.sh` — within bands.
- The branch of this change itself lands through the new merge gate.

## Rejected alternatives

- **GitHub pull requests** — the person chose local merge.
- **Merge then let CI check `main` afterwards** — a broken commit could land.
- **Checking CI by branch name** — a green run on an earlier push would vouch
  for later commits.
- **Moving reviews to CI** — the person chose to keep them local.
