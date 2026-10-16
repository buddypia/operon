# Spec: a per-project knowledge base distilled from agent conversations, embedded with EmbeddingGemma 2

- **Intent**: `./intent.md`
- **Status**: draft — stays draft until the person chooses the runtime (Flagged concerns)

## Requirements

Each names the test in `src/tests.rs` that fails without it. Tests run against an
`Embedder` seam (Design): CI uses `FixtureEmbedder`, which returns the reference
vectors stored in `tests/fixtures/embedding-golden.json` for the texts listed
there and refuses any other text, so ranking tests run on real EmbeddingGemma 2
vectors without the 310 MB model. What only the real model can prove is the
fidelity script (15) and the Acceptance run.

1. **Real embeddings.** `compute_in_process_similarity`, `rank_transcripts_in_process`,
   `rank_fallback_candidates_in_process`, `mark_embedded_model_ready`, and the
   `SEMANTIC_*` constants from change 142 are removed; semantic ranking takes its
   vectors from an `Embedder`. — `semantic_search_never_scores_by_word_overlap`
   (with `FixtureEmbedder`, a query sharing no word with the right document
   still ranks it first; a document sharing words but not meaning ranks below).
2. **Pinned, verified model.** `ggml-org/embeddinggemma-2-GGUF`, file
   `embeddinggemma-2-Q8_0.gguf`, revision `bfcd298762cc34d0357ece5ebdd31791a3a374d8`,
   309 855 456 bytes, SHA-256
   `2188ac1deca4b77dffefd603c2776a9d76d9d74ec01841392982ebb840b09135` (read
   from the Hugging Face API and re-hashed locally on 2026-10-16). Ready means: the
   final file exists, and a sidecar `embeddinggemma-2-Q8_0.gguf.verified`, written
   only after the hash matched, records that hash, the size, and the file's
   modification time; readiness is a `stat` compared with the sidecar, never a
   re-hash and never the sidecar alone. —
   `model_is_ready_only_when_size_and_mtime_match_the_verified_sidecar`,
   `model_download_command_is_fixed_and_resumable`.
3. **Prompts and shape.** Queries `task: search result | query: {text}`,
   documents `title: {title, or none} | text: {text}`, exchanges being ranked
   `task: classification | query: {text}`; vectors L2-normalised, 768 floats. —
   `embedding_inputs_carry_the_model_task_prompts`.
4. **Opt-in per project.** No conversation is read for knowledge and no judge is
   called for a project whose knowledge base is not enabled. —
   `pipeline_skips_projects_that_did_not_opt_in`.
5. **Exchanges, incrementally.** A conversation splits into exchanges (one person
   turn and the agent turns answering it). Each conversation keeps a cursor: a run
   ranks and judges only exchanges past it, at most 400 per conversation per run,
   each truncated to 4 000 characters for embedding. —
   `conversation_splits_into_bounded_exchanges`,
   `a_grown_conversation_is_processed_only_past_its_cursor`.
6. **First step ranks, it does not decide.** Exchanges under 120 characters are
   skipped. The rest are ranked by
   `score = mean_top3(cos to KEEP_EXEMPLARS) − mean_top3(cos to DROP_EXEMPLARS) + structure`,
   where the exemplars are labelled example exchanges shipped as constants
   (decision with reason, cause and fix, procedure, pitfall, project fact; versus
   thanks, "continue", raw listings, retry chatter) and `structure` adds 0.02 each
   for a code span, a file path, an error line, and a stated reason (`ため`,
   `because`, `原因`). The top 8 with `score ≥ −0.02` go to the second step.
   Measured on 2026-10-16 with abstract one-line anchors the margins were within
   ±0.05 and did not separate keep from drop; that is why this step is a ranking
   and why exemplars replace anchors. —
   `local_ranking_puts_labelled_keep_exchanges_above_drop_ones` (fixture: every
   labelled keep exchange ranks above the median drop exchange; the labelled test
   exchanges are held out — none is an exemplar, and the test asserts that).
