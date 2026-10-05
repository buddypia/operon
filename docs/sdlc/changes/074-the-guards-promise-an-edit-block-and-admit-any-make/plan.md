# Plan: update worktree guards and narrow the trunk make pattern

- **Spec**: `./spec.md`
- **Approved**: 2026-09-24
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `.claude/scripts/worktree-new.mjs` | overwritten: default base from create-pr `base_branch` |
| `.cli/hooks/worktree-owner-tracker.mjs` | overwritten: `formatLockNotice` asks `isEditLockEnforced` |
| `.cli/lib/test-lock.mjs` | overwritten: `isEditLockEnforced` |
| `.claude/.bundle-receipt.json` | source commit 41bb180f, three new hashes |
| `.claude/config/worktree-policy.json` | the make pattern becomes an end-anchored grammar |
| `src/tests.rs` | the 073 test gains four refused make commands and three allowed ones |

## Order of work

1. Apply the guard updates into the worktree. The three customizable files are kept.
2. Rewrite the make pattern, then probe it with the commands this repository
   runs on `main` and with a set it must refuse.
3. Extend the test, and watch it fail with the 073 pattern restored.
4. Run the three gates, the band check and the staleness check.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The pattern refuses a documented `make` spelling | a landing session refused on `main` | the probe in step 2; the test pins `make wt.new BR=`, `make -C <dir> wt.new BR=`, `make q.check` |
| The pattern loosens again | make runs any command on `main` | the extended test |
| `detectDefaultBase` misreads Operon's config | `wt.new` branches from the wrong base or fails | smoke run: it resolves `main` |

## Proof of completion

- `cargo fmt --check` prints nothing.
- `cargo test --locked` prints `test result: ok. 562 passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` prints nothing past the compile lines.
- The extended test fails with the 073 make pattern restored.
- The staleness check prints `current` with an empty `locally_modified`.

## Departures from the plan

- Review round 1 (rust-reviewer, approve-with-nits 0 3), all three taken:
  - Nit 1: the end anchor refused `make q.check 2>&1` and `>/dev/null`, which
    073 admitted. The grammar now ends with fd and `/dev/null` redirects, and
    the test pins `make q.check 2>&1` as allowed.
  - Nit 2: the spec's env-prefix limit now names `MAKEFILES` and the other make
    flag variables.
  - Nit 3: the allowed `-C` case uses the fixed `/tmp/operon` instead of the
    temp fixture path, which a `TMPDIR` with a space or `+` would break.
