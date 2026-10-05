# Plan: give the pipeline state, a readiness gate, a screen gate, risk-driven autonomy, and a completion contract

- **Spec**: `./spec.md`
- **Approved**: 2026-09-01
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `docs/sdlc/templates/state.yaml` | new — the per-change state file, and the one place the status set is written |
| `docs/sdlc/risk.yaml` | new — surface → `<tier> <autonomy> <screen> <paths>` |
| `scripts/check-readiness.sh` | new — Go / Conditional Go / No-Go over a change directory |
| `.claude/skills/sdlc/references/state-and-resume.md` | new — statuses, transitions, what `resume` must say |
| `.claude/skills/sdlc/references/readiness-gate.md` | new — the phases, and what the script cannot judge |
| `.claude/skills/sdlc/references/screen-approval.md` | new — the ASCII-layout gate, three-revision cap |
| `.claude/skills/sdlc/SKILL.md` | becomes the orchestrator: state first, gates at the stage boundaries, references for detail |
| `.claude/hooks/gate-stop.sh` | consult `state.yaml`'s contract as well as the tree |
| `.claude/hooks/guard-stage.sh` | new — `PreToolUse(Edit\|Write)`: refuse a `src/` edit that has not passed the stage gate |
| `docs/sdlc/changes/00{1..6}-…/state.yaml` | backfilled, so `every_change_directory_carries_its_state` is honest |
| `scripts/check-transcript-vocabulary.sh` | declare `not-wired:` and why |
| `docs/sdlc/changes/008-…/state.yaml` | this change's own state — the first instance, and the fixture the guard reads |
| `docs/sdlc/changes/007-…/state.yaml` | backfilled, so the guard has two instances and one of them is `done` |
| `scripts/harness-metrics.sh` | emit `states`, `risk_surfaces`, `reference_bytes` |
| `docs/sdlc/bands.yaml` | bands for the three new metrics |
| `docs/sdlc/README.md` | the orchestrator, the state file, the gates; what was added this time |
| `docs/sdlc/lessons.md` | entry 011 |
| `src/tests.rs` | six new guards |
| `CLAUDE.md` | one pointer line |
| `AGENTS.md` | one line: read `state.yaml` first |

## Order of work

Data before the documents that read it, guards last so they can be watched
failing against files that already exist.

1. `docs/sdlc/templates/state.yaml` and `docs/sdlc/risk.yaml`. The template is
   the SSOT for the status set, so it comes first and the guard reads the set out
   of it.
2. `docs/sdlc/changes/008-…/state.yaml`, then backfill `007-…/state.yaml`. Two
   instances, one `in-progress` and one `done`, so the guard is not written
   against a single shape.
3. `scripts/check-readiness.sh`, then run it against changes 003–008 and fix what
   it finds. Running it on the existing artifacts before trusting it is the point:
   a checker that has never rejected anything is a checker nobody has tested.
4. The three `references/` files, then rewrite `.claude/skills/sdlc/SKILL.md`
   around them. Measure the skill's own size at this step — requirement 9 fails if
   the detail stayed in `SKILL.md`.
5. `.claude/hooks/gate-stop.sh` — read the contract, keep the stamp behaviour.
6. `scripts/harness-metrics.sh`, then `docs/sdlc/bands.yaml`. Metrics first: a
   band on a key nothing emits fails the existing control-bands guard.
7. `docs/sdlc/README.md`, `CLAUDE.md`, `AGENTS.md`.
8. `src/tests.rs` — the four guards.
9. `docs/sdlc/lessons.md` entry 011.
10. Verify by mutation: break a status, break a risk path, empty a flagged-concern
    answer, rename a reference. Each must fail. Restore.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The readiness checker flags legitimate prose — `<uuid>`, `Vec<Range<usize>>` — and gets ignored | a No-Go on every spec, then the script stops being run | requirement 5: backticked spans are stripped first. Verified by running it over changes 003–008, which contain both shapes |
| The status set drifts between the template and the guard | a status nothing accepts, or a guard accepting anything | the guard reads the set out of `docs/sdlc/templates/state.yaml` rather than restating it — lesson 004 |
| `state.yaml` becomes a file nobody updates, and lies | a session trusts `stage: build` on work that is done | the contract block: `gate-stop.sh` refuses a finish with a `pending` machine item, so a stale state is felt at the stop boundary rather than read silently |
| The detail lands in `SKILL.md` instead of `references/`, and every invocation pays for it | `steering_bytes` rises by ~9 KB | measured at step 4; `reference_bytes` makes the split visible |
| A reference is renamed and the skill goes on naming the old path | the skill reaches for a document that is not there | `every_reference_the_skill_names_exists`, plus `harness_documents_only_name_paths_that_exist` once it walks the references |
| The stop hook's new contract check makes it impossible to finish | a loop | the stamp is unchanged, and the contract check only names items — the refusal is still once per tree |

## Proof of completion

Measured, not estimated.

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. 308 passed; 0 failed; 6 ignored`
  (314 total, +4 from this change).
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-readiness.sh --all` — **Conditional Go, 0 blocking, 6
  warning**. No committed spec was rejected. The warnings are five changes with
  no `state.yaml` (this change introduces the file; 007 and 008 have one) and
  `002`, which has no `spec.md` because it took a bugfix-shaped route.
  The first cut *did* reject four committed specs, by demanding the literal word
  "Answer" in a flagged concern; those specs answer in ordinary prose, and the
  checker was wrong, not the corpus. Now it checks for a body, not a keyword.
- `bash scripts/harness-metrics.sh` — `states` 2, `risk_surfaces` 10,
  `reference_bytes` 11 637, `invariant_tests` 18 → 22, `routes` 6 and `rules` 4
  unchanged, `unsafe_blocks` 2 and `subprocess_sites` 29 unchanged.
