# Spec: a per-project knowledge base distilled from agent conversations, embedded with EmbeddingGemma 2

- **Intent**: `./intent.md`
- **Status**: draft

## Requirements

Each names the test in `src/tests.rs` that fails without it.

1. **Real embeddings.** The keyword-overlap heuristic and the marker-file
   "download" from change 142 are gone; `SearchEngineMode::EmbeddedGemma2` means
   vectors from EmbeddingGemma 2. —
   `semantic_search_never_scores_by_word_overlap`,
   `model_is_ready_only_when_the_pinned_file_hashes_match`.
2. **Pinned model.** The model is `ggml-org/embeddinggemma-2-GGUF` file
   `embeddinggemma-2-Q8_0.gguf` at revision
   `bfcd298762cc34d0357ece5ebdd31791a3a374d8`, 309 855 456 bytes, SHA-256
   `2188ac1deca4b77dffefd603c2776a9d76d9d74ec01841392982ebb840b09135`. A file
   whose size or hash differs is never loaded. —
   `model_is_ready_only_when_the_pinned_file_hashes_match`,
   `model_download_command_is_fixed_and_resumable`.
3. **Prompts.** Queries are embedded as `task: search result | query: <text>`,
   documents as `title: <title or none> | text: <text>`, classification as
   `task: classification | query: <text>`; vectors are L2-normalised and stored
   at 768 dimensions. — `embedding_inputs_carry_the_model_task_prompts`.
4. **Opt-in per project, nothing by default.** No conversation is read for
   knowledge, and no agent CLI is called, for a project whose knowledge base is
   not enabled. — `pipeline_skips_projects_that_did_not_opt_in`.
5. **Exchanges.** A conversation is split into exchanges (one person turn and
   the agent turns that answer it), each capped at 4 000 characters, at most
   400 per conversation. — `conversation_splits_into_bounded_exchanges`.
6. **Local first judgement.** Each exchange is scored against fixed anchor
   descriptions (worth keeping: decision and reason, cause and fix, procedure,
   pitfall, project fact; not worth keeping: greeting, progress check, file
   listing, retry chatter). At most 8 exchanges per conversation, scoring at or
   above `KNOWLEDGE_CANDIDATE_MIN_SCORE`, go further. —
   `local_judge_keeps_only_the_top_scoring_exchanges`.
7. **Second judgement by the conversation's own CLI.** The candidates go, in one
   call per conversation, to the CLI the conversation was held in (Claude Code →
   `claude -p … --permission-mode plan`, Codex → `codex exec -s read-only`,
   Antigravity → `agy --print --sandbox`), with a 120 s timeout, a 16 KiB output
   ceiling, and at most `KNOWLEDGE_DAILY_JUDGE_CALLS` (30) calls per project per
   day. Its JSON answer (0–5 entries of kind, title ≤ 40 characters, body ≤ 400
   characters, candidate index) is parsed defensively; anything else yields no
   entries and the conversation is retried once on a later run. —
   `judge_command_uses_the_conversations_cli_read_only`,
   `judge_answer_parses_or_yields_nothing`, `judge_calls_stop_at_the_daily_cap`.
8. **Dedupe and reinforce.** A new entry within cosine 0.90 of an existing one
   reinforces that entry (recurrence + 1, new source added) instead of being
   inserted. — `a_recurring_entry_reinforces_instead_of_duplicating`.
9. **Freshness and removal.** Freshness is `exp(-Δt / τ)`, where Δt is the time
   since the entry was last reinforced or referenced and
   `τ = 30 days × (1 + ln(1 + references + recurrences))`. Each pipeline run
   deletes unpinned entries older than 14 days whose freshness is below 0.10, and,
   above 2 000 entries, the least fresh unpinned ones. Pinned entries never
   decay out. — `freshness_halves_slower_for_used_entries`,
   `stale_unpinned_entries_are_removed_and_pinned_ones_kept`.
10. **References count.** Opening an entry in the Knowledge tab, or an entry being
    among the results an agent receives from `operon knowledge search`, records a
    reference. — `search_hits_returned_to_an_agent_count_as_references`.
11. **Search.** Knowledge search ranks entries by cosine to the query; a 2 000
    entry project answers from memory without reading transcripts. Each result
    names its conversation(s), which open as the CLI session list does today. —
    `knowledge_search_ranks_by_meaning_not_words`.
