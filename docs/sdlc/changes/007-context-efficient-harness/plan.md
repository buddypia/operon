# Plan: adopt four mechanisms, and pay for them by making conditional knowledge conditional

- **Spec**: `./spec.md`
- **Approved**: 2026-08-31
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `docs/sdlc/routes.yaml` | new — six work-type routes, positional tuples |
| `docs/sdlc/templates/handoff.md` | new — the artifact an exhausted ceiling produces |
| `.claude/rules/rust.md` | new — Rust craft axes, pinned-API rule; `paths:` `src/**/*.rs`, `Cargo.toml` |
| `.claude/rules/palette-and-glyphs.md` | new — colour and icons, moved out of `CLAUDE.md` |
| `.claude/rules/transcripts.md` | new — transcript vocabulary and terminal identity, moved out of `CLAUDE.md` |
| `.claude/rules/identifiers.md` | new — identifier SSOT, moved out of `CLAUDE.md` |
| `.claude/skills/root-cause/SKILL.md` | new — the Iron Law and its phases |
| `.claude/agents/rust-reviewer.md` | new — scoped Rust reviewer |
| `.claude/hooks/gate-stop.sh` | new — `Stop`: an unproven Rust change blocks the first finish |
| `.claude/settings.json` | wire the `Stop` hook |
| `.claude/skills/sdlc/SKILL.md` | classify into a route before allocating a directory; auto-stop section |
| `CLAUDE.md` | five deep-dive sections become pointer lines |
| `REVIEW.md` | name the Rust reviewer and the routing table as passes |
| `docs/sdlc/README.md` | the rules mechanism, the routes, what was omitted and why |
| `docs/sdlc/lessons.md` | entry 010 |
| `docs/sdlc/bands.yaml` | bands for `rules`, `routes`, `conditional_steering_bytes`; `hooks`/`skills`/`subagents` re-baselined |
| `scripts/harness-metrics.sh` | emit `rules` and `conditional_steering_bytes`; count the new guards |
| `src/tests.rs` | three new guards; `harness_documents()` walks `.claude/rules` |
| `AGENTS.md` | one line: the routes file is the entry point for a development request |

## Order of work

The tree compiles between every step; `src/tests.rs` is edited last on purpose,
so the guards are written against files that already exist and can be watched
failing when those files are mutated.

1. `docs/sdlc/routes.yaml` and `docs/sdlc/templates/handoff.md` — the data first,
   because the skill and the guard both read them.
2. The `.claude/rules/` files, each carrying its `paths:` list. Content
   moved verbatim where it was already correct.
3. `CLAUDE.md` — replace the five moved sections with pointer lines. Measure
   `always_loaded_bytes` here; requirement 6 either holds at this step or the plan
   is wrong.
4. `.claude/skills/root-cause/SKILL.md` and `.claude/agents/rust-reviewer.md`.
5. `.claude/hooks/gate-stop.sh`, then its wiring in `.claude/settings.json`.
   Wiring second: `every_hook_the_settings_file_wires_exists_and_is_executable`
   fails on an unwired script, so this is the one pair that must land together.
6. `.claude/skills/sdlc/SKILL.md`, `REVIEW.md`, `AGENTS.md`,
   `docs/sdlc/README.md` — the documents that point at what now exists.
7. `scripts/harness-metrics.sh` and `docs/sdlc/bands.yaml`. Metrics before bands:
   a band on a key nothing emits fails the existing control-bands guard.
8. `src/tests.rs` — the three new guards, and `harness_documents()` extended to
   walk `.claude/rules` so the path check covers the new documents.
9. `docs/sdlc/lessons.md` entry 010.
10. Verify by mutation: delete a `paths:` block, delete a route's ceiling, and
    confirm each fails the suite. Restore.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A moved policy stops being applied because the rule never fires on a new file | a colour literal reaches a new drawing file | `design_md_documents_exactly_what_the_app_paints`, `every_theme_stays_readable`, and the icon and transcript guards — all unmoved, all still the enforcement |
| A rule is added later without `paths:` and quietly becomes always-loaded | `always_loaded_bytes` under-reports; the band stops telling the truth | `every_rule_declares_the_paths_it_applies_to` |
| A backticked path in a new document does not exist | an agent searches for a file that is not there — lesson 002 | `harness_documents_only_name_paths_that_exist`, once `harness_documents()` walks `.claude/rules` |
| The `Stop` hook blocks the same tree repeatedly and the session cannot finish | a loop the user has to break by hand | the stamp keyed on the tree state; requirement 9, verified by finishing twice |
| The `Stop` hook's stamp path drifts from the guard that reads it | the guard passes having checked nothing — lesson 004 | the guard reads the path out of the hook rather than restating it |
| `steering_bytes` crosses 125 000 and the change should have stopped | `bash scripts/check-bands.sh` names diagnose | the band itself; the reading is recorded below |
| A new route names a stage the pipeline does not have | the skill routes into nothing | `every_route_the_pipeline_offers_names_stages_that_exist` |

## Proof of completion

Measured, not estimated.

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. 304 passed; 0 failed; 6 ignored`
  (310 total, +3 from this change).
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-bands.sh` — `bands: 1 of 18 metrics breached`.
  `always_loaded_bytes` 12 392 → 9 661, under its 9 716 baseline: requirement 6
  holds and a pre-existing breach is cleared. The remaining breach is
  `steering_bytes` at 127 176, **diagnose**; see the diagnosis below.
