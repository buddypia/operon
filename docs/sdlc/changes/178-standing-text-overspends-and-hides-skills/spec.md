# Spec: the standing instructions fit Harness²'s budget and name every skill

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. The text Claude Code loads on every turn — `CLAUDE.md` with the `AGENTS.md`
   it imports — is at most 8,000 characters, the cap Harness² puts on Codex's
   `AGENTS.md` (`harness2/substrate/codex.py`, `always_on_max=8_000`). Codex and
   Antigravity load `AGENTS.md` alone, which the same sum bounds. `CLAUDE.md`
   imports `AGENTS.md` by an `@AGENTS.md` line, or the sum describes nothing.
   `AGENTS.md` states the order for placing new knowledge — a skill, a script,
   a subagent, a hook or a test, and this file last, paid for by a cut.
   Fails on main's tree (9,133 characters):
   `harness_text_every_turn_loads_stays_within_eight_thousand_characters`.
2. Every directory under `.claude/skills/` that holds a `SKILL.md` is named in
   `AGENTS.md` on a line that carries its backticked path
   `.claude/skills/<name>/SKILL.md` and says when to read it (the line contains
   `when `, in any case) — Harness²'s trigger line, in the one file all three
   CLIs load. Fails on main's tree for all six — no line there says when, and
   three of the skills are not named at all:
   `harness_agents_md_says_when_to_read_every_skill`.
3. Each hooks file has the shape its CLI dispatches, and the check refuses a
   mutant of each shape it accepts:
   `every_hook_file_has_the_shape_its_cli_dispatches`.
   - `.codex/hooks.json` — Harness²'s `check_hooks`: valid JSON; a non-empty
     top-level `hooks` object; every event one Codex dispatches (Harness²'s
     Codex set, joined with the events `hook_events(CliProvider::Codex)`
     registers); each mapped to a list of objects with a `hooks` list; each
     hook `type: "command"` with a non-empty `command`.
   - `.claude/settings.json` — the same under its `hooks` key, with events from
     `VALID_EVENT_TYPES` in `.cli/lib/hook-registry.mjs` joined with those
     `hook_events(CliProvider::Claude)` registers — the two places this
     repository records what Claude Code dispatches.
   - `.claude/hooks.json` (Antigravity reads it as `.agents/hooks.json`) — named
     groups, each an object holding `enabled` (if present, a boolean) and events from
     `hook_events(CliProvider::Gemini)`; an event that table marks as needing a
     matcher is grouped as `[{matcher, hooks: [...]}]`, the rest are flat
     handler lists, as `.cli/lib/cli-adapter-utils.mjs` also records; every
     handler `type: "command"` with a non-empty `command`.
   - Mutants refused, inside the test: a misspelled event, an event mapped to an
     object, an entry with no `hooks` list, a `prompt` type, an empty command,
     an Antigravity `Stop` wrapped in a group, an Antigravity `PreToolUse` left
     flat, a non-boolean `enabled`, malformed JSON, an empty `hooks`. Each
     mutant must be refused for its own reason — the test names the message it
     expects — so removing any one check fails the test, not only removing all.
4. The entry sentence moves to where every CLI reads it:
   `one_skill_is_the_entry_point_and_the_root_documents_name_it` requires
   "`.claude/skills/feature-pilot/SKILL.md` is the entry point" in `AGENTS.md`
   and the `@AGENTS.md` import in `CLAUDE.md`, and still refuses either file
   calling another skill the entry. Fails on main's tree, whose `AGENTS.md`
   names the path without the sentence.
5. `every_document_that_names_the_gates_names_the_same_three` drops
   `CLAUDE.md`, which no longer lists the gates, and adds
   `.claude/skills/feature-pilot/references/rust-build-chain.md`, which now
   carries the table of gate commands and their healthy output.
6. `always_loaded_bytes` in `docs/sdlc/bands.yaml` is re-based on the reading
   after the cut, with its tiers under 8,000 bytes, so the band warns before the
   test refuses; `bash scripts/check-bands.sh` passes.

## Behaviour

No screen changes; the app is not rebuilt. The people who read anything new are
whoever runs the suite. The failure messages, in the suite's Japanese:

- Requirement 1, over budget: `CLAUDE.md と AGENTS.md は毎ターン読み込まれ、合計
  {n} 文字で上限 8000 文字を超えています。足した分は削って払ってください —
  手順は skill (AGENTS.md に When の行)、決まった操作は script、大きな入力から
  小さな答えを返す調査は subagent、文章で守られない規則は hook かテストへ`.
  Import missing: `CLAUDE.md が @AGENTS.md を読み込んでいません — 毎ターンの
  文字数が数えられません`. Order missing: `AGENTS.md に、知識を置く順番 (skill,
  script, subagent, hook, 最後にこのファイル) の行がありません`.
- Requirement 2: `{name}: AGENTS.md に .claude/skills/{name}/SKILL.md をいつ読むか
  を書いた行がありません — 名前を呼ばれない skill は開かれません`.
- Requirement 3, one line per problem, prefixed by the file: `未知のイベント
  {event} — {cli} が読むのは {events}`, `{event} の値が配列ではありません`,
  `{event}[{i}] に hooks の配列がありません`, `{event}[{i}] の type が
  "command" ではありません`, `{event}[{i}] の command が空です`, `{event} は
  matcher と hooks でまとめる形です`, `{event} はまとめずに handler を並べる形
  です`, `enabled が真偽値ではありません`; and, when a mutant passes, `変異 {what} を見逃しました — 検査が検査して
  いません`.
