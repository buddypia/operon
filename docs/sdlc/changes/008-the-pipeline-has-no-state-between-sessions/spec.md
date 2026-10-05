# Spec: give the pipeline state, a readiness gate, a screen gate, risk-driven autonomy, and a completion contract

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Each change directory carries `state.yaml`: a flat, positional, parser-proof
   file naming the route, the stage, the status, attempts spent against the
   route's ceiling, questions asked against a ceiling, the autonomy level, the
   risk tier, the readiness verdict, and the screen-approval verdict.
2. `state.yaml` carries a `resume` line — one sentence that tells a session
   arriving cold what to do next — and a `contract` block of completion items,
   each with a verdict and the command that decides it.
3. The statuses a change may be in are a closed set, and it includes the three
   that are not progress: `awaiting-user`, `blocked`, `failed`. A session that
   cannot proceed records which one, rather than stopping without a record.
4. `scripts/check-readiness.sh <change-dir>` decides whether a spec may enter
   stage 3, exiting `0` for Go, `2` for Conditional Go, and `1` for No-Go. It
   fails a spec that keeps a template placeholder, a `TODO`/`TBD`/`FIXME`, a
   `Status:` still reading `draft`, an empty Policy-conformance cell, a flagged
   concern with no answer, a missing mandatory section, or an Acceptance section
   that covers none of its numbered requirements.
5. The readiness checker ignores anything inside backticks. `Vec<Range<usize>>`
   and `<uuid>` appear in committed specs as code and prose; a checker that
   flagged them would be a checker people learn to skip. Lesson 003's rule, one
   document over.
6. `docs/sdlc/risk.yaml` maps repository surfaces to a risk tier, an autonomy
   level, and whether a change touching them needs its screen approved before
   implementation. Its content is `REVIEW.md`'s "when a human must look" list
   made machine-readable, not a new policy.
7. A change whose files match a `screen yes` surface records `screen: pending`
   and may not enter stage 3 until it reads `approved`. The approval is an ASCII
   layout of the screen, agreed before the code exists, capped at three revisions.
8. `.claude/hooks/gate-stop.sh` reads `state.yaml` when one exists and refuses a
   finish whose contract still has a `pending` machine item, naming the item and
   its command.
9. The `sdlc` skill is the single entry point and holds the orchestration; its
   detail lives in `.claude/skills/sdlc/references/`, which loads only when the
   skill decides it needs it. No new rival entry point is created.
10. `scripts/harness-metrics.sh` emits `states`, `risk_surfaces`, and
    `reference_bytes`, all banded. Reference prose that nothing counts is hidden
    cost, and this change is mostly reference prose.
11. Every mechanism above is **wired to something that can refuse**, not only to
    a guard that checks it is well-formed. A gate script is invoked by a hook or a
    workflow, or declares in its own header why it must not be. Added after the
    fact: the first pass shipped seven tables and two enforcers, and the gap was
    found by being asked what had actually been built. See lesson 012.
12. Every mechanism above has a guard in `src/tests.rs`. The readiness gate's
    guard runs the script against a spec built to fail and against a committed
    spec that should pass — its behaviour, not its existence.
13. `CLAUDE.md` grows by at most one pointer line, and `always_loaded_bytes`
    stays under its 9 716 baseline.

## Behaviour

No user-visible behaviour changes; `src/` gains tests only.

What changes is what a session knows when it arrives. Today it opens
`docs/sdlc/changes/007-…/plan.md`, reads `Status: done`, and has to read the
whole document to learn anything more. After this change it reads `state.yaml`
and gets, in nine lines: the route, the stage, `2 of 3` attempts spent, `high`
risk, `paused` autonomy because the change touches `.claude/hooks/`, readiness
`go`, screen `n/a`, and a `resume` line saying what to do next.

The states that are not progress are the point of the closed set. A session that
has asked its seventh question moves to `awaiting-user` and records the
assumptions it adopted instead of asking an eighth. A session that has failed the
route's ceiling moves to `failed` and writes the handoff change 007 added. Both
are currently expressed by the session simply stopping, which is indistinguishable
from the session finishing.

The readiness gate is deliberately blunt where it can be and silent where it
cannot. It does not judge whether a design is good. It judges whether the document
is *filled in*, which is the failure that actually happens: change 007's own spec
went to `approved` carrying five stale backticked paths.

## Design

| New file | What it is |
|---|---|
| `docs/sdlc/templates/state.yaml` | the per-change state file, as a template |
| `docs/sdlc/risk.yaml` | surface → `<tier> <autonomy> <screen> <paths>` |
| `scripts/check-readiness.sh` | the readiness gate; Go / Conditional / No-Go |
| `.claude/skills/sdlc/references/state-and-resume.md` | the status set, the transitions, what `resume` must say |
| `.claude/skills/sdlc/references/readiness-gate.md` | the phases, what each rejects, and what the script cannot judge |
| `.claude/skills/sdlc/references/screen-approval.md` | the ASCII-layout gate and its three-revision cap |

### Design decisions and rationale