- `always_loaded_bytes` 9 876. Requirement 12 asked for under 9 716 and this
  misses by 160; see the departures.
- The reference split works: `.claude/skills/sdlc/SKILL.md` grew 6 992 → 8 861
  (+1 869) while 11 637 bytes went to `references/`. Inlined, the orchestrator
  would have cost ~13 KB of steering instead of 1.9 KB.
- `.claude/hooks/gate-stop.sh` exercised directly: it names change 008's three
  pending machine contract items and skips 007, whose status is `done`.
- Mutation, watched failing rather than asserted:
  - a `status` outside the closed set → named, with the set printed;
  - a `resume` reading `"WIP"` → "次の一手を述べていません";
  - **widening the set in `docs/sdlc/templates/state.yaml` makes the guard accept
    the new value with no edit to the guard** — the positive direction, and the
    point of reading the SSOT rather than restating it;
  - a risk surface pointing at a path that is not there (src/persistence.rs, deliberately not in backticks — lesson 003) → "何も分類しない surface";
  - `SKILL.md` renaming a reference → caught twice, by
    `every_reference_the_skill_names_exists` and by the path check;
  - disabling the readiness gate's flagged-concern check → **passed on the first
    attempt.** See lesson 011; the guard was rewritten and now fails.

## The steering_bytes diagnosis, second reading

Change 007 recorded this at 127 176 and deferred the instrument defect. It is now
136 645, and the composition settles the question:

| | bytes | read before code? |
|---|---:|---|
| `DESIGN.md` | 32 109 | no — the design system, consulted when painting |
| `docs/sdlc/lessons.md` | 22 162 | no — an append-only ledger, consulted at stage 6 |
| `docs/sdlc/README.md` | 18 891 | no — the pipeline, consulted when running it |
| everything else | 63 483 | mostly no: skills load when invoked, templates when copied |
| **`always_loaded_bytes`** | **9 876** | **yes — and it is healthy, under its warn tier** |

`steering_bytes`' own comment says it measures "what an agent pays before it reads
a single line of code". Two files are 40 % of it and neither is loaded before
anything. The metric measures total harness prose, which is a real quantity, but
its band then fires for recording a lesson and for documenting a decision — the
two things stage 6 exists to make people do. It has now fired three times, each
time for exactly that, and each time the honest response was to write more prose
explaining why.

Not fixed here, for the reason given in `spec.md`. The diagnosis is complete
enough that the change which fixes it is mechanical: split the aggregate into
`always_loaded_bytes` (exists), `conditional_steering_bytes` (exists),
`reference_bytes` (exists), and a `ledger_bytes` for the append-only documents,
then band each for what it actually is. That is change 009's whole content.

## Departures from the plan

- **The plan had no enforcement step, and that was its central defect.** Steps 1-9
  built tables, a script, references, and guards; nothing wired the tables to
  anything that could refuse, and step 10's mutations could not see it because
  every mutation deleted part of a mechanism and watched a guard fail — none asked
  whether anything *but* the guard would have noticed. Found by being asked what
  had actually been built. `.claude/hooks/guard-stage.sh`,
  `every_gate_script_is_wired_or_declares_why_not`, and
  `every_change_directory_carries_its_state` were added afterwards, and
  requirement 11 in `spec.md` now says so. Lesson 012.
- **Six state files backfilled, not two.** The plan backfilled only 007, to give
  the guard a second shape. Making `every_change_directory_carries_its_state` a
  real invariant meant all six, and doing it surfaced change 002: parked at
  `draft` for months with nothing in the repository saying so. It is now
  `awaiting-user`.
- **The backfill was caught cutting corners.** Four of the six wrote
  `three-gates: "machine passed"` with no third field, and
  `every_state_file_names_a_route_and_a_stage_that_exist` refused them. The guard
  worked on its author.

- **The readiness gate's first cut rejected four committed specs.** It demanded
  the literal word "Answer" in a flagged concern. Changes 001, 004, 005, and 007
  answer theirs in prose. Running the checker over the existing corpus before
  trusting it — step 3 of the plan, and the only reason this was caught — showed
  the checker was wrong. It now requires a body of at least 80 characters, which
  the template's own 43-character stub does not clear.
- **The readiness guard passed with the gate switched off.** Written up as lesson
  011. Not a departure from the plan so much as the plan's step 10 doing its job:
  the plan said watch each guard fail, and one of them did not.
- **`always_loaded_bytes` is 9 876, not under 9 716.** Requirement 12 also said
  "at most one pointer line" in `CLAUDE.md`; the orchestrator needs `state.yaml`,
  `routes.yaml`, and `risk.yaml` to be discoverable, which is three. The
  requirement was written before the shape was known and it was wrong, not the
  implementation. Genuine duplication was removed on the way — `AGENTS.md` is
  `@`-imported into `CLAUDE.md`, so the routing pointer had been paid for twice.
  The reading is well inside the warn tier at 11 000, and `always_loaded_bytes`
  went 12 392 → 9 876 across changes 007 and 008 together.
- **A shared `flat_block` parser.** `control_bands` and `work_routes` each had
  their own copy of the same twenty lines, and this change would have added a
  third and fourth for `state.yaml` and `risk.yaml`. Four parsers for one file
  format is the identifier-SSOT rule wearing a different hat, so they were
  collapsed into one.
- **`states` has no band.** The count rises as changes accumulate and the five
  pre-008 changes have no state file, so a `min` band would be noise and a `max`
  band would punish doing the work. It is emitted and watched, not banded.
