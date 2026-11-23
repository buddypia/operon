# Plan: nothing proves a change in the running application

- **Spec**: `./spec.md`
- **Approved**: 2026-11-23
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `scripts/verify-app.py` | new: launch, doctor, run, eval-step, shot, tree, stop |
| `src/tests.rs` | eight tests, one per requirement |
| `docs/features/palette/spec.md` and at least two more feature specs | the section `## Seen in the running app`, only for journeys run live |
| `.claude/skills/feature-pilot/references/verify-in-the-app.md` | new: the five parts |
| `.claude/skills/feature-pilot/SKILL.md` | the verify step names the reference |
| `.claude/skills/sdlc/references/screen-approval.md` | the screen is looked at again after the build |
| `docs/sdlc/README.md` | one row in the built table |
| `docs/sdlc/changes/314-…/` | `verification.yaml`, `mutations.py`, the records |

Risk surfaces of `docs/sdlc/risk.yaml`: none of these files is under a `paused`
surface, and none under `gate-configuration`. `harness` (medium) is reached
only through the byte counts `scripts/harness-metrics.sh` reads.

## Order of work

1. The tests for requirements 1, 2, 5, 6 and 8, watched failing (no script).
2. `scripts/verify-app.py`: `launch --plan`, `stop`, `eval-step`, the seed,
   then `launch`, `doctor`, `run` and the evidence, until those five pass.
3. Run it live against the empty home and the seeded project.
4. The tests for requirements 3 and 4 (`every_step_seen_in_the_running_app_…`,
   `verify_app_lists_the_features_whose_documents_carry_steps`), watched
   failing; then the steps for the features that hold live.
5. The test for requirement 7 (`the_pipeline_sends_a_change_that_paints_to_the_running_app`),
   watched failing; then the reference and the three documents.
6. `mutations.py`, `verification.yaml`, the verifiers, the records.

The tree compiles between every step: only `src/tests.rs` changes in the crate.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A run touches the person's data or tmux | their sessions change | requirement 1's test; the acceptance check of the real data folder's modification time |
| `stop` ends the installed Operon | the person's window closes | requirement 2's test; the pid is matched to the recorded binary path |
| Steps go stale as strings change | a run fails on a renamed label | requirement 3's test fails first, with no window |
| A window left open after a failed run | a stray Operon on screen | requirement 8's test fails a step and looks for the process and the folder |
| A run interrupted by a signal | a stray Operon and scratch folder | the handler stops what was started; checked by hand once, since the suite cannot signal itself mid-run cheaply |
| A stale binary is driven | evidence of old code | `doctor` compares the binary with the newest file under `src/` |
| No accessibility permission | every step fails to read | `doctor` exits 2 with the setting to change, before any step |
| The always-loaded text or steering grows a tier | `check-bands.sh` | the contract's bands item |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked verify_app_`, then `cargo test --locked every_step_seen_in_the_running_app`,
  then `cargo test --locked the_pipeline_sends_a_change_that_paints` — each
  passes; the whole suite in CI.
- `python3 docs/sdlc/changes/314-nothing-proves-a-change-in-the-running-app/mutations.py`
  — every mutant caught.
- `python3 scripts/verify-app.py run palette` — exit 0, an evidence folder with
  a picture of the open palette.

## Departures from the plan
