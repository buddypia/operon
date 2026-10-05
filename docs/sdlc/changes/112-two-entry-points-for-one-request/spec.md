# Spec: one entry point for a development request

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Exactly one skill description claims to be the entry point for a
   development request, and `CLAUDE.md` and `AGENTS.md` both name that
   skill's file — `one_skill_is_the_entry_point_and_the_root_documents_name_it`.
2. Every skill that tells a session to write a change directory says to run
   `make wt.new` before it, and the entry skill opens the worktree before it
   hands over to the stage that writes —
   `a_skill_that_opens_a_change_makes_the_worktree_first`.
3. Every skill directory is named by the path of its `SKILL.md` in a flat
   catalog under `.claude/skills/` — `every_skill_is_named_in_a_flat_catalog`.
4. The entry skill keeps no record of its own. It points at `state.yaml`,
   `intent.md`, `spec.md`, `plan.md`, `approvals.log`, and `review.yaml`.
5. `bash scripts/check-bands.sh` exits 0, and `always_loaded_bytes` stays
   below 13000.

## Behaviour

Nothing a person sees in the application changes. An agent given a development
request reads `.claude/skills/feature-pilot/SKILL.md`. That skill opens the
worktree, runs stages 1–2 through `.claude/skills/sdlc/SKILL.md`, builds in Rust
with `.claude/skills/feature-pilot/references/rust-build-chain.md`, and lands with
`git merge --no-ff`.

## Design

- **Placement verdict.** `EXTEND`: the steering documents and the harness tests
  in `src/tests.rs` grow; no application module changes.
- **Skills.** The brought-in set collapses to one skill and one reference. The
  sub-skills it carried are dropped. Each duplicated something this repository
  already has: `sdlc` for the record and gates, `root-cause` for defects,
  `REVIEW.md` and `.claude/agents/` for review, and `screen-approval.md` for
  screens. The same holds for the feature folder, context JSON, project config,
  domain map, and ownership index, which `docs/sdlc/README.md` lists as
  deliberately not built.
- **Edits to existing skills.** `sdlc` gives up the entry claim and gains the
  worktree step in §3.
- **Flat catalog.** `development-skills.md` (removed by change 114) names the skills for
  Antigravity CLI.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | no drawing code changes |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | no glyph changes |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | the entry skill path is written in the root documents and checked by requirement 1 rather than restated in the test |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | the store is untouched |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | the tests read files only |
| Documentation — user-facing docs change in all three languages together | no | no user-facing document changes |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | none added |
| Budgets — any new scan or output path states its byte and item ceiling | yes | the tests read the files under `.claude/skills/`, a bounded set of files |

## Flagged concerns

- **Codex CLI and Antigravity CLI run no commit gate** — answered: the entry
  skill tells those sessions which checks to run by hand. Registering the hook
  is out of scope (see `intent.md`).

## Acceptance

- `cargo test --locked` passes, including the three tests named in Requirements.
- `make q.check` passes.
- `bash scripts/check-bands.sh` exits 0.
- Each new test watched failing under a mutation of the claim it guards.

## Rejected alternatives

- Keep the brought-in set and raise the bands — two records of one position
  drift, and the hooks read only one of them.
- Make `sdlc` the entry and `feature-pilot` a stage-3 helper — the person asked
  for `feature-pilot` as the named entry.
