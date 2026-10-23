# Spec: The channel CLI focuses exclusively on the `canary` and `stable` channels

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `bash scripts/operon-channel.sh status` (and running with no arguments)
   prints the repository `HEAD` commit, the running Operon process state, and a
   line for each of the two channels — `canary` (`$OPERON_DIST_DIR/Operon.app`)
   and `stable` (`$OPERON_STABLE_APP`, default `/Applications/Operon Stable.app`)
   — reading the commit stamp key out of `scripts/package-macos.sh` rather than
   restating it, and omitting `installed:` and `stable-previous:` lines from
   `status`. Test:
   `the_channel_cli_reports_status_and_reads_its_stamp_key_from_the_packager`.
2. `bash scripts/operon-channel.sh canary [--build|-b] [--full] [--no-open]`
   packages `$OPERON_DIST_DIR/Operon.app` through `scripts/package-macos.sh`
   when `--build` is given or when `$OPERON_DIST_DIR/Operon.app` is absent
   (forwarding `--full` when requested), leaves `$OPERON_STABLE_APP` untouched,
   and unless `--no-open` is given stops any running Operon process and opens
   `$OPERON_DIST_DIR/Operon.app`. Test:
   `the_channel_cli_builds_and_launches_canary_without_touching_stable`.
3. `bash scripts/operon-channel.sh stable --promote` (and `promote`) refuses
   with exit 66 when `$OPERON_DIST_DIR/Operon.app` does not exist; when it
   exists, it stages a copy of `$OPERON_DIST_DIR/Operon.app` and swaps it
   atomically into `$OPERON_STABLE_APP` through
   `scripts/replace-macos-bundle.sh` with
   `$OPERON_DIST_DIR/Operon.stable-previous.app` as the retained prior stable
   bundle, keeping `$OPERON_DIST_DIR/Operon.app` in place, and (for
   `stable --promote` without `--no-open`, or `promote --open`) stops any
   running Operon process and opens `$OPERON_STABLE_APP`. Test:
   `the_channel_cli_promotes_canary_to_stable_and_rolls_back`.
4. `bash scripts/operon-channel.sh stable [--no-open]` refuses with exit 66 when
   `$OPERON_STABLE_APP` is absent, and otherwise stops any running Operon
   process and opens `$OPERON_STABLE_APP` (unless `--no-open` is given).
   `bash scripts/operon-channel.sh stable --rollback [--no-open]` (and
   `rollback [--no-open]`) refuses with exit 66 when
   `$OPERON_DIST_DIR/Operon.stable-previous.app` is absent, and otherwise stops
   any running Operon process, swaps
   `$OPERON_DIST_DIR/Operon.stable-previous.app` back into `$OPERON_STABLE_APP`
   through `scripts/replace-macos-bundle.sh`, and opens `$OPERON_STABLE_APP`
   unless `--no-open` is given. Test:
   `the_channel_cli_promotes_canary_to_stable_and_rolls_back`.
5. `bash scripts/operon-channel.sh` rejects `dev`, unknown subcommands, and
   `stable --promote --rollback` with exit 64 and usage on stderr; `quit` stops
   any running `Operon` or `operon` process (exiting 75 if it does not exit
   within `OPERON_QUIT_WAIT_SECONDS`); `-h`/`--help` exits 0 with usage on
   stdout. Test:
   `the_channel_cli_refuses_missing_bundles_and_unknown_subcommands`.

## Behaviour

`scripts/operon-channel.sh` exposes the two-channel (`canary` and `stable`)
workflow:

- `status` (default): prints `head:`, `running:`, `canary:`, and `stable:`.
- `canary [--build|-b] [--full] [--no-open]`: builds `dist/Operon.app` if
  requested or missing, quits any running Operon process, and launches
  `dist/Operon.app` (unless `--no-open`).
