# Spec: worktree isolation, enforced in all three CLIs

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `git commit` issued with the working directory on `main` is refused, and the
   refusal names the command that opens an isolated tree instead.
2. `git commit` issued from inside an isolated tree is permitted. Not by
   `git -C` from this one: `.claude/hooks/gate-commit.sh` refuses a commit aimed
   at another checkout, because the cargo gates it runs are this directory's and
   would be vouching for code they never compiled. The tree commits for itself,
   from a session whose project directory it is. `AGENTS.md` says so, because
   a common convention is `git -C` and following it here
   produces a refusal with no obvious cause.
3. `git reset --hard`, `git push --force`, `git clean -f` and `git stash clear`
   are refused. `git restore --staged` is permitted, because it does not touch
   the working tree.
4. Requirements 1–3 hold in Claude Code, Codex CLI and Antigravity CLI. A guard
   registered in one CLI only does not satisfy this.
5. `make wt.new BR=feature/<task>` creates the branch and its tree, and is the
   only sanctioned way to create a branch; `git checkout -b`, `git switch -c`
   and `git branch <name>` are refused.
6. Every one of operon's six existing hook registrations survives with its
   event, matcher, command and timeout unchanged, and none of the added hooks
   runs before one of them on the same event.
7. `make q.check` runs this repository's three gates — `cargo fmt --check`,
   `cargo clippy -D warnings`, `cargo test` — and exits non-zero if any fails.
8. The runtime state the added guards write is not tracked.
9. `AGENTS.md` states what is now enforced, and states that `create-pr` is inert
   here, in the same place a session reads the rest of the git rules.
10. `cargo test --locked` passes, including the three tests that read this
    repository's own documents and tables.

## Behaviour

Nothing a person sees in the application changes. What changes is what a session
sees at a terminal:

- On `main`, at `git commit`: a refusal naming `make wt.new BR=feature/<task>`.
- At session start on `main`: a warning that AI work is standardised to run in
  an isolated worktree, suppressible with `ALLOW_MAIN_SESSION=1` for read-only
  exploration.
- At `git reset --hard`: a refusal naming the reversible alternative.

## Design

No Rust module gains anything. The change is four surfaces:

- **Guards.** Node scripts under `.cli/hooks/` — `commit-guard`,
  `destructive-git-guard`, `worktree-policy-guard`,
  `worktree-session-owner-guard`, `worktree-shipping-guard`,
  `pre-ship-review-guard`, `worktree-owner-tracker`, `trunk-start-warning`,
  `milestone-deck-warning` — with their shared libraries under `.cli/lib/` and
  their entry scripts under `.claude/scripts/`. They run out of process and the
  Rust build does not see them.
- **Registration, three times.** The `hooks` block of `.claude/settings.json`
  for Claude Code,
  `.codex/hooks.json` for Codex, `.claude/hooks.json` for Antigravity under the
  namespace `operon-guards`. The existing six Claude registrations are kept in
  place and the new ones appended after them, so `gate-commit.sh` still reaches
  a `git commit` before `commit-guard.mjs` does.
- **Entry points.** `Makefile` gains `wt.new` and `wt.run` from the bundle, and
  `q.check` / `q.fix` written for this repository: the bundle derives those from
  a `package.json` and there is none, so they are the cargo three.
- **Documents.** Two lines in `AGENTS.md`; `.claude/state/` in `.gitignore`.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | Nothing is drawn. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | No mark is added. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | no | No Rust identifier is added. The one string three files must agree on is each hook's command path, and the three CLI registration files are generated from one bundle definition rather than typed three times. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | The store is not touched and its schema does not move. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | partly | No new child is spawned *by the application*. The added children are spawned by the CLI outside the Rust process and never through `src/exec.rs`, so that rule does not reach them; they are judged instead by `subprocess-safety-reviewer`, which `docs/sdlc/risk.yaml` requires here because the diff is on the `gate-configuration` surface. |
| Documentation — user-facing docs change in all three languages together | no | No user-facing document changes; `AGENTS.md` is read by sessions, not users. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | Every added guard reads git and the local filesystem only. No `unsafe` is added — no Rust is added. |
| Budgets — any new scan or output path states its byte and item ceiling | yes | The added `.claude/skills/create-pr/` is the only always-discoverable text; measured below, and `bash scripts/check-bands.sh` is in the contract. |

## Flagged concerns

- **`create-pr` is inert and stays.** It shells out to `gh` and an SSH `origin`
  and this repository has neither, so every path through it fails at its first
  git call. It arrives because it is one bundle with the guards. Resolved by
  documenting it as inert in `AGENTS.md` rather than deleting it, so that a
  session reads the reason before it reads the skill. The alternative —
  deleting it — was rejected because it makes the bundle un-reappliable.
- **The refusal text tells the reader to do something this repository refuses.**
  `commit-guard.mjs` denies a commit on `main` with a message recommending
  `git -C .worktrees/<branch> commit` and, failing that, the `create-pr` skill.
  Both are dead ends here: `.claude/hooks/gate-commit.sh` refuses the `git -C`
  form, and `create-pr` needs a remote this repository does not have. So the
  guard fires correctly and then misdirects, at the exact moment a session is
  reading it for instructions. Resolved for now in `AGENTS.md`, which a session
  has loaded before it ever sees the refusal. Not resolved in the guard itself:
  editing it would make the file deviate from the receipt, so the configuration
  is managed directly. Recorded here so the
  next session that hits the refusal is not the one who has to work it out.

- **`steering_bytes` was already over its threshold at HEAD.** Measured at
  155025 against a 150000 threshold with a stale 91262 baseline, before this
  change touched anything. This change adds 11328 of it. The band is not made
  true by this change and is not made true by reverting it; entry in
  `docs/sdlc/lessons.md` is the place that settles what the metric should be
  measuring, and that is deliberately not this change's business.
- **A document in the tree names a file that is not there, for a reason that is
  not this change.** Change 049's `plan.md` names a check script three times in
  backticks that nothing has written yet; that work is in flight in the same
  tree and is not staged here. Dead paths stay out of backticks — entry 003 in
  `docs/sdlc/lessons.md` — and this change follows that rule in its own files
  but does not edit another change's.

## Acceptance

- `git commit` on `main` is refused, and so is the same commit through
  `git -C .worktrees/<branch>` — by a different guard, for the reason in
  requirement 2.
- `git reset --hard` is refused; `git restore --staged <file>` is not.
- `make wt.new BR=feature/x DRY=1` reports the tree it would create.
- All three CLI registration files name every guard, and `.claude/settings.json`
  still names all six of the hooks it named at HEAD, in the same order.
- `cargo test --locked` passes.

## Rejected alternatives

- **Write the rule into `AGENTS.md` and stop there.** That is what the
  repository already had; change 050 is the record of it not working.
- **Write the guards by hand for this repository.** Three CLIs times nine guards
  is the kind of thing that is correct on the day it is written. The harness has
  a receipt and an init command, so it can be managed cleanly.
- **Take a wider harness.** Tried first, and rolled back: it brought
  a second entry point competing with `.claude/skills/sdlc/SKILL.md`, an
  uncompressed copy of the skill this repository had already deliberately
  compressed into `root-cause`, and TypeScript/React tooling for a Rust desktop
  app. `reference_bytes` went from 11637 to 456009 against a 50000 threshold,
  which is how it was caught.
