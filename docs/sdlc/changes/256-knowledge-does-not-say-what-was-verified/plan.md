# Plan: a knowledge entry says whether its conversation confirmed it

- **Spec**: `./spec.md`
- **Approved**: 2026-11-07
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/tests.rs` | eight `knowledge_256_*` tests; `judged_answer` callers unchanged, a sibling helper writes `verified`; `NewEntry` literals gain `verified: false` |
| `src/knowledge.rs` | prompt, `JudgedEntry`/`NewEntry`/`KnowledgeEntry` field, schema column and additive migration in `open`, `entries`, `add_or_reinforce`, `process_conversation`, `knowledge_cli` |
| `src/app/knowledge.rs` | `verified_label` helper, called from the entry row and the open heading |
| `src/i18n_tables.rs` | 検証済み, 未確認, `{kind}・{status}` in every table |
| `README.md`, `README.ja.md`, `README.ko.md` | one line in the knowledge bullets |

## Order of work

1. Tests for Req 1–7 against the current code, with the fields they need added
   as `verified: false` so the crate compiles; watch each fail for its own reason.
2. `JudgedEntry`, `NewEntry`, `KnowledgeEntry` fields; prompt; parser (Req 1–2).
3. Schema column, migration in `open`, `entries`, `add_or_reinforce` (Req 3–4).
4. `process_conversation` passes the flag (Req 5).
5. CLI text and JSON (Req 6).
6. Tab label and i18n rows (Req 7).
7. README lines; `mutations.py`; `verification.yaml`.

The tree compiles after every step.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| `ALTER TABLE` runs on every open and fails when the column exists | knowledge tab error on second open | Req 4 opens the same file twice |
| GUI run and agent search open an old file at once; the second `ALTER` fails with duplicate column | knowledge tab or `knowledge search` errors once | `knowledge_256_adding_verified_twice_still_opens` |
| Old builds break on the new column | an older Operon cannot read knowledge | Req 4 asserts `schema_version` stays 1 and an insert naming only the old columns succeeds |
| Reinforcement downgrades a verified entry | label flips back to 未確認 | Req 3 |
| Prompt grows past its ceiling | judge prompt truncated | Req 1 and existing `judge_prompt_stays_under_its_ceiling` |
| Draw path does work per frame | stutter | the label reads the in-memory snapshot only; rust-reviewer |
| A string added without every table | missing translation | `every_message_has_a_row_in_every_table`-class tests in the suite |

## Proof of completion

- `cargo test --locked knowledge_256_` — `8 passed; 0 failed`.
- `cargo test --locked knowledge_` and `cargo test --locked judge_` — all pass.
- `make q.fast` — fmt and clippy clean.
- `python3 docs/sdlc/changes/256-knowledge-does-not-say-what-was-verified/mutations.py` — every mutation caught.
- CI green on the branch head.

## Departures from the plan

None yet.