- `bash scripts/harness-metrics.sh` — `rules` 4, `routes` 6,
  `conditional_steering_bytes` 10 235, `invariant_tests` 15 → 18,
  `evals` 5 → 6,
  `subagents` 2 → 3, `skills` 3 → 4, `hooks` 4 → 5, `tests_in_ci` 301 → 304,
  `unsafe_blocks` 2 unchanged, `subprocess_sites` 29 unchanged,
  `unwrapped_spawns` 2 unchanged.
- The `Stop` hook exercised directly: first call exit 2 with the file list and
  the three commands, second call on the identical tree exit 0 and silent,
  `stop_hook_active: true` exit 0. Requirement 9 holds in both halves.
- Mutation, watched failing rather than asserted — each of the three guards was
  seen red before it was trusted:
  - stripping the frontmatter from `.claude/rules/rust.md` →
    `every_rule_declares_the_paths_it_applies_to` fails with "paths: の無い規則は
    毎ターン読み込まれます";
  - dropping the attempts column from the `bugfix` route → 4 values not 5;
  - renaming a stage to `verify` and a `requires` path to a typo →
    both reported by `every_route_the_pipeline_offers_names_stages_that_exist`;
  - writing the stamp path a second time in `.claude/hooks/gate-stop.sh` →
    `the_stop_gate_stamp_is_spelled_in_exactly_one_place` fails at 2 occurrences.
- `harness_documents_only_name_paths_that_exist`, once extended to
  `.claude/rules`, caught five real stale paths in this change's own `spec.md` and
  `plan.md` (rule files renamed during step 2) and one backticked path fragment in
  a new `src/tests.rs` comment. All six were defects, not false positives.

## The steering_bytes diagnosis

Required by the band, and recorded here because a diagnose tier whose
investigation is not written down is a warn with extra steps.

**What moved.** 107 077 before, 127 176 after: +20 099. Attributable to
`.claude/skills/root-cause/SKILL.md` (~5 000), `.claude/agents/rust-reviewer.md`
(~3 300), lesson 010 (~2 700), `docs/sdlc/templates/handoff.md` (~2 300), and
growth in `docs/sdlc/README.md`, `.claude/skills/sdlc/SKILL.md`, and `REVIEW.md`,
less 2 940 returned by `CLAUDE.md` shrinking.

**When.** All of it in this change. The pre-existing 107 077 against a 91 262
baseline is older drift, most of it `DESIGN.md`, `CONTRIBUTING.md`, and the
lessons ledger growing since the baseline was set.

**What the investigation found, beyond the number.** The route table had been
written in three places — `docs/sdlc/routes.yaml`, the `sdlc` skill, and
`docs/sdlc/README.md` — which is the identifier-SSOT violation this repository
has a policy and a test against, arrived at from the other direction. Deleting
the README copy and two other duplications recovered ~2 700 bytes and made the
change better, which is the argument for taking a band seriously rather than
re-baselining it.

**What remains, and why it is not cut.** ~2 200 bytes over the threshold. Every
remaining addition is required by a numbered requirement in `spec.md`; cutting
further means dropping a requirement, which is a decision for stage 1 and not for
a session trying to get a number down. The band's rule for `diagnose` is
read-only investigation, not a halt — `propose` at 150 000 is the tier that sends
a change back. The second flagged concern in `spec.md` records the instrument
defect the investigation surfaced and defers it to its own change.

## Departures from the plan

- **Four rule files, not three.** The plan named `vocabularies.md` and
  `terminal-identity.md`; the tree has `palette-and-glyphs.md`,
  `transcripts.md`, and `identifiers.md`. Merging colour, icons, transcript kinds,
  and the identifier SSOT into one file would have forced its `paths:` list to the
  union of nearly every module, which defeats the point of scoping — the colour
  rule would then load while editing `src/store.rs`. Splitting by *what the paths
  actually are* gives three tight scopes instead of one loose one. Terminal
  identity merged into `transcripts.md` because their path lists are identical:
  both are about a store another product writes.
- **A third guard.** `the_stop_gate_stamp_is_spelled_in_exactly_one_place` was
  not in the plan. Writing the hook made it obvious that a second literal of the
  stamp path turns the gate into a loop, and that is lesson 004's shape in a shell
  script.
- **`always_loaded_bytes` landed at 9 661, not the ~8 400 the spec estimated.**
  The five moved sections were 4 527 bytes but the pointer table and the
  `AGENTS.md` line cost ~1 400 back. Requirement 6 holds — under the 9 716
  baseline — with less margin than assumed.
- **`steering_bytes` reached diagnose rather than staying at warn.** Handled
  above rather than absorbed silently; the trimming it forced found a real
  three-way duplication of the route table.
- **`routes.yaml` has five columns, not four.** `requires` was added so a route
  can name a document that must be satisfied on top of its gate — that is how
  `bugfix` binds to `.claude/skills/root-cause/SKILL.md` as data rather than as
  prose. Caught by writing the guard, which is the order the plan intended.
- **No new pre-approved commands in `.claude/settings.json`.** The plan listed
  them; nothing this change added needs one.
