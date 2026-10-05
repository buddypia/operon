# Spec: <the change>

- **Intent**: `./intent.md`
- **Status**: draft | approved | superseded

## Requirements

Numbered, each one independently checkable. A requirement nobody can fail is not
a requirement.

1.
2.

## Behaviour

What the user sees and does, including the states that are not the happy one:
empty, loading, interrupted, denied, and too large. Japanese strings for anything
a person reads; describe them here so review does not discover them in the diff.

## Design

The shape of the solution: which modules gain what, which types change, what
crosses a process boundary. Enough that `plan.md` is a matter of ordering.

## Policy conformance

Answer each line, or state that it does not apply. These are the policies this
repository enforces with tests, so "does not apply" is a claim the suite checks.

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | | |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | | |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | | |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | | |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | | |
| Documentation — user-facing docs change in all three languages together | | |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | | |
| Budgets — any new scan or output path states its byte and item ceiling | | |

## Flagged concerns

Things this spec cannot settle on its own. Each needs an answer before stage 3
begins; record the answer inline rather than in a separate thread.

- **<concern>** — why it is unresolved, what would resolve it.

## Acceptance

How the finished change is judged, as commands and observations:

- `cargo test --locked` passes, including <the new test names>.
- <what to look at in the running app, and what it should show>

## Rejected alternatives

One line each, with the reason. This is the section that stops the same argument
from being had twice.