12. **Session search uses the index.** With the model ready, 「ローカル履歴」 also
    ranks the indexed conversations of enabled projects by meaning, before the
    raw transcript scan finishes. —
    `local_history_search_returns_indexed_sessions_first`.
13. **Agents can ask.** `Operon.app/Contents/MacOS/Operon knowledge search
    "retry policy" --limit 5 --json` (query free text, `--limit` and `--json`
    optional) run inside a project directory (or one of its
    worktrees) prints that project's top entries and exits 0; outside a known,
    enabled project it prints a Japanese reason on stderr and exits 2; it never
    opens a window or takes the instance lock. —
    `knowledge_cli_resolves_the_project_from_a_worktree`,
    `knowledge_cli_refuses_an_unknown_or_disabled_project`.
14. **Store shape.** The knowledge base is one SQLite file per project under the
    app data directory, opened in WAL mode with a busy timeout, with a
    `schema_version` the code refuses to read when newer than it knows. —
    `knowledge_database_round_trips_and_refuses_a_newer_schema`.
15. **Fidelity.** `scripts/check-embedding-fidelity.sh` embeds the four golden
    texts in `tests/fixtures/embedding-golden.json` (reference vectors from
    sentence-transformers) with the shipped runtime and fails below cosine 0.995.
    Its own fixture is checked by `embedding_golden_fixture_is_well_formed`.

## Behaviour

### Settings › データ — the model

Replaces change 142's block. States and strings:

- Not downloaded: 「意味検索（EmbeddingGemma 2）」, 「この Mac 上で動くモデルです。
  会話の内容は外部に送りません。」, button 「モデルを取得 (310MB)」.
- Downloading: progress bar, 「取得中… {done}MB / {total}MB」, button 「中止」. The
  partial file stays so a later 「モデルを取得」 resumes it.
- Verifying: 「確認中…」.
- Ready: 「使用可能 ({device} / {size}MB)」 where device is 「GPU (Metal)」 or
  「CPU」 as reported by the runtime, button 「モデルを削除」.
- Failed: 「取得できませんでした: {reason}」 (network, disk full, hash mismatch:
  「ファイルが壊れていたため破棄しました」), button 「再試行」.
- Deleting the model while a project has knowledge enabled keeps the entries;
  search and collection pause with 「モデルがないため停止中」 in that project's tab.

### Project › ナレッジ (new tab)

See `screen.md`. Disabled state explains what is collected and offers 「このプロジェクトで
ナレッジを集める」; enabling without a model offers 「モデルを取得」 first. Enabled
state: search field 「ナレッジを検索」, status line 「{n} 件 ・ 最終更新 {time} ・ 本日の判定 {used}/{cap} 回」,
entry rows (kind badge, title, freshness bar, 「参照 {n}」, source CLI and date), an
entry opens to its body, sources (「会話を開く」), 「固定」/「固定を外す」, 「削除」.
Empty enabled state: 「まだナレッジはありません。このプロジェクトで会話を続けると、
重要なやり取りがここに追加されます。」 Collection in progress: 「会話を整理しています… ({done}/{total})」.
Judge CLI missing or failing: 「{cli} で判定できませんでした。次回の更新で再試行します。」 on
the status line, never a modal. Daily cap reached: 「本日の判定回数の上限に達しました。明日続きを処理します。」
Turning collection off: 「収集を止める」 keeps entries; 「ナレッジを削除」 removes the
file after a confirmation inside the tab (no OS dialog).

The tab shows the line agents need: 「エージェントから参照: `<path to Operon> knowledge search "…"`」
with 「コピー」.

### Sessions › ローカル履歴

With the model ready, indexed results appear first labelled with their project;
the existing scan's results follow. Without the model, behaviour is today's
keyword search, and the zero-result hint points at Settings › データ.

### When it runs

On launch (after 30 s), every 10 minutes, and when the person enables a project.
A conversation is processed when its transcript has not changed for 5 minutes,
so a live conversation is not judged mid-way. At most 20 conversations per run.
All of it on a background thread; the frame reads only the last result.

## Design

