# Ensure clean self-contained documentation and configuration

## Problem

Operon's historical change records, configuration templates, script comments, and test docstrings contained historical notes referencing external tooling, adaptation guides, or sync origins. All documentation, comments, and specifications must describe Operon's own architecture cleanly, directly, and self-containedly.

## Proposed Change

- Clean up historical change records (changes 007, 051, 052, 073, 074, 114, 116, 117, 118, 120) to state Operon's own specifications directly.
- Rename change directory `007-ported-ecosystem-costs-context-it-does-not-earn` to `007-context-efficient-harness`.
- Clean up historical scrub lists so that closed change directories do not retain lists of obsolete identifiers.
- Update `.claude/.bundle-receipt.json`, `.gitignore`, `Makefile`, `.claude/scripts/create-pr/ops.mjs`, `.claude/skills/create-pr/config.json`, `.cli/lib/hook-anchors.mjs`, `.cli/lib/hook-registry.mjs`, and `.cli/lib/utils.mjs` to describe local harness configurations directly.
- Update doc comments in `src/cli.rs` and `src/tests.rs` to refer to target agent CLIs and local harness generation.
- Ensure all tests pass under `cargo test --locked`.

## Risk and Review

Touches documentation, comments, and configuration headers. Does not alter application runtime behavior or runtime dependencies.
