# Spec: the app's wording is rewritten where it reads as machine-made

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. R1: the idle status group, and so every idle row, chip, tab tooltip and
   palette entry that reads `SessionStatusGroup::message_id`, is worded
   「アイドル」 in Japanese, "Idle" in English and 「유휴」 in Korean; the needs-you
   group reads 「확인 필요」 in Korean (Japanese 要対応 and English "Needs you"
   unchanged). The idle tab hint is 「ターンを終え、次の依頼を出せます」 /
   "Turn finished; ready for your next request" / 「턴을 마쳤습니다. 다음 요청을 보낼
   수 있습니다」, and the idle chip hint is 「ターンを終え、次の依頼を出せるセッション」 /
   "Sessions that finished their turn" / 「턴을 마치고 다음 요청을 보낼 수 있는 세션」.
   No Japanese group word contains 「待」, no Korean one 「대기」, and neither hint
   contains 「待」, 「대기」 or "waiting". Pinned by
   `the_idle_group_word_does_not_read_as_waiting_on_you` (fails on `main`: the
   word is 「手すき」).
2. R2: no English or Korean row carries Japanese script (kana, CJK ideographs,
   「」) and no row in any table puts a backslash on screen. Pinned by
   `no_translation_carries_japanese_script_or_a_stray_backslash` (fails on
   `main`: Korean 「병合합니다」 and 「停止」, and three rows whose `\\\"` draws
   backslashes before a quote).
