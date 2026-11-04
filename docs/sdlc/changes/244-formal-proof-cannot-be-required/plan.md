# Plan: formal proof cannot be required until Kani is measured

- **Spec**: n/a until the probe has measured; see `./state.yaml`
- **Approved**: 2026-11-04
- **Status**: approved

## Order of work

1. **Probe.** `.github/workflows/kani-probe.yml`, on this branch only: install
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
| `.github/workflows/kani-probe.yml` | added for the probe, removed before landing |
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
