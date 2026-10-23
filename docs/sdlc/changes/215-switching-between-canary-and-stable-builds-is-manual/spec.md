# Spec: A channel CLI builds, switches, promotes, and rolls back canary and stable bundles

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `bash scripts/operon-channel.sh status` (and running with no arguments)
   prints the repository `HEAD` commit, the running Operon process state, and a
   line for each channel slot — `canary` (`$OPERON_DIST_DIR/Operon.app`),
   `stable` (`$OPERON_STABLE_APP`, default `/Applications/Operon Stable.app`),
   `stable-previous` (`$OPERON_DIST_DIR/Operon.stable-previous.app`), and
   `installed` (`$OPERON_INSTALLED_APP`, default `/Applications/Operon.app`) —
   reading the commit stamp key out of `scripts/package-macos.sh` rather than
   restating it, and reporting `check-installed-build.sh`'s verdict for present
   bundles. Test:
   `the_channel_cli_reports_status_and_reads_its_stamp_key_from_the_packager`.
2. `bash scripts/operon-channel.sh canary [--build] [--full] [--no-open]`
   packages `$OPERON_DIST_DIR/Operon.app` through `scripts/package-macos.sh`
   when `--build` is given or when `$OPERON_DIST_DIR/Operon.app` is absent
   (forwarding `--full` when requested), leaves `$OPERON_STABLE_APP` untouched,
   and unless `--no-open` is given stops any running Operon process and opens
   `$OPERON_DIST_DIR/Operon.app`. Test:
   `the_channel_cli_builds_and_launches_canary_without_touching_stable`.
3. `bash scripts/operon-channel.sh promote` refuses with exit 66 when
   `$OPERON_DIST_DIR/Operon.app` does not exist; when it exists, it stages a
   copy of `$OPERON_DIST_DIR/Operon.app` and swaps it atomically into
   `$OPERON_STABLE_APP` through `scripts/replace-macos-bundle.sh` with
   `$OPERON_DIST_DIR/Operon.stable-previous.app` as the retained prior stable
   bundle, keeping `$OPERON_DIST_DIR/Operon.app` in place. Test:
   `the_channel_cli_promotes_canary_to_stable_and_rolls_back`.
4. `bash scripts/operon-channel.sh stable` refuses with exit 66 when
   `$OPERON_STABLE_APP` is absent, and otherwise stops any running Operon
   process and opens `$OPERON_STABLE_APP`. `bash scripts/operon-channel.sh
   rollback [--no-open]` refuses with exit 66 when
   `$OPERON_DIST_DIR/Operon.stable-previous.app` is absent, and otherwise stops
   any running Operon process, swaps
   `$OPERON_DIST_DIR/Operon.stable-previous.app` back into `$OPERON_STABLE_APP`
   through `scripts/replace-macos-bundle.sh`, and opens `$OPERON_STABLE_APP`
   unless `--no-open` is given. Test:
   `the_channel_cli_promotes_canary_to_stable_and_rolls_back`.
5. `bash scripts/operon-channel.sh dev` stops any running Operon process and
   runs `cargo run --release --locked` in the repository; `quit` stops any
   running Operon process without launching another; `-h`/`--help` exits 0 with
   usage on stdout; any unknown subcommand or option exits 64 with usage on
   stderr. Test:
   `the_channel_cli_refuses_missing_bundles_and_unknown_subcommands`.

## Behaviour

`scripts/operon-channel.sh` is a command-line script in `scripts/`, so its
terminal output is English like the rest of `scripts/` (see
`scripts/check-installed-build.sh`). The README sections in `README.md`,
`README.ja.md`, and `README.ko.md` document the command in English, Japanese,
and Korean.

Subcommands:

- `status` (default): prints `repository`, `head`, `running`, and one summary
  block per bundle (`canary`, `stable`, `stable-previous`, `installed`) showing
  path, commit stamp (`missing`, `unstamped`, or the commit SHA), and
  `check-installed-build.sh` output.
- `canary [--build|-b] [--full] [--no-open]`: builds `dist/Operon.app` if
  requested or missing, quits any running Operon process, and launches
  `dist/Operon.app` (unless `--no-open`).
- `dev`: quits any running Operon process and execs `cargo run --release
  --locked`.
- `promote [--open]`: copies `dist/Operon.app` into a staging directory obok
  `dist/` and atomically swaps it into `/Applications/Operon Stable.app` via
  `scripts/replace-macos-bundle.sh`, retaining the previous stable bundle at
  `dist/Operon.stable-previous.app`. If `--open` is passed, quits any running
  Operon and launches `/Applications/Operon Stable.app`.
- `stable`: quits any running Operon process and launches
  `/Applications/Operon Stable.app`.
- `rollback [--no-open]`: quits any running Operon process, atomically swaps
  `dist/Operon.stable-previous.app` back into `/Applications/Operon Stable.app`
  via `scripts/replace-macos-bundle.sh`, and launches
  `/Applications/Operon Stable.app` (unless `--no-open`).
- `quit`: quits any running Operon process and waits up to
  `OPERON_QUIT_WAIT_SECONDS` (10 seconds) for `Operon` and `operon` processes
  to exit; exits 1 with a diagnostic if a process is still running after the
  deadline.

States:

