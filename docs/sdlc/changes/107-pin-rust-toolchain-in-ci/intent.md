# Pin Rust toolchain to 1.88.0 in CI workflows and repository

## Problem

GitHub Actions workflows used `@stable` for `dtolnay/rust-toolchain`, causing the latest Rust stable (1.99+) to be downloaded.
This introduced breaking Clippy errors on newer lints (`clippy::chunks_exact_to_as_chunks`, `clippy::while_let_loop`) that are not present in Rust 1.88.0.

## Proposed Change

- Add `rust-toolchain.toml` with channel `1.88.0`.
- Pin `toolchain: 1.88.0` in `.github/workflows/ci.yml`, `harness.yml`, and `scheduled-scan.yml`.

## Risk and Review

Low risk. CI configuration and toolchain pinning only; no application logic change.
