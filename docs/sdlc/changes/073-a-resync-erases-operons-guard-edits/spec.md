# Spec: consolidate worktree guard configuration into dedicated policy files

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. The worktree isolation configuration is consolidated and `bundle-staleness.mjs` reports
   status as current with an empty `locally_modified` list.
2. Guard configuration options are externalized into dedicated Operon configuration files
   rather than embedded in scripts, and the classification is written down below.
3. On `main`, a Bash command that is not worktree lifecycle, read-only
   inspection, the landing merge, or a gate or release script is still refused
   (058). The list lives in `.claude/config/worktree-policy.json` as
   `trunk_bash_allowlist`, a customizable configuration file.
4. A Bash command that runs in, or names a path in, a worktree another live
   session owns is still refused (058), through `session_owner_scope: "all_bash"`
   in the same file.
5. `src/tests.rs` fails if either switch is turned off or loses its Claude Code
   wiring in `.claude/settings.json`.
6. The three gates pass, and the test count does not drop and the ignored count
   stays at six.

## Behaviour

No screen changes. What an agent sees:

- On `main`, `cargo build` is refused with the message
  `[Worktree Policy] Command not on the trunk allowlist on main: ...`, which
  points at `make wt.new BR=feature/...`. `make wt.new`, `make -C` with a
  `wt.new` target, `git merge --no-ff`, `cargo test --locked`,
  `bash scripts/check-review.sh` and the read-only commands still run.
- A command aimed at another session's live worktree is refused with the
  `[Session Owner Guard]` message.
- Three things 058 allowed on `main` are now refused, because tightening
  found them too wide: `git checkout`, a `git merge` that is
  not `--no-ff` or `--ff-only`, and `git stash drop`. So are output redirects
  into files, `GIT_*=` assignments, and a `node` script whose name merely ends
  in `ops`.
- `make wt.run` and `node .claude/scripts/wt-run.mjs` are refused on `main`.
  Both were on 058's list, but wt.run runs its command in the first active
  worktree, or in the main checkout when there is none, and neither guard can
  see that command. Found in review round 1.

## Design

Classification of local guard extensions into dedicated configurations:

| Component | Functionality | Classification | Resolution |
|---|---|---|---|
| `.claude/scripts/create-pr/ops.mjs` | no-remote base ref and cleanup (071) | Standardized | `baseRef`, `no_remote` sync status, trace-based cleanup |
| `.cli/hooks/worktree-shipping-guard.mjs` | Stop names a merged worktree left behind (071) | Standardized | `isLandedOnBase` and its landed section |
| `.cli/hooks/worktree-policy-guard.mjs` | hard-coded trunk Bash allowlist (058) | Configurable policy | patterns in `.claude/config/worktree-policy.json` |
| `.cli/hooks/worktree-session-owner-guard.mjs` | ownership check on every Bash target (058) | Configurable policy | `session_owner_scope: "all_bash"` in the same file |
| `.cli/lib/hook-registry.mjs` | Codex and Antigravity Bash registration of both guards (058) | Standardized | Consolidated registry, regenerated hook files |
| `.cli/lib/cli-adapter-utils.mjs` | Antigravity `toolCall.args.Cwd` folded into `cwd` (058) | Standardized | normalized `commandCwd`, read by commit-guard, worktree-policy-guard and session-owner |
| same | lowercase `toolCall.args.cwd` | obsolete, dropped | no captured payload uses it |
| `.cli/hooks/destructive-git-guard.mjs` | refuse `git config core.worktree` (058) | Standardized | protected-config classifier |
| `.claude/scripts/worktree-new.mjs` | `--session-id` and `SESSION_ID` owner record (058) | obsolete, dropped | lease model: tracker records owner on `wt.new`, and first edit or commit claims unowned worktree |
| `.claude/skills/create-pr/SKILL.md` | full reference paths | Standardized | Consistent relative paths |
| `.claude/skills/create-pr/references/merge-conflict-resolution.md` | Rule paths formatting | Standardized | Clean markdown formatting |

The Operon configuration remainder is the allowlist itself and the two switches, in
`.claude/config/worktree-policy.json`, and their Claude Code wiring in
`.claude/settings.json` (both guards on the Bash matcher with no `if`).

The allowlist is anchored by the guard on both sides and judged per
simple command, with review narrowing applied: the ops script is
pinned to its path, the merge to `--no-ff` and `--ff-only`, and file-writing
options of `git log`, `git diff`, `git grep`, `find`, `make` and `cargo clippy`
are excluded. It keeps `wc` and `shasum` (added on `main` after 058) and adds
`make -C` before a `wt.new` or `q.check` target, the spelling the operator's
global rules require. Thirteen patterns.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | No drawing code changes. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No glyphs change. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | The allowlist has one home, the policy file, and the new test reads it from there rather than restating it. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | The store is not touched. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | The app gains no child. The new test starts `node` through `run_command_with_timeout` with a 30 second limit and the repository pointers removed by `node_in`. |
| Documentation — user-facing docs change in all three languages together | No | No README changes; the harness documents that change are the skill files. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | The guards run locally; no `unsafe`. |
| Budgets — any new scan or output path states its byte and item ceiling | No | No new scan or output path in the app. |

## Flagged concerns

- **The allowlist is narrower than 058's** — an agent on `main` that used
  `git checkout` or a plain `git merge` is now refused. Answered: landing is a
  `git merge --no-ff` from `main`, which is allowed, and switching branches in
  the main checkout is not part of any documented procedure here. Both are too wide.
- **Codex hook commands changed** — the regenerated `.codex/hooks.json` resolves
  the dispatcher through the main checkout instead of the current worktree, so
  the command strings, and with them Codex's trust hashes, change. Answered:
  Codex asks once to trust the new commands; this is a local re-approval, not a
  behaviour change, and it is recorded as an open risk rather than worked around.
- **Antigravity worktrees start unowned** — the dropped `worktree-new.mjs` edit
  wrote an owner from `SESSION_ID` when one was set. Answered: lease management
  has the first edit or commit claim an unowned worktree, and Antigravity's
  PostToolUse payload never carried the command, so the env route was the only
  one and it depended on a variable no CLI documents.

## Acceptance

- `node .../bundle-staleness.mjs --target` this repository prints
  `current` and an empty `locally_modified`.
- `cargo fmt --check` prints nothing; `cargo test --locked` prints
  `ok. N passed; 0 failed; 6 ignored` with N one more than on `main`;
  `cargo clippy --locked -- -D warnings` prints nothing past the compile lines.
- `the_trunk_allowlist_and_the_ownership_check_are_switched_on_here` passes, and
  fails when `trunk_bash_allowlist.enabled` is false or `session_owner_scope` is
  `commit`.
- The 071 tests pass unchanged against `ops.mjs` and Stop guard.

## Rejected alternatives

- Hard-code policies directly in guard scripts: creates ongoing maintenance drift.
- Use a generic 17-pattern test list verbatim: it omits `wc` and `shasum`,
  which `main` added after 058, and keeps `git stash drop`, which nothing here
  needs on `main`.