| Component | Design | Rationale |
|---|---|---|
| State format | `state.yaml`, flat positional | A flat file cannot drift from its parser — the property `docs/sdlc/bands.yaml` and `docs/sdlc/routes.yaml` were already written for. |
| State transitions | Six stages as `stage`, and a closed `status` set (`in-progress`, `awaiting-user`, `blocked`, `failed`, `done`, `abandoned`) | Clear distinction between sequential pipeline progress (`stage`) and execution condition (`status`). |
| Resume summary | Single `resume` line | Immediate context restoration for returning sessions in a single human-readable line. |
| Readiness Gate | `scripts/check-readiness.sh` | One standalone bash script owning validation phases (structural, integrity, policy conformance) without external runtime dependencies. |
| Product scope | Focus on local tool workflow | Local-first desktop tool: streamlined checks without unnecessary product funnels, paywalls, or remote telemetry overhead. |
| Screen approval gate | `references/screen-approval.md` | ASCII layout, three-way verdict, three-revision cap, Before/After on UI modifications. |
| Interaction limit | `questions: "<asked> <ceiling>"` and `awaiting-user` | Caps repetitive queries and prevents autonomous loops from spinning indefinitely. |
| Risk classification | `docs/sdlc/risk.yaml` | Scoped to Operon's specific surfaces: persisted store, `unsafe`, `PATH` setup, bundle swap, gate configuration. |
| Completion contract | `contract` block checked by `.claude/hooks/gate-stop.sh` | Enforced mechanically by commit and stop hooks rather than advisory self-reporting. |
| Skill footprint | Six direct pipeline stages | Lean harness design; stages directly structure the workflow without redundant sub-skill layers. |

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | nothing is painted. The screen-approval gate governs future changes that paint; it adds no role itself |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | no glyph is drawn |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | the status set, the stage names, and the tier names are each written once. Stages come from the same six the routes guard checks; the status set lives in `docs/sdlc/templates/state.yaml` and the guard reads it from there rather than restating it |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | `state.yaml` is a committed document, not persisted application state. Nothing under `~/.operon` changes |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | no new `Command::new` in `src/`. `scripts/check-readiness.sh` is bash reading files; the stop hook already spawns `git` |
| Documentation — user-facing docs change in all three languages together | No | `README.md` and its translations are unchanged; every document touched is harness-internal English |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | nothing reaches the network; no dependency added; `unsafe_blocks` stays at 2 |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | `reference_bytes` is the ceiling for the reference tier and is banded. `scripts/check-readiness.sh` reads one directory of markdown and prints a bounded verdict |

## Flagged concerns

- **`steering_bytes` is at `diagnose` before this change starts.** — Answer: the
  bulk of this change is reference prose under `.claude/skills/sdlc/references/`,
  which `steering_bytes` does not count *and should not* — a reference loads only
  if the skill reaches for it, which is the same distinction `.claude/rules/`
  earns with `conditional_steering_bytes`. Requirement 10 makes that tier visible
  as `reference_bytes` rather than leaving it uncounted. The `steering_bytes`
  breach is not cleared by this change and its band is not moved; the reading is
  recorded in `plan.md`, and the formula defect flagged in change 007 stays open
  as its own change. Adding a tier metric is not the same act as re-baselining a
  breached one, and the distinction is the whole reason to write it down here.
- **A closed status set will be wrong eventually.** — Answer: that is the point of
  a closed set, and the guard names the file the set lives in so widening it is a
  one-line diff with a visible reason. An open set is what produced `None`
  meaning two different things in lesson 007.
- **The readiness gate can be satisfied without the spec being good.** — True and
  stated, in `references/readiness-gate.md`: the script judges whether the
  document is filled in, never whether the design is right. Requirement 4's list
  is a floor. Presenting it as a quality verdict would be worse than not having it.
- **The screen gate has no fixture to test against.** — Answer: it is a
  conversation, not a script, so its guard is that `risk.yaml` declares which
  surfaces need it and `state.yaml` cannot read `approved` without a route through
  the skill. That is weaker than a script and is labelled as such; an eval is the
  honest home for the rest and is deferred rather than faked.

## Acceptance

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`,
  including `every_state_file_names_a_route_and_a_stage_that_exist`,
  `every_risk_surface_names_paths_that_exist`,
  `the_readiness_gate_refuses_a_spec_that_is_not_filled_in`, and
  `every_reference_the_skill_names_exists`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-readiness.sh docs/sdlc/changes/008-the-pipeline-has-no-state-between-sessions`
  exits 0 and prints `Go`.
- The same command against a spec with its flagged-concern answers removed exits
  1 and names the concern.
- `bash scripts/harness-metrics.sh` — `states`, `risk_surfaces`, and
  `reference_bytes` present and non-zero; `always_loaded_bytes` still under 9 716;
  `unsafe_blocks` 2 and `subprocess_sites` 29 unchanged.
- `.claude/hooks/gate-stop.sh` refuses once while this change's own contract has a
  `pending` machine item, and names it.

## Rejected alternatives

- **A second orchestrator skill beside `sdlc`, with its own name and shape.** Two entry points for one job. The first question a session would
  face is which to use, and the answer would be in neither.
- **A JSON state file with a schema.** Buys validation this repository gets
  from a test anyway, and costs the corruption mode that makes a repair skill
  necessary.
- **Adding sixteen sub-skills.** Each stage already has a document saying
  what it produces and a gate saying when it is done. A skill per stage would be
  ten files restating six.
- **Keeping the readiness gate as a prose checklist in the skill.** It is a list
  of string checks over a markdown file. That is a script, and a checklist an
  agent runs against itself is the mechanism change 007 declined to adopt.
- **Putting the detail in `SKILL.md` rather than `references/`.** Would land ~9 KB
  in the tier that loads on every invocation of the skill, for material needed on
  a minority of invocations.
- **Fixing the `steering_bytes` formula here, to make room.** Change 007 flagged
  the defect and deferred it precisely so that the change needing the room is not
  the change that adjusts the instrument.
