# Spec: Add trivial work route to avoid subagent review explosion

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Add a new `trivial` route to `docs/sdlc/routes.yaml` following the 5-positional tuple format (`<stages> <attempts> <gate> <requires> <review>`).
2. The `trivial` route must specify `stages` as `build,test`, `attempts` as `1`, `gate` as `.claude/hooks/gate-commit.sh`, `requires` as `-`, and `review` as `none`.
3. The `trivial` route must be formally documented in `docs/sdlc/README.md` and `.claude/skills/sdlc/SKILL.md` with explicit qualification criteria:
   - Modifications <= 2 files and <= 20 lines of code (LOC).
   - Confined to non-high-risk surfaces (does not touch store, unsafe, path configuration, or gate configuration).
   - Trivial typo fixes, small internal adjustments, or single-test additions.
4. The route addition must satisfy the comprehensive integrity test `every_route_the_pipeline_offers_names_stages_that_exist` in `src/tests.rs`.

## Behaviour

When an agent (or human) starts a small, self-contained task qualifying as trivial, it configures `state.yaml` with:
```yaml
state:
  route: "trivial"
  stage: "build"
  attempts: "0 1"
```
During pre-commit checks (`scripts/check-review.sh` called by `.claude/hooks/gate-commit.sh`):
- `scripts/check-review.sh` parses `route: "trivial"`, maps `none` to an empty required reviewer set (`required=""`).
- As long as no touched high-risk surface adds a reviewer, the commit passes without invoking subagents (`rust-reviewer`, `durability-reviewer`, `subprocess-safety-reviewer`).
- Deterministic verification (`cargo fmt`, `cargo test`, `cargo clippy`) runs as usual.
- The endless ping-pong review cycle is eliminated for trivial tasks.

## Design

1. `docs/sdlc/routes.yaml`:
   Append the following entry to the `routes:` mapping:
   ```yaml
   # Very small internal changes (<=2 files, <=20 LOC), typo fixes, or trivial adjustments.
   # Gated by deterministic tests; review subagents waived.
   trivial: "build,test 1 .claude/hooks/gate-commit.sh - none"
   ```

2. `docs/sdlc/README.md`:
   Update Stage 0 routing documentation to list `trivial` alongside existing routes, explaining when to use it and the 1-attempt ceiling.

3. `.claude/skills/sdlc/SKILL.md`:
   Add `trivial` to the route table in Section 1 ("Does it belong here, and on which route?"), documenting its definition and scope rules.

4. `src/tests.rs`:
   The test `every_route_the_pipeline_offers_names_stages_that_exist` will dynamically discover `trivial` from `routes.yaml` and verify its presence in `SKILL.md`, `README.md`, and its structural validity.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | No | Does not touch UI palettes or colors. |
| Icons | No | Does not add or modify icon vocabularies. |
| Identifier SSOT | Yes | The route name `trivial` is defined in `routes.yaml` (SSOT) and referenced consistently in docs and skills. |
| Durability | No | Persisted application store schema is untouched. |
| Subprocess safety | Yes | Gate configuration surface is classified as high-risk; this change adheres to all gate-commit requirements. |
| Documentation | Yes | Documentation and skill references are kept in sync with the new route. |
| Local-first | Yes | Entirely local tooling and pipeline definitions. |
| Budgets | Yes | Does not introduce any unbounded output or execution paths. |

## Flagged concerns

None. The `trivial` route does not weaken security or durability because `scripts/check-review.sh` automatically widens the required reviewer set if any high-risk surface (`store`, `unsafe-and-path`, `bundle-swap`, `gate-configuration`, `dependencies`, or `Command::new`/`unsafe`) is touched by the diff.

## Acceptance

- `every_route_the_pipeline_offers_names_stages_that_exist` passes in `cargo test --locked`.
- `every_state_file_names_a_route_and_a_stage_that_exist` passes in `cargo test --locked`.
- `cargo fmt --check`, `cargo test --locked`, and `cargo clippy --locked -- -D warnings` all pass.
- `scripts/check-readiness.sh` returns Go for `docs/sdlc/changes/059-add-trivial-work-route`.

## Rejected alternatives

- Relying only on unrecorded one-line fixes (no SDLC directory): Rejected because many small tasks still benefit from a traceable change directory and branch name, but do not warrant 3 LLM review subagents.
- Setting `review: craft` for trivial changes: Rejected because running `rust-reviewer` for a 5-line typo or formatting fix still causes diff digest shifts and retry loops.
