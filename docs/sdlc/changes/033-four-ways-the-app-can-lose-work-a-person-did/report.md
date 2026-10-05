# Bugfix: four ways this application can destroy work a person did

- **Route**: `bugfix` — no intent and no spec by design. The review is the bug
  report, and the reproductions below are the intent.
- **Skill**: `.claude/skills/root-cause/SKILL.md`
- **Source**: the stage-5 review of changes 018–026 by `rust-reviewer` ×3,
  `subprocess-safety-reviewer`, and `durability-reviewer`. All five returned
  *do not approve*; sixteen Important findings. This change takes the four that
  lose work somebody did. The other twelve are listed in `./remaining.md`.

Findings A and B were reported independently by three of the five reviewers.

## A — two background keys guard one git index

**Cause.** `record_staged_change` claims
`BackgroundKey::GitMutation(project)`; `draft_commit_message` claims `BackgroundKey::CommitMessage(project)`. Both call
`git_stage_exactly`, whose first move is `git reset --quiet -- .`.
`spawn_background` fences per key, so the two run at once against one index, and
neither button is disabled by the other's task.

**Reproduction.** Tick three files, press 「N ファイルをコミット」. The commit
task finishes `git add` and enters `git commit`, which runs this repository's own
pre-commit hook — `cargo test`, minutes. The draft button is still enabled, so
press 「AI に書かせる」. Its `git reset --quiet -- .` empties the index. `git
commit` re-reads the index after the hook and records an empty tree, or fails
with `no changes added to commit`. The staging a person chose is gone either
way. The reverse order — pressing commit during the draft's 120-second CLI run —
writes the same index twice.

**Fix.** One question, asked in one place: `git_worktree_busy` names every key
that writes an index, and both actions and both buttons read it. A guard that
restated the key list at each call site is the shape `.claude/rules/identifiers.md`
forbids.

## B — `git reset`'s exit status is discarded

**Cause.** The reset inside `git_stage_exactly` ran through
`run_command_with_timeout(&mut reset, …)?`. The
`?` catches a spawn failure, a timeout, and an output overrun. A non-zero exit
is dropped on the floor. The `git add` eleven lines below checks
`output.status.success()`, so the two halves of one function disagree.

**Reproduction.** An agent running in the same worktree holds `.git/index.lock`
for a moment — a plain `git status` refreshing the index is enough. `git reset`
exits 128 with `Unable to create '…/index.lock': File exists` and is ignored.
Tens of milliseconds later the lock is gone and `git add` succeeds. A file that
was staged by hand and then unticked rides into the commit — which is the exact
invariant this function's own doc comment says the reset exists to enforce.

**Fix.** Check the status and carry the stderr, the same way the `add` does.

## C — a hunk selection made outside this application is destroyed silently

**Cause.** `git_stage_exactly` stages whole files. A person who ran `git add -p`
in the terminal pane and staged one hunk of three has a *partially* staged file:
content in the index that is neither `HEAD` nor the working tree. The reset
drops it and the `add` stages all three hunks.

**Reproduction.** `git add -p src/app.rs` in the terminal pane, stage hunk 1 of
3. In the Changes view that file is ticked, because a tracked modification is
ticked by default. Press either 「コミット」 *or* 「AI に書かせる」 — the draft
alone is enough, no commit needed. The two hunks that were deliberately left out
are now in the commit. Recovery for the commit is `git reset --soft HEAD~1`;
recovery for the hunk selection is nothing — the index tree is never written to
the object store, so redoing it by hand is the only route.

**Fix.** Refuse, and say which files. A file is partially staged when it appears
in both `git diff --cached --name-only` and `git diff --name-only`; Operon's own
staging never produces that state, because it stages whole files. So the refusal
cannot fire on Operon's own leftovers — which is what rules out the naive check
of "is anything staged", a check that would refuse forever after one failed
commit. Documented in all three READMEs, because an application that rewrites
your index should say so before you find out.

## E — an unreadable notes file is silently replaced with an empty one

**Cause.** `load_diff_comments` ended in `.ok()`, so a
file that cannot be parsed becomes an empty set with the reason discarded. That
much is deliberate and tested. What is not: the next `persist_diff_comments`
writes that empty set over the file.

**Reproduction.** A sidecar holding twelve notes fails to parse — a shape change
in a build, or ordinary corruption; there is no version field and it is outside
`STORE_SCHEMA_VERSION`. Operon opens with no notes and says nothing. Write one
note. The file now holds one, and the twelve are gone with no copy anywhere.
`.claude/skills/durability-invariants/SKILL.md` asks for the opposite: a
malformed legacy sidecar is preserved for diagnosis rather than repaired.

**Fix.** A file that exists, is non-empty, and does not parse is renamed aside
before anything replaces it, and the person is told where it went — the same
move `recover_unreadable_store` already makes for the store itself.

## Two tests in this commit are not this change's

`the_release_gate_consults_the_preconditions_script` and
`the_release_preconditions_script_has_no_bypass` belong to change 032, written
by another session working in this repository at the same time. They were
unstaged in `src/tests.rs` when this change staged that file whole, so they
landed under this commit's message.

Recorded rather than reverted: pulling two hunks out of a landed commit to
re-land them elsewhere is churn in a file both sessions are editing, and churn
in that file is what caused this. 032's plan.md names this commit from the other
direction, so a reader arriving at either finds the pointer.

The mistake was staging a file rather than a change, knowing somebody else was
in it.

## Guards

Each is watched failing under a mutation aimed at what it guards, and each
mutation is confirmed to have landed on the line it was aimed at.

## Not done here

The twelve remaining Important findings, in `./remaining.md`. Two of them —
the editor's root for a worktree session, and the resolved-path cache key —
need a decision rather than a fix.