- **Missing bundle on `promote`, `stable`, or `rollback`**: prints a clear
  message naming the missing bundle path and the subcommand that creates it,
  and exits 66 (`EX_NOINPUT`).
- **Process refuses to quit within deadline**: prints the remaining PIDs and
  exits 1 without launching a second instance or swapping under a live lock.
- **Interrupted during `promote` or `rollback`**: `trap cleanup EXIT` removes
  any temporary staging directory, and `scripts/replace-macos-bundle.sh`
  guarantees atomic swap and rollback on signal.

## Design

**Placement: `EXTEND`.** The repository already owns bundle creation and atomic
bundle swapping in `scripts/package-macos.sh`,
`scripts/replace-macos-bundle.sh`, and `scripts/check-installed-build.sh`. A
new shell script `scripts/operon-channel.sh` composes those three scripts
without modifying any of them. Route `modify`.

- `scripts/operon-channel.sh`:
  - Written in POSIX / macOS stock `/bin/bash` 3.2 syntax (`set -euo pipefail`,
    `unset GIT_DIR GIT_WORK_TREE` before git queries, no bash-4-only features).
  - Reads `source_commit_key` from `scripts/package-macos.sh` using the same
    `sed` extraction as `scripts/check-installed-build.sh`, so the stamp key
    remains spelled in one place (`scripts/package-macos.sh`).
  - Environment overrides for deterministic testing without touching `/Applications`
    or real running processes:
    - `OPERON_REPO_ROOT` (defaults to the script's parent repository)
    - `OPERON_DIST_DIR` (defaults to `$repo_root/dist`)
    - `OPERON_STABLE_APP` (defaults to `/Applications/Operon Stable.app`)
    - `OPERON_INSTALLED_APP` (defaults to `/Applications/Operon.app`)
    - `OPERON_PACKAGER` (defaults to `$repo_root/scripts/package-macos.sh`)
    - `OPERON_CHANNEL_OPEN_CMD` (defaults to `open`)
    - `OPERON_CHANNEL_QUIT_HOOK` (when set in tests, called instead of
      `osascript` / `pgrep`)
- `src/tests.rs`:
  - Three tests named in Requirements 1–5 that exercise `scripts/operon-channel.sh`
    in isolated temporary directories, plus a doc guard test
    `the_readmes_document_the_channel_cli` that verifies `README.md`,
    `README.ja.md`, and `README.ko.md` name `scripts/operon-channel.sh` and its
    subcommands (`canary`, `promote`, `stable`, `rollback`, `status`).
- `README.md`, `README.ja.md`, `README.ko.md`:
  - Document `bash scripts/operon-channel.sh` under "Package a macOS .app".

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | No UI drawing changes. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No icons added or changed. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | `OperonSourceCommit` is read out of `scripts/package-macos.sh`, never retyped; `the_channel_cli_reports_status_and_reads_its_stamp_key_from_the_packager` verifies this. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes (read) | No store or sidecar change. Bundle promotion and rollback delegate to `scripts/replace-macos-bundle.sh` (`RENAME_SWAP`) and refuse to launch or swap while an Operon process holds the instance lock. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No Rust production subprocess call sites are added; the CLI is a standalone shell script in `scripts/`. |
| Documentation — user-facing docs change in all three languages together | Yes | `README.md`, `README.ja.md`, and `README.ko.md` are updated together and held by `the_readmes_document_the_channel_cli`. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Local bundle management only; no network calls or `unsafe`. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | Process quit polling is bounded by `OPERON_QUIT_WAIT_SECONDS` (default 10 seconds, 200 iterations of 50 ms). |

## Flagged concerns

- **Promoting a bundle while Operon is running could race on the single-instance lock if `--open` is requested.** Answered: `promote` writes to `/Applications/Operon Stable.app` via `scripts/replace-macos-bundle.sh` without touching the local session index, and whenever `canary`, `stable`, `rollback`, `dev`, or `promote --open` launches a bundle, `quit_running_operon` stops any running Operon process and waits for both `Operon` and `operon` processes to exit before opening the target bundle.

## Acceptance

- `cargo test --locked the_channel_cli_` and
  `cargo test --locked the_readmes_document_the_channel_cli` pass, including:
  - `the_channel_cli_reports_status_and_reads_its_stamp_key_from_the_packager`
  - `the_channel_cli_builds_and_launches_canary_without_touching_stable`
  - `the_channel_cli_promotes_canary_to_stable_and_rolls_back`
  - `the_channel_cli_refuses_missing_bundles_and_unknown_subcommands`
  - `the_readmes_document_the_channel_cli`
- `bash scripts/operon-channel.sh status` in the repository exits 0 and prints
  the status of `canary`, `stable`, `stable-previous`, and `installed`.

## Rejected alternatives

- **Put channel management subcommands inside the compiled `Operon` binary.**
  Replacing or building the `Operon` binary while it is running or before it
  has been compiled requires an external script anyway, and a bash script in
  `scripts/` needs no compilation before first use.
- **Overwrite `/Applications/Operon.app` for both canary and stable.**
  `AGENTS.md` and `scripts/check-installed-build.sh` tie `/Applications/Operon.app`
  to the latest `main` release-binary commit; keeping the pinned stable build at
  `/Applications/Operon Stable.app` and battle-testing in `dist/Operon.app`
  prevents routine `main` merges from overwriting the pinned stable build.
