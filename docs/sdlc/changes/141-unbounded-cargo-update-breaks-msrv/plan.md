# Plan: declare project MSRV in Cargo manifest and stabilize dependencies

- **Spec**: `./spec.md`
- **Approved**: 2026-10-10
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `Cargo.toml` | add `rust-version = "1.88.0"` under `[package]` |
| `Cargo.lock` | updated `libc` to `0.2.190` while preserving `uuid` at `1.24.1` |
| `src/tests.rs` | new test: verify Cargo.toml rust-version matches .github/workflows/ci.yml toolchain |
| `docs/sdlc/changes/141-unbounded-cargo-update-breaks-msrv/` | SDLC pipeline tracking documents |

## Order of work

1. Write the test `cargo_manifest_declares_msrv_matching_ci` in `src/tests.rs` and watch it fail because `Cargo.toml` lacks `rust-version`.
2. Add `rust-version = "1.88.0"` to `Cargo.toml`.
3. Watch the test pass.
4. Verify mutation: changing the expected version in test or removing `rust-version` breaks the test.
5. Run the commit gates: `cargo fmt --check`, `cargo clippy --locked -- -D warnings`, and targeted test run.

## Risks

- **Crates violating MSRV:** Future updates without `--locked` could attempt to pull dependencies needing a newer rustc version. The manifest key makes `cargo` reject those packages automatically when resolving.

## Proof of completion

- `cargo test --locked cargo_manifest_declares_msrv_matching_ci` passed in 0.68s.
- `cargo fmt --check` and `cargo clippy --locked -- -D warnings` exit 0.
- `scripts/check-readiness.sh` reports Go.
- `scripts/check-review.sh` reports SHIP (rust-reviewer approved round 1 at diff 776213d496c1a511).

## Departures from the plan

- Included `libc 0.2.190` patch bump in `Cargo.lock` as verified low-risk dependency update within the 1.88.0 MSRV.
