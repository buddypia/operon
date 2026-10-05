# Spec: Suppress target directory build artifact bloat

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `Cargo.toml` configures `debug = 1` for `[profile.dev]` and `[profile.test]`, suppressing bulky type/variable debug symbols while retaining line-level debug tables for panic/test stack traces.
2. A new maintenance script `scripts/prune-target.sh` safely removes stale test binaries (`target/debug/deps/operon-*`), keeping only the most recent build artifacts, and supports `--dry-run` to preview deletions and reclaimed bytes without mutating disk state.
3. `Makefile` exposes `wt.prune-target` (or `prune.target`) so developers and agents can execute routine target garbage collection with a single, guarded command.
4. Regression tests in `src/tests.rs` verify that `Cargo.toml` contains `[profile.dev]` and `[profile.test]` with `debug = 1`, and that `scripts/prune-target.sh` exists, is executable, and runs `--dry-run` successfully.

## Behaviour

- Running `cargo test` builds binaries with `debug = 1`. Stack traces in test failures continue to report file paths and line numbers accurately.
- Running `scripts/prune-target.sh` with `--dry-run` lists obsolete artifacts and reports total reclaimable space.
- Running `scripts/prune-target.sh` deletes obsolete test binaries and reports reclaimed megabytes/gigabytes.
- If `target/` does not exist or has no stale artifacts, `scripts/prune-target.sh` exits cleanly with code 0 and announces that there is nothing to prune.

## Design

- `Cargo.toml`:
  ```toml
  [profile.dev]
  debug = 1

  [profile.test]
  debug = 1
  ```
- `scripts/prune-target.sh`:
  A portable bash script.
  - Locates `target/debug/deps` and `target/release/deps`.
  - For tests/binaries matching `operon-*` (excluding `.d` dependency files and `.rmeta`/`.rlib`), detects obsolete instances older than the newest mtime, or when `--older-than <days>` is passed.
  - Safely deletes corresponding `.dSYM` directories and binary executables.
  - Supports `--dry-run`.
- `Makefile`:
  - Adds `.PHONY: prune.target` running `bash scripts/prune-target.sh`.
- `src/tests.rs`:
  - `dev_and_test_profiles_suppress_debug_symbols`: pins `debug = 1` in `Cargo.toml`.
  - `prune_target_script_runs_dry_run_cleanly`: invokes `scripts/prune-target.sh` with `--dry-run` and asserts exit status 0.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | No UI colour change. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No new icons. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | No persisted identifier change. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | Persisted SQLite store is untouched. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | Tooling script only; no new application subprocesses. |
| Documentation — user-facing docs change in all three languages together | No | Internal developer workflow and build configuration only. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | 100% local bash and cargo configuration. Zero network calls. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | `scripts/prune-target.sh` limits path traversals specifically to `target/*/deps` and `target/*/incremental`. |

## Flagged concerns

None.

## Acceptance

- `cargo test --locked` passes, including the new tests `dev_and_test_profiles_suppress_debug_symbols` and `prune_target_script_runs_dry_run_cleanly`.
- `cargo clippy --locked -- -D warnings` and `cargo fmt --check` pass.
- `scripts/prune-target.sh` executed with `--dry-run` exits with code 0.

## Rejected alternatives

- `cargo sweep`: Requires third-party cargo plugin installation (`cargo install cargo-sweep`), violating local-first zero-external-install requirements.
- Setting `debug = 0`: Removes all stack trace line numbers, making panic and test debugging unviable.
- Setting `split-debuginfo = "unpacked"` without `debug = 1`: Leaves huge type metadata intact, reducing less than 15% of bloat compared to `debug = 1`'s 60-80% reduction.
