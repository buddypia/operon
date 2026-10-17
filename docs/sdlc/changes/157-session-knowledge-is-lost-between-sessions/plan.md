# Plan: a per-project knowledge base distilled from agent conversations, embedded with EmbeddingGemma 2

- **Spec**: `./spec.md`
- **Approved**: 2026-10-17
- **Status**: approved

The person chose the runtime on 2026-10-17 (「OK」 to "llama-cpp-2 through a
public fork until a release carries the support") and approved `screen.md` in
the same answer. The fork exists: `https://github.com/buddypia/llama-cpp-rs`,
branch `operon/embeddinggemma-2`, commit
`48eedeb3c5b21ceafc06b4fae953a0d332d1e037` — upstream `224a1a34` (0.1.161) plus
the llama.cpp submodule moved to `2a667032dc788c3a071d538e0b572e45a3f12731`
(carries ggml-org/llama.cpp#30054) and the two zeroed fields in
`llama-cpp-2/src/timing.rs`. Built against it with rustc 1.88.0 on 2026-10-17:
Metal, 225 tokens in 13 ms, cosine to the reference 0.9999–1.0000 on the four
probe texts.

## Files that change

| File | Change |
|---|---|
| `Cargo.toml`, `Cargo.lock` | `llama-cpp-2 = { git = "https://github.com/buddypia/llama-cpp-rs", rev = "48eedeb3…", default-features = false, features = ["metal"] }`; comment saying why a git revision and when it goes back to crates.io |
| `src/knowledge.rs` (new) | database (open, schema, migrate-refuse, privacy), exchanges and cursor, ranking, self-feed filter, judge prompt and answer parser, dedupe/reinforce, freshness and removal, pipeline run with budgets, search, session index search, CLI entry `run_cli` |
| `src/knowledge/embed.rs` (new) | `Embedder` trait, `Device`, `LlamaEmbedder`, process-wide lazy `Mutex`, prompt helpers, model paths, readiness (`stat` vs `.verified`), download/verify job, `FixtureEmbedder` (`#[cfg(test)]`) |
| `src/exec.rs` | `run_command_cancellable`; `run_command_with_output_limit` becomes a call to it with a flag never set |
| `src/agents.rs` | `knowledge_judge_command(provider, prompt)` beside `commit_message_command` |
| `src/cli.rs` | remove change 142's stand-ins (`SEMANTIC_*`, `embedded_model_*`, `mark_embedded_model_ready`, `compute_in_process_similarity`, `rank_*_in_process`) and the `mode` branch in `search_local_transcripts_in_with_mode` |
| `src/models.rs` | `EmbeddedModelDownloadState` replaced by `ModelState` (Missing / Downloading{done,total} / Verifying / Ready{device} / Failed(reason)). Neither derives `Serialize`, so no persisted shape moves; `SearchEngineMode` (persisted) unchanged |
| `src/config.rs` | the constants listed in spec Design, plus the model URL built from repo, revision, file |
| `src/main.rs` | `mod knowledge;` and re-export; the `knowledge` argument branch before `configure_command_path()` — no `unsafe`, no `PATH` change |
| `src/app.rs` | `ProjectTab::Knowledge`; `BackgroundKey::{KnowledgeRun, KnowledgeSearch, KnowledgeSessionSearch, ModelDownload, ModelReadiness}`; matching `BackgroundResult`s; `KnowledgeView` per project; `model_state`; download cancel flag set in `on_exit`; run schedule (30 s, 10 min, on enable) |
| `src/app/screens.rs` | the ナレッジ tab as drawn in `screen.md`; Settings › データ model block replaced; ローカル履歴 「意味で一致」 group above today's results |
| `src/glyphs.rs` | `ICON_KNOWLEDGE` and its `ICON_VOCABULARY` row |
| `src/i18n_tables.rs` | a row in every table for every new string |
| `src/tests.rs` | the tests the spec names; change 142's four tests (`in_process_similarity_and_ranking_handles_edge_cases`, `rank_transcripts_in_process_handles_edge_cases`, `embedded_model_cache_lifecycle_and_state`, `in_process_semantic_search_ranks_or_falls_back`) removed with the code they pinned |
| `tests/fixtures/embedding-golden.json` (new) | reference vectors (eg2 server, 768-d) for the exemplars, the ranking/search/dedupe test texts, and the fidelity texts |
| `scripts/check-embedding-fidelity.sh` (new) | embeds the golden texts through the shipped runtime (`Operon knowledge embed-golden`, a hidden subcommand reading the fixture) and fails below 0.995 |
| `CLAUDE.md` | the module map gains `src/knowledge.rs` and `src/knowledge/embed.rs` |
| `README.md` | the knowledge base, the model download, cmake for building, the agent command, what is sent to which CLI, Antigravity keeping judge runs — in all three languages |

## Order of work

The tree compiles after every step.

1. **Dependency.** Add the git dependency; one `cargo build` adds its packages
   to `Cargo.lock` without moving any locked version (checked with `git diff
   Cargo.lock`: only additions), then `cargo build --locked` passes. Record the
   person's `dependencies` approval in `approvals.log` against the new
   `Cargo.toml`. Confirm `cargo clippy --locked -- -D warnings` stays clean with
   the C++ build in the graph.
2. **Contracts first.** Golden fixture generated from the eg2 server
   (`.tmp/` script, not committed) and `embedding_golden_fixture_is_well_formed`;
   database round-trip and newer-schema refusal (Req 18), privacy (Req 16); judge
   answer parser fixtures (Req 7). Watch each fail, then write `src/knowledge.rs`'s
   database and parser.
3. **`run_command_cancellable`** (Req 19): both tests, then the refactor of
   `run_command_with_output_limit`.
4. **Embedder seam.** `Embedder`, prompts (Req 3), `FixtureEmbedder`;
   `LlamaEmbedder` behind the `Mutex`, ≤ 8 texts and ≤ 2 048 tokens per lock, logs
   silenced, context 2 048, mean pooling, token-boundary truncation; device from
   the backend (`GPU (Metal)` only when the Metal backend is the one loaded).
5. **Model file** (Req 2): readiness, the fixed resumable curl command, verify,
   rename, sidecar, re-hash-once for a final file without sidecar, full-size
   `.part` skipping curl; change 142's directory removed when it holds only
   `model_ready.json`.
6. **Replace change 142** (Req 1), in one step so the tree keeps compiling:
   `ModelState` in `src/models.rs`, `model_state` on `OperonApp` (initialised
   from readiness), the `ModelDownload` / `ModelReadiness` background jobs, and
   the Settings › データ block redrawn from `model_state` — then change 142's
   functions, constants, `EmbeddedModelDownloadState`, the `mode` branch in
   `search_local_transcripts_in_with_mode`, and its four tests are removed.
   `semantic_search_never_scores_by_word_overlap` is written here against
   `FixtureEmbedder` and stays red until step 8 supplies the ranking it calls.
7. **Pipeline** in requirement order: opt-in (4), exchanges and cursor (5),
   ranking with exemplars and structure (6), self-feed (8), judge command and
   prompt ceiling and daily cap and pause (7), dedupe (9), freshness and removal
   (10), unfinishable conversations (17), another model's vectors (21). Then
   **When it runs** from the spec's Behaviour: a conversation whose transcript
   changed in the last `KNOWLEDGE_IDLE_SECONDS` is left for later
   (`a_conversation_still_being_written_is_left_for_later`); a run stops at 200
   listed, 20 processed, 1 500 embeddings, or 180 s, with the clock and limits
   passed in so the test does not sleep (`a_run_stops_at_each_of_its_budgets`);
   enabled projects run one after another inside one job
   (`enabled_projects_are_processed_one_after_another`). One failing test each,
   then code.
8. **Search** (12), references (11), session index search (13).
9. **CLI** (14, 20): `main.rs` branch, project resolution from a worktree,
   exit codes, JSON-only stdout.
10. **App wiring**: background keys and results, the schedule (30 s after
    launch, every 10 minutes, on enable — `the_knowledge_run_is_scheduled_after_launch_and_every_ten_minutes`),
    one `BackgroundKey::KnowledgeRun` for every project so a second run cannot
    start while one is going (`starting_a_knowledge_run_while_one_is_running_starts_nothing`),
    `KnowledgeView`, `on_exit` cancel. Then the ナレッジ tab and the ローカル履歴
    group per `screen.md`, glyph, strings.
11. **Fidelity script** and the hidden `embed-golden` subcommand; run it.
12. **Docs**: `README.md` (three languages), `CLAUDE.md` module map.
13. **Verify**: `make q.fast`; `cargo test --locked knowledge`, `embedding`,
    `judge`, `cancel`, `semantic`, `model_`, `local_history`; the six ignored
    tests are not affected (restore and the shared archive are read, not
    changed) — run `cargo test --locked -- --ignored` anyway because `src/cli.rs`
    changes, or state that it could not run.
14. **Mutations**: for each guard, break the code it guards (drop the opt-in
    check, the cursor advance, the 120-char floor, the exemplar subtraction, the
    knowledge-command filter, the 0.98 copy check, the 0.85 dedupe, the τ
    reference term, the pinned exemption, the daily cap, the pause-not-attempt
    rule, the sidecar mtime check, the 0700/0600 modes, the schema refusal, the
    cancel kill, the model-mismatch rebuild and its search/dedupe gate, the
    JSON-only stdout, the idle rule, each run budget, the single run key) and
    watch the named test turn red.
15. **Acceptance** on this Mac per spec, then package.

### What the evaluator asked the plan to carry (spec approval, 2026-10-16)

- **A conversation that pauses every run.** A pause leaves cursors as they
  were, so a transcript whose prompt makes its CLI exit non-zero each time would
  stop every run at the same place. The paused conversation's `sessions` row
  records `paused_at` (a column the spec's schema did not list); each run
  orders conversations with no `paused_at` first, newest first, then paused
  ones by oldest `paused_at`, so every other conversation is judged before it
  and a poisoned one only ever blocks itself. A judged answer clears
  `paused_at`. A paused call spends nothing — not an attempt, not one of the
  30 daily calls. Test: `a_conversation_that_paused_is_tried_after_the_others`.
- **0.70 on session titles.** The 0.70 search floor was measured query →
  entry. Session index search (Req 13) embeds `title: {session title} | text:
  {first exchange}`; step 8 measures query → session on twenty real local
  sessions against the eg2 server before relying on it. A different floor goes
  into `KNOWLEDGE_SESSION_MIN_COSINE` and under Departures, with the numbers.
- **Re-embedding counts against the budget.** Req 21's rebuild re-embeds
  `entries.vector`, `sources.vector`, and `sessions.vector` and spends from the
  same 1 500 embeddings per run, so it can take several runs. Until it
  finishes, the database holds vectors of both models, so while `meta.model`
  differs from the pinned model nothing compares against it: search (12, 13)
  returns nothing for that project and the tab's status line says
  「ナレッジを新しいモデルで作り直しています… ({done}/{total})」 (a new string in an
  existing state slot), dedupe (9) and the copy check (8) are not reached
  because the run spends its whole budget on the rebuild and judges nothing.
  Progress is `meta.rebuild_table` and `meta.rebuild_after_rowid`; the last row
  sets `meta.model`. Test: `vectors_from_another_model_are_rebuilt_not_mixed`
  searches and runs the pipeline against a half-rebuilt database and asserts no
  result and no judge call, then finishes the rebuild and asserts both.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Git dependency fails to fetch on a contributor's machine or CI (submodule) | `cargo build` errors before compiling | CI `package` and `test` jobs on the branch push |
| C++ build slows CI past its timeout | red CI on time | first branch push; `Swatinem/rust-cache` caches the build |
| Embedding on the GUI thread | stutter on search | the `Mutex` is only taken in background jobs; `rust-reviewer` reads every `embed` caller |
| A query waits behind a long run | search over a second | ≤ 8 texts / 2 048 tokens per lock (0.13 s measured) |
| Two processes (GUI and CLI) write one database | `SQLITE_BUSY` | WAL, 2 s busy timeout, CLI writes only reference counts |
| Judge run lands in the project's session list | the pipeline judges its own run | cwd outside the project; the acceptance check; `judge_command_is_read_only_ephemeral_and_runs_outside_the_project` |
| `agy --print` asks to trust the judge directory | every Antigravity judge times out (a pause) | measured in step 7 before relying on it; if it prompts, the directory is trusted by the same mechanism Operon already uses, or Antigravity judging is reported as unsupported in the tab |
| Model file half-written or tampered | wrong vectors | hash before rename; readiness bound to size and mtime |
| Download outlives the app | orphan curl | `on_exit` sets the cancel flag; curl's own `--max-time` |
| Fixture vectors drift from the runtime | ranking tests pass on stale vectors | `embedding_golden_fixture_is_well_formed` and the fidelity script |
| A secret in a conversation reaches another vendor | privacy | only the conversation's own CLI receives it; stated in the disabled tab and `README.md` |
| Antigravity transcript path not a file at scan time | session skipped silently | `full_history_restore_transcript` resolves it; an error is `unreadable` (Req 17), shown in the tab count |
| Draw path reads the database | stutter | the tab reads `KnowledgeView` only; filled by background results |
| A missing translation row | i18n table test fails | existing i18n drift tests |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked knowledge` and the other filters in step 13 — all pass.
- CI on the branch: `test result: ok. N passed; 0 failed; 6 ignored`.
- `bash scripts/check-bands.sh` — within bands.
- `bash scripts/check-embedding-fidelity.sh` — every cosine ≥ 0.995.
- The packaged app on this Mac: the spec's Acceptance list, each observed.

## Corrections to the spec's Design

- `BackgroundKey::KnowledgeRun(Uuid)` becomes `BackgroundKey::KnowledgeRun`:
  the Behaviour section runs every enabled project one after another in one job,
  and a per-project key would let two run at once.
- `sessions` gains `paused_at` (above).

## Departures from the plan

1. **`Embedder` has no `device()`.** The device is a property of the loaded
   runtime, not of each call; `detect_device()` reads it (or the backend's
   device list, without loading the model) for 「使用可能 (GPU (Metal) / 310MB)」.
2. **`sources` and `sessions` keep their text.** `sources.text` and
   `sources.recorded_at`, `sessions.text` and `sessions.vector`: a model change
   re-embeds from what was stored instead of re-reading transcripts that may be
   gone (the evaluator's "rebuild text source" minor).
3. **Two more background keys.** `KnowledgeLoad(Uuid)` and `KnowledgeEdit(Uuid)`
   read and write a project's database off the frame; the plan named only the
   run and the searches.
4. **`ModelState::Downloading` carries nothing.** Progress is an `AtomicU64`
   the download thread writes and the frame reads, so no message per chunk.
5. **Source dates are relative** (「3日前」) in the detail pane, as the rest of
   the app shows times.
6. **The CLI exits 3 mid-rebuild** with 「ナレッジを新しいモデルで作り直しています…」
   rather than answering from half-rebuilt vectors (the evaluator's minor).
7. **Every context reserves eight sequences.** llama.cpp 2a66703 aborts the
   process when `n_seq_max` is 3, 5, 6, or 7; found by the fidelity script's
   last chunk of six (lesson 074).
8. **The model is dropped before every exit.** `release_shared_embedder` runs
   in `on_exit`, after `run_native` returns, and before the CLI's
   `process::exit`, and refuses later loads: Metal's teardown asserts while a
   model is loaded (lesson 074).
9. **Idle unload moved to the frame.** The plan released an idle model at the
   end of a run, where it had just been used and never qualified. The frame
   now tries once a minute (`EMBEDDING_IDLE_CHECK_SECONDS`) with `try_lock`, so
   an embedding in progress is never waited on.
10. **llama.cpp and ggml logs are switched off before the first init.**
    `void_logs` came too late for ggml's Metal device lines, which reached the
    knowledge command's stderr.
11. **`project_for_directory` prefers the nearest registered directory**, and
    reads a worktree list only for a project that is its repository's top
    level: a plain folder nested in another checkout claimed every worktree of
    that checkout (`knowledge_cli_refuses_an_unknown_or_disabled_project`
    caught it).
12. **`CliJudge::command`** builds the child separately, so the test reads its
    program, arguments, and directory without running an agent CLI.
13. **The fidelity entry point** is `Operon knowledge embed-golden <fixture>`
    (`KNOWLEDGE_EMBED_GOLDEN_SUBCOMMAND`), held to
    `EMBEDDING_FIDELITY_MIN_COSINE` = 0.995, with
    `embed_golden_fails_when_the_runtime_drifts_from_the_reference`.
14. **Mutations (step 14), each watched turning its test red:** opt-in check;
    cursor skip; 120-char floor; drop-exemplar subtraction (survived the
    median-only assertion — the test now also holds keep above and drop below
    the −0.02 candidate floor); knowledge-command filter; 0.98 copy check; 0.85
    dedupe; τ reference term; pinned capacity; daily cap; pause spending an
    attempt; sidecar mtime; 0700 and 0600 modes; schema refusal; cancel kill;
    rebuild skipped; search during a rebuild (survived while the stale vectors
    were zeros — they now sit exactly on the query); JSON-only stdout; idle
    rule; processed budget; embedding and wall budgets (each guarded twice, in
    the loop and in `Budget::spend`, so mutated in both); single run key;
    search and session thresholds; cited references; the n_seq_max and exit
    release (lesson 074).
15. **Fidelity, measured 2026-10-17:** 46 golden texts through the shipped
    runtime on Apple M4 Max (Metal), lowest cosine 0.99987.
16. **Review round 1 (do-not-approve from all three) moved the code:**
    `delete_model` unloads without ending the process's use of the model;
    exit waits for a cancelled download's `curl` to be reaped; every run write
    goes through `write_if_enabled` and 「ナレッジを削除…」 is one
    `clear_and_disable` transaction, so a run in flight cannot write a cleared
    or switched-off project back; a conversation past 400 exchanges keeps its
    file stamp unrecorded until read to the end; a failed tab load offers
    「再試行」; a download that does not finish drops the enable it was started
    for; `prune` cannot wrap its count. Each has a test watched failing with
    the fix reverted.
17. **The six live-CLI tests:** two pass; four fail on the step that checks a
    resumed conversation renders in a native terminal. Two of those four were
    run on a clean `main` checkout on 2026-10-17 and fail the same way, so the
    failure is this machine's CLIs, not this change.
