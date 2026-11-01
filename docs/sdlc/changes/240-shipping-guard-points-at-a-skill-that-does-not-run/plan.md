# Plan: the Stop notice points at a landing that does not run here

- **Spec**: n/a (`bugfix` route — the bug report is in `./state.yaml`)
- **Approved**: 2026-11-01
- **Status**: approved

## Root cause

`buildBlockMessage` in `.cli/hooks/worktree-shipping-guard.mjs` is the bundle's
generic text: a squash landing through `/create-pr ship-worktree`, gated on a
`.tmp/worktree-<branch>/PLAN.md` checklist and a review panel. Operon never
adopted that flow, and the guard has no way to be told the project lands
differently, so every Operon worktree reads as "PLAN.md missing" and is sent
to a skill the settings deny.

## The fix

Change 073's rule: what is Operon's goes declaratively in
`.claude/config/worktree-policy.json`, and the guard reads it.

- **Policy**: a `landing_steps` array of lines, `{worktree}` and `{branch}`
  substituted, holding AGENTS.md's landing: push the branch and wait for CI,
  `git merge --no-ff --no-edit {branch}` and `git push origin main` from the
  main checkout, `ops.mjs cleanup-worktree --worktree "{worktree}"`, with
  feature-pilot §4 named for the order and the review record.
- **Guard**: when `landing_steps` is present, the notice says committed work
  lands by those steps and prints them for each unlanded worktree; the
  `/create-pr` line, the PLAN.md tag and both PLAN.md sections, and the review
  panel are not printed; the base-freshness note drops its `ship-worktree`
  clause. The uncommitted-work and merged-but-present sections are unchanged.
  Without the key the text is exactly as before.
- **Deletion**: the twelve unreached modules listed in `state.yaml`, and their
  entries in `.claude/.bundle-receipt.json`, so the inventory says what is
  there. A resync from the bundle source would bring them back; the receipt's
  `$comment` names them so the next resync can drop them deliberately.

## Files that change

| File | Change |
|---|---|
| `.claude/config/worktree-policy.json` | `landing_steps` |
| `.cli/hooks/worktree-shipping-guard.mjs` | read `landing_steps`; print them instead of the ship-worktree flow |
| `.claude/scripts/{review-deck,ship-deck,ship-deck-bridge,followup-debt-tracker}.mjs` | deleted |
| `.claude/scripts/lib/{debt-resolution,deck-visuals,doc-outline,doc-section-render,review-deck-core,review-deck-render,review-origin,ship-deck-render}.mjs` | deleted |
| `.claude/.bundle-receipt.json` | entries for the twelve removed; `$comment` names them |
| `src/tests.rs` | a test that renders the notice with this repository's policy |

## Order of work

1. Test: a fixture with this repository's policy and one committed, unmerged
   worktree. The notice names `git merge --no-ff --no-edit <branch>` and the
   cleanup command for that worktree, and names neither `/create-pr` nor
   `PLAN.md`. Watch it fail on the unedited guard.
2. Policy and guard; the test passes, and
   `the_stop_guard_names_a_merged_worktree_the_session_still_owns` still does.
3. Delete the twelve; re-run the reachability check (`.tmp/240/reach.mjs`) —
   nothing left unreached that was meant to stay — and the hook test groups.
4. Mutate: drop the policy key, drop the PLAN.md suppression, drop the
   `{worktree}` substitution; each goes red.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A deleted module is loaded by a path the import scan cannot see (a spawned script name) | a hook or ops command throws on a missing file | `git grep` of every name (done: only bundle-internal or the denied PR flow); hook test groups |
| The notice stops naming uncommitted work | work lost at Stop | the uncommitted section is untouched; existing test |
| A resync restores the deleted files | they reappear unused | the receipt `$comment`; harmless if it happens |

## Proof of completion

- the new test fails before step 2, passes after.
- `cargo test --locked` groups `worktree`, `guard`, `hook`, `stop` pass.
- `make q.fast` clean; CI green on the branch.

## Departures from the plan

None yet.
