# Plan: Suppress target directory build artifact bloat

- **Spec**: `./spec.md`
- **Approved**: 2026-09-21
- **Status**: done

This is the plan produced in plan mode and accepted before any file was edited.
If the implementation departs from it, update this file — an abandoned plan is
worse than no plan, because the next reader trusts it.

## Files that change

| File | Change |
|---|---|
| `Cargo.toml` | Add `[profile.dev]` and `[profile.test]` sections specifying `debug = 1` |
| `scripts/prune-target.sh` | New script for safely purging stale test binaries and old incremental caches |
| `Makefile` | Add `prune.target` target to run `scripts/prune-target.sh` |
| `src/tests.rs` | Add verification tests for profiles and prune script |
| `docs/sdlc/changes/060-suppress-target-bloat/state.yaml` | Track progress through the stages |

## Order of work

1. Update `Cargo.toml` with `[profile.dev]` and `[profile.test]` (`debug = 1`).
2. Create `scripts/prune-target.sh` with `--dry-run` and target scanning logic, and make it executable.
3. Add `prune.target` to `Makefile`.
4. Add regression tests to `src/tests.rs` verifying profile configuration and dry-run execution.
5. Execute gates: `cargo fmt --check`, `cargo test --locked`, `cargo clippy --locked -- -D warnings`.
6. Run `scripts/prune-target.sh` with `--dry-run` to verify expected behavior.
7. Complete reviewer verification, record in `review.yaml`, and confirm `bash scripts/check-review.sh` passes.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Over-aggressive pruning deletes currently needed source or release binaries | Active build fails with missing files | Script explicitly filters to `deps/operon-*` executables and `.dSYM`, never touches release bundle or `.rlib`/`.rmeta` |
| Line numbers lost in panic stack traces | Stack traces omit source file and line numbers | `debug = 1` specifically includes line tables |
| Permission errors on execution | `make prune.target` fails with `Permission denied` | `chmod +x scripts/prune-target.sh` and test in `src/tests.rs` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — all tests pass, including the new tests `dev_and_test_profiles_suppress_debug_symbols` and `prune_target_script_runs_dry_run_cleanly`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `scripts/prune-target.sh` executed with `--dry-run` — exits 0 and reports status cleanly.
- `bash scripts/check-review.sh docs/sdlc/changes/060-suppress-target-bloat` — reports `✅ SHIP`.

## Departures from the plan

None yet.
