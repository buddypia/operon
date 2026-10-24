# Spec: fold the durability reviewer into the Rust reviewer and the durability skill

- **Intent**: `./intent.md`
- **Status**: draft

## Requirements

1. `scripts/check-review.sh` asks `rust-reviewer` for the `store` and
   `bundle-swap` surfaces, and a `full` route asks for `rust-reviewer` and
   `subprocess-safety-reviewer`. `the_review_gate_requires_the_reviewers_review_md_names`
   keeps REVIEW.md's table and the script's mapping equal; the gate fixture
   tests in `src/tests.rs` that name the required set say the new set.
2. `durability-reviewer`'s definition under `.claude/agents/` is gone, so a verdict recorded
   under that name is refused as a reviewer that is not defined — the gate's
   existing check, unchanged.
3. The procedure the durability reviewer held — the order to check in, the
   interruption-timeline failure statement, and the sync-injection test
   expectation — is a **Reviewing a diff against this** section of
   `.claude/skills/durability-invariants/SKILL.md`, and nowhere else.
4. Every reviewer the gate asks for the `store` surface tells itself to read
   that skill and that section. New test
   `the_reviewer_the_store_asks_for_reads_the_durability_procedure` reads the
   reviewer `reviewer_for_surface` names for `store` and `bundle-swap`, and
   fails when its definition does not name the skill and the section, or the
   skill lacks the section.
5. No document an agent is steered by names `durability-reviewer` or counts
   three reviewers: `REVIEW.md`, `docs/sdlc/routes.yaml` comments,
   `docs/sdlc/README.md`, `.claude/skills/feature-pilot/references/rust-build-chain.md`,
   `.github/workflows/claude-review.yml`. `harness_documents_only_name_paths_that_exist`
   already fails on a path that is not there.
6. The eval that decided this is kept in `eval.md` beside this spec.

## Behaviour

No screen changes. A person sees one fewer reviewer in the gate's
`必要なレビュアー:` line for a store, bundle-swap, or `full` change, and the
refusal text is otherwise as it was. A session that records a
`durability-reviewer` verdict is refused with the existing
"定義されていないレビュアーです" message.

## Design

- Placement: `EXTEND` — the reviewer set and its gate exist; this narrows them.
- `reviewer_for_surface`: `store | bundle-swap) echo rust-reviewer`. `full)`
  needs the two remaining reviewers.
- `rust-reviewer.md`: the description gains the durability lens on those
  surfaces; **Order to check in** gains item 7, which points at the skill's
  section; **What not to report** stops handing persistence to a reviewer that
  no longer exists; the verdict refuses a `CommittedButNotSynced` mapped onto
  the failure branch.
- The durability skill keeps `user-invocable: false` and its description; the
  session reads the same section when it changes the store, which is the
  progressive disclosure the fold relies on.
- Bootstrapping: main's judge reads this branch's commits. This diff touches
  `gate-configuration` and no `store` or `bundle-swap` path, so main asks for
  `rust-reviewer` (route `modify`) and `subprocess-safety-reviewer`; both stay
  defined. A store change in flight elsewhere that recorded a durability
  verdict is refused after this lands; `git worktree list` shows none, and the
  three remote branches left (178, 180, 186) are closed records.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | no | nothing paints |
| Icons | no | no mark |
| Identifier SSOT | yes | the surface→reviewer mapping stays in `reviewer_for_surface` alone; REVIEW.md's table is checked against it by the existing test |
| Durability | yes | the guarantees are unchanged and still reviewed on every store diff; only who reads them changes |
| Subprocess safety | no | no child process added or changed |
| Documentation | yes | `README*.md` do not name the reviewers (checked: `git grep durability-reviewer -- 'README*'` is empty) |
| Local-first | no | nothing leaves the machine |
| Budgets | no | no scan or output path |

## Flagged concerns

- **This relaxes a gate.** One required reviewer fewer on `store`,
  `bundle-swap`, and `full`. `approval.md` gives that to a person; asked before
  editing.

## Acceptance

- `cargo test --locked the_review_gate` and the gate fixture tests pass with
  the new set; `the_reviewer_the_store_asks_for_reads_the_durability_procedure`
  passes and is watched failing when item 7 is removed from `rust-reviewer.md`.
- `git grep -n durability-reviewer -- . ':!docs/sdlc/changes' ':!docs/sdlc/lessons.md'`
  prints only test comments that quote history.
- `bash scripts/check-review.sh` on a staged `src/store.rs` edit names
  `rust-reviewer`.

## Rejected alternatives

- **Fold every reviewer into one now.** Subprocess raised Importants in ten
  changes; folding it untested risks the reviewer that finds the most. One fold
  at a time, measured.
- **Keep durability for `store` only and drop it from `full`.** Keeps the
  definition and the duplicated procedure, which is the cost the eval priced.
- **Fold durability into the subprocess reviewer.** Durability ran beside the
  Rust reviewer in 28 of 29 runs, and the store is Rust code; bundle-swap is
  shell, but its review is an interruption timeline, which the skill section
  states for either reader.
- **Accept retired reviewer names as no-ops in the gate.** That widens what the
  judge accepts, and a verdict from a reviewer that does not exist is the shape
  a fabricated approval takes.
