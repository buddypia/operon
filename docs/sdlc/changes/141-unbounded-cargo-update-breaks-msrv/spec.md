# Spec: declare project MSRV in Cargo manifest and stabilize dependencies

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. **Manifest declares MSRV.** `Cargo.toml` under `[package]` declares `rust-version = "1.88.0"`. The declared version exactly matches the toolchain configured in `.github/workflows/ci.yml`.
   — `cargo_manifest_declares_msrv_matching_ci`
2. **Locked dependencies compile under MSRV.** The dependencies locked in `Cargo.lock` must compile cleanly under the project's declared MSRV without introducing any crate that requires a newer compiler.
   — `cargo_lock_respects_msrv_and_passes_all_checks`

## Behaviour

- Running `cargo check --locked`, `cargo clippy --locked`, or `cargo test --locked` under Rust 1.88.0 completes successfully without compiler errors or warnings.
- Any future dependency addition or update that requires Rust > 1.88.0 is rejected by Cargo's resolver or fails the manifest consistency test.
- No user-facing application behavior or visual UI metrics change.

## Design

- **Placement:** EXTEND. The change touches `Cargo.toml`, `Cargo.lock`, and adds an invariant test in `src/tests.rs`.
- **Manifest:** Add `rust-version = "1.88.0"` to the `[package]` table in `Cargo.toml`.
- **Lockfile:** Keep `uuid` at version `1.24.1` so that the Rust 1.89.0 requirement introduced in `uuid 1.27.0` is prevented from breaking the locked build. Update safe compatible transitive dependencies that do not exceed Rust 1.88.0.
- **Test:** A new test in `src/tests.rs` parses `Cargo.toml` (extracting `rust-version`) and `.github/workflows/ci.yml` (extracting `toolchain: 1.88.0`), asserting that the two values are identical.
- **Persisted shape:** none.
- **External commands:** None spawned by the application. `cargo` commands run during CI and local build gates.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | no | no drawing or styling changed |
| Icons | no | no icon font assets changed |
| Identifier SSOT | yes | MSRV 1.88.0 is anchored in Cargo.toml and checked against CI |
| Durability | no | SQLite schema and models are untouched |
| Subprocess safety | no | no subprocess invocation sites added |
| Documentation | yes | internal SDLC change documentation fully maintained |
| Local-first | yes | all dependencies remain local-first with zero cloud telemetry |
| Budgets | no | binary size and memory budgets remain within bands |

## Flagged concerns

- **Transitive crates bumping MSRV unexpectedly.** Some crates published on crates.io bump their minimum supported Rust version in minor or patch releases. Declaring `rust-version` in `Cargo.toml` and enforcing locked builds via `Cargo.lock` in CI ensures that inadvertent upgrades never reach main silently without a test failure.

## Acceptance

- `cargo_manifest_declares_msrv_matching_ci` passes, and fails when `rust-version` is removed or mismatched with `ci.yml`.
- `cargo fmt --check`, `cargo clippy --locked -- -D warnings`, and `cargo test --locked` all pass cleanly on Rust 1.88.0.
- `scripts/check-readiness.sh` outputs Go.

## Rejected alternatives

- **Upgrading the entire project to Rust 1.89+ immediately.** Upgrading the compiler requires coordinating developer environments, updating CI configurations, and testing potential compiler regressions. It should be an intentional toolchain upgrade rather than an unplanned side-effect of a library patch.
- **Leaving `rust-version` omitted.** Omitting `rust-version` leaves the dependency graph vulnerable to broken resolution whenever `cargo update` is run by developers or automated tooling.
- **Blindly upgrading eframe to 0.33.** Eframe 0.33 contains extensive breaking API changes in window event handling and fonts, which would introduce severe UI regression risks if bundled with a routine dependency update.
