# Intent: Build artifacts in target consume unbounded disk space

- **Status**: approved
- **Opened**: 2026-09-21

## Problem

Compiling and testing the workspace during agent-driven workflows causes `target/` to accumulate tens to hundreds of gigabytes (reaching 128 GB) of stale build artifacts, debug binaries, and incremental compilation cache. Because Cargo never garbage-collects obsolete binaries with old hashes, frequent automated test and build cycles exhaust disk space rapidly.

## Who feels it, and when

Anyone running repeated development, automated testing, and release packaging sessions on the machine, particularly across multiple worktrees and agent runs where test suites run repeatedly.

## Desired outcome

1. Routine development and test builds produce significantly smaller artifacts (at least 50% smaller binary artifacts) while retaining actionable stack traces with file names and line numbers on test failure or panic.
2. An automated and safe pruning script is available in the repository to clean up obsolete test binaries and stale build artifacts without wiping the entire build cache.
3. Disk space growth in `target/` is bounded and manageable.

## Constraints this change inherits

- macOS only.
- Local-first: no external dependencies, no telemetry.
- Test failures and panics must retain source file and line number information in stack traces (`debug = 1` maintains line tables).
- Release build profile and packaged bundle integrity (`/Applications/Operon.app`) must remain unaffected.
- No network access during builds (`--locked` preserved).

## Systems likely affected

- Build configuration (`Cargo.toml`).
- Developer tooling and maintenance scripts (`scripts/`, `Makefile`).

## Open questions

None. The user has explicitly selected the combination of Cargo profile configuration (`debug = 1`) and a repository pruning script.

## Not in scope

- Rewriting dependencies to reduce compile-time size.
- Changing release build profile optimizations (`[profile.release]` remains intact).
