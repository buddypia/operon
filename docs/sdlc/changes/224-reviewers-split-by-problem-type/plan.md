# Plan: fold the durability reviewer into the Rust reviewer and the durability skill

- **Spec**: `./spec.md`
- **Approved**: pending — a person's, because the change relaxes a gate; built on the branch meanwhile, not landed
- **Status**: in progress

## Files that change

| File | Change |
|---|---|
| `.claude/skills/durability-invariants/SKILL.md` | gains **Reviewing a diff against this** (the durability reviewer's order, failure statement, test expectation) |
| `.claude/agents/rust-reviewer.md` | description and item 7 read that section; persistence no longer handed off; verdict refuses a mis-mapped `CommittedButNotSynced` |
| `.claude/agents/durability-reviewer.md` | deleted |
| `scripts/check-review.sh` | `store`, `bundle-swap` → `rust-reviewer`; `full` → rust + subprocess; header comments |
| `REVIEW.md` | passes 2 and 2b, and the gate table row |
| `docs/sdlc/routes.yaml`, `docs/sdlc/README.md`, `.claude/skills/feature-pilot/references/rust-build-chain.md`, `.github/workflows/claude-review.yml` | name two reviewers and the skill |
| `src/tests.rs` | fixture stops copying the deleted file; expected required sets; new `the_reviewer_the_store_asks_for_reads_the_durability_procedure` |

## Order of work

1. Write the new test first and watch it fail against today's tree (the store
   maps to `durability-reviewer`, whose definition does not name the section).
2. Skill section, `rust-reviewer.md`, delete `durability-reviewer.md`.
3. `scripts/check-review.sh` mapping and `full` set; REVIEW.md table and
   passes — the two move together because a test reads them against each other.
4. Fixture expectations in `src/tests.rs`.
5. The remaining documents.
6. Run the named tests; mutate item 7 out of `rust-reviewer.md` and watch the
   new test go red; restore.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| main's judge asks for a reviewer this branch deleted | commit refused | the diff touches no `store`/`bundle-swap` path; checked with `bash scripts/check-review.sh` before committing |
| a fixture still expects three reviewers | red gate test | `cargo test --locked the_review_gate` and the gate fixture tests |
| a document still names the deleted file | red harness test | `harness_documents_only_name_paths_that_exist` |
| the folded reviewer reads past a durability defect | an escape after landing | `eval.md` is the baseline to re-run; recorded as the risk in `state.yaml` |

## Proof of completion

- `cargo test --locked the_review_gate` — all pass.
- `cargo test --locked the_reviewer_the_store_asks_for_reads_the_durability_procedure` — passes; red with item 7 removed.
- `cargo test --locked harness_documents_only_name_paths_that_exist` — passes.
- `make q.fast` — clean.

## Departures from the plan

None yet.