- `stable [--promote|-p] [--rollback|-r] [--no-open]`:
  - with no action flag: quits any running Operon process and launches
    `/Applications/Operon Stable.app` (unless `--no-open`).
  - with `--promote` / `-p`: atomically promotes `dist/Operon.app` into
    `/Applications/Operon Stable.app` (retaining the previous stable bundle at
    `dist/Operon.stable-previous.app`), quits any running Operon process, and
    launches `/Applications/Operon Stable.app` (unless `--no-open`).
  - with `--rollback` / `-r`: quits any running Operon process, atomically
    restores `dist/Operon.stable-previous.app` into
    `/Applications/Operon Stable.app`, and launches
    `/Applications/Operon Stable.app` (unless `--no-open`).
- `promote`, `rollback`, and `quit` remain accepted as convenience aliases;
  `dev` is removed.

States:

- **Missing bundle on `stable`, `stable --promote`, or `stable --rollback`**:
  prints a clear message naming the missing bundle path and the command that
  creates it, and exits 66 (`EX_NOINPUT`).
- **Process refuses to quit within deadline**: prints an error to stderr and
  exits 75 without launching a second instance or swapping under a live lock.
- **Interrupted during promote or rollback**: `trap cleanup_staging EXIT`
  removes any temporary staging directory.

## Design

**Placement: `REPLACE` / `EXTEND` in `scripts/operon-channel.sh`.** Route
`modify`.

- `scripts/operon-channel.sh`:
  - Removes `cmd_dev`, `installed_app`, and `installed_build_freshness` from
    `status`.
  - Adds `--promote` (`-p`) and `--rollback` (`-r`) flags to `cmd_stable`
    (refusing both together with exit 64).
- `src/tests.rs`:
  - Updates `the_channel_cli_reports_status_and_reads_its_stamp_key_from_the_packager`
    to verify `status` outputs only `head:`, `running:`, `canary:`, and `stable:`
    (no `installed:` or `stable-previous:`).
  - Updates `the_channel_cli_promotes_canary_to_stable_and_rolls_back` to test
    `stable --promote`, `stable`, and `stable --rollback`.
  - Updates `the_channel_cli_refuses_missing_bundles_and_unknown_subcommands`
    to verify `dev` and `stable --promote --rollback` exit 64.
  - Updates `the_readmes_document_the_channel_cli` to assert the two-channel
    documentation in `README.md`, `README.ja.md`, and `README.ko.md`.
- `README.md`, `README.ja.md`, `README.ko.md`:
  - Document `status`, `canary --build`, `stable`, `stable --promote`, and
    `stable --rollback`.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | No UI drawing changes. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No icons added or changed. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | `OperonSourceCommit` is read out of `scripts/package-macos.sh`, never retyped; `the_channel_cli_reports_status_and_reads_its_stamp_key_from_the_packager` verifies this. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes (read) | No store or sidecar change. Bundle promotion and rollback delegate to `scripts/replace-macos-bundle.sh` (`RENAME_SWAP`). |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No Rust production subprocess call sites are added. |
| Documentation — user-facing docs change in all three languages together | Yes | `README.md`, `README.ja.md`, and `README.ko.md` are updated together and held by `the_readmes_document_the_channel_cli`. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Local bundle management only; no network calls or `unsafe`. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | Process quit polling is bounded by `OPERON_QUIT_WAIT_SECONDS` (default 10 seconds, 100 iterations of 100 ms). |

## Flagged concerns

- **Combining `--promote` and `--rollback` on `stable` would be ambiguous.** Answered: `cmd_stable` tracks both flags and exits 64 with a usage error if both are passed.

## Acceptance

- `cargo test --locked channel_cli` passes all 5 tests:
  - `the_channel_cli_reports_status_and_reads_its_stamp_key_from_the_packager`
  - `the_channel_cli_builds_and_launches_canary_without_touching_stable`
  - `the_channel_cli_promotes_canary_to_stable_and_rolls_back`
  - `the_channel_cli_refuses_missing_bundles_and_unknown_subcommands`
  - `the_readmes_document_the_channel_cli`
- `bash scripts/operon-channel.sh status` prints `head:`, `running:`, `canary:`, and `stable:` and exits 0.

## Rejected alternatives

- **Keep `dev` and `installed` alongside `canary` and `stable`.** Rejected because the person explicitly asked for just `stable` and `canary` ("stable, canaryだけで良さそう").
