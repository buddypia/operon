# Plan: Add trivial work route to avoid subagent review explosion

- **Spec**: `./spec.md`
- **Approved**: 2026-09-21
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `docs/sdlc/routes.yaml` | Add `trivial` route definition (`build,test 1 .claude/hooks/gate-commit.sh - none`) |
| `docs/sdlc/README.md` | Add `trivial` to the Stage 0 route listing with qualification criteria |
| `.claude/skills/sdlc/SKILL.md` | Add `trivial` to the route table in Section 1 |
| `src/tests.rs` | Update `the_review_gate_has_no_bypass` to recognize merge commit `exit 0` in check-review.sh |

## Order of work

1. Update `docs/sdlc/routes.yaml` to declare `trivial`.
2. Update `docs/sdlc/README.md` to document the new route.
3. Update `.claude/skills/sdlc/SKILL.md` to document the new route.
4. Run `cargo test --locked every_route_the_pipeline_offers_names_stages_that_exist`.
5. Fix `the_review_gate_has_no_bypass` in `src/tests.rs` to allow merge-commit exit.
6. Run the full cargo test and lint gates (`cargo fmt --check`, `cargo test --locked`, `cargo clippy --locked -- -D warnings`).
7. Perform stage 5 review check and record `review.yaml`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Inconsistent route table representation across docs | `every_route_the_pipeline_offers_names_stages_that_exist` fails in test suite | `cargo test --locked` fails |
| Accidentally weakening high-risk surface reviews | High-risk surfaces being committed without required reviewers | `scripts/check-review.sh` enforces surface widening regardless of route |

## Proof of completion

- `cargo fmt --check` passes cleanly.
- `cargo test --locked` passes all 496 tests with exactly 6 ignored.
- `cargo clippy --locked -- -D warnings` passes without warnings.
- `bash scripts/check-review.sh docs/sdlc/changes/059-add-trivial-work-route` returns `SHIP`.

## Departures from the plan

- Updated `the_review_gate_has_no_bypass` in `src/tests.rs` because previous commit `9c31163` introduced merge commit exit 0 in `check-review.sh` without updating the test's success-pattern assertion.
