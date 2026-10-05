# Spec: update worktree guards and narrow the trunk make pattern

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. The worktree guards are updated, and `bundle-staleness.mjs` reports them
   `current` with an empty `locally_modified` list.
2. On `main`, the make pattern in `trunk_bash_allowlist` admits only `make`,
   then an optional `-C <dir>`, then either `wt.new` with any of `BR=`, `BASE=`
   and `DRY=` (the variables the Makefile reads) or `q.check`, then nothing
   but redirects to a file descriptor or `/dev/null`. Every value is limited
   to `[\w./-]+`.
3. `the_trunk_allowlist_and_the_ownership_check_are_switched_on_here` fails if
   `make ... --eval=` or `make ... SHELL=` is admitted, and still passes
   `make wt.new BR=...`, `make -C <dir> wt.new BR=...`, `make q.check` and
   `make q.check 2>&1`.
4. The three gates pass. The test count does not drop, and the ignored count
   stays at six.

## Behaviour

No screen changes. What an agent sees:

- After a `fix/*` commit that adds a test, the `[test-lock]` notice says the
  commit will be refused, and that editing is not blocked in this repository,
  because `isEditLockEnforced` finds no `coverage-threshold-guard`.
- `make wt.new` with no `--base` branches from create-pr `base_branch`
  (`main` here), as before in practice. A bad configured value now fails
  instead of silently picking another branch.
- On `main`, `make wt.new BR=feature/x --eval=...`, `make --eval=... q.check`,
  `make wt.new BR=x SHELL=/tmp/sh`, `make wt.new BR=x -j4`,
  `make wt.new BR=x q.check` and `make wt.new BR=$(...)` are refused with the
  `[Worktree Policy] Command not on the trunk allowlist on main`
  message. `make wt.new BR=...`, `make -C <dir> wt.new BR=... BASE=main DRY=1`
  `make q.check` and `make q.check 2>&1 | tail -20` still run.

## Design

The update changes only the three files touched and their receipt
hashes. The customizable files (the policy, the create-pr config, the pre-ship
panel sections) are kept, the hook files regenerate unchanged, and settings
wiring is unchanged.

The make pattern goes from a negative lookahead on `-f|--file|--makefile` to a
positive, end-anchored grammar:

```
make\s+(?:-C\s+[\w./-]+\s+)?(?:wt\.new(?:\s+(?:BR|BASE|DRY)=[\w./-]+)*|q\.check)(?:\s+\d*>\s*(?:&\d+|/dev/null))*\s*$
```

The guard judges one simple command at a time, so `\s*$` ends at the command,
and `&&`, `;` and `|` are judged separately as before. A redirect to a file
descriptor or `/dev/null` stays in the words the guard matches (it refuses
redirects into files itself), so the grammar names those at the end. Without
them, `make q.check 2>&1` would have been refused where 073 admitted it
(review round 1, nit 1).

Known limits, left open:

- The guard strips env assignments before it matches (`worktree-policy-guard.mjs`
  header), and refuses only `GIT_*` and `CDPATH`. So `MAKEFLAGS=--eval=... make
  wt.new BR=x`, `MAKEFILES=/tmp/x.mk make q.check` and `env MAKEFLAGS=x make
  q.check` are still admitted: GNU make reads flags from `MAKEFLAGS`, `MFLAGS`
  and `GNUMAKEFLAGS`, and extra makefiles from `MAKEFILES`. A `SHELL=` prefix is
  harmless, because GNU make on POSIX does not take `SHELL` from the
  environment. A future hardening could address `LOCATION_ENV_RE` or a sibling
  regex covering `MAKEFLAGS|MFLAGS|GNUMAKEFLAGS|MAKEFILES`. This change does not
  patch the managed file. (`PATH=` and `RUSTC_WRAPPER=` prefixes pass as well,
  but they reach the allowlisted `cargo test --locked` just the same, so they
  are not specific to make.)
- `-C` still accepts any directory made of safe characters, so
  `make -C /tmp/elsewhere wt.new` runs that Makefile's `wt.new`. The pattern
  cannot name this checkout without hard-coding a machine path.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | No | No drawing code changes. |
| Icons | No | No glyphs change. |
| Identifier SSOT | No | The allowlist has one home, the policy file. The test reads it from there. |
| Durability | No | The store is not touched. |
| Subprocess safety | Yes | The app gains no child. The extended test uses the existing `run_command_with_timeout` call, which has a 30 second limit. |
| Documentation | No | No README changes. |
| Local-first | Yes | The guards and the sync run locally, and there is no new `unsafe`. |
| Budgets | No | No new scan or output path in the app. |

## Flagged concerns

- **`make wt.new` with extra flags is now refused on `main`**, for example
  `-j4` or `--dry-run`. Answered: no documented procedure here uses them, and
  `DRY=1` is the Makefile's own dry-run switch.

## Acceptance

- `bundle-staleness.mjs --target` this repository prints `current` and an
  empty `locally_modified`.
- `cargo fmt --check` prints nothing. `cargo test --locked` prints
  `ok. 562 passed; 0 failed; 6 ignored`. `cargo clippy --locked -- -D warnings`
  prints nothing past the compile lines.
- The extended test fails when the 073 make pattern is restored.

## Rejected alternatives

- Keep the negative lookahead and add `--eval` and `SHELL=` to it: make has more
  command-running options (`-E`, `MAKEFLAGS=`, `.SHELLFLAGS=` and any variable a
  recipe expands), so a deny list stays open.
- Patch `LOCATION_ENV_RE` in the worktree guard to catch `MAKEFLAGS=`: that
  edits a generated file, and the next update would erase it (the problem
  073 fixed).
