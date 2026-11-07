# Plan: a conversation that proves an entry wrong replaces it

- **Spec**: `./spec.md`
- **Approved**: 2026-11-08
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/tests.rs` | six `knowledge_257_*` tests; existing `judge_prompt` / `parse_judge_answer` callers pass an empty known list / `0`; `JudgedEntry` literal gains `replaces: None` |
| `src/config.rs` | `KNOWLEDGE_JUDGE_KNOWN_ENTRIES` |
| `src/knowledge.rs` | `KnownEntry`, prompt section, parser field, `related_entries`, `replace_entry`, `AddOutcome::Replaced`, `RunReport.replaced`, run wiring |
| `README.md`, `README.ja.md`, `README.ko.md` | one sentence in the knowledge bullets |

## Order of work

1. Signatures and fields with today's behaviour (`known` ignored, `replaces: None`,
   `related_entries` empty, `replace_entry` returning `false`) so the tests
   compile; write the six tests and watch each fail for its own reason.
2. Prompt and parser (Req 1–2).
3. `related_entries` and `replace_entry` (Req 3–4).
4. Run wiring, report, budget, and one replacement per entry (Req 5–6).
5. README sentences; `mutations.py`; `verification.yaml`.

The tree compiles after every step.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Prompt passes its ceiling with known entries | judge prompt cut, candidates missing | Req 1 at the limits; `judge_prompt_stays_under_its_ceiling` |
| A pinned entry is rewritten | person's pinned text changes | Req 4 (`replace_entry` refuses) and Req 3 (never offered) |
| `replaces` index mapped to the wrong entry | an unrelated entry overwritten | Req 5 asserts which entry changed and that the other did not |
| Extra embeddings on projects with no entries | slower first runs | Req 5 asserts no query embedding with an empty base |
| Query embeddings pass the run's budget | a run embeds more than 1,500 texts | Req 6 through `Budget::spend` |
| Two items replace one entry | the first replacement lost | Req 5: each offered entry replaced at most once per answer |
| Replacement written for a disabled project | knowledge reappears after clearing | stays inside `write_if_enabled`, as `add_or_reinforce` does |

## Proof of completion

- `cargo test --locked knowledge_257_` — `6 passed; 0 failed`.
- `cargo test --locked knowledge_` and `cargo test --locked judge_` — all pass.
- `make q.fast` — fmt and clippy clean.
- `python3 docs/sdlc/changes/257-knowledge-keeps-a-contradicted-entry/mutations.py` — every mutation caught.
- CI green on the branch head.

## Departures from the plan

- `KnowledgeDatabase::entry_count` lost its `#[cfg(test)]`: the run asks it
  whether the base has anything to offer before embedding queries.
- Packaging is deferred to one run after the article's last step lands.
