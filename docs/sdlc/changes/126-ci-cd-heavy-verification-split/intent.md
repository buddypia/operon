# Intent: offload heavy verifications and integration test suites to CI/CD

- **Status**: approved
- **Opened**: 2026-10-06

## Problem

Local iteration and packaging loops in Operon currently incur substantial latency because full test executions take over 80 seconds (with single integration tests such as `the_review_gate_refuses_what_it_says_it_refuses` running over 60 seconds across real git repositories).
Furthermore:
1. `scripts/package-macos.sh` runs `cargo fmt --check`, `cargo test --locked`, `cargo clippy --locked -- -D warnings`, and `cargo build --release --locked` by default without any opt-in fast path for rapid local verification.
2. The GitHub Actions CI workflow in `.github/workflows/ci.yml` was originally marked as not running due to a lack of a remote. With `origin` now configured, CI should actively run the full exhaustive test matrix, including:
   - Fast lint & formatting checks (`cargo fmt --check`, `cargo clippy --locked -- -D warnings`)
   - Exhaustive test execution (`cargo test --locked`)
   - Cross-CLI ignored restore tests (`cargo test --locked -- --ignored` where feasible)
   - App bundle build & packaging validation (`bash scripts/package-macos.sh`)
   - Control band metrics verification (`bash scripts/check-bands.sh`, `bash scripts/harness-metrics.sh`)
3. Local development and packaging should support an explicit fast mode (`--fast` or `OPERON_FAST_PACKAGE=1`) that delegates the heavy test suite to CI/CD while preserving formatting and clippy gates, without breaking the default invariant checked by `the_packager_runs_the_three_gates_before_it_builds`.

## Who feels it, and when

Developers and agents packaging local app builds or iterating on code who experience 80s+ local wait times for repetitive test runs that are better handled concurrently in CI/CD.

## Desired outcome

1. `.github/workflows/ci.yml` is updated to a complete multi-job CI workflow running fast checks, the full test suite, bundle packaging, and control band checks on push to `main` and pull requests.
2. `scripts/package-macos.sh` supports an explicit opt-in fast mode via `--fast` or `OPERON_FAST_PACKAGE=1` which skips `cargo test --locked` while still running `cargo fmt --check` and `cargo clippy --locked -- -D warnings`. Default invocation without arguments or env var preserves the exact three gates to satisfy `the_packager_runs_the_three_gates_before_it_builds`.
3. Invariant test in `src/tests.rs` validates that `--fast` skips `cargo test --locked` while retaining fmt, clippy, and release build.
4. Changes are merged to `main` and pushed to `origin/main` so GitHub Actions executes the full CI/CD verification matrix.

## Constraints this change inherits

- macOS only; GitHub Actions runner `macos-latest`.
- Zero regressions in existing 714 unit tests; preserve `every_document_that_names_the_gates_names_the_same_three` and `the_packager_runs_the_three_gates_before_it_builds`.
- Local-first architecture; no network dependencies during local offline builds.

## Systems likely affected

- `.github/workflows/ci.yml`
- `scripts/package-macos.sh`
- `src/tests.rs`

## Open questions

None.

## Not in scope

- Deleting existing unit or integration tests from `src/tests.rs`.