**Placement**: `NEW_MODULE` — `src/knowledge.rs` (pipeline, decay, database,
search, CLI entry) and `src/knowledge/embed.rs` (model file, download, runtime).
`git grep` for knowledge/embedding found only change 142's stand-ins in
`src/cli.rs` (`compute_in_process_similarity`, `rank_*_in_process`,
`embedded_model_*`, `mark_embedded_model_ready`), which this change removes; no
module owns a knowledge concern today. `CLAUDE.md`'s map gains both lines.

**Runtime**: `llama-cpp-2` with `default-features = false, features = ["metal"]`
(the person's decision is pending; see Flagged concerns). One process-wide
`Embedder` behind a `Mutex` (inference is serialised, as eg2 does for MPS),
loaded lazily on a background thread, dropped after 10 idle minutes. Context
2 048 tokens, mean pooling; longer text is truncated at a token boundary.
Measured on this Mac (M-series, 2026-10-16): load 0.3–0.5 s, a 549-token
document 60 ms, a short query 10 ms, cosine to the sentence-transformers
reference 0.9999–1.0000.

**Download**: `curl -L --fail -C - --silent -o <dir>/<file>.part <pinned URL>`
through `run_command_with_timeout` (30 min, 4 KiB output ceiling) on a
background thread; progress is the `.part` file's size polled by that thread
once a second. Then SHA-256 over the file (`shasum -a 256` through the same
helper, 2 min), rename to the final name, write nothing else. Cache directory:
`embedded_model_directory()` (kept), now `…/models/embeddinggemma-2-q8_0`.

**Database** (`<app data>/knowledge/<project uuid>.sqlite3`, `rusqlite`, already
a dependency): tables `meta(key, value)` (`schema_version`, `enabled`,
`model`, `dimensions`), `entries(id, kind, title, body, created_at,
reinforced_at, referenced_at, references, recurrences, pinned, vector)`,
`sources(entry_id, provider, session_id, transcript_path, exchange_index)`,
`sessions(provider, session_id, transcript_path, mtime, size, processed_at,
attempts, title, vector)`, `judge_calls(day, count)`. All statements use
`rusqlite::params!`; writes in one transaction per conversation. The GUI and the
CLI may open the same file; WAL and a 2 s busy timeout make that safe, and a busy
write is retried on the next run.

**Opt-in** lives in the per-project database's `meta.enabled`, not in
`store-v2.json`: `src/store.rs` and `src/models.rs` gain no field and
`STORE_SCHEMA_VERSION` does not move. `SearchEngineMode` keeps its variants.

**Reading conversations**: `scan_native_cli_sessions` (`src/cli.rs`) lists them;
`read_transcript_conversation` (`src/history.rs`) reads the classified turns, so
`TRANSCRIPT_VOCABULARY` stays the one judge of record kinds.

**Judge command**: `knowledge_judge_command(provider, prompt)` in `src/agents.rs`,
built like `commit_message_command` (same read-only switches), run through
`run_command_with_output_limit` with `cwd` = the project.

**CLI entry**: `src/main.rs` checks `std::env::args().nth(1) == Some("knowledge")`
before `configure_command_path()` and the instance lock, and calls
`knowledge::run_cli(args) -> i32`. The CLI reads `store-v2.json` read-only to
map the working directory (via `git rev-parse --show-toplevel` and the worktree
list) to a project id.

**App wiring**: `ProjectTab::Knowledge`; `BackgroundKey::{KnowledgeRun(project),
KnowledgeSearch, ModelDownload}`; the draw path reads `KnowledgeView` (entries,
status, last search) held on `OperonApp`.

**Constants** (`src/config.rs`): `KNOWLEDGE_CANDIDATE_MIN_SCORE`,
`KNOWLEDGE_DAILY_JUDGE_CALLS = 30`, `KNOWLEDGE_MAX_ENTRIES = 2000`,
`KNOWLEDGE_DEDUPE_COSINE = 0.90`, `KNOWLEDGE_TAU_DAYS = 30`,
`KNOWLEDGE_REMOVE_FRESHNESS = 0.10`, `KNOWLEDGE_MIN_AGE_DAYS = 14`,
`KNOWLEDGE_SESSIONS_PER_RUN = 20`, `KNOWLEDGE_IDLE_SECONDS = 300`, the model
URL, revision, size and hash, and `KNOWLEDGE_CLI_SUBCOMMAND = "knowledge"`
(shared by `src/main.rs` and the copied line in the tab).

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | Yes | The freshness bar and kind badges use existing roles (`accent`, `muted`, `success`, `warning`); no new role. To be confirmed against `DESIGN.md` at the screen. |
| Icons | Yes | The tab needs one mark; `ICON_KNOWLEDGE` (Phosphor `BRAIN` or `LIGHTBULB`) in `src/glyphs.rs` with an `ICON_VOCABULARY` entry. |
| Identifier SSOT | Yes | The subcommand word, model file name and hash, and database file name are constants in `src/config.rs`, read by both the GUI and the CLI; the CLI round trip is tested. |
| Durability | Yes | No change to `store-v2.json`. The knowledge database is separate, versioned, and refuses a newer schema; the model is written as `.part` and renamed only after the hash matches. Read against `.claude/skills/durability-invariants/SKILL.md` before stage 3; `durability-reviewer` reviews. |
| Subprocess safety | Yes | `curl`, `shasum`, `git`, and the three judge CLIs go through `src/exec.rs` with the timeouts and ceilings above; judge arguments are fixed switches plus one prompt argument, no shell. `subprocess-safety-reviewer` reviews. |
| Documentation | Yes | `README.md` (all three languages) gains the knowledge base, the model download, and the agent command. |
| Local-first | Yes | Embeddings run locally; the model is fetched only on the person's click; the judge is the CLI that already holds the conversation. No new `unsafe` in Operon (the crate's FFI is its own). |
| Budgets | Yes | Per run: 20 conversations, the existing per-file transcript ceiling, 400 exchanges × 4 000 characters, 8 candidates, one judge call with a 24 KiB prompt and 16 KiB answer, 30 calls/day; 2 000 entries per project; model 310 MB. |

