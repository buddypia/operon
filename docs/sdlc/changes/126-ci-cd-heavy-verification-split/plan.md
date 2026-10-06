# Plan: offload heavy verifications and integration test suites to CI/CD

- **Spec**: `./spec.md`
- **Approved**: 2026-10-06
- **Status**: approved

This is the plan produced in plan mode and accepted before any file was edited.
If the implementation departs from it, update this file — an abandoned plan is
worse than no plan, because the next reader trusts it.

## Files that change

| File | Change |
|---|---|
| `scripts/package-macos.sh` | Add `--fast` and `OPERON_FAST_PACKAGE` support skipping test gate while preserving fmt/clippy gates and default 3-gate sequence |
| `.github/workflows/ci.yml` | Update header for active remote, structure parallel jobs for fast checks, full test-suite, macos packaging, and bands |
| `src/tests.rs` | Add unit test `the_packager_fast_mode_skips_tests_when_requested` |

## Order of work

1. Update `scripts/package-macos.sh` to parse `--fast` / `OPERON_FAST_PACKAGE=1`.
2. Update `.github/workflows/ci.yml` with structured jobs while preserving canonical gate command strings.
3. Add unit test `the_packager_fast_mode_skips_tests_when_requested` in `src/tests.rs`.
4. Run `cargo test --locked the_packager` to verify both default and fast packager modes.
5. Run readiness gate `bash scripts/check-readiness.sh docs/sdlc/changes/126-ci-cd-heavy-verification-split`.
6. Run `cargo fmt --check` and `cargo clippy --locked -- -D warnings`.
7. Prepare `review.yaml`, commit in worktree, merge `--no-ff` into `main`, cleanup worktree, and push to `origin/main`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Default packager stops running all three gates | Test failure in `the_packager_runs_the_three_gates_before_it_builds` | Existing test in `src/tests.rs` fails |
| Invariant document check fails if gate strings are moved or rephrased | `every_document_that_names_the_gates_names_the_same_three` fails | Existing test in `src/tests.rs` fails |

## Proof of completion

- `cargo fmt --check` — clean.
- `cargo clippy --locked -- -D warnings` — clean.
- `cargo test --locked the_packager` — passes.
- `every_document_that_names_the_gates_names_the_same_three` — passes.
- `bash scripts/check-readiness.sh docs/sdlc/changes/126-ci-cd-heavy-verification-split` — exits 0 (Go).

## Departures from the plan

None yet.
