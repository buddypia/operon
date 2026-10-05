# Plan: a worktree starts from the mainline, and a taken name gets the next one

- **Spec**: `./spec.md`
- **Approved**: 2026-09-05
- **Status**: in progress

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | `WORKTREE_NAME_MAX_ATTEMPTS`. |
| `src/git.rs` | `WorktreeBase`; `detect_worktree_base`; `worktree_name_candidate`; `is_worktree_name_available`; `configure_created_worktree`. |
| `src/app.rs` | `create_worktree` resolves the base and walks candidates; `BackgroundResult::WorktreeCreated` carries the branch and the base display; the notice names both. |
| `src/i18n_tables.rs` | EN and KO rows for the new messages, in sorted position. |
| `src/tests.rs` | The eight tests named in the spec's Acceptance. |
| `README.md`, `README.ja.md`, `README.ko.md` | One bullet each. |

## Order of work

1. `src/config.rs` ceiling and `src/git.rs` pure helpers (`worktree_name_candidate`) with their tests. Compiles; green.
2. `src/git.rs` git-touching helpers, tested against real repositories built in a temporary directory by `git init` — a fixture repository with an `origin` remote whose `origin/HEAD` is set, one without it, one with neither.
3. `src/app.rs` call site and the result shape; `src/i18n_tables.rs` rows. Compiles; green; clippy clean.
4. The three READMEs.
5. Gates, bands, a manual creation in the running app, commit.

The tree compiles between every step.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| `origin/HEAD` exists but dangles (the remote's default branch was renamed) | Creation fails on a repository that used to work | The symref target is verified with `rev-parse --verify` before it is used; detection falls through to the probes. Test: a repository whose `origin/HEAD` names a deleted ref. |
| The candidate loop hides a real failure by trying the next name | A person asks for `fix` and silently gets `fix-7` | Availability is three positive checks, not "the add failed"; a failing `git worktree add` is still reported, never retried. The confirmation names the branch. |
| `push.autoSetupRemote` is clobbered for someone who set it to false on purpose | Their first push behaves differently than they configured | It is read with `git config --get push.autoSetupRemote` at every scope first and only written when that produces nothing. Test: a repository with it set to `false` is left at `false`. |
| A tag named `main` wins over the branch | The worktree starts from a tag | The base is passed as `refs/heads/main`, never `main`. Test: a repository holding both, asserting the resulting commit. |
| The extra git calls make the button feel slow | A visible pause on a large repository | The calls are `rev-parse` and `config` against local refs, inside the background task that already existed; nothing is added to a draw path. |
| `subprocess_sites` band moves | `scripts/check-bands.sh` warns | The new calls are counted; if the band moves it is recorded rather than worked around. |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `ok`, eight tests higher than before, `6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-bands.sh` — no new breach beyond what is recorded.
- Each new test watched failing by mutation: the fallback list truncated; the
  candidate arithmetic off by one; the qualification stripped; the
  already-set check removed.
- In the running app: from a project sitting on a feature branch, a new worktree
  is at `origin/main`'s tip and says so; the same name again yields `-2`.

## Departures from the plan

- **The whole creation moved into `src/git.rs`.** The plan had `src/app.rs`
  resolving the base and walking the candidates inside its background closure.
  It became one function, `create_worktree_from_mainline`, and the closure in
  `src/app.rs` shrank to a call. Two reasons: the loop is the part worth
  testing and it is testable directly this way rather than through an
  application; and `src/app.rs` is over its size warn band, so
  `largest_module_lines` fell from 8862 to 8837 across this change instead of
  rising.
- **`BackgroundResult::WorktreeCreated` lost its `destination` field.** It was
  computed before the work started, which stops being true once the name can
  change. The destination now arrives inside `CreatedWorktree` beside the branch
  and the base, so the three things the confirmation names cannot disagree.
- **Three `must_use` warnings from change 017 were fixed in passing.**
  `cargo test` printed them on every run. They are in test code, so the
  documented clippy gate never saw them; `--all-targets` does, and a warning the
  suite prints every run is a warning people stop reading.
