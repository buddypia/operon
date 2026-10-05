# Plan: apply the AI-native SDLC playbook to Operon

- **Spec**: `./spec.md`
- **Approved**: 2026-08-27
- **Status**: done, with departures — see below

## Files that change

| File | Change |
|---|---|
| `docs/sdlc/README.md` | the pipeline: six stages, roles, environment tiers, skip conditions, measurement, and what this project deliberately does not do |
| `docs/sdlc/templates/` | `intent.md`, `spec.md`, `plan.md` — the spec template's policy table is where stage 2 does its work |
| `docs/sdlc/lessons.md` | the incident ledger, seeded with four entries, each with a Guard column |
| `docs/sdlc/bands.yaml` | control bands and response tiers for sixteen harness metrics |
| `REVIEW.md` | the review policy: four passes, Important vs. Nit, exclusions, human threshold |
| `.claude/skills/sdlc/SKILL.md` | runs a change through the stages |
| `.claude/hooks/guard-write.sh` | generated-output and credential guards (was inline in `settings.json`) |
| `.claude/hooks/guard-bash.sh` | the release gates: two `deny`, four `ask` |
| `.claude/hooks/gate-commit.sh` | the three gates plus the test-erosion check, at the commit boundary |
| `.claude/settings.json` | wires the three hooks; keeps the post-edit `cargo fmt` and the allowlist |
| `scripts/check-bands.sh` | judges a metrics reading against the bands |
| `scripts/run-evals.sh` | runs each eval in a detached worktree of a committed ref |
| `scripts/harness-metrics.sh` | six new counters; `steering_bytes` widened to the new documents |
| `evals/` | README plus five evals, each seeded from a real incident |
| `.github/workflows/` | `harness.yml`, `claude-review.yml`, `scheduled-scan.yml` |
| `src/tests.rs` | four new invariant tests, three existing ones widened, one guard de-literalised |
| `src/i18n_tables.rs` | doc comment corrected to describe what is true |
| `CLAUDE.md`, `AGENTS.md`, `CONTRIBUTING.md`, `.github/PULL_REQUEST_TEMPLATE.md` | pointers into the above |

## Order of work

1. Hooks first, unwired — they are the part that can break the session that
   writes them, so they get written and hand-exercised before `settings.json`
   points at them.
2. Stage 1–2 documents and templates, then `REVIEW.md`, then the `sdlc` skill.
3. `bands.yaml` and `check-bands.sh` together; the file and its parser are one
   unit.
4. `evals/` and `run-evals.sh`.
5. Workflows.
6. Wire `settings.json`.
7. Steering-document edits.
8. `src/tests.rs` last, because it checks everything above and cannot be written
   before the things it checks exist.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A hook blocks legitimate work | the session cannot commit or write | each hook exercised by hand across its whole decision table before wiring |
| A document names a mechanism that is not wired | reads as coverage, does nothing | `every_hook_the_settings_file_wires_exists_and_is_executable`, in both directions |
| A band watches a key the script does not print | never fires, reads as green | `the_control_bands_only_name_metrics_the_metrics_script_emits` |
| `steering_bytes` grows without limit | every task pays for it | a band on `steering_bytes`, and the requirement to report the delta |
| The path check trips on a document *discussing* a removed path | a true statement fails a true rule | it did — see Departures |

## Proof of completion

- `bash scripts/check-bands.sh` — `bands: 15 metrics within their bands`, with
  `gate_seconds` skipped because it is only measured under `--gates`.
- `actionlint .github/workflows/*.yml` — no output.
- `bash scripts/run-evals.sh --list` — five evals.
- Every hook decision exercised by hand: 8 write cases, 10 bash cases, 3 commit
  cases.
- The metric delta, reported.
- `cargo fmt --check`, `cargo test --locked`, `cargo clippy --locked -- -D warnings`
  — **not run**. See Departures.

## Departures from the plan

**The crate does not compile, and did not before this change.** An unfinished
i18n migration is in the working tree *and in the index*: 175 compile errors.
`cargo test` and `cargo clippy` therefore could not run, and `cargo fmt` cannot
walk the module tree at all. Ten of those errors were unambiguous syntax damage
and were repaired here; the rest are a separate piece of work, written up as
`docs/sdlc/changes/002-i18n-migration-incomplete/intent.md`.

The four new tests are written but unexecuted. What they assert was verified by
reimplementing each check in shell — the content holds; the Rust that asserts it
is uncompiled. This is stated rather than papered over, and it is the reason
this plan's status is "done, with departures".

**The path check needed an escape it was not designed with.** Widening
`harness_documents_only_name_paths_that_exist` to source comments immediately
failed on `docs/sdlc/lessons.md` and on the new test's own doc comment: both
correctly record that a path no longer exists, and both named it in backticks.
The rule is right and the documents were right; what was missing was the
distinction between a name being pointed at and a name being discussed. Removed
paths are now written without backticks, and both places say why so the next
reader does not helpfully restore them.

**`invariant_tests` rose to 15, not 16.** The plan's arithmetic was wrong: eleven
plus four is fifteen. The band baseline is set from the measurement, not from the
plan.

**`/Applications/Operon.app` was not replaced.** `AGENTS.md` asks for it after
every change, and it cannot be done: packaging requires a build, and the crate
does not build. Nothing in this change alters the binary — every Rust edit is in
`src/tests.rs`, in a doc comment, or a syntax repair of code that could not
compile — so the installed app is not stale with respect to it.