7. **Second step: the conversation's own CLI, outside the project.** One call per
   conversation per run, with the candidates, to the CLI that held it: `claude -p
   --output-format text --permission-mode plan --no-session-persistence`, `codex
   exec --skip-git-repo-check -s read-only --ephemeral`, `agy --print --sandbox`.
   Its working directory is `~/Library/Application Support/com.local.operon/knowledge/judge/`, never the project, so
   no judge run is listed among the project's conversations and no project hook
   fires. 120 s timeout, 16 KiB answer ceiling, prompt at most 64 KiB (each
   candidate cut to its first 800 characters of the person's turn and 1 600 of the
   agent's), at most 30 calls per project per day. The answer is a JSON array of
   0–5 `{kind, title ≤ 40 chars, body ≤ 400 chars, candidate}`. An answer that
   is not that yields no entries and counts as one attempt for that conversation.
   A CLI that is missing, not signed in, timed out, or exited non-zero is a
   pause, not an attempt: the run stops judging, leaves every cursor and attempt
   count as it was, and the next run tries again. —
   `judge_command_is_read_only_ephemeral_and_runs_outside_the_project`,
   `judge_prompt_stays_under_its_ceiling`, `judge_answer_parses_or_yields_nothing`,
   `judge_calls_stop_at_the_daily_cap`.
8. **Not fed by its own output.** An exchange is not a candidate when it contains
   the knowledge command (`KNOWLEDGE_CLI_SUBCOMMAND` followed by `search`), or when
   its vector is within cosine 0.98 of an exchange already recorded as a source
   (Operon's own restore copies a conversation into another CLI's store; the copy
   is not a second occurrence). A person asking the same thing word for word in a
   new conversation is also skipped by that rule; the answer that follows is a
   different exchange and is ranked as usual. —
   `exchanges_quoting_knowledge_results_or_copies_are_not_candidates`.
9. **Dedupe and reinforce.** A new entry within cosine 0.85 of an existing one
   reinforces it (recurrences + 1, source added) instead of being inserted.
   Measured 2026-10-16: paraphrases 0.907–0.927, unrelated project statements
   0.684–0.727. — `a_recurring_entry_reinforces_instead_of_duplicating`.
10. **Freshness and removal.** `freshness = exp(−Δt / τ)`, Δt since the entry was
    last reinforced or referenced, `τ = 30 days × (1 + ln(1 + reference_count +
    recurrences))`. Each run deletes unpinned entries with freshness < 0.10 (69 days
    unused at the smallest τ), then, while more than 2 000 entries remain, the
    least fresh unpinned one. When pinned entries alone reach 2 000, new entries
    are not added and the status says so. —
    `freshness_decays_slower_for_used_entries`,
    `stale_unpinned_entries_are_removed_and_pinned_ones_kept`,
    `a_full_base_of_pinned_entries_stops_adding`.
11. **References count.** Opening an entry in the tab, or an entry being among the
    results printed by the knowledge command, adds one reference. —
    `search_hits_returned_to_an_agent_count_as_references`.
12. **Knowledge search.** Entries ranked by cosine to the query, shown at or above
    0.70 (measured 2026-10-16, query to document: right answers 0.746–0.897, the
    best wrong one 0.717, an off-topic query's best 0.628); each result names its conversations, which open as the CLI session list
    does today. — `knowledge_search_ranks_by_meaning_not_words`.
13. **Session search uses the index.** With the model ready, 「ローカル履歴」 starts a
    second job, separate from today's transcript scan, that ranks the indexed
    conversations (Claude Code, Codex, and Antigravity) of enabled projects by
    cosine ≥ 0.70; its results show as soon as it finishes, above the scan's. —
    `local_history_search_returns_indexed_sessions_first`.
14. **Agents can ask.** `/Applications/Operon.app/Contents/MacOS/Operon knowledge search "retry policy"
    --limit 5 --json` (query free text, `--limit` default 5, `--json` optional) run
    inside a project or one of its worktrees prints that project's entries and
    exits 0. Unknown or not enabled project: Japanese reason on stderr, exit 2.
    Model missing: 「モデルが取得されていません。Operon の設定 › データ で取得してください。」,
    exit 3. llama.cpp's logs are silenced so stdout is only the result. It opens no
    window, takes no instance lock, and writes only reference counts. —
    `knowledge_cli_resolves_the_project_from_a_worktree`,
    `knowledge_cli_refuses_an_unknown_or_disabled_project`.
15. **Fidelity.** `scripts/check-embedding-fidelity.sh` embeds the golden texts
    with the shipped runtime and fails below cosine 0.995 to their reference
    vectors. `embedding_golden_fixture_is_well_formed` checks the fixture (768
    floats each, unit length) and that every exemplar constant is in it, so an
    exemplar cannot change without regenerating its vector.
16. **Private on disk.** The knowledge directory is created 0700 and each
    database 0600 before SQLite opens it; SQLite gives its `-wal` and `-shm` the
    database's mode. — `knowledge_files_are_private`.
17. **Conversations that cannot be finished.** A transcript the reader refuses
    (over `TRANSCRIPT_FILE_MAX_BYTES` or its line limit) or cannot read is marked
    `too-large` or `unreadable` with its size and mtime and is not read again
    until either changes. Three unparseable answers mark it `failed`; `failed`
    is tried again after 7 days or when the transcript changes, and the tab's
    「処理できなかった会話 {n} 件 [再試行]」 clears every `failed` mark at once. A
    paused CLI (Req 7) never moves a conversation toward `failed`. —
    `unfinishable_conversations_are_not_reread_every_run`,
    `an_unavailable_judge_pauses_without_spending_attempts`.
18. **Store shape.** One SQLite file per project, WAL, 2 s busy timeout, a
    `schema_version` the code refuses when newer than it knows. —
    `knowledge_database_round_trips_and_refuses_a_newer_schema`.

19. **Cancellable children.** `run_command_cancellable` kills its child within
    0.1 s of the flag being set and returns a cancelled error;
    `run_command_with_output_limit` behaves as before. —
    `cancelled_command_is_killed_and_reported`,
    `output_limited_command_behaves_as_before`.
20. **The CLI's output contract.** With `--json`, stdout is one JSON array and
    nothing else; with the model missing it exits 3 and prints nothing on stdout. —
    `knowledge_cli_json_is_the_only_stdout`,
    `knowledge_cli_exits_3_without_the_model`.
21. **A database from another model.** When `meta.model` differs from the pinned
    model, the database's vectors are not compared with new ones: entries and
    sessions are re-embedded from their stored text on the next run (no judge
    call), then `meta.model` is updated. —
    `vectors_from_another_model_are_rebuilt_not_mixed`.

## Behaviour

### Settings › データ — the model

Replaces change 142's block. The existing `search_engine_mode` radio stays and is
re-labelled 「キーワードのみ」 / 「意味検索」; 「意味検索」 takes effect only when the
model is ready. A store that already says `EmbeddedGemma2` (from change 142) shows
「意味検索」 selected with 「モデルを取得 (310MB)」 beneath it and searches by keyword
until the model is ready.

- Not downloaded: 「意味検索（EmbeddingGemma 2）」, 「この Mac 上で動くモデルです。
  会話の内容は外部に送りません。」, 「モデルを取得 (310MB)」.
- Downloading: progress bar, 「取得中… {done}MB / {total}MB」, 「中止」. 「中止」 kills
  curl; the `.part` file stays and the next 「モデルを取得」 resumes it.
- Verifying: 「確認中…」.
- Ready: 「使用可能 ({device} / 310MB)」, device 「GPU (Metal)」 or 「CPU」 as the
  runtime reports, 「モデルを削除」.
- Failed: 「取得できませんでした: {reason}」 — 「ネットワークに接続できませんでした」,
  「空き容量が足りません」, 「ファイルが壊れていたため破棄しました」 — and 「再試行」.
- Deleting the model keeps every knowledge base; collection and search pause and
  each enabled project's tab says 「モデルがないため停止中」.
- Change 142's directory `…/models/embeddinggemma-2-270m` is removed on first
  launch when it holds only `model_ready.json`.

### Project › ナレッジ (new tab)

As drawn in `screen.md`. Disabled: what is collected, where it is stored, that
candidates go to the conversation's own CLI, and 「このプロジェクトでナレッジを集める」
(or 「モデルを取得して始める (310MB)」 without a model). Enabled: 「ナレッジを検索」,
status 「{n} 件 ・ 最終更新 {time} ・ 本日の判定 {used}/{cap} 回」, rows (kind badge
決定/原因/手順/注意/事実, title, freshness bar, 「参照 {n}」, CLI and date), an opened
row shows body, sources with 「会話を開く」, 「固定」/「固定を外す」, 「削除」.

- Empty: 「まだナレッジはありません。このプロジェクトで会話を続けると、重要なやり取りがここに追加されます。」
- Running: 「会話を整理しています… ({done}/{total})」.
- Judge unavailable: 「{cli} で判定できませんでした。次回の更新で再試行します。」
- Cap: 「本日の判定回数の上限に達しました。明日続きを処理します。」
- Full of pinned entries: 「固定したナレッジが上限 (2000 件) に達したため、新しいナレッジを追加していません。」
- 「収集を止める ▾」: 「収集を止める (ナレッジは残す)」, 「ナレッジを削除…」 → inline
  「{n} 件のナレッジを削除します。元に戻せません。 [削除する] [キャンセル]」.
- 「エージェントから参照: /Applications/Operon.app/Contents/MacOS/Operon knowledge search "…"」 (the running executable's own path) with 「コピー」.

A project removed from Operon keeps its knowledge file; re-adding the same path
does not find it (new id). Removing it is out of scope; stated in `README.md`.

### Sessions › ローカル履歴

Model ready: indexed results first under 「意味で一致」 with their project; today's scan
under 「キーワードで一致」. No model: today's behaviour; the zero-result hint points
at Settings › データ.

### When it runs

30 s after launch, every 10 minutes, and when a project is enabled. A conversation
is processed once its transcript has been unchanged for 5 minutes. Per run, per
project: the 200 newest conversations from `scan_native_cli_sessions` (newest
first), at most 20 processed, and at most 1 500 embeddings or 3 minutes of wall
clock; the rest wait for the next run. Enabled projects run one after another in one
background job, so the embedding and judge budgets above apply per project and
a run over many projects takes longer, never runs two at once. Background thread only; the frame reads
the last result.

## Design

**Placement**: `NEW_MODULE`. `src/knowledge.rs` — pipeline, ranking, freshness,
database, search, CLI entry. `src/knowledge/embed.rs` — `Embedder` trait,
`LlamaEmbedder`, model file, download, readiness. `git grep` for knowledge and
embedding finds only change 142's stand-ins in `src/cli.rs`, removed here; no
module owns this concern. `CLAUDE.md` gains both lines.

**Embedder seam**: `trait Embedder { fn embed(&self, texts: &[String]) ->
anyhow::Result of one 768-float vector per text; fn device(&self) -> Device; }`. `LlamaEmbedder` (production)
and `FixtureEmbedder` (`#[cfg(test)]`).

**Runtime** (pending the person, Flagged concerns): `llama-cpp-2`,
`default-features = false, features = ["metal"]`. One process-wide
`Mutex` around an optional `LlamaEmbedder`, locked per `embed` call of at most 8 texts and
2 048 tokens in total, so a query waits behind at most one such call (8 texts of
225 tokens took 0.13 s, measured 2026-10-16). Loaded lazily on a
background thread, dropped after 10 idle minutes. Context 2 048 tokens, mean
pooling; longer text truncated at a token boundary. Measured on this Mac
(2026-10-16, probes not committed): load 0.3–0.5 s, a 549-token document 60 ms, a
short query 10 ms, cosine to sentence-transformers 0.9999–1.0000.

**Download**: `curl --location --fail --silent --show-error --continue-at -
--max-time 1800 --output embeddinggemma-2-Q8_0.gguf.part` followed by the pinned URL. `src/exec.rs` gains
`run_command_cancellable(command, timeout, limits, cancel: &AtomicBool)` — the
same loop as `run_command_with_output_limit`, which becomes a call to it with a
flag that is never set; the existing behaviour is unchanged. 「中止」 and app exit
(`on_exit`) set the flag and the child is killed. Progress is the `.part` size,
stat-ed once a second by the download thread. After curl: `shasum -a 256` through
`run_command_with_timeout` (120 s); match → rename to the final name, then write
the sidecar atomically (`write_file_atomically`); mismatch → delete `.part`. A
crash leaves curl to its own `--max-time`; a resume racing an orphan can only
produce a hash mismatch, which discards and re-fetches. A final file without a sidecar (crash between
rename and sidecar) is re-hashed once, not re-fetched; a `.part` already at full
size skips curl (a range request past the end returns 416) and goes straight to
the hash.

**Database** `~/Library/Application Support/com.local.operon/knowledge/{project uuid}.sqlite3` (`rusqlite`, already a
dependency): `meta(key, value)` — `schema_version`, `enabled`, `model`,
`dimensions`; `entries(id, kind, title, body, created_at, reinforced_at,
referenced_at, reference_count, recurrences, pinned, vector)`;
`sources(entry_id, provider, session_id, transcript_path, exchange_index, vector)`;
`sessions(provider, session_id, transcript_path, mtime, size, cursor, state,
attempts, processed_at, title, vector)`; `judge_calls(day, count)`. Every value
through `rusqlite::params!`; one transaction per conversation. GUI and CLI may
open the same file: WAL, 2 s busy timeout, a busy write retried next run.

**Opt-in** lives in each database's `meta.enabled`. `src/store.rs` and
`src/models.rs` gain nothing; `STORE_SCHEMA_VERSION` does not move. (The intent's
guess that the flag would live in the store is corrected here.)

**Reading conversations**: `scan_native_cli_sessions` (`src/cli.rs`, worktrees
included through `project_and_worktree_paths`) lists them;
`read_transcript_conversation` (`src/history.rs`) reads classified turns from the
`CliSession.path` each scan returns, Antigravity's included, so
`TRANSCRIPT_VOCABULARY` stays the one judge of record kinds.

**Judge command**: `knowledge_judge_command(provider, prompt)` in `src/agents.rs`,
next to `commit_message_command`, run through `run_command_with_output_limit`.

**CLI entry**: `src/main.rs` branches on `std::env::args().nth(1) ==
Some(KNOWLEDGE_CLI_SUBCOMMAND)` before `configure_command_path()` and the instance
lock, calls `knowledge::run_cli(args) -> i32` and exits with it. The branch adds
no `unsafe` and does not touch `PATH` resolution; the CLI uses the caller's
`PATH`. It reads `store-v2.json` read-only to map the working directory to a
project: the directory's `git rev-parse --show-toplevel` (or, without git, its
ancestors) against each project's path and worktree paths.

**App wiring**: `ProjectTab::Knowledge`; `BackgroundKey::{KnowledgeRun(Uuid),
KnowledgeSearch, KnowledgeSessionSearch, ModelDownload, ModelReadiness}`;
`OperonApp` holds `KnowledgeView` (entries, status, last search) and
`model_state`, both written only by background results.

**Constants** (`src/config.rs`): the model URL, revision, file name, size, hash;
`KNOWLEDGE_CLI_SUBCOMMAND = "knowledge"`; `KNOWLEDGE_DAILY_JUDGE_CALLS = 30`,
`KNOWLEDGE_MAX_ENTRIES = 2000`, `KNOWLEDGE_DEDUPE_COSINE = 0.85`,
`KNOWLEDGE_COPY_COSINE = 0.98`, `KNOWLEDGE_TAU_DAYS = 30`,
`KNOWLEDGE_REMOVE_FRESHNESS = 0.10`, `KNOWLEDGE_CANDIDATE_MIN_SCORE = -0.02`,
`KNOWLEDGE_SEARCH_MIN_COSINE = 0.70`, `KNOWLEDGE_SESSIONS_PER_RUN = 20`,
`KNOWLEDGE_EMBEDDINGS_PER_RUN = 1500`, `KNOWLEDGE_RUN_SECONDS = 180`,
`KNOWLEDGE_IDLE_SECONDS = 300`, `KNOWLEDGE_JUDGE_PROMPT_MAX_BYTES = 65536`.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | Yes | Freshness bar and kind badges use existing roles (`accent`, `muted`, `warning`); no new role. Checked again at the screen against `DESIGN.md`. |
| Icons | Yes | `ICON_KNOWLEDGE` (Phosphor) in `src/glyphs.rs` with an `ICON_VOCABULARY` entry, for the tab. |
| Identifier SSOT | Yes | Subcommand word, model file name, hash, and database file name are `src/config.rs` constants used by both the GUI and the CLI; the copied line in the tab is built from the same constant and tested. |
| Durability | Yes | `store-v2.json` untouched. Knowledge database versioned and refusing a newer schema; files private (16); model renamed only after its hash matches, readiness bound to size and mtime (2). `durability-reviewer` reviews. |
| Subprocess safety | Yes | `curl`, `shasum`, `git`, and the three judge CLIs go through `src/exec.rs` with the timeouts and ceilings stated; curl is cancellable and killed on exit; judge arguments are fixed switches plus one prompt, no shell; judges run outside the project. `subprocess-safety-reviewer` reviews. |
| Documentation | Yes | `README.md` in all three languages: the knowledge base, the model download, cmake for building, the agent command, what is sent to which CLI. |
| Local-first | Yes | Embeddings run locally; the model is fetched only on the person's click; the judge is the CLI that already holds that conversation. No `unsafe` added in Operon's sources. |
| Budgets | Yes | Per run per project: 200 listed, 20 processed, 1 500 embeddings or 180 s, 400 exchanges × 4 000 chars, 8 candidates, one 64 KiB prompt and 16 KiB answer per conversation, 30 judge calls a day; 2 000 entries; model 310 MB; embed batches of 8. |

## Flagged concerns

- **Runtime is the person's decision (`dependencies`, paused).** Recommended
  `llama-cpp-2`. Its published 0.1.160 vendors llama.cpp from 2026-10-05, one day
  before EmbeddingGemma 2 support (ggml-org/llama.cpp#30054); with upstream
  llama.cpp swapped in, it works (numbers above). Until a release vendors it, the
  dependency is a git revision of a fork carrying only the submodule bump and a
  two-field fix to `timing.rs`. That means: a public fork under the person's
  account (outward-facing, waits for them); `Cargo.lock` pinning a git revision
  that review reads as such; and building Operon from source fetching the fork
  and its llama.cpp submodule. The fork is replaced by the crates.io release when
  one carries the support. Measured alternatives: `ort` rc.11 + ONNX int8 (same
  fidelity, CPU only, 206 ms per 549 tokens; its CoreML provider rejects unbounded
  shapes and, padded, ran at 180 ms); Ollama, LM Studio, `llama-server` (Metal,
  but installed by the person).
- **Build needs cmake** (and Xcode's clang). GitHub's `macos-latest` has it;
  `README.md` says so for contributors.
- **Judge excerpts reach the CLI's vendor** — only conversations held in that same
  CLI. Said in the disabled-state text.
- **Antigravity keeps its judge runs.** `agy` has no ephemeral switch, so each
  judge call stays in the person's own Antigravity history, under the judge
  directory rather than the project. `README.md` says so. Whether `agy --print`
  asks to trust that directory is unverified; if it does, the call times out and
  is a pause (Req 7), and stage 3 checks it before relying on it.
- **Intel Macs have no Metal path this change claims.** llama.cpp runs the same model on their CPU (slower, untested here: no Intel Mac is available); the Ready label says 「CPU」 so it never claims a GPU it is not using.
- **First-step quality is unproven on real conversations.** The fixture shows
  ordering on labelled examples; whether the second step keeps most of what the
  first passes is observed in the Acceptance run and recorded in `plan.md`.

## Acceptance

- `cargo test --locked` passes, including every test named above.
- `bash scripts/check-embedding-fidelity.sh` prints every cosine ≥ 0.995 with the
  model downloaded.
- In the app on this Mac: Settings › データ fetches the model, shows progress,
  survives 「中止」 and resumes, and ends at 「使用可能 (GPU (Metal) / 310MB)」.
- Enable 「ナレッジ」 for Operon; after a run, entries drawn from this repository's
  recent conversations appear, each opening its conversation; 「復元で最初のターンが
  消える」 finds a restore-related entry; no judge conversation appears in the
  project's CLI session list.
- From a worktree shell, `/Applications/Operon.app/Contents/MacOS/Operon knowledge search "restore" --json` prints JSON
  only, and the entry's 「参照」 rises in the tab.
- `ls -l` on the knowledge directory shows `drwx------` and `-rw-------`.

## Rejected alternatives

- Keep change 142's heuristic as a fallback — it is what made the label untrue.
- The eg2 server — the person wants Operon distributable without it.
- `candle` / `mlx-rs` — no EmbeddingGemma 2 implementation; writing one is the
  fidelity risk this spec measures against.
- Abstract anchor sentences for the first step — measured, they do not separate.
- A field in `store-v2.json` for the opt-in — moves the store schema for a flag
  the knowledge database can hold.
- Storing knowledge in the project's repository — conversations hold secrets.
- Judging every exchange with the agent CLI — spends quota on small talk.
- Running the judge in the project directory — its own run becomes a conversation
  the pipeline would judge next.
- Injecting knowledge at session start through hooks — out of scope; agents ask.
