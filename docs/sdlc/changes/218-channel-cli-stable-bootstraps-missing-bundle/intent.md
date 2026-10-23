# Intent: `operon-channel.sh stable` bootstraps `/Applications/Operon Stable.app` from `dist/Operon.app` on first run

- **Date**: 2026-10-24
- **Route**: `bugfix` (see `docs/sdlc/routes.yaml`)
- **Status**: approved

## Request

> `~/dev/buddypia/operon (main) » bash scripts/operon-channel.sh stable`
> `error: stable bundle not found at /Applications/Operon Stable.app — run 'operon-channel.sh stable --promote' after verifying canary`
> `---`
> `上記のエラーが出ている`

When the person ran `bash scripts/operon-channel.sh stable` on a checkout where `dist/Operon.app` was already built but `/Applications/Operon Stable.app` had never been created yet, the CLI exited `66` instead of initializing `/Applications/Operon Stable.app` from `dist/Operon.app` and launching it.

## Root Cause

- **Named cause**: `cmd_stable` in `scripts/operon-channel.sh` unconditionally exited `66` whenever `/Applications/Operon Stable.app` did not exist yet (`[ ! -d "$stable_app" ]`), even when `dist/Operon.app` (`$canary_app`) was already present on disk to seed the initial stable bundle.
- **Placement**: `EXTEND` in `scripts/operon-channel.sh` (`cmd_stable`) and `src/tests.rs` (`the_channel_cli_promotes_canary_to_stable_and_rolls_back` and `the_channel_cli_refuses_missing_bundles_and_unknown_subcommands`).

## Problem

On a machine where `dist/Operon.app` already exists, running `bash scripts/operon-channel.sh stable` for the first time fails with `error: stable bundle not found at /Applications/Operon Stable.app`, forcing an extra manual `--promote` step before `stable` can launch.

## Users & Context

Developers and dogfooders running `bash scripts/operon-channel.sh stable` on macOS.

## Desired Outcome

1. When `bash scripts/operon-channel.sh stable` (with or without `--no-open`) runs and `/Applications/Operon Stable.app` does not exist yet, if `dist/Operon.app` (`$canary_app`) exists on disk, `stable` automatically promotes `dist/Operon.app` into `/Applications/Operon Stable.app` and launches it (unless `--no-open`).
2. Once `/Applications/Operon Stable.app` exists, `bash scripts/operon-channel.sh stable` leaves `/Applications/Operon Stable.app` untouched (does not overwrite it with a newer `dist/Operon.app`) and simply switches to `/Applications/Operon Stable.app`.
3. When neither `/Applications/Operon Stable.app` nor `dist/Operon.app` exists on disk, `bash scripts/operon-channel.sh stable` exits `66` with a clear message instructing to run `operon-channel.sh canary --build`.

## Scope

### In scope

- `scripts/operon-channel.sh`: bootstrap `$stable_app` from `$canary_app` via `do_promote "$no_open"` when `[ ! -d "$stable_app" ] && [ -d "$canary_app" ]`.
- `src/tests.rs`: verify that `stable` bootstraps `/Applications/Operon Stable.app` when missing and `dist/Operon.app` exists, that subsequent `stable` runs do not overwrite an existing `/Applications/Operon Stable.app`, and that `stable` exits `66` when both bundles are missing.

### Out of scope

- Modifying `scripts/package-macos.sh` or `scripts/replace-macos-bundle.sh`.
