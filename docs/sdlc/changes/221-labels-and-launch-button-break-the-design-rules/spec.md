# Spec: the sub-tabs, the idle group and the launch button say what the product's documents promise

- **Intent**: `./intent.md`
- **Screen**: `./screen.md`
- **Status**: approved

## Requirements

1. The three sub-views under a project's changes tab draw labels that come from
   the translation tables: 「変更ファイル」, 「差分」, 「コミット履歴」. Pinned by
   `the_git_sub_tabs_are_translated_message_ids` (fails today: the labels are
   the ASCII words `Changes`, `Diff`, `History`).
2. The group of idle sessions is worded 「指示待ち」 on its chip, on every row in
   it, and in the palette, because every one of them reads
   `SessionStatusGroup::message_id`. The word 「待機中」 is no longer a group
   word, so it no longer names two states. Pinned by
   `the_idle_group_word_is_not_the_queued_session_word`.
3. The launch sheet's start button is filled with the `accent` role and inked
   `on_accent`, whichever agent is selected. Pinned by
   `the_launch_button_is_the_accent_whichever_agent_is_selected`, which draws
   the sheet once per agent and reads the fill behind the button's text.
4. The documents that name the idle *filter* use the window's word, on exactly
   these lines: `README.md:108`, `README.ja.md:22` and `:99`, `README.ko.md:22`
   and `:97`, `docs/MANUAL.ja.md:202`, `:205` and `:435`, `docs/MANUAL.ko.md:201`,
   `:204` and `:436`. `docs/MANUAL.ja.md:152-153` and `docs/MANUAL.ko.md:150-151`
   speak of queued sessions and keep their word, so the test reads those named
   lines (by the phrase around the word) and never greps the bare word. Pinned by
   `the_documents_name_the_idle_filter_with_the_windows_word`.

## Behaviour

- Japanese: sub-tabs 「変更ファイル」 · 「差分」 · 「コミット履歴」. The filter
  chip and the status word on a row read 「指示待ち」. English: "Changed
  files" · "Diff" · "Commit history", and "Idle" for the group, as before.
  Korean: 「변경된 파일」 · 「차이점」 · 「커밋 기록」, and 「지시 대기」 for the group.
- Sentences about queued sessions keep 「待機中」 (a queued session's own status
  word is not it; the word is how the notices and hints speak of one) (「キャンセルできるのは待機中の
  セッションだけです。」, 「待機中のセッションを今開始します」); their rows are
  untouched.
- The launch button text is unchanged apart from a leading play mark, which is
  how `primary_button` draws every filled button: `▶  Codex CLI を起動  ⌘↩`.
  When the button cannot be pressed (`launch_ready` is false) it is drawn
  disabled, as it already was.
- No empty, loading, error or too-large state changes: nothing is added to any
  of them.

## Design

Placement: `EXTEND` — three existing symbols grow a different value.

- `src/app.rs`: `GitView::label` returns `tr("変更ファイル")`, `tr("差分")`,
  `tr("コミット履歴")`.
- `src/ui/widgets.rs`: `SessionStatusGroup::message_id` returns 「指示待ち」 for
  `Free`.
- `src/app/screens.rs`: in the launch sheet footer, the hand-built filled button
  (`agent_accent`, `readable_text_on`) becomes `primary_button(ui, palette,
  ICON_START, label)` inside `add_enabled_ui(launch_ready, …)`. `agent_accent`
  stays where it paints a vendor mark.
- `src/i18n_tables.rs`: `EN_TABLE` and `KO_TABLE` gain 「変更ファイル」,
  「コミット履歴」, 「指示待ち」 and lose 「待機中」, each in its sorted position;
  no row for a string nothing asks for (a test refuses it).
- Documents: `README.md`, `README.ja.md`, `README.ko.md`, `docs/MANUAL.ja.md`,
  `docs/MANUAL.ko.md`.
- No persisted shape, no external command, no background work; the draw path
  reads what it read before.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | Yes, uses an existing role | `primary_button` already draws `accent` filled with `on_accent` ink on the toolbar (`src/app/screens.rs:140`), so this pairing is on screen today and no new ink-on-surface pair is created; the contrast test near `src/tests.rs:7355` reads `readable_text_on(fill)` against the fill and the focus-ring test at `src/tests.rs:7532` already draws `primary_button`. No new role, no literal. DESIGN.md's rule "The button that starts a session is `accent` whichever CLI is selected" (Do's and Don'ts) is the one being met. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No new mark | `ICON_START` already exists and is in the vocabulary. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | The group word is read from `message_id` by the chip, the row and the palette; no second copy is written. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | Nothing persisted changes; the filter state is not stored (README: nothing is selected at start). |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | The button still sets the same `launch` flag; the launch path and its gates are not touched. |
| Documentation — user-facing docs change in all three languages together | Yes | `README.md`, `README.ja.md`, `README.ko.md` and the two manuals that name the word change in the same commit; `docs/MANUAL.md` says "Idle" already. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | No | Wording and a button fill. |
| Budgets — any new scan or output path states its byte and item ceiling | No | No scan or output path is added. |

## Flagged concerns

None. The only judgement is wording (「指示待ち」, 「変更ファイル」, 「コミット履歴」),
which the evaluator is asked to read against the intent.

## Acceptance

- `cargo test --locked the_git_sub_tabs_are_translated_message_ids the_idle_group_word_is_not_the_queued_session_word the_launch_button_is_the_accent_whichever_agent_is_selected the_documents_name_the_idle_filter_with_the_windows_word` passes, and each fails on `main`.
- `cargo test --locked every_translation every_status every_message_id every_source` passes: sorted tables, a row in each table for each id, no row nobody asks for.
- `make q.fast` passes.
- In the running app: a project's changes tab shows 「変更ファイル」 / 「差分」 /
  「コミット履歴」; the Sessions sidebar chip reads 「指示待ち」; picking each of
  the three agents in the launch sheet leaves the start button orange.

## Rejected alternatives

- Keep the English sub-tab words and call them product vocabulary: `worktree` and
  `PR` are kept for that reason, because every neighbouring sentence uses them;
  `Changes`, `Diff` and `History` have ordinary Japanese words and a sibling tab
  (「変更」) already uses one.
- Rename the queued-session word instead: it appears in four sentences in three
  languages; the idle word appears as one id.
- Call the idle group 「完了」: a session that finished its turn is not finished,
  and 「終了」 already means the terminal ended.
