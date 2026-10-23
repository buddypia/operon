# Plan: A channel CLI builds, switches, promotes, and rolls back canary and stable bundles

- **Spec**: `./spec.md`
- **Approved**: 2026-10-24
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `scripts/operon-channel.sh` | New channel management CLI (`status`, `canary`, `dev`, `promote`, `stable`, `rollback`, `quit`) delegating to `scripts/package-macos.sh`, `scripts/replace-macos-bundle.sh`, and `scripts/check-installed-build.sh` |
| `src/tests.rs` | Five tests covering status/stamp-key SSOT, canary build/launch, promote/rollback, missing-bundle/usage refusals, and README documentation sync |
| `README.md`, `README.ja.md`, `README.ko.md` | Document `bash scripts/operon-channel.sh` under the macOS packaging section in all three languages |

## Order of work

1. Run `bash scripts/check-readiness.sh` and obtain evaluator approvals for
   `intent.md`, `spec.md`, and `plan.md`; commit the planning artifacts.
2. Add the five tests in `src/tests.rs` (`the_channel_cli_*` and
   `the_readmes_document_the_channel_cli`) and watch them fail before
   `scripts/operon-channel.sh` and the README updates exist.
3. Implement `scripts/operon-channel.sh` (compatible with `/bin/bash` 3.2) and
   update `README.md`, `README.ja.md`, and `README.ko.md`.
4. Run `make q.fast` and `cargo test --locked channel_cli` to verify all new
   and existing script/harness tests pass.
5. Write and run `mutations.py` in the change directory, including claim
   mutations on the script's output and README text.
6. Run `rust-reviewer` and `subprocess-safety-reviewer`, record verdicts in
   `review.yaml`, settle `state.yaml` to `done`, commit, push, wait for CI,
   merge to `main`, and clean up the worktree.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Bash 4 syntax in `scripts/operon-channel.sh` fails under macOS `/bin/bash` 3.2 | `every_shell_script_in_the_repository_parses_under_macos_stock_bash` fails | `cargo test --locked bash` and `/bin/bash -n scripts/operon-channel.sh` |
| `promote` consumes `dist/Operon.app` instead of staging a copy | `dist/Operon.app` disappears after `promote` | `the_channel_cli_promotes_canary_to_stable_and_rolls_back` asserts `dist/Operon.app` remains intact after `promote` |
| `source_commit_key` literal retyped in `scripts/operon-channel.sh` | Drift if the packager key ever changes | `the_channel_cli_reports_status_and_reads_its_stamp_key_from_the_packager` checks `operon-channel.sh` reads the key from `package-macos.sh` |
| Documentation updated in one language only | `README.md`, `README.ja.md`, and `README.ko.md` drift | `the_readmes_document_the_channel_cli` asserts all three READMEs document the script and its subcommands |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked the_channel_cli_` and
  `cargo test --locked the_readmes_document_the_channel_cli` — `test result: ok. 5 passed; 0 failed`.
- The five new tests fail before `scripts/operon-channel.sh` and the README
  updates are added.
- `mutations.py` in the change directory catches every mutation.
- `bash scripts/operon-channel.sh status` prints the current channel status and
  exits 0.

## Departures from the plan

- None.
