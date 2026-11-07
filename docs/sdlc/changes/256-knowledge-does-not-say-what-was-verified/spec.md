# Spec: a knowledge entry says whether its conversation confirmed it

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. The judge prompt asks, for every kept item, a boolean `verified`: true only
   when the conversation itself shows the knowledge was checked — a test or a
   command ran and its output confirmed it, or the person confirmed it — and
   false for a hypothesis, a plan, or a claim nobody checked. It says what the
   flag is for (a later conversation relies on a confirmed entry without checking
   it again and checks an unconfirmed one first) and asks a confirmed item's body
   to say how it was confirmed. The prompt still fits
   `KNOWLEDGE_JUDGE_PROMPT_MAX_BYTES`.
   Test: `knowledge_256_judge_prompt_asks_whether_it_was_verified`.
2. `parse_judge_answer` carries `verified` into `JudgedEntry`: JSON `true` is
   true; `false`, absent, or any non-boolean is false, so an answer in the old
   shape still parses whole.
   Test: `knowledge_256_judge_answer_carries_verified`.
3. A new entry is stored with its `verified`; reinforcing an existing entry with
   a verified item makes it verified, and reinforcing with an unverified item
   leaves a verified entry verified. `KnowledgeDatabase::entries` returns it.
   Test: `knowledge_256_verified_is_stored_and_never_downgraded`.
4. A database created before this change (an `entries` table without the
   column, holding rows) opens, gains the column, returns its rows as unverified,
   and keeps `schema_version` 1, so an older Operon still opens it; an insert
   naming only the old columns still succeeds.
   Test: `knowledge_256_an_older_database_gains_verified_without_a_new_version`.
   The GUI's run and an agent's `knowledge search` may open the same old file at
   once, both see the column missing, and both add it: the one whose
   `ALTER TABLE` loses sees "duplicate column name". Adding the column succeeds
   whenever the column is present afterwards, so the loser opens normally.
   Test: `knowledge_256_adding_verified_twice_still_opens` (the add run against
   a table that already has the column returns `Ok`).
5. A distillation run stores what the judge said: a run whose judge answers one
   verified and one unverified item leaves one verified and one unverified entry.
   Test: `knowledge_256_a_run_stores_what_the_judge_said_was_verified`.
6. `Operon knowledge search` shows it: `--json` rows carry `"verified": true|false`;
   the text form puts 検証済み or 未確認 after the kind inside the brackets.
   Test: `knowledge_256_search_says_which_entries_were_verified`.
7. The knowledge tab shows 検証済み or 未確認 on each entry row and in the open
   entry's heading.
   Test: `knowledge_256_the_tab_says_which_entries_were_verified`.

## Behaviour

- Text search output, one entry:
  `[原因・検証済み] 共有の一時ディレクトリ (83%)` then the body on the next line.
  An unconfirmed one: `[原因・未確認] …`. The bracket is built with
  `tf!("{kind}・{status}", …)`.
- JSON adds one key per row: `"verified": true`. No other key changes.
- Knowledge tab, list row: after the kind label, a 12px label 検証済み in
  `text_muted` or 未確認 in `text_faint`. Open entry heading: the same label after
  the kind label. No new icon, colour role, control, or layout.
- Entries kept before this change read 未確認 until a later conversation confirms
  the same knowledge. Nothing judges old entries again.
- Empty, error, too-large, interrupted states are unchanged: the flag rides on
  answers and rows that already exist.

## Design

Placement verdict: **EXTEND** — `src/knowledge.rs` already owns the judge, the
database, and the CLI; `src/app/knowledge.rs` owns the tab
(`git grep -n "fn judge_prompt\|fn add_or_reinforce\|fn ui_knowledge_entry_row" src`).

- `src/knowledge.rs`
  - `judge_prompt`: the JSON object gains `"verified": true | false` and two
    sentences of purpose.
  - `JudgedEntry.verified: bool`; `parse_judge_answer` reads
    `item.get("verified").and_then(Value::as_bool).unwrap_or(false)`.
  - `NewEntry.verified: bool`; `KnowledgeEntry.verified: bool`.
  - Schema: `KNOWLEDGE_SCHEMA` creates `entries` with
    `verified INTEGER NOT NULL DEFAULT 0` for new files. `KnowledgeDatabase::open`
    adds the column to an existing table when `PRAGMA table_info(entries)` lacks
    it, through `add_verified_column`, which runs
    `ALTER TABLE entries ADD COLUMN verified INTEGER NOT NULL DEFAULT 0` and, on
    an error, reads `table_info` again and returns `Ok` when the column is now
    there (another process won the race), the error otherwise.
    Additive, like the `layout` table of change 171: a 157-era build names its
    columns in every `SELECT` and `INSERT`, so it ignores the column and its
    inserts take the default; `KNOWLEDGE_SCHEMA_VERSION` stays 1.
  - `add_or_reinforce`: insert writes `verified`; reinforce runs
    `verified = MAX(verified, ?)`.
  - `process_conversation`: passes `entry.verified` into `NewEntry`.
  - `knowledge_cli`: JSON key, text bracket via `tf!`.
- `src/app/knowledge.rs`: one helper `verified_label(ui, palette, verified)` used
  by the row and the heading. It reads the snapshot already in memory; no I/O in
  the draw path.
- `src/i18n_tables.rs`: rows for 検証済み, 未確認, `{kind}・{status}` in every table.
- No persisted shape outside the private knowledge database; no new child
  process; no new dependency.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | yes, read | uses existing `text_muted` and `text_faint`; status roles (`success` = working right now, per `DESIGN.md`) are not borrowed |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | words only, no icon |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes, read | the column is named in SQL inside `src/knowledge.rs` like every other column of the private database, and Req 4 round-trips an old file through the migration; the JSON key is written once in the CLI and asserted by Req 6 |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | read | the skill governs `src/store.rs`; this is the knowledge database, whose rule (change 157/171) is that an additive shape older builds ignore keeps version 1; Req 4 tests the old file both ways |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | no new child; the judge call is unchanged |
| Documentation — user-facing docs change in all three languages together | yes, read | the knowledge bullets in `README.md`, `README.ja.md`, and `README.ko.md` each gain one line: a note says whether its conversation confirmed it (検証済み / 未確認), and search reports it; the manuals do not describe knowledge |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | none added |
| Budgets — any new scan or output path states its byte and item ceiling | yes | prompt still bounded by `KNOWLEDGE_JUDGE_PROMPT_MAX_BYTES` (Req 1 asserts it); answer still bounded by `KNOWLEDGE_JUDGE_ANSWER_MAX_BYTES` |

## Flagged concerns

- **A judge may call everything verified.** Resolved: the prompt names what
  counts as evidence and what does not; the flag only labels, it never ranks or
  removes, so an over-generous judge costs a wrong label, not lost knowledge.

## Acceptance

- `cargo test --locked knowledge_256_` passes all eight tests, each watched
  failing first.
- `cargo test --locked knowledge_` passes (existing knowledge tests unchanged).
- `make q.fast` clean.
- In the app, an entry distilled from a conversation that ran a passing test
  shows 検証済み in プロジェクト › ナレッジ.

## Rejected alternatives

- A schema version bump: an older Operon would refuse the database for a column
  it can safely ignore.
- Ranking verified entries higher in search: changes what agents find on a label
  nobody has measured; labelling first.
- A separate "evidence" column: the body already holds how it was confirmed, and
  a second field doubles what the 400-character budget must carry.
