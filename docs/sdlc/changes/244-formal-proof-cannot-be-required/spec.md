# Spec: formal proof is required where it serves

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. A change that touches `src/agents.rs` is refused unless its record holds a
   `formal-proof` obligation, and one that names no `#[kani::proof]` function
   in `src/tests.rs` — or a function without that attribute directly above it
   — is refused as evidence of nothing.
   `a_launch_gate_change_must_name_a_proof_harness_that_exists`.
2. The CI `verify` job installs Kani 0.68.0, the measured version, before it
   runs the obligations, and only when a record of the branch names
   `formal-proof`; the method's command shape is the one that reaches a
   harness in the test module (`cargo kani --tests --harness `).
   `the_verify_job_installs_kani_before_it_runs_a_recorded_proof`.
3. The harness is invisible to every build but `cargo kani`, with warnings as
   errors. `mutations.py` (`harness-outside-cfg-kani`, `cfg-kani-undeclared`).

## Behaviour

No person-facing behaviour. A refused record reads, from the commit gate:
`❌ BLOCK — <change> · <id> の「<command>」に当たる #[kani::proof] が src/tests.rs にありません`
and, when the proof is missing,
`❌ BLOCK — <change> · launch-gate には formal-proof の検証が必要です`.

## Design

As the plan's departures table. Placement: `EXTEND` — the evidence table and
the gate grow a row and a check; nothing new is a module.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | no | nothing paints |
| Icons | no | nothing paints |
| Identifier SSOT | yes | the surface name and the prefix are read out of the real tables by `the_verify_job_installs_kani_before_it_runs_a_recorded_proof` |
| Durability | no | the store is untouched |
| Subprocess safety | no | no child process in `src/`; Kani runs in CI only |
| Documentation | no | no user-facing document describes the evidence table |
| Local-first | yes | Kani is never a crate dependency; `Cargo.toml` only declares `cfg(kani)` |
| Budgets | yes | a proof obligation runs under the existing 900 s ceiling; measured 419 s |

## Flagged concerns

None open.

## Acceptance

- `cargo test --locked` passes, including the two tests above.
- `python3 docs/sdlc/changes/244-formal-proof-cannot-be-required/mutations.py` catches every mutation.
- The probe runs recorded in `plan.md`: the proof succeeds, and fails with the newline put back.

## Rejected alternatives

- Requiring formal-proof on the whole `subprocess` surface: `src/git.rs` and
  `src/tmux.rs` hold no pure gate a proof could serve.
- Installing Kani in every `verify` run: seven minutes on branches that record no proof.
