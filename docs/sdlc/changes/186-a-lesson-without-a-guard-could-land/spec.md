# Spec: the suite admits a lesson only with its rule and a guard that ran

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `every_lesson_is_admitted_with_its_rule_and_a_guard_that_ran` runs
   `bash scripts/pipeline-indicators.sh --lessons` on this repository and
   fails unless it exits 0, printing what the script printed — so the rule for
   what counts as a guard this repository holds stays in the script.
2. The same test reads `docs/sdlc/lessons.md` through one function,
   `lesson_admission_problems(lessons: &str) -> Vec<String>`, which reports:
   an entry heading with no title after its number and dash; an entry with
   neither `**The rule:**` nor `**The rule.**`; an entry with no
   `**Guard.**`; and, for entries numbered from `FIRST_LESSON_WITH_WATCHED_GUARD`
   (081) on, a Guard paragraph that does not say `watched failing` (whitespace
   between the words is free, since the file is wrapped). Numbers ascend by one
   from 001, newest last. Each problem names its entry.
3. Mutants inside the test, each refused by its own message: an entry with no
   rule, an entry with no guard, an entry from 081 on whose guard never says
   watched failing, a skipped number, an entry with no title. An entry numbered
   080 with no "watched failing" is accepted, so the threshold is tested both
   ways.
4. The script half is shown to refuse too: a fixture repository holding a copy
   of the script, a lessons file whose only guard names a test absent from its
   `src/tests.rs`, and that test file — the script exits 1 and names the entry.
5. `docs/sdlc/lessons.md`'s header says what the suite now refuses.

## Behaviour

No screen. The failure messages, in the suite's Japanese:

- `lessons.md: pipeline-indicators.sh --lessons が失敗しました — Guard が指すテストか eval が無い教訓があります:\n{output}`
- `{entry}: 見出しに題がありません — どの場面の教訓か分かりません`
- `{entry}: The rule がありません — 何を守る教訓か分かりません`
- `{entry}: Guard がありません — 教訓を守るものがありません`
- `{entry}: Guard が watched failing と言っていません — 失敗するところを見ていない guard は推測です`
- `{entry}: 番号が {expected} ではありません — 新しい教訓は末尾に連番で足します`
- A mutant that passes: `変異 {what} を見逃しました — 検査が検査していません`.
- An empty or unreadable `lessons.md` fails with what is missing.

## Design

Placement: `EXTEND`. The test sits beside
`every_lesson_cited_by_number_is_a_lesson_that_exists` in `src/tests.rs`; the
fixture for requirement 4 follows `eval_runner_fixture`'s pattern (a temporary
directory holding a copy of the script, so the script's own `cd` to its parent
lands there). `FIRST_LESSON_WITH_WATCHED_GUARD` is a constant in the test,
beside the function that reads it. No persisted shape, no new external command
beyond `bash` on a repository script as the neighbouring script tests run it,
no background thread, nothing on the draw path.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | nothing painted |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | no glyph |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes, in spirit | the guard-existence rule is the script's, asked rather than restated (lesson 004) |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | no store |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | test-only `bash` on a repository script, as the neighbouring script tests do |
| Documentation — user-facing docs change in all three languages together | no | `lessons.md` is an English agent document |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | none added |
| Budgets — any new scan or output path states its byte and item ceiling | yes | reads one named file and runs one script over it; nothing added to the always-on text |

## Flagged concerns

- **Whether every current entry passes** — measured on 2026-10-21: all 82
  name a guard the script finds; 069 and 070 write `**The rule.**` rather than
  `**The rule:**`, which requirement 2 accepts; 081 and 082 say watched failing.

## Acceptance

- `cargo test --locked every_lesson_is_admitted_with_its_rule_and_a_guard_that_ran`
  passes, and the rest of the suite in CI.
- Each check in `lesson_admission_problems` watched failing when removed, on
  the mutant it owns; the script fixture watched passing when its guard names a
  test that exists.

## Rejected alternatives

- Reimplementing the guard-existence rule in Rust: two copies of one rule drift
  (lesson 004).
- Requiring watched failing of every entry: 001 to 080 were compressed and many
  no longer say it; rewriting them is churn, and the threshold makes the rule
  bind from where it was adopted.
- A commit hook on `lessons.md`: the suite in CI is where Harness²'s validator
  lives here (change 178).
