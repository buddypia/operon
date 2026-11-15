# Intent: the toolchain and the dependencies have fallen a year behind

- **Status**: approved
- **Opened**: 2026-11-13

## Problem

The person asked to bring Rust and every version up to date
("Rust最新バージョンとかバージョン最新にして"), and, offered the choice of
leaving the GUI framework for a later change, chose to take all of it in this
one ("egui 0.37 まで一気にやる").

The compiler the project is pinned to is twelve releases old (1.88.0 against
1.100.0 on 2026-11-13). The GUI framework is six breaking releases old (0.31
against 0.37), and three other direct dependencies are one or two breaking
releases behind (the file dialog 0.15 against 0.18, SQLite bindings 0.40
against 0.42, and the test-only TOML parser 0.8 against 1.2). Every fix and
improvement upstream since then is out of reach, and each release skipped makes
the eventual migration larger: the last attempt (change 141) already left the
GUI framework out because two breaking releases looked too big to take at once.

## Who feels it, and when

The person, every time a crate they want needs a newer compiler or a newer GUI
framework, and every session that runs `cargo update` and has to hold versions
back by hand to stay inside the 1.88.0 ceiling.

## Desired outcome

1. The project builds, lints and tests on Rust 1.100.0, and every place that
   names the toolchain names that one version.
2. Each direct dependency is on its newest stable release, except where a
   stated reason holds it back.
3. Operon looks and behaves as it did before: the same screens, the same
   icons, the same terminal, the same stored data opened without migration.

## Constraints this change inherits

- macOS only; immediate-mode GUI, moving from `eframe`/`egui` 0.31 to 0.37.
- Local-first: no telemetry, no accounts, no cloud calls.
- The persisted store is on real machines and must open unchanged.
- The embedding runtime is a pinned fork until crates.io vendors a llama.cpp
  that knows EmbeddingGemma 2 (change 157).

## Systems likely affected

`Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, the CI workflows that pin
the toolchain, `src/main.rs` (fonts, window options), `src/app.rs` and
`src/app/`, `src/ui/`, and the egui-driven tests in `src/tests.rs`.

## Open questions

None for the person: the scope was their answer.

## Not in scope

- New behaviour, restyling, or using new egui features.
- Moving the embedding runtime off its fork (crates.io 0.1.161 still lacks the
  model) or `libc` to 1.0 (alpha only).
