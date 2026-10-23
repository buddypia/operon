# Plan: The channel CLI focuses exclusively on the `canary` and `stable` channels

- **Spec**: `./spec.md`
- **Approved**: 2026-10-24
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `scripts/operon-channel.sh` | Narrow `status` to `canary` and `stable`, add `--promote` (`-p`) and `--rollback` (`-r`) to `stable`, and remove `dev` |
| `src/tests.rs` | Update the five `the_channel_cli_*` and `the_readmes_document_the_channel_cli` tests to assert the two-channel contract |
| `README.md`, `README.ja.md`, `README.ko.md` | Update the channel CLI section in all three languages to show `canary` and `stable` (`--promote`, `--rollback`) |

## Order of work

1. Validate readiness (`bash scripts/check-readiness.sh`) and record evaluator approvals in `approvals.log`.
2. Update the five tests in `src/tests.rs` to assert the two-channel contract and watch them fail against the current `scripts/operon-channel.sh` and READMEs.
3. Update `scripts/operon-channel.sh`, `README.md`, `README.ja.md`, and `README.ko.md`.
4. Run `make q.fast`, `cargo test --locked channel_cli`, and `bash scripts/check-bands.sh`.
5. Write and run `mutations.py` in the change directory.
6. Run `rust-reviewer` and `subprocess-safety-reviewer`, write `review.yaml`, settle `state.yaml` to `done`, commit, push, wait for CI, merge to `main`, and clean up the worktree.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| `status` still prints `installed:` or `dev` still runs | Test failure in `the_channel_cli_reports_status_and_reads_its_stamp_key_from_the_packager` or `the_channel_cli_refuses_missing_bundles_and_unknown_subcommands` | `cargo test --locked channel_cli` |
| `stable --promote` consumes `dist/Operon.app` instead of staging a copy | `dist/Operon.app` missing after `stable --promote` | `the_channel_cli_promotes_canary_to_stable_and_rolls_back` |
| Trilingual README drift | One README misses `stable --promote` or `stable --rollback` | `the_readmes_document_the_channel_cli` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past compile lines.
- `cargo test --locked channel_cli` — `test result: ok. 5 passed; 0 failed`.
- `python3 docs/sdlc/changes/216-the-channel-cli-exposes-only-canary-and-stable-workflows/mutations.py` — catches all mutations.

## Departures from the plan

- None.
