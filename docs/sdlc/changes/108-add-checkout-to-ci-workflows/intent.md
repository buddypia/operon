# Add checkout step to CI and Harness workflows

## Problem

The CI and Harness workflows were previously never executed in GitHub Actions because the repository had no git remote.
When triggered upon push, `ci.yml` (fmt / clippy / test) and `harness.yml` (invariants / metrics / bands) failed immediately with:
`error: could not find Cargo.toml in /Users/runner/work/operon/operon or any parent directory`
because `actions/checkout@v4` was missing from the steps.

## Proposed Change

- Add `- uses: actions/checkout@v4` to `checks` job in `.github/workflows/ci.yml`.
- Add `- uses: actions/checkout@v4` to `harness` job in `.github/workflows/harness.yml`.

## Risk and Review

Low risk. CI configuration only; does not affect application runtime logic.
