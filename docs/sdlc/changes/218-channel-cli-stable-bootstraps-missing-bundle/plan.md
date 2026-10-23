# Plan: `operon-channel.sh stable` bootstraps `/Applications/Operon Stable.app` from `dist/Operon.app` on first run

- **Spec**: `./spec.md`
- **Approved**: 2026-10-24
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `scripts/operon-channel.sh` | In `cmd_stable`, when `$stable_app` does not exist and `$canary_app` exists, delegate to `do_promote "$no_open"`; when neither exists, exit `66` with a `canary --build` hint |
| `src/tests.rs` | Update `the_channel_cli_promotes_canary_to_stable_and_rolls_back` to test first-run `stable` bootstrap + non-overwriting subsequent `stable` launch + `stable --promote` + `stable --rollback`, and update `the_channel_cli_refuses_missing_bundles_and_unknown_subcommands` |

## Order of work

1. Validate readiness (`bash scripts/check-readiness.sh`) and record evaluator approvals in `approvals.log`.
2. Update `the_channel_cli_promotes_canary_to_stable_and_rolls_back` and `the_channel_cli_refuses_missing_bundles_and_unknown_subcommands` in `src/tests.rs` and watch them fail before editing `scripts/operon-channel.sh`.
3. Update `cmd_stable` in `scripts/operon-channel.sh`.
4. Run `make q.fast`, `cargo test --locked channel_cli`, and `bash scripts/check-bands.sh`.
5. Write and run `mutations.py` in the change directory.
6. Run `rust-reviewer` and `subprocess-safety-reviewer`, write `review.yaml`, settle `state.yaml` to `done`, commit, push, wait for CI, merge to `main`, and clean up the worktree.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Plain `stable` overwrites an already-existing `$stable_app` when `$canary_app` changes | `$stable_app/marker` unexpectedly updated on plain `stable` | `the_channel_cli_promotes_canary_to_stable_and_rolls_back` |
| Missing both `$stable_app` and `$canary_app` does not exit `66` | Exit code != 66 in `the_channel_cli_refuses_missing_bundles_and_unknown_subcommands` | `the_channel_cli_refuses_missing_bundles_and_unknown_subcommands` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past compile lines.
- `cargo test --locked channel_cli` — `test result: ok. 5 passed; 0 failed`.
- `python3 docs/sdlc/changes/218-channel-cli-stable-bootstraps-missing-bundle/mutations.py` — catches all mutations.

## Departures from the plan

- None.
