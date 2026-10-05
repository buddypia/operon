# Plan: one "+", one search, four status words

- **Spec**: `./spec.md`
- **Approved**: 2026-09-27
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/app/keymap.rs` | `session.new` row, label 「新しいセッション」, default `Mod+N`; `palette.open` label 「検索・操作を開く」; module doc's action count |
| `src/app.rs` | `open_new_session_from_sidebar` → `open_new_session`; `open_command_palette`; `show_sessions_needing_you` generalised to `show_status_group(group)`; `session.new` chord beside the page chords |
| `src/app/screens.rs` | toolbar buttons; palette actions `[_; 8]` with 新しいセッション and 履歴を開く first, and their two arms in `activate_palette_entry`; palette window title 「検索・操作」; sidebar header (「履歴」 only); sidebar row status word hover; library heading 「履歴」; Home tiles per group (no プロジェクト tile), `AI セッションを始める` removed, Projects list 「フォルダを選択…」 secondary, `active/queued/completed_count` deleted; dependency label from the status view |
| `src/ui/widgets.rs` | `SessionStatusKind::label` → group message id; `Free` → 「待機中」; `GROUPS`/`index()` in the mock's order; view label stored translated; doc comment on why |
| `src/models.rs` | `SessionStatus::label` deleted (no callers left) |
| `src/i18n_tables.rs` | rows for every new id in spec "Behaviour" in every table; the unused ids it names removed |
| `src/tests.rs` | `every_session_status_kind_belongs_to_exactly_one_group` keyed on the kind (uniqueness and reachability); `reports_what_a_session_is_actually_doing` and the test at ~11417 assert `.kind`; the three toolbar coordinate tests find each control's rect; three new tests in the spec |
| `README.md`, `README.ja.md`, `README.ko.md` | keyboard paragraph: new session and ⌘N; the chip word 空き / 유휴 / Free |

## Order of work

1. Keymap row, `open_new_session`, `open_command_palette`, chord handler;
   palette rows. Compiles.
2. Toolbar buttons; sidebar header; library heading.
3. Status words in `widgets.rs`; every drawing site translated; dependency
   label; Home tiles with `show_status_group`.
4. i18n rows; tests; README ×3.
5. Gates, mutation check of each new test, rust-reviewer, commit, package,
   install, look at the screen.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A new id lacks an EN or KO row | Japanese in the English UI | `every_status_label_is_one_of_the_four_group_words` asserts `translation_for(En/Ko, id)` for every group word and new id (tests run in Ja, so comparing to `tr` would prove nothing) |
| A palette row does nothing | a dead action | the new-session test activates both new rows and asserts the state change |
| A per-state test goes vacuous | WORKING vs WAITING no longer checked | those tests assert `.kind`, keyed on `format!("{kind:?}")` |
| A toolbar test clicks the wrong control | passes while testing nothing | each coordinate test finds its control's rect from the frame |
| ⌘N reaches the terminal as well as opening the form | a stray keystroke in the agent | same `chord_pressed` path as ⌘1–3, which is already live beside a focused terminal; checked on screen |
| Toolbar buttons become drag handles | a click moves the window | each rect registered; the toolbar hit-test asserts it |
| Palette chord drift | the row shows ⌘N while the handler listens elsewhere | the existing `the_palette_draws_the_chord_the_handler_listens_for` extended to `session.new` |
| Home counts differ from the sidebar | two numbers for one thing | both call `status_group_counts` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- The three new tests, each watched failing with its rule inverted.
- Installed app: toolbar and sidebar as in `./screen.md`; ⌘N opens the form.

## Departures from the plan

Carried from the round-3 evaluation, recorded here rather than moving the
approved digests:

- The unused ids removed are the six named in the spec's Behaviour section;
  the Design bullet naming two of them defers to that list.
- The README chip lists are reordered to 要対応 / 実行中 / 待機中 / 終了, not only
  re-worded.
- screen.md's status-word row names the set; its order is the chips' new order.

Found while building:

- The empty-Home sentence pointed at the project's 「概要」 tab to start a
  session; it now points at the toolbar's 「新しいセッション」 (new id, old row
  removed).
- The toolbar tests click where the frame drew each control
  (`drawn_text_rect`), not typed coordinates; the new-session click is proven
  by the no-project notice it raises.
- README.ko.md named the 要対応 chip 요대응 while the app says 대응 필요; the
  chip list now uses the app's words.
