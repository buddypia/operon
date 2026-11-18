# Spec: ナレッジ

## Behaviour
1. Knowledge is opt-in per project. On the project's 「ナレッジ」 tab a disabled project shows an explanation, 「このプロジェクトでナレッジを集める」, and when the model is absent 「モデルを取得して始める (310MB)」. No database is created and no conversation read until it is enabled. 「収集を止める (ナレッジは残す)」 disables collection and keeps entries; 「ナレッジを削除…」 asks 「削除する」 / 「キャンセル」 and clears and disables.
2. The embedding model is EmbeddingGemma 2 (`ggml-org/embeddinggemma-2-GGUF`, a pinned revision, Q8_0, 768 dimensions). It is downloaded once, checked against a pinned SHA-256 and byte size, and used on this Mac only. Settings → データと復旧 → 「意味検索（EmbeddingGemma 2）」 can fetch (「モデルを取得 (310MB)」), cancel (「中止」), retry (「再試行」), delete (「モデルを削除」), and choose 「ローカル履歴の検索」 between 「キーワードのみ」 and 「意味検索」.
3. A run starts 30 s after launch and every 600 s, or on 「今すぐ更新」. It lists at most 200 sessions per run, processes at most 20 conversations, embeds at most 1,500 texts, and stops after 180 s. A second run is not started while one is running.
4. A conversation is split into exchanges (a person turn with the agent's reply). Exchanges under 120 characters are dropped and long ones capped at 4,000; at most 400 per conversation. Exchanges are ranked locally against fixed keep/drop exemplars plus a structure bonus; at most 8 candidates per conversation go on.
5. An exchange that quotes the knowledge command, or whose vector is within cosine 0.98 of an exchange already recorded as a source, is not a candidate.
6. Candidates go to one judge call per conversation, run by the CLI that held the conversation (read-only, ephemeral) with a working directory outside the project. If that CLI is not installed or paused, the remaining CLIs are tried in order (Claude Code, Codex, Antigravity). A provider that paused stays skipped for the rest of the run. Timeout 120 s, prompt ≤ 64 KiB (person turn cut to 800 chars, agent turn to 1,600), answer ≤ 16 KiB. At most 30 judge calls per project per UTC day.
7. The judge answers 0 to 5 entries, each with a kind (「決定」「原因」「手順」「注意」「事実」), a title (≤ 40 chars), a body (≤ 400 chars), a boolean `verified`, and optionally a `replaces` naming an offered existing entry. A new entry within cosine 0.85 of an existing one reinforces it instead of adding; `verified` is never downgraded by reinforcement.
8. An entry the judge says is out of date replaces the related, unpinned entry it names, keeping its history; pinned entries are never replaced. Related entries are offered only while the run's embedding budget lasts.
9. Freshness is `exp(−days / τ)` with τ = 30 days × (1 + ln(1 + references + recurrences)). Pruning removes entries below 0.10. A pinned entry is not pruned. At most 2,000 entries per project; when pinned entries fill the limit, runs only reinforce.
10. A conversation that cannot be judged is a failed attempt only when the answer was unusable; a missing, signed-out, timed-out or non-zero CLI is a pause that spends nothing. After 3 failed attempts a conversation is retried after 7 days; 「再試行」 forces it.
11. The tab's status line is one of: 「会話を整理しています… ({done}/{total})」, 「モデルがないため停止中」, 「ナレッジを新しいモデルで作り直しています…」, 「{cli} で判定できませんでした。次回の更新で再試行します。」, 「本日の判定回数の上限に達しました。明日続きを処理します。」, 「固定したナレッジが上限 (2000 件) に達したため、新しいナレッジを追加していません。」, or 「{n} 件 ・ 最終更新 {updated} ・ EmbeddingGemma 2 ({device}) ・ 本日の判定 {used}/{cap} 回」. A count 「処理できなかった会話 {n} 件」 offers 「再試行」.
12. 「表示:」 switches 「一覧」 and 「マップ」. 「並び:」 offers 「鮮度」, 「参照回数」, 「新しい順」; 「種類:」 offers 「すべて」 and the five kinds. The search box 「ナレッジを検索」 ranks entries by cosine to the query and shows those at 0.70 or above; none gives 「一致するナレッジはありません。」.
13. Each entry shows its kind, title, 「検証済み」 or 「未確認」, a freshness bar and 「参照 {n}」. The open entry offers 「会話を開く」, 「固定」 / 「固定を外す」 and 「削除」. Opening an entry counts a reference.
14. The map places every entry as a point from its vector: neighbours by meaning (8 nearest, edges at cosine ≥ 0.72), groups by modularity (at least 3 members; named 16 chars from the most central title, rest 「その他」), layout by a bounded iteration, outlined by convex hulls. Shape encodes kind, radius references (3 to 9 px), opacity freshness (floor 0.25), a ring marks a pinned entry. Hover shows 「{kind} ・ 鮮度 {p}% ・ 参照 {n}」. Selecting a point or a list row opens the same entry. Search and kind filter dim non-matches. Drag pans, scroll or pinch zooms around the pointer, and group titles show at every zoom while entry titles show past zoom 1.8. The right panel lists 「グループ」 with 「グループ一覧」 to return.
15. Saved positions are keyed by a fingerprint of each vector. An entry that has not changed keeps its place; changed vectors lay out afresh.
16. At the start of a Claude Code or Codex session, a hook runs `Operon knowledge brief` and hands the CLI at most 10 titles (pinned, then verified, then freshest; ≤ 1,500 chars) with the search command. Antigravity gets nothing.
17. `Operon knowledge search "<query>" [--limit N] [--json]` (default limit 5) resolves the project from the working directory, including a worktree, and prints ranked hits. With `--json` stdout is only the JSON array, and rows carry `verified`.
18. Entries and conversation vectors live in `knowledge/<project id>.sqlite3` in Operon's data folder; the directory is mode 0700 and files 0600.

## States
- Model missing: collection is stopped, the tab says 「モデルがないため停止中」, and `Operon knowledge search` exits 3.
- Model download: progress, 「確認中…」, 「中止」, failure 「取得できませんでした: {reason}」 with 「再試行」; a download that exits the app is stopped on exit.
- Knowledge load fails: the tab offers 「再試行」. Map fails: 「マップを作れませんでした: {error}」 with 「再試行」; while computing: 「配置を計算しています…」.
- No entries: 「まだナレッジはありません。このプロジェクトで会話を続けると、重要なやり取りがここに追加されます。」.
- A map result computed for an older snapshot, or for fewer entries than now, is not drawn.
- The CLI refuses an unknown or disabled project, wrong arguments (exit 64 with usage), and the brief prints nothing for a project outside every registered folder, with no database, disabled, or empty.
- A database with a newer schema than the app knows is refused. An older one gains the `verified` column and the map table without a version bump.

## Invariants
- Nothing is read, judged or stored for a project that has not enabled knowledge.
- Conversation content is embedded locally and never sent to a model service; only ranked candidates go to the user's own agent CLI.
- The judge runs outside the project folder, so its runs do not appear among the project's conversations.
- The map layout, neighbour computation and brief never run in the draw path; the brief gives up after 3 s and prints nothing.
- Knowledge is never written into the repository.
- A pinned entry is never replaced, pruned, or counted as free space.
- A recorded `verified` is never lowered by reinforcement.
- The brief and the knowledge command need no model to answer the brief; search does.

## Code and tests
- Symbols:
  - `src/knowledge.rs::KnowledgeDatabase`: SQLite store (entries, sources, vectors, conversations, map positions).
  - `src/knowledge.rs::run_project`: one project's run; `process_conversation` handles one conversation.
  - `src/knowledge.rs::rank_candidates`, `exchange_score`: local ranking against exemplars.
  - `src/knowledge.rs::judge_prompt`, `parse_judge_answer`, `CliJudge`: the second step.
  - `src/knowledge.rs::freshness`: decay formula.
  - `src/knowledge.rs::search_entries`, `knowledge_cli`, `knowledge_brief`, `knowledge_brief_cli`: search and the agent-facing commands.
  - `src/knowledge/embed.rs::download_model`, `delete_model`, `SharedEmbedder`: model lifecycle and runtime.
  - `src/knowledge/map.rs::compute_map`, `map_groups`, `convex_hull`: map computation.
  - `src/app/knowledge.rs::ui_knowledge`, `knowledge_status_line`, `ui_model_block`: the tab, status line, settings block.
  - `src/ui/knowledge_map.rs::MapView`: map view and coordinates.
  - `src/config.rs::KNOWLEDGE_DAILY_JUDGE_CALLS`, `KNOWLEDGE_MAX_ENTRIES`, `KNOWLEDGE_BRIEF_ENTRIES`: the limits.
- Tests:
  - `knowledge_search_ranks_by_meaning_not_words`: search by meaning.
  - `judge_command_is_read_only_ephemeral_and_runs_outside_the_project`: judge isolation.
  - `judge_prompt_stays_under_its_ceiling`: prompt bound.
  - `judge_calls_stop_at_the_daily_cap`: 30 calls per day.
  - `exchanges_quoting_knowledge_results_or_copies_are_not_candidates`: no self-feeding.
  - `freshness_decays_slower_for_used_entries`: decay.
  - `knowledge_files_are_private`: modes 0700 and 0600.
  - `cli_judge_falls_back_to_an_available_cli_and_skips_failed_ones`: fallback.
  - `an_unavailable_judge_pauses_without_spending_attempts`: pause rule.
  - `knowledge_ui_shows_embedding_gemma_device_and_available_cli_summary`: status line and card.
  - `knowledge_cli_json_is_the_only_stdout`: JSON output contract.
  - `knowledge_cli_exits_3_without_the_model`: missing model.
  - `knowledge_edits_create_a_database_only_on_enable`: opt-in.
  - `a_knowledge_tab_that_failed_to_load_offers_a_retry`: load failure.
  - `knowledge_map_positions_go_with_their_entry`: stable positions.
  - `knowledge_map_lays_out_the_largest_base_in_time`: layout budget.
  - `knowledge_map_is_computed_off_the_frame`: off-frame computation.
  - `knowledge_map_and_list_share_the_open_entry`: shared selection.
  - `knowledge_256_the_tab_says_which_entries_were_verified`: verified mark.
  - `knowledge_257_a_run_replaces_what_the_judge_says_is_out_of_date`: replacement.
  - `knowledge_258_brief_lists_pinned_then_verified_then_freshest`: brief order.
  - `knowledge_258_a_slow_brief_gives_up_and_says_nothing`: brief timeout.

## Decisions inherited
- Change 157: knowledge is opt-in per project, uses a pinned local model, ranks locally first and lets the conversation's own CLI judge second, outside the project, capped at 30 calls per day.
- Change 171: the map is computed off the frame from vectors and positions are kept per fingerprint so entries do not jump between runs.
- Change 225: no single vendor CLI is required; the judge prefers the conversation's CLI and falls back to any installed one, with no settings dropdown. The golden exemplars are pinned and not edited.
- Change 256: each entry records whether it was verified, and reinforcing never lowers it.
- Change 257: an entry a later conversation shows wrong is replaced (keeping history), never a pinned one.
- Change 258: sessions receive titles and the search command through a hook, bounded by 10 entries and 3 s, and a missing or empty knowledge base says nothing.
