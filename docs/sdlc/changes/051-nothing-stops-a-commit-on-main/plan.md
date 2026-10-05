# Plan: worktree isolation, enforced in all three CLIs

- **Spec**: `./spec.md`
- **Approved**: 2026-09-16
- **Status**: departed from (see below)

## Files that change

Eighty-five files. Listing each one would be a listing, not a plan, so they are
grouped by what decides their content. Only the last group was written by hand;
everything above it is copied by the bundle and must not be hand-edited, because
the receipt at `.claude/.bundle-receipt.json` records a hash per file and a
local edit is what the next re-apply reports as a conflict.

| File | Change |
|---|---|
| `.cli/hooks/*.mjs` (9) | The guards. Copied. |
| `.cli/lib/*.mjs` (26) | Their shared libraries. Copied. |
| `.cli/_cli-dispatch.mjs` | The Codex/Antigravity entry point that delegates to the same guards. Copied. |
| `.claude/scripts/**` (34) | `worktree-new`, `wt-run`, and the `create-pr` operations they call. Copied. |
| `.claude/config/*.json` (4) | The policy the guards read: protected branches, section templates. Copied. |
| `.claude/skills/create-pr/**` (5) | The skill, with its references. Copied. |
| `.claude/skills/git-worktree-isolation-skills.md` | Flat catalogue stub so Antigravity, which scans `.agents/skills/*.md` flat, discovers the skill at all. Copied. |
| `.agents` | Symlink to `.claude`, so Codex finds the skills without a second copy. Created. |
| `.claude/settings.json` | Ten registrations appended after the existing six. Merged. |
| `.codex/hooks.json`, `.claude/hooks.json` | The same guards, for the other two CLIs. Generated. |
| `.claude/.bundle-receipt.json` | What was taken, at which hash, and the command that re-applies it. Generated. |
| `Makefile` | `wt.new` and `wt.run` from the bundle; `q.check` and `q.fix` by hand. |
| `.gitignore` | The bundle's block, plus `.claude/state/` by hand. |
| `AGENTS.md` | Two lines by hand. |

## Order of work

There is no compile step, so "the tree compiles between steps" does not apply.
What does apply is that the tree must not be left in a state where a guard is
registered and its file is absent, which is a CLI that errors on every tool call.

1. Copy the files. Registration is written in the same operation, so the window
   above never opens.
2. Define `q.check` and `q.fix`. The bundle derives these from a `package.json`;
   this repository is Rust and has none, so it reported the derivation as unmet
   rather than guessing, and they are written by hand as the cargo three.
3. Add `.claude/state/` to `.gitignore` — the guards write session ownership
   there, and the bundle's own ignore block does not cover it.
4. Verify by behaviour, not by reading: attempt each refusal and each permission.
5. Run the suite, fix what this change broke in it.
6. Write this change directory, obtain the two verdicts the diff requires, commit.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A merge into `.claude/settings.json` drops or reorders one of the six existing hooks | A native gate silently stops firing; commits start passing that should not | `subprocess-safety-reviewer`, which `docs/sdlc/risk.yaml` requires on this surface; and a direct diff of `HEAD:.claude/settings.json` against the staged copy |
| A guard is registered for one CLI and not the others | The rule holds in Claude and not in Codex, which is worse than not holding at all, because it is believed | Each of the three registration files read back and counted |
| A copied document names a path that does not exist here | `cargo test` fails | `harness_documents_only_name_paths_that_exist` |
| The copied skills push the harness past its byte bands | Nothing, until the harness is too large to load | `bash scripts/check-bands.sh` |
| The sync brings more than the guards | A second entry point competing with the `sdlc` skill; a session that follows the wrong one | `reference_bytes` in `bash scripts/harness-metrics.sh` — this is what actually caught it |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok`, with the pre-existing change-049
  failure noted in `spec.md` as not this change's.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `git commit` on `main` refused; the same commit under `git -C .worktrees/<b>`
  accepted.
- `git reset --hard` refused; `git restore --staged <file>` accepted.
- `make wt.new BR=feature/x DRY=1` reports the tree it would create.
- `git show HEAD:.claude/settings.json` and `git show :.claude/settings.json`
  agree on all six original registrations, in order.

## Departures from the plan

Two, and the first is the reason this document says "departed from" rather than
"done".

**Harness scoping and trimming.** The initial draft had included auxiliary
orchestration files alongside the worktree guards: 189 files. It was caught by
this repository's own instruments. `reference_bytes` went from a baseline of
11637 to 456009 against a 50000 threshold, and `steering_bytes` from 91262 to
431397. Reviewing the draft found three things the byte count pointed at: a
redundant orchestrator claiming to be an entry point, which `.claude/skills/sdlc/SKILL.md`
already is; an uncompressed debugging script, which this repository had already
cleanly scoped in `.claude/skills/root-cause/SKILL.md`; and skills written for a
src/features directory, `.tsx` files and OpenAPI schemas — none of which this
Rust immediate-mode desktop app has, which is why that path stays out of
backticks, per entry 003 in `docs/sdlc/lessons.md`. One hundred and nine
unnecessary files were removed before tracking, keeping only the worktree
isolation tooling.

**The merge moved one number nobody asked it to move.** Alongside appending its
ten registrations, the update raised the `Stop` timeout on
`.claude/hooks/gate-stop.sh` from 30 seconds to 120. Nothing requested it and
nothing recorded it; it was found by `subprocess-safety-reviewer`, on the
surface that exists for exactly this, by diffing `HEAD:.claude/settings.json`
against the staged copy. A longer budget is not a weaker gate — a hook that
times out is a hook that did not run, so 120 makes the Stop gate *more* likely
to finish, which is why nothing failed and why nothing would have. That is the
argument for restoring it rather than keeping it: an undiscussed widening that
happens to be harmless is still a gate configuration this repository did not
choose. It is back at 30. If 30 is too short for what `gate-stop.sh` now does,
that is worth its own change, with the measurement in front of it.

**`q.check` could not be derived and was not guessed.** The bundle builds it from
a `package.json`'s scripts. There is none here. It reported
`no_target_scripts_to_derive_from` rather than writing an empty gate, which is
the right failure — an empty gate that reports "passed" is worse than a missing
one — and the target was written by hand against this repository's three cargo
commands.
