# Plan: the standing instructions fit Harness²'s budget and name every skill

- **Spec**: `./spec.md`
- **Approved**: 2026-10-18
- **Status**: approved

## Measured before writing

Lesson 060: a CLI's behaviour is measured, not assumed. Fixtures in a scratch
directory, 2026-10-18, codex-cli 0.163.0, agy 1.4.2, claude 2.1.302; a skill
whose body tells the agent to print a token, and an `AGENTS.md` without (A) and
with (B) a When-line naming it.

- `AGENTS.md` is always-on for Codex and Antigravity: an instruction placed only
  there was followed by both, unprompted (1 of 1 each).
- When the prompt used the skill description's own words ("release note"),
  every CLI opened the skill with no When-line: A 6 of 6, B 6 of 6. So
  Harness²'s "an unnamed skill is never opened" does not hold for these three
  CLIs in the easy case — each advertises a skill's name and description.
- With the situation only implied (a test green locally and red in CI; the
  description says "intermittent"), the same: A 9 of 9, B 9 of 9. In a fixture
  with one skill, the When-line changed nothing on any CLI.
- What the fixtures cannot show is a crowded listing. The Claude Code session
  that wrote this change carries the person's global skills too, and its
  listing gave `durability-invariants`, `root-cause`, and `sdlc` a name and no
  description — the three skills `AGENTS.md` did not name. There nothing
  advertised when to open them. Requirement 2 stands on that: the When-line is
  the description that survives when the listing does not, at a cost the
  budget in requirement 1 already pays for. Codex's listing under the same
  load was not measured.

## Files that change

| File | Change |
|---|---|
| `src/tests.rs` | three new tests (requirements 1–3), the entry test reads `AGENTS.md` (4), the gate-list test's documents (5) |
| `AGENTS.md` | When-lines for all six skills; the placement order; the pipeline and release paragraphs replaced by their When-lines; the guard description dropped |
| `CLAUDE.md` | what `AGENTS.md` already says goes; plan mode, the module map, and the rules pointer table stay |
| `.claude/skills/feature-pilot/references/rust-build-chain.md` | §4 carries the gate commands and their healthy output |
| `.claude/skills/root-cause/SKILL.md` | the intermittent-test example beside the flake rule |
| `docs/sdlc/README.md` | the homes of knowledge, in Harness²'s order; stage 4's pointer |
| `docs/sdlc/bands.yaml` | `always_loaded_bytes` re-based after the cut |
| `docs/sdlc/changes/178-…/` | this change's record |

## Order of work

1. Write the three new tests and the two test edits; run them by name on the
   current `CLAUDE.md` and `AGENTS.md`, and record requirements 1, 2, and 4
   failing for the stated reason (requirement 2 for all six skills).
   Requirement 3 passes on the tree: remove its checks one at a time — the
   event check, the grouping check, the type check, the empty-command check —
   and record the test failing on the mutant that check owns each time, then
   restore the file byte for byte (`cmp`).
2. Rewrite `AGENTS.md` and `CLAUDE.md`; move the gate table, the intermittent
   example, and update `docs/sdlc/README.md`. Run the named tests until green,
   with `harness_documents_only_name_paths_that_exist`,
   `claude_md_maps_every_module_that_exists`, the landing, asking, and
   reporting tests, and `no_eval_prompt_is_pasted_into_the_steering`.
3. Measure `cat CLAUDE.md AGENTS.md | wc -c` and `wc -m`; re-base the band;
   `bash scripts/check-bands.sh`.
4. `make q.fast`; commit; `bash scripts/run-evals.sh` against the commit.
5. Review per the route and the surfaces touched; close `state.yaml`; push, CI,
   land, clean up.

The tree compiles between every step; step 1 leaves the suite red on purpose
until step 2.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A moved sentence was the one an eval relies on | an eval regresses | `scripts/run-evals.sh` over the commit, 004 and 006 in particular |
| A cut takes a line another test pins | a red named test | the pinned tests run by name in step 2, the suite in CI |
| The Claude Code event vocabulary in `hook-registry.mjs` is incomplete | a valid event refused | it is joined with `hook_events(CliProvider::Claude)`; a refusal names the file and the event |
| A bundle regeneration rewrites a hooks file into another shape | the shape test fails on the regenerated file | the test itself — that is the point |
| The band and the test disagree near the cap | one fires without the other | bytes are never fewer than characters, so the band warns first |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` in CI — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `harness_text_every_turn_loads_stays_within_eight_thousand_characters`,
  `harness_agents_md_says_when_to_read_every_skill`, and
  `one_skill_is_the_entry_point_and_the_root_documents_name_it` fail on main's
  root documents and pass after; `every_hook_file_has_the_shape_its_cli_dispatches`
  fails with its checker broken and passes restored.
- `cat CLAUDE.md AGENTS.md | wc -m` at most 8,000; `bash scripts/check-bands.sh`
  prints `bands: N metrics within their bands`.
- No screen to look at: the app is not rebuilt.

## Departures from the plan

- **Eval 004 regressed, and the cap left the prose.** Against the first
  commit (5d9278c) eval 004 failed 0 of 1 where main (029eb90) passed 1 of 1:
  asked for an empty notify module, the agent declined, and one of its
  reasons was that the module map line would grow "an instruction file whose
  character count is capped". The placement line had said "with `CLAUDE.md`
  it stays under 8,000 characters". The number is the test's to enforce, and
  its failure message says what to cut; in prose it read as a reason not to
  add the line a new module owes. So the line now says "a new rule goes in
  this order" and no longer quotes the cap — the form Harness² asks for, a
  check rather than a sentence. 7,347 characters, 7,395 bytes.
- **Evals 004 and 006 fail on main as well.** Rerun with this change closed
  (b4b0e94) and against main (029eb90) side by side: 004 failed 3 of 3 on
  each, every agent stopping at feature-pilot's `DUPLICATE` because
  `send_macos_notification` already exists; 006 failed every judged run on
  each (1 here, 1 on main) with the rest timing out, the agent asking where
  the badge would be used rather than commit code nothing calls. Main's single
  004 pass this morning was the lucky sample. Both prompts ask for code with
  no caller, which the commit gate and lesson 005 now argue against; fixing
  the evals is outside this change.
