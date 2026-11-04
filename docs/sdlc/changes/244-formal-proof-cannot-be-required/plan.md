# Plan: formal proof cannot be required until Kani is measured

- **Spec**: n/a until the probe has measured; see `./state.yaml`
- **Approved**: 2026-11-04
- **Status**: approved

## Order of work

1. **Probe.** the probe workflow (kani-probe.yml, removed before landing), on this branch only: install
   `kani-verifier`, `cargo kani setup`, then
   `cargo kani --tests --harness proof_safe_agent_command_admits_no_line_break`,
   each step timed into the job summary. The harness sits in `src/tests.rs`
   under `#[cfg(kani)]`, so neither the build nor the suite sees it;
   `Cargo.toml` declares `cfg(kani)` so rustc does not warn about it.
2. **Read the run.** Three outcomes, decided by the measurement and not before:
   - **It verifies, in a time CI can pay on each change that needs it**: the
     `verify` job learns to install Kani when an obligation names
     `formal-proof`, and `verification-methods.yaml` requires `formal-proof` on
     the surfaces it can serve. That is a tightening of gate configuration;
     its own plan section is written then, before the edit.
   - **It verifies, but too slowly to pay per change**: the table says so with
     the numbers, and formal-proof stays optional.
   - **It does not build this crate**: the table says why, with the error, and
     formal-proof stays optional.
3. **Remove the probe workflow** in every outcome: it runs on this branch only,
   and a landed workflow nobody triggers is a file that only looks like a check.
   The harness stays if it verified.

## Files that change

| File | Change |
|---|---|
| the probe workflow (kani-probe.yml, removed before landing) | added for the probe, removed before landing |
| `src/tests.rs` | the `#[cfg(kani)]` harness |
| `Cargo.toml` | `[lints.rust] unexpected_cfgs` declares `cfg(kani)` |
| `docs/sdlc/verification-methods.yaml` | the measured verdict on `formal-proof` |
| `docs/sdlc/changes/244-…/` | the records |

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The probe takes a sixth macOS job while a landing runs five | another branch's CI queues | runs on this branch only, and is removed |
| Kani's toolchain differs from 1.88 and the crate does not compile under it | the build step fails | that is outcome three, recorded with its error |
| The `cfg(kani)` declaration hides a misspelt cfg | a typo in another cfg is not warned about | `check-cfg` names exactly `cfg(kani)`; every other cfg is still checked |

## Departures from the plan

- **Measured: outcome one.** CI run 41802936778 on `ff32070`, `macos-latest`:
  `kani-verifier` 0.68.0, install 29 s (a prebuilt binary), `cargo kani setup`
  11 s, `cargo kani --tests --harness proof_safe_agent_command_admits_no_line_break`
  419 s — about 325 s compiling the crate, llama-cpp-sys-2 and eframe included,
  and 94 s proving: 686 checks, 0 failed, `VERIFICATION:- SUCCESSFUL`. That
  fits the 900 s each obligation is given in the `verify` job.
- **So formal-proof becomes requirable, where it serves.** Kani proves
  properties of pure functions over symbolic input; in this crate those are
  the launch gates in `src/agents.rs`. Written before the edit:

  | File | Change |
  |---|---|
  | `docs/sdlc/risk.yaml` | a surface `launch-gate`, `medium supervised no src/agents.rs` |
  | `docs/sdlc/verification-methods.yaml` | `launch-gate: "formal-proof"`; the `formal-proof` prefix becomes `cargo kani --tests --harness ` (the harnesses sit in the test module); the comment states the measurement |
  | `scripts/check-review.sh` | a `formal-proof` obligation must name a `#[kani::proof]` function that exists in `src/tests.rs`, as a `cargo test` one must name a test |
  | `.github/workflows/ci.yml` | the `verify` job installs Kani 0.68.0 when an obligation of the branch names `formal-proof`, and only then |
  | the probe workflow (kani-probe.yml, removed before landing) | also watches the harness fail with the newline put back into the gate; removed before landing |

  All of it adds a requirement; none removes or narrows one, so the
  evaluator decides it (`approval.md`, gate configuration, tightening). A
  change to `src/agents.rs` from here on records the existing harness as its
  `formal-proof` obligation, which costs its CI run about seven minutes.
- **The harness fails on the defect it guards.** CI run 41806210171 on
  `348940a`: with `'\n'` put back into the gate's whitespace arm, Kani reports
  `VERIFICATION:- FAILED` on `assertion failed: !command.trim().contains(['\n',
  '\r', ';'])`; unmutated, the same run proved it again in 380 s. The probe
  workflow is removed in the commit after it.
- **This change records no formal-proof obligation of its own**: the method
  table that judges its commits is main's, whose prefix is still the old one.
  The two probe runs above are its proof evidence; the first change to
  `src/agents.rs` after it lands records the harness under the new prefix.