3. R3: no row uses a term this change retires — ja 「手すき」 「手が空い」 「永続化」
   「上限付き」; en "Attempting to", "Land worktree", "landing", "Level with",
   "in two goes"; any language, the menu name "Discover local CLI sessions",
   which no menu carries (it is 「ローカル CLI セッションを検出」 / "Detect local CLI
   sessions" / 「로컬 CLI 세션 감지」); ko 「덩어리」 「차이점」 「신선도」 「대응 필요」 「키 할당」 「당신」
   「시간 창」 「리포지토리」. Pinned by `no_message_uses_a_retired_term` (fails on
   `main` for each list).
4. R4: the 187 messages listed in `wording.md` read as listed there — 228
   cells: 42 Japanese message ids re-keyed at every call site, 94 English and
   92 Korean translations — and every table stays complete, sorted, and keeps
   each message id's placeholder names. Pinned by
   `every_translation_table_is_sorted_by_message_id`,
   `every_message_id_has_a_row_in_every_table`,
   `every_translation_keeps_the_placeholder_names_of_its_message_id`,
   `every_message_the_code_shows_has_a_row_in_every_table` and
   `every_message_row_is_one_the_code_shows`, which fail when a re-keyed id
   is left behind at a call site or in a table.
5. R5: README.md, README.ja.md, README.ko.md, docs/MANUAL.md,
   docs/MANUAL.ja.md and docs/MANUAL.ko.md quote the new labels wherever they
   quoted a reworded one — the idle filter 「アイドル」, the Korean 「확인 필요」,
   the English "Merge worktree into main" — and describe the idle state as a
   finished turn: `| **アイドル** | ターンを終え、次の依頼を出せる状態 |` and
   `| **유휴** | 턴을 마치고 다음 요청을 보낼 수 있는 상태 |`. Pinned by
   `the_documents_name_the_idle_filter_with_the_windows_word`, which also refuses
   「手すき」, 「대응 필요」 and "Land worktree into main" anywhere in the six files.

## Behaviour

Status chips, before → after:

- ja: 要対応 · 実行中 · 手すき · 終了 → 要対応 · 実行中 · アイドル · 終了
- en: unchanged (Needs you · Running · Idle · Finished)
- ko: 대응 필요 · 실행 중 · 유휴 · 종료 → 확인 필요 · 실행 중 · 유휴 · 종료

Idle tab tooltip: 「アイドル / ターンを終え、次の依頼を出せます」 (before
「手すき / ターンを終えて手が空いています」).

Representative rows (all 187 are in `wording.md`):

| Where | Before | After |
|---|---|---|
| ja error | Codex の再生レコードのエンコード: {error} | Codex の再生レコードをエンコードできませんでした: {error} |
| ja notice | …ディスクへの永続化を確認できませんでした。 | …ディスクに保存されたことを確認できませんでした。 |
| en menu | Land worktree into main… | Merge worktree into main… |
| en notice | Attempting to run .operon/run.sh. | About to run .operon/run.sh. |
| en branch | Level with {p0}. | Up to date with {p0}. |
| en notice | {chord} is claimed by both \\\"{first}\\\" … (backslashes on screen) | {chord} is claimed by both "{first}" and "{second}". |
| ko label | 5시간 창 | 5시간 한도 |
| ko tab | 차이점 | 변경 내용 |
| ko notice | 현재 worktree의 변경 사항을 main 브랜치에 병合합니다. | …병합합니다. |
| ko notice | 계정 "{name}"을(를) 추가했습니다. | '{name}' 계정을 추가했습니다. |

No state, layout, order, colour, count or failure path changes; empty, error,
too-large and interrupted states show the same rows, reworded where listed.

## Design

Placement: EXTEND — the wording lives in `src/i18n_tables.rs`
(`EN_TABLE`, `KO_TABLE`) and in the Japanese message ids at their call sites.
`SessionStatusGroup::message_id` and `session_status_view` in
`src/ui/widgets.rs` return the new idle id and hints. The re-keyed ids are
replaced as exact string literals across `src/` — call sites in `src/app.rs`,
`src/app/screens.rs`, `src/app/knowledge.rs`, `src/history.rs`, `src/cli.rs`,
`src/files.rs`, `src/git.rs`, `src/agents/accounts.rs`,
`src/store.rs` and `src/ui/widgets.rs`, and the assertions in `src/tests.rs` —
and each table is re-sorted by message id. In `src/store.rs` (two
`CommittedButNotSynced` notices) and `src/git.rs` (one test-run notice) only
the literal text changes: no write order, persisted shape, cancellation path,
or command. `src/agents.rs` is left untouched, so its launch gate is not a
surface of this change. None of the ids is compared against persisted data, so no stored file
changes meaning. No external command, background work, or draw-path cost.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | No | No colour or role changes. |
| Icons | No | No glyph changes. |
| Identifier SSOT | Yes | Each message id still lives once in code and once per table, read through `tr`/`tf!`; ids are renamed, not duplicated. |
| Durability | Yes | `.claude/skills/durability-invariants/SKILL.md` read: `CommittedButNotSynced` means the data is on disk and only the directory sync is unconfirmed. The reworded notices keep that certainty (「ディスクに保存されたことを確認できませんでした」 / "could not confirm it reached the disk") and still say Operon retries; the `WriteOutcome` branches are untouched. |
| Subprocess safety | No | No child process. |
| Documentation — three languages together | Yes | The READMEs and manuals in ja/en/ko change in the same commit as the labels they quote. |
| Local-first | No | Wording only. |
| Budgets | No | No scan or output path. |

## Flagged concerns

None. Which rows change, and to what, is the wording judgement recorded in
`wording.md` and read against the intent.

## Acceptance

- `cargo test --locked the_idle_group_word_does_not_read_as_waiting_on_you no_translation_carries_japanese_script_or_a_stray_backslash no_message_uses_a_retired_term the_documents_name_the_idle_filter_with_the_windows_word` passes, and each fails on `main`.
- `cargo test --locked every_translation every_message` passes.
- `cargo test --locked sidebar_251_marks_and_accessible_filters workspace_session_tab_shows_its_status_glyph` passes (the longer idle word still fits).
- `make q.fast` passes.

## Rejected alternatives

- Idle word 「待機中」 / Korean 「대기 중」: names a queued session (change 221)
  and reads as waiting on the person (change 259).
- 「完了」: reads as the 終了 group.
- 「応答済み」: false for a session that has not been given a request yet.
- 「休止中」: reads as paused by someone.
- 「手すき」 kept: the person rejected it.
- Rewriting every row: churn without a reason costs the documents and the
  reader's memory of the labels; only rows that read wrong change.
- One change per language: the status word, its tests and the documents span
  all three, and a half-moved term is the inconsistency this change removes.
