# Spec: the idle group is worded 「手すき」

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. R1: the idle status group, and so every idle row, chip, tab tooltip and
   palette entry that reads `SessionStatusGroup::message_id`, is worded
   「手すき」 in Japanese, "Idle" in English and 「유휴」 in Korean, and no
   Japanese group word contains 「待」 nor any Korean one 「대기」. Pinned by
   `the_idle_group_word_does_not_read_as_waiting_on_you` (fails on `main`: the
   word is 「指示待ち」).
2. R2: both hover explanations of the idle state stop saying "waiting": the
   tab's hint for `SessionStatusKind::Idle` is 「ターンを終えて手が空いています」 /
   "Turn finished; free for a new request" / 「턴을 마치고 손이 비어 있습니다」, and the
   chip's `SessionStatusGroup::Free.hint_id()` is 「ターンを終えて手が空いているセッション」 /
   "Sessions that finished their turn and are free" / 「턴을 마치고 손이 비어 있는
   세션」. Pinned by `the_idle_group_word_does_not_read_as_waiting_on_you`, which
   also refuses 「待」, 「대기」 and "waiting" in either hint, and by
   `workspace_session_tab_shows_its_status_glyph` hovering a live idle tab.
3. R3: README.md, README.ja.md, README.ko.md, docs/MANUAL.ja.md and
   docs/MANUAL.ko.md name the idle filter 「手すき」 / 「유휴」 wherever they named
   「指示待ち」 / 「지시 대기」 — the hero captions, the filter lists, the manuals'
   state tables and their troubleshooting lines — and the state tables describe
   it as a finished turn rather than a wait:
   `| **手すき** | ターンを終えて手が空いている。次の依頼を出すかは自由 |` and
   `| **유휴** | 턴을 마치고 손이 비어 있는 상태. 다음 요청은 자유 |`. Pinned by
   `the_documents_name_the_idle_filter_with_the_windows_word`, which also
   refuses the old words anywhere in the five files.

## Behaviour

Before: chips 要対応 · 実行中 · 指示待ち · 終了; an idle tab's tooltip
「指示待ち / 次の依頼を待っています」.
After: chips 要対応 · 実行中 · 手すき · 終了; the tooltip
「手すき / ターンを終えて手が空いています」; hovering the 手すき chip reads
「ターンを終えて手が空いているセッション」 (before: 「起動していて、次の依頼を待っているセッション」). Korean chips 대응 필요 · 실행 중 · 유휴 · 종료.
English is unchanged except the hint. No new state, no new failure path; empty
lists, filters, and counts behave as before.

## Design

Placement: EXTEND. `SessionStatusGroup::message_id` in `src/ui/widgets.rs`
returns 「手すき」 for `Free` and `hint_id` the new chip hint;
`session_status_view` returns the new hint for `SessionStatusKind::Idle`.
`src/i18n_tables.rs` replaces the rows for 「指示待ち」, 「次の依頼を待っています」 and
「起動していて、次の依頼を待っているセッション」 in `EN_TABLE` and `KO_TABLE`, each at
its sorted position. Existing tests that name the old words move to the new ones.
No persisted shape, external command, background work, or draw-path cost.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | No | No colour or role changes. |
| Icons | No | No glyph changes. |
| Identifier SSOT | No | Message ids live in one place each, read through `tr`. |
| Durability | No | No store, cancellation, migration, or byte limit. |
| Subprocess safety | No | No child process. |
| Documentation — three languages together | Yes | README and manual in ja/ko and README.md change in the same commit. |
| Local-first | No | Wording only. |
| Budgets | No | No scan or output path. |

## Flagged concerns

None. The wording (「手すき」, 「유휴」) is the judgement, read against the intent.

## Acceptance

- `cargo test --locked the_idle_group_word_does_not_read_as_waiting_on_you the_documents_name_the_idle_filter_with_the_windows_word workspace_session_tab_shows_its_status_glyph` passes, and the first two fail on `main`.
- `cargo test --locked every_translation every_message_id every_status` passes.
- `make q.fast` passes.

## Rejected alternatives

- 「待機」/「待機中」: names a queued session (change 221).
- 「次の依頼待ち」: keeps 「待ち」, the reading this change removes.
- 「完了」: reads as the 終了 group.
- 「空き」: reads as an empty slot rather than a free agent.
- Merging 要対応 into the idle group: they differ in whether the session moves
  without the person, which is what the filter is for.
