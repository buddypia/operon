# Intent: The channel CLI focuses exclusively on the `canary` and `stable` channels

- **Status**: approved
- **Opened**: 2026-10-24

## Problem

> 「これが言ったようにどう分けて利用したらいいか教えて」
> 「stable, canaryだけで良さそう」

After reviewing the channel table for `scripts/operon-channel.sh` (which listed `stable`, `canary`, `dev`, `stable-previous`, and `installed`), the person asked to narrow the model to just `stable` and `canary`.

Exposing `dev`, `installed`, and `stable-previous` in `status` and requiring separate top-level subcommands outside `canary` and `stable` adds unnecessary surface area to a two-channel workflow.

## Who feels it, and when

- A person checking `bash scripts/operon-channel.sh status` or reading the README: they see four bundle slots (`canary`, `stable`, `stable-previous`, `installed`) and a `dev` subcommand when they only want to manage `canary` and `stable`.
- A person promoting or rolling back `stable`: they should be able to run `stable --promote` or `stable --rollback` directly under the `stable` channel command.

## Desired outcome

- `bash scripts/operon-channel.sh status` (and running with no arguments) reports `head:`, `running:`, `canary:`, and `stable:` only.
- `bash scripts/operon-channel.sh canary [--build|-b] [--full] [--no-open]` builds and launches the `canary` bundle (`dist/Operon.app`).
- `bash scripts/operon-channel.sh stable [--promote|-p] [--rollback|-r] [--no-open]` switches to `/Applications/Operon Stable.app`, promoting `canary` into `stable` when `--promote` is passed and rolling back to the retained previous stable bundle when `--rollback` is passed (while keeping `promote`, `rollback`, and `quit` aliases working and removing `dev`).
- `README.md`, `README.ja.md`, and `README.ko.md` document only the two-channel (`canary` and `stable`) workflow.

## Constraints this change inherits

- macOS only (`scripts/operon-channel.sh` must parse under `/bin/bash` 3.2).
- Local-first: no telemetry, no accounts, no cloud calls.
- `source_commit_key` is read from `scripts/package-macos.sh` rather than hardcoded.
- `README.md`, `README.ja.md`, and `README.ko.md` stay synchronized.

## Systems likely affected

`scripts/operon-channel.sh`, `src/tests.rs`, `README.md`, `README.ja.md`, and `README.ko.md`.

## Open questions

None.

## Not in scope

- Modifying `scripts/package-macos.sh` or `scripts/replace-macos-bundle.sh`.
- Changing any Rust code outside `src/tests.rs`.
