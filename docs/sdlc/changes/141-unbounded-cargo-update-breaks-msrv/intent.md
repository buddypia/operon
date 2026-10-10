# Intent: unbounded dependency updates break the 1.88.0 MSRV

- **Status**: approved
- **Opened**: 2026-10-10

## Problem

The person asked to analyze and execute library and dependency updates step-by-step ("라이브러리와 의존성 최신화 등을 하나씩 하나씩 분석하고 왜 진행해야 하는지... 전부 다 진행해줘... 진행해").

During the analysis, running `cargo update` attempts to resolve `uuid v1.27.0`, which requires Rust `1.89.0`. However, the local development environment and GitHub Actions CI ([`ci.yml`](../../.github/workflows/ci.yml)) are strictly pinned to Rust `1.88.0`. Because [`Cargo.toml`](../../Cargo.toml) omits the `rust-version` manifest key, Cargo's resolver has no knowledge of the project's MSRV, allowing compiler-incompatible dependencies to enter `Cargo.lock` and immediately break compilation across CI jobs.

## Who feels it, and when

Every developer and automated session running `cargo update` or updating locked dependencies, whenever a transitive or direct crate bumps its MSRV beyond Rust 1.88.0. CI fails immediately on `cargo clippy`, `cargo test`, and `cargo fmt`.

## Desired outcome

1. [`Cargo.toml`](../../Cargo.toml) declares `rust-version = "1.88.0"` so the project MSRV is machine-readable and enforced.
2. A deterministic test in [`src/tests.rs`](../../src/tests.rs) verifies that the manifest MSRV matches the CI toolchain declared in `.github/workflows/ci.yml`.
3. `Cargo.lock` has safe compatible dependencies updated without violating the 1.88.0 MSRV or introducing breaking API changes.

## Constraints this change inherits

- Operon's local development and CI toolchain is Rust `1.88.0`. No dependency requiring Rust > 1.88.0 may be introduced.
- Core UI dependencies (`eframe 0.31.1`, `egui-phosphor 0.9.0`) have major breaking API changes in 0.33 and must not be bumped blindly in this change.
- Persistence and durability invariants for SQLite (`rusqlite`) and session models must be strictly preserved.

## Systems likely affected

[`Cargo.toml`](../../Cargo.toml), [`Cargo.lock`](../../Cargo.lock), and [`src/tests.rs`](../../src/tests.rs).

## Open questions

None for the person: the person approved proceeding with the MSRV safety guard and safe dependency updates.

## Not in scope

- Migrating `eframe` from 0.31 to 0.33: that involves rewriting UI event loops, ViewportBuilder, and font layering, which requires a dedicated future change with screen verification.