- Absent states: an unreadable or non-JSON hooks file, a `settings.json` with no
  `hooks`, an empty `AGENTS.md`, and a `hook-registry.mjs` with no
  `VALID_EVENT_TYPES` set each fail with what is missing rather than passing
  vacuously.

## Design

Placement: `EXTEND` — every change is to an existing concern. The new tests sit
in `src/tests.rs` beside `every_skill_loads_under_codex_and_antigravity` and
reuse `skill_documents`; the hook check is one function,
`hook_shape_problems(cli, text) -> Vec<String>`, called for the three files and
for the mutants. Constants: `ALWAYS_ON_CHARACTER_CAP` (8,000) and
`CODEX_HOOK_EVENTS` (the set Harness² validates for Codex, the one vocabulary
this repository did not already hold); Antigravity's events and their grouping
come from `hook_events(CliProvider::Gemini)` in `src/tmux/hooks.rs`, and the
Claude Code set is read out of `hook-registry.mjs`, rather than restated.
`serde_json` is already a dependency.

Text moves, none deleted outright without a home that loads when it is needed:

| From | What | To |
|---|---|---|
| `CLAUDE.md` | entry point, stopping, commit and CI, review policy, release | already in `AGENTS.md` — the copy goes |
| `CLAUDE.md` | a mistake made twice becomes a check | `AGENTS.md`, rewritten as the placement order |
| `CLAUDE.md` | gate commands, healthy output, ignored tests, metrics | `rust-build-chain.md` §4, read at the verify step |
| `CLAUDE.md` | a test that fails only sometimes is a bug | `root-cause` skill, beside its flake rule |
| `CLAUDE.md` | route children through `src/exec.rs` | folded into the module map's `src/exec.rs` line; `.claude/rules/rust.md` arrives with every `.rs` file and says it in full, and `unwrapped_spawns` is banded |
| `AGENTS.md` | the pipeline paragraph | When-lines for `feature-pilot` and `sdlc` |
| `AGENTS.md` | the release paragraph | When-line for `ship` |
| `AGENTS.md` | what `destructive-git-guard.mjs` blocks | dropped: the guard refuses with its own reason |

`docs/sdlc/README.md` changes where it says what loads when (the four homes, and
stage 4's pointer to the gate table). No persisted shape, no external command,
no background thread, nothing on the draw path.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | nothing painted |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | no glyph |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes, in spirit | the Claude Code event set is read from `hook-registry.mjs`, not copied; each moved sentence keeps one home (the table above) |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | no store, no persisted shape |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | the tests spawn nothing |
| Documentation — user-facing docs change in all three languages together | no | `README*.md` say nothing about the steering files; the changed documents are English-only agent documents |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | none added; the CLI probes ran by hand against local fixtures |
| Budgets — any new scan or output path states its byte and item ceiling | yes | the tests read six named, tracked files (`CLAUDE.md`, `AGENTS.md`, the three hook files, `hook-registry.mjs`) and the existing `skill_documents` listing of one directory, as the neighbouring root-document and skill tests do; `MAX_HARNESS_FILE_BYTES` stays with `harness_documents`, which walks directories it does not name. The always-on budget itself is the ceiling this change adds |

## Flagged concerns

- **Whether the three CLIs open a skill nothing names** — Harness² states it for
  Codex; lesson 060 says measure. Answered in `plan.md` under "Measured before
  writing": in one-skill fixtures all three CLIs opened the skill from its
  description alone (15 of 15), so Harness²'s claim does not hold as stated
  here. Requirement 2 stands on the crowded case instead — the session that
  wrote this change listed the three unnamed skills with no description — and
  its test's comment says so rather than repeating Harness²'s claim.

## Acceptance

- `cargo test --locked harness_` and `cargo test --locked every_hook_file_has_the_shape`
  pass, as do `one_skill_is_the_entry_point_and_the_root_documents_name_it`,
  `every_document_that_names_the_gates_names_the_same_three`,
  `harness_documents_only_name_paths_that_exist`,
  `claude_md_maps_every_module_that_exists`, and the rest of the suite in CI.
- Requirements 1, 2, and 4 watched failing on main's `CLAUDE.md` and
  `AGENTS.md`; requirement 3's mutants each watched refused, and the test
  watched failing with the checker's event check removed.
- `cat CLAUDE.md AGENTS.md | wc -m` reads at most 8,000;
  `bash scripts/check-bands.sh` passes.
- `bash scripts/run-evals.sh` over the committed steering is no worse than
  before, eval 006 (the rules pointer) and 004 (the module map) in particular.

## Rejected alternatives

- A new PostToolUse hook checking these rules at write time: Harness² enforces
  them in its validator before a harness runs, which is the suite here; no
  trajectory shows them broken inside the tool loop; Antigravity's PostToolUse
  returns no message, and the Codex and Antigravity hook files are generated by
  the vendored bundle.
- Making the band the cap: a `propose` breach is a signal with an answered-breach
  path (lesson 025), not a refusal, and it counts bytes.
- Counting bytes in the test: Harness² counts characters; the band keeps bytes,
  which are never fewer, so it fires first.
- Moving the rules pointer table or the module map out: eval 006 measured the
  table as what carries a new file to its rule, and the map is the workspace
  fact used on nearly every task — the kind Harness² ranks most valuable.
- When-lines in `CLAUDE.md`: Codex and Antigravity never load it.
- `.agents/skills/` paths in the When-lines: the repository writes
  `.claude/skills/`, and `.agents` is a pinned link to `.claude`.
- Deleting the `create-pr` skill or the flat catalogue stub: both are owned by
  the vendored bundle and would come back; `create-pr` gets a line saying when
  it would apply and that it does not run here.
