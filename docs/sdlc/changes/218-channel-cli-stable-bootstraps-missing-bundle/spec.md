# Spec: `operon-channel.sh stable` bootstraps `/Applications/Operon Stable.app` from `dist/Operon.app` on first run

- **Intent**: `./intent.md`
- **Screen**: `n/a`
- **Status**: approved

## Summary

Update `cmd_stable` in `scripts/operon-channel.sh` so that when `/Applications/Operon Stable.app` (`$stable_app`) does not exist yet and `dist/Operon.app` (`$canary_app`) is present, running `operon-channel.sh stable` automatically initializes `$stable_app` from `$canary_app` (via `do_promote "$no_open"`) and opens `$stable_app` (unless `--no-open`). Once `$stable_app` exists, running `operon-channel.sh stable` switches to `$stable_app` without overwriting it.

## Requirements

### Functional

1. **First-run bootstrap of `stable` when `canary` exists**:
   - When `bash scripts/operon-channel.sh stable [--no-open]` is invoked without `--promote` or `--rollback` and `[ ! -d "$stable_app" ]`:
     - If `[ -d "$canary_app" ]`, `cmd_stable` delegates to `do_promote "$no_open"`, staging a copy of `$canary_app`, atomically placing it at `$stable_app` via `scripts/replace-macos-bundle.sh` (keeping `$canary_app` intact), quitting any running Operon, and opening `$stable_app` (unless `--no-open`).
2. **Pinned `stable` once created**:
   - When `[ -d "$stable_app" ]` and neither `--promote` nor `--rollback` is passed, `cmd_stable` does not copy or overwrite `$stable_app` even if `$canary_app` has changed; it quits any running Operon and opens `$stable_app` (unless `--no-open`).
3. **Refusal when both `stable` and `canary` are missing**:
   - When `[ ! -d "$stable_app" ]` and `[ ! -d "$canary_app" ]`, `cmd_stable` writes an error to `stderr` naming `canary --build` and exits `66`.

### Non-functional

- **Atomicity & Staging Cleanup**: First-run bootstrap reuses `do_promote`, preserving staging copy cleanup via `cleanup_staging` trap.

## Affected Surfaces

| File / Module | Change Type | Notes |
|---|---|---|
| `scripts/operon-channel.sh` | Modify (`EXTEND`) | Bootstrap `$stable_app` from `$canary_app` when `$stable_app` is missing |
| `src/tests.rs` | Modify (`EXTEND`) | Test first-run `stable` bootstrap from `canary`, subsequent non-overwriting `stable` launch, and missing-bundle refusal |

## Failure Modes & Recovery

| Failure | Detection | Recovery |
|---|---|---|
| Neither `$stable_app` nor `$canary_app` exists | `[ ! -d "$stable_app" ] && [ ! -d "$canary_app" ]` | Exit `66` with message naming `canary --build` |
| `replace-macos-bundle.sh` fails during bootstrap | Non-zero exit from helper | Remove staging copy and exit `1` |

## Acceptance Criteria

- [ ] **AC1**: When `$stable_app` does not exist and `$canary_app` exists, `bash scripts/operon-channel.sh stable` creates `$stable_app` from `$canary_app` (keeping `$canary_app` intact), quits any running Operon, and opens `$stable_app` (`the_channel_cli_promotes_canary_to_stable_and_rolls_back`).
- [ ] **AC2**: Once `$stable_app` exists, modifying `$canary_app` and running `bash scripts/operon-channel.sh stable` (without `--promote`) opens the existing `$stable_app` without overwriting it (`the_channel_cli_promotes_canary_to_stable_and_rolls_back`).
- [ ] **AC3**: When neither `$stable_app` nor `$canary_app` exists, `bash scripts/operon-channel.sh stable` exits `66` and mentions `canary --build` on `stderr` (`the_channel_cli_refuses_missing_bundles_and_unknown_subcommands`).

## Verification Mapping

| Criterion | Test (`src/tests.rs`) |
|---|---|
| AC1 | `the_channel_cli_promotes_canary_to_stable_and_rolls_back` |
| AC2 | `the_channel_cli_promotes_canary_to_stable_and_rolls_back` |
| AC3 | `the_channel_cli_refuses_missing_bundles_and_unknown_subcommands` |
