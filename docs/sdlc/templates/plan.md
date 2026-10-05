# Plan: <the change>

- **Spec**: `./spec.md`
- **Approved**: YYYY-MM-DD
- **Status**: approved | in progress | done | departed from (see below)

This is the plan produced in plan mode and accepted before any file was edited.
If the implementation departs from it, update this file — an abandoned plan is
worse than no plan, because the next reader trusts it.

## Files that change

| File | Change |
|---|---|
| `src/…` | |

## Order of work

Numbered, so that the tree compiles between steps wherever it can. Say where it
cannot.

1.
2.

## Risks

What could break that the compiler will not catch. For each one, what makes it
visible.

| Risk | How it shows up | What catches it |
|---|---|---|
| | | |

## Proof of completion

The commands and observations that say this is done. Include the output that
counts as healthy, not just the command.

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- <the new test that fails before this change and passes after>
- <what to look at in the running app>

## Departures from the plan

Filled in during implementation. What changed, and why the plan was wrong.