## Flagged concerns

- **Runtime crate is the person's decision (new dependency, `paused`).**
  Recommended: `llama-cpp-2`. Its published 0.1.160 vendors llama.cpp from
  2026-10-05, one day before EmbeddingGemma 2 support (ggml-org/llama.cpp#30054);
  measured with upstream llama.cpp swapped in, it works. Until a release vendors
  it, the dependency is a git revision of a fork whose only changes are the
  llama.cpp submodule bump and the two-field fix in `timing.rs`; creating that
  fork is outward-facing and waits for the person. Alternatives measured: `ort`
  1.88-compatible rc.11 + ONNX int8 (same fidelity, CPU only, 206 ms per 549
  tokens; CoreML provider rejects unbounded shapes and, with padding, is no
  faster); Ollama / LM Studio / `llama-server` (Metal, but the person must install
  them).
- **Build needs cmake.** GitHub's `macos-latest` image ships it; a contributor
  without it gets the crate's build error. `README.md` names it.
- **Judge sends excerpts to the CLI's vendor.** Only conversations from that same
  CLI, which already hold that text. Stated in the disabled-state explanation.
- **Intel Macs** run the same model on the CPU (no Metal acceleration claimed;
  the Ready label says 「CPU」).

## Acceptance

- `cargo test --locked` passes, including every test named above.
- `bash scripts/check-embedding-fidelity.sh` prints all four cosines ≥ 0.995 on
  this Mac with the model downloaded.
- In the running app: enable 「ナレッジ」 for Operon itself, wait for a run, and see
  entries drawn from this repository's recent conversations, each opening its
  conversation; a search for 「復元で最初のターンが消える」 returns a restore-related entry.
- From a worktree shell, `Operon knowledge search "restore" --json` prints entries
  and the tab's 「参照」 count rises.
- The settings label says 「GPU (Metal)」 on this Mac.

## Rejected alternatives

- Keep change 142's heuristic as a fallback — it is what made the label untrue.
- The eg2 server — the person wants Operon distributable without it.
- `candle` / `mlx-rs` — no EmbeddingGemma 2 implementation; writing one is the
  fidelity risk this spec measures against.
- A field in `store-v2.json` for opt-in — moves the store schema for a flag the
  knowledge database can hold itself.
- Storing knowledge in the project's repository — conversations hold secrets.
- Judging every exchange with the agent CLI — spends the person's quota on small talk.
- Injecting knowledge at session start through hooks — out of scope; agents ask.
