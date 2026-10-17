# Spec: classify the enforcement plane as gate configuration

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. The `gate-configuration` row of `docs/sdlc/risk.yaml` also covers, each one
   measured in the tree:
   - `.cli` — the guards `.claude/settings.json`, `.claude/hooks.json`
     (Antigravity, through the `.agents` link) and `.codex/hooks.json` run on
     every command, their shared library, and their dispatcher;
   - `.claude/config` — the policy those guards read (`worktree-policy.json`:
     protected branches, the trunk allowlist) and the approval vocabulary
     `.cli/lib/approval-vocabulary.mjs` reads;
   - `.claude/hooks.json`, `.codex/hooks.json` — the other two CLIs' hook
     registrations (`.claude/settings.json` is already on the row);
   - `scripts/ci-verified.sh` — what `.claude/hooks/gate-merge.sh` asks before
     admitting a merge; `scripts/run-bounded.sh` — the wrapper both that merge
     gate's local suite and `scripts/ci-verified.sh`'s `gh` query run under, so
     an edit to it can make either report success (evaluator round 1);
     `scripts/check-readiness.sh` — the verdict `.claude/hooks/guard-stage.sh`
     holds stage 3 behind;
   - `.github/workflows` — the CI run `scripts/ci-verified.sh` accepts as proof.
   `every_risk_surface_names_paths_that_exist` already fails on a path that is
   not there. `scripts/check-installed-build.sh` stays off the row:
   `.claude/hooks/gate-stop.sh` only prints its report, under a 5-second bound
   with its errors discarded, and refuses nothing on it.
2. `every_hook_an_agent_cli_runs_is_gate_configuration` fails when a hook
   registration file at the repository's top level — a `<.dir>/hooks.json` or
   `<.dir>/settings.json` holding at least one `command` — or a repository file
   any of those commands names, is not under a `gate-configuration` path. The
   registration files are found by looking, not listed, so a fourth CLI's file
   is judged the day it lands. A command is split on whitespace and both quote
   characters; a word names a repository file if the word itself, or the part
   after any `/` in it, is a file under the repository root once canonicalised
   — which reads `${CLAUDE_PROJECT_DIR}/.cli/…`, `)/.cli/_cli-dispatch.mjs`
   (the dispatcher behind `$(dirname …)`), and a bare `.cli/hooks/…`. Watched
   failing on today's table, naming the `.cli/hooks` guards and both
   `hooks.json` files.

## Behaviour

No user-visible behaviour. For sessions: a diff touching any path above is
`high` / `paused`, needs `plan.md` judged before editing (a relaxation is the
person's, a tightening the evaluator's, per `approval.md`), and pulls in
`subprocess-safety-reviewer` through `scripts/check-review.sh`'s existing
mapping. The failure message names each uncovered file and the row to add it to.

## Design

- `docs/sdlc/risk.yaml`: the row keeps its four positional values; the paths
  list grows, and its comment says what the row now holds and why (the
  enforcement plane: what the gates run, what they read, what they trust).
- `src/tests.rs`: the new test reads the row through `flat_block`, the parser
  the existing risk test uses, and judges words as requirement 2 says. Paths are
  canonicalised so `.agents/…` reads as `.claude/…`, and a candidate outside
  the root (`/usr/bin/…`, `..`) is not a repository file.
- Prefixes are directories, the conventional CODEOWNERS granularity: `.cli`,
  `.claude/config` and `.github/workflows` also hold files that enforce nothing
  (section templates, `ship-scale.mjs`, `scheduled-scan.yml`); every workflow
  can reach the repository's secrets, and splitting the others file by file
  would leave the next guard added beside them unclassified.
- No change to `scripts/check-review.sh`, `REVIEW.md`, or `approval.md`: the
  row's reviewer and approval split already exist.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | no | nothing paints |
| Icons | no | no glyph |
| Identifier SSOT | yes | the test reads the row and the registration files; it lists neither |
| Durability | no | no store |
| Subprocess safety | no | the test spawns nothing |
| Documentation | no | no user-facing document |
| Local-first | no | nothing leaves the machine |
| Budgets | no | reads three small JSON files |

## Flagged concerns

- **More changes stop to be judged.** Changes like 173 and 174 now land as
  `paused` with a plan approval and the safety reviewer. That is the point: 173
  relaxed what main permits, which `approval.md` already says is the person's.

## Acceptance

- `cargo test --locked every_hook_an_agent_cli_runs_is_gate_configuration
  every_risk_surface_names_paths_that_exist the_review_gate` passes.
- On a diff touching `.cli/hooks/…`, `bash scripts/check-review.sh --index`
  requires `subprocess-safety-reviewer`.

## Rejected alternatives

- A new surface row for guards: one more name `scripts/check-review.sh`,
  `REVIEW.md`, and `approval.md` would each have to learn, for the same tier,
  reviewer, and approval split gate configuration already has.
- Listing each guard file: the next guard would be unclassified again; prefixes
  plus the registration test close it.
- Scanning gate scripts for every `scripts/*.sh` they mention: most mentions
  are messages and comments, so the check would be noise; the three the gates
  actually run and act on are listed.
