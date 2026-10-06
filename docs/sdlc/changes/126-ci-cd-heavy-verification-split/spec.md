# Spec: offload heavy verifications and integration test suites to CI/CD

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. In `.github/workflows/ci.yml`:
   - Remove the obsolete "NOT RUNNING. This repository has no git remote" comment now that `origin` is configured.
   - Configure a structured workflow with separate jobs:
     - `checks`: runs `cargo fmt --check` and `cargo clippy --locked -- -D warnings`.
     - `test-suite`: runs the exhaustive test suite `cargo test --locked` on `macos-latest`.
     - `package`: runs `bash scripts/package-macos.sh` to compile release binary, generate Info.plist, code-sign, and package `Operon.app`.
     - `bands`: runs `bash scripts/harness-metrics.sh` and `bash scripts/check-bands.sh`.
   - Maintain literal occurrences of `cargo fmt --check`, `cargo test --locked`, `cargo clippy --locked -- -D warnings` to satisfy `every_document_that_names_the_gates_names_the_same_three`.
2. In `scripts/package-macos.sh`:
   - Inspect positional arguments for `--fast` and environment variable `OPERON_FAST_PACKAGE`.
   - If fast mode is enabled:
     - Log `[packager] Fast mode: skipping test gate (delegated to CI/CD); running fmt & clippy.`
     - Execute `cargo fmt --check 9>&-`
     - Execute `cargo clippy --locked -- -D warnings 9>&-`
     - Skip `cargo test --locked 9>&-`
   - If fast mode is NOT enabled (default):
     - Execute `cargo fmt --check 9>&-`
     - Execute `cargo test --locked 9>&-`
     - Execute `cargo clippy --locked -- -D warnings 9>&-`
     - Execute `cargo build --release --locked 9>&-`
     in that exact sequence, preserving invariant `the_packager_runs_the_three_gates_before_it_builds`.
3. In `src/tests.rs`:
   - Add unit test `the_packager_fast_mode_skips_tests_when_requested` asserting that passing `--fast` skips `test --locked` while still running `fmt --check`, `clippy --locked -- -D warnings`, and `build --release --locked`.

## Behaviour

- Running `bash scripts/package-macos.sh --fast` or `OPERON_FAST_PACKAGE=1 bash scripts/package-macos.sh` packages the application locally in seconds instead of over a minute, while verifying formatting and clippy lints.
- Pushing to GitHub triggers GitHub Actions to run the full verification matrix across parallel jobs.
- Running `bash scripts/package-macos.sh` without flags runs all 3 gates as before.

## Design

Verdict: `EXTEND` — adds opt-in fast path to packaging script and expands CI workflow.

## Non-functional requirements

- Latency: local packaging with `--fast` avoids 80s+ test run.
- Reliability: 100% adherence to existing invariants and gates.
- Zero network dependencies for local offline packaging.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | no | No UI color changes |
| Icons | no | No icon changes |
| Identifier SSOT | yes | `OPERON_FAST_PACKAGE` and `--fast` documented and tested |
| Durability | no | No storage schema changes |
| Subprocess safety | yes | Packaging subprocesses run with bounded error handling and fd 9 closed |
| Documentation | no | Internal developer tooling and CI enhancement |
| Local-first | yes | Local offline packaging remains fully functional |
| Budgets | no | No new scanning budgets |

## Flagged concerns

- **Packager test regression**: `the_packager_runs_the_three_gates_before_it_builds` requires that the default invocation runs all 3 gates in order. Fast mode must strictly be opt-in via explicit flag or env var.
- **Document gates invariant**: `every_document_that_names_the_gates_names_the_same_three` requires `.github/workflows/ci.yml` and `scripts/package-macos.sh` to contain the three canonical gate command strings.

## Target surfaces

- `.github/workflows/ci.yml`
- `scripts/package-macos.sh`
- `src/tests.rs`

## Rejected alternatives

- Removing `cargo test --locked` unconditionally from local packager — Rejected because release builds without explicit flags must remain self-gating per sdlc 096 and test invariant `the_packager_runs_the_three_gates_before_it_builds`.
- Disabling CI workflow — Rejected because GitHub Actions provides essential parallel verification for heavy 80s+ test suites and release bundle checks.

## Acceptance

- `cargo fmt --check` passes with no output.
- `cargo test --locked` passes with 0 failures.
- `cargo clippy --locked -- -D warnings` passes with 0 warnings.
- `the_packager_runs_the_three_gates_before_it_builds` continues to pass cleanly.
- `the_packager_fast_mode_skips_tests_when_requested` passes cleanly.
- `every_document_that_names_the_gates_names_the_same_three` passes cleanly.
