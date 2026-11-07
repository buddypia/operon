# Spec: a conversation that proves an entry wrong replaces it

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `judge_prompt(candidates, known)` lists the existing entries it is given, as
   `[E1]`, `[E2]`, … with each entry's title and body, before the candidates.
   It says that an item may carry `"replaces": <E number>` when the conversation
   shows that existing entry is wrong or out of date, that the item then takes
   the entry's place, and that `replaces` is left out when the item only repeats
   or adds to an entry. With no known entries, the existing-entries section and
   the `replaces` instruction are absent. The prompt stays inside
   `KNOWLEDGE_JUDGE_PROMPT_MAX_BYTES` with the most known entries and every
   candidate at its limit.
   Test: `knowledge_257_judge_prompt_offers_existing_entries_to_replace`.
2. `parse_judge_answer(answer, offered, known)` carries `replaces` into
   `JudgedEntry` as `Some(index)` (zero-based) for an integer `1..=known`, and
   `None` when it is absent, zero, past `known`, or not an integer. Old-shape
   answers parse as before.
   Test: `knowledge_257_judge_answer_carries_replaces`.
3. `KnowledgeDatabase::related_entries(query_vectors)` returns the unpinned
   entries whose vector reaches `KNOWLEDGE_SEARCH_MIN_COSINE` against any of the
   given query vectors, best first, at most `KNOWLEDGE_JUDGE_KNOWN_ENTRIES` (5);
   none when there are none.
   Test: `knowledge_257_related_entries_are_near_unpinned_and_bounded`.
4. `KnowledgeDatabase::replace_entry(id, entry, source, now)` sets that entry's
   kind, title, body, vector, and `verified` to the new item's (a confirmation
   of the old content says nothing about the new), keeps its id, `created_at`,
   reference count, and earlier sources, adds the new source, counts a
   recurrence, sets `reinforced_at` to now, and returns `true`. It changes
   nothing and returns `false` when the entry is pinned or gone.
   Test: `knowledge_257_replacing_an_entry_keeps_its_history_and_spares_pinned`.
5. A run embeds each candidate as a query, offers the related entries to the
   judge, and applies its answer: an item whose `replaces` names an offered
   entry replaces it (`RunReport.replaced` counts it, the entry count does not
   change); one whose entry was pinned or deleted meanwhile is added or
   reinforced as today. Items are applied in the answer's order and each offered
   entry is replaced at most once per answer: a second item naming the same
   entry is added or reinforced as today. A project with no entries embeds
   nothing extra.
   Test: `knowledge_257_a_run_replaces_what_the_judge_says_is_out_of_date`.
6. The query embeddings are spent through the run's budget (`Budget::spend`),
   never past it: when what remains cannot cover one query per candidate, or
   the run's time is up, no existing entries are offered, nothing extra is
   embedded, and the conversation is judged as before.
   Test: `knowledge_257_an_exhausted_budget_offers_no_existing_entries`.

## Behaviour

- Nothing new is drawn. In プロジェクト › ナレッジ a replaced entry shows its new
  title and body under the same row; its freshness restarts from the replacement
  and its reference count stays. On the map it may move, since its meaning moved.
- `Operon knowledge search` returns the new content.
- A pinned entry is never offered to the judge for replacement, so it never
  changes; a contradicting item about it is stored as today.
- No new error, empty, or interrupted state: a judge that ignores `replaces`
  leaves today's behaviour.

## Design

Placement verdict: **EXTEND** — `src/knowledge.rs` owns the judge, the run, and
the database (`git grep -n "fn judge_prompt\|fn process_conversation\|fn add_or_reinforce" src`).

- `src/config.rs`: `KNOWLEDGE_JUDGE_KNOWN_ENTRIES: usize = 5`.
- `src/knowledge.rs`
  - `KnownEntry { id, title, body }` (what the judge is shown).
  - `judge_prompt(&[Candidate], &[KnownEntry])`: when `known` is non-empty, a
    paragraph on `replaces` and an `Existing entries` block after the
    instructions and before the candidates. Titles and bodies are already capped
    at 40 and 400 characters, so the block is bounded by five of them; the
    candidates keep the existing byte check.
  - `JudgedEntry.replaces: Option<usize>`; `parse_judge_answer` gains `known`.
  - `related_entries(&self, &[Vec<f32>]) -> Result<Vec<KnownEntry>>`: one pass over
    `entry_vectors` plus a `pinned = 0` filter.
  - `replace_entry(...) -> Result<bool>`: one `UPDATE … WHERE id = ? AND pinned = 0`;
    when it changed a row, `insert_source`.
  - `AddOutcome::Replaced(i64)`, `RunReport.replaced`.
  - `process_conversation`: after the cap and pause checks and before the prompt,
    when the database has entries and `budget.spend(candidates.len())` succeeds,
    embed `query_input(&candidate.text)` for each candidate and take
    `related_entries`; otherwise `known` is empty. Pass `known` to `judge_prompt`
    and its length to `parse_judge_answer`. In the write, keep a set of replaced
    indices: an item with `replaces: Some(i)` not yet in the set tries
    `replace_entry(known[i].id, …)`; when that returns `false`, or `i` was
    already replaced, the item goes to `add_or_reinforce`.
- No schema change, no new child process, no new dependency, no drawing change.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no, read | nothing is drawn differently |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | none |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes, read | the `replaces` key is written in the prompt and read by the parser; Req 1 and Req 2 assert both sides, and the bound is one constant in `src/config.rs` |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | read | no schema change; the write stays inside the existing `write_if_enabled` transaction, so a disabled or cleared project writes nothing |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | the judge call is unchanged, one per conversation |
| Documentation — user-facing docs change in all three languages together | yes | one sentence added to the knowledge bullets of `README.md`, `README.ja.md`, `README.ko.md` |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | none added; no extra judge call |
| Budgets — any new scan or output path states its byte and item ceiling | yes | at most 8 query embeddings per judged conversation, spent through `Budget::spend` so the run's 1,500 is never exceeded (Req 6); at most 5 known entries of ≤ 440 characters each in the prompt, which keeps its 64 KiB ceiling (Req 1) |

## Flagged concerns

- **A judge could replace an entry that was only related.** Resolved: the prompt
  limits `replaces` to an entry the conversation shows is wrong or out of date;
  pinned entries are never offered; the source of the old content stays recorded
  under the entry. Recoverable by pinning what must not change.

## Acceptance

- `cargo test --locked knowledge_257_` passes all six tests, each watched
  failing first.
- `cargo test --locked knowledge_` and `cargo test --locked judge_` pass.
- `make q.fast` clean.

## Rejected alternatives

- Replacing on similarity alone (newest wins above the dedupe cosine):
  paraphrases would churn wording on every repeat, and a contradiction is
  often below that cosine.
- A separate offline "dream" pass over old transcripts: spends judge calls the
  30-a-day budget cannot spare; the judge already reads the new conversation.
- Deleting the old entry and adding a new one: loses its references, its
  sources, and the person's sense of "the same entry".
