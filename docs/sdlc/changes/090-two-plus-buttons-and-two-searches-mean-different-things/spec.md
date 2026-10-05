# Spec: one "+", one search, four status words

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. A keymap action `session.new`, label 「新しいセッション」, default `Mod+N`,
   is added to `KEYMAP_ACTIONS`. While the palette is closed, its chord calls
   `open_new_session` (today's `open_new_session_from_sidebar`, renamed; same
   project choice and same no-project notice).
2. The toolbar's right side, right to left: settings icon (unchanged); a
   primary button `ICON_ADD` reading `tf!("新しいセッション  {chord}")` with the
   chord of `session.new` (just 「新しいセッション」 when unbound), calling
   `open_new_session`; a framed secondary button `ICON_SEARCH` reading
   `tf!("検索・操作  {chord}")` with the chord of `palette.open` (just 「検索・操作」
   when unbound),
   opening the palette exactly as the ⌘K chord does (focus requested, selection
   reset); then the rate limits as today. Chord text comes from the live keymap;
   an unbound action shows its label alone. The toolbar project-add icon is
   removed. Every drawn control is registered as a toolbar control so a click
   on it never starts a window drag.
3. Adding a project stays reachable by `project.add` (⌘O), drag-and-drop, the
   Projects screen buttons, and its ⌘K row — all unchanged.
4. The palette's actions gain two rows at the top, in this order:
   「新しいセッション」 (`session.new`, drawing its chord) and 「履歴を開く」
   (no chord), which selects the Sessions page and opens the session library.
   The existing rows keep their order after them. `activate_palette_entry`
   gains an arm for each; neither may fall through to the no-op arm.
   The palette window title and the `palette.open` label become 「検索・操作」 and
   「検索・操作を開く」, so the control has one name.
5. The sidebar header keeps 「プロジェクト別セッション」 and the count; its "+" and
   magnifier are removed; a small text button 「履歴」 at its right opens the
   session library. The per-project "+" on project rows is unchanged.
6. The session library's heading 「セッションツール」 becomes 「履歴」. Its content
   and 「ターミナルに戻る」 are unchanged.
7. The group `SessionStatusGroup::Free` is named 「待機中」 (message id), not 「空き」.
   Its hint is unchanged. `SessionStatusGroup::GROUPS` and `index()` become
   要対応, 実行中, 待機中, 終了 (NeedsYou, Busy, Free, Finished), the mock's order,
   so the chips and the Home tiles read the same. The filter is in-memory
   only; nothing persisted depends on the order.
8. Every per-session status label a person reads is the group word of its
   state: `SessionStatusKind::label` returns `self.group().message_id()`, and
   `SessionStatusView.label` stores it already translated
   (`tr(kind.label()).to_owned()`), so every drawing site that reads the view is
   translated without change. The per-state hint and the per-state tone are
   unchanged, so a failed session is still drawn in the failure colour. The
   sidebar row's status word gains `on_hover_text(status.hint)`, as the chip
   already has, so the per-state sentence is on hover everywhere the word is.
9. Home's four tiles become four group tiles, one per group in
   `SessionStatusGroup::GROUPS` order (the プロジェクト tile goes, as in the mock;
   the Projects page is one tab away), labelled with the group word, counted by
   `status_group_counts` (the sidebar's count, unread included), toned
   `palette.status(group.tone())` as the chips are, with icons 要対応
   `ICON_ATTENTION`, 待機中 `ICON_STATUS_IDLE`, 実行中 `ICON_STATUS_RUNNING`, 終了
   `ICON_STATUS_DONE`. A group tile calls `show_status_group(group)`, which is
   today's `show_sessions_needing_you` generalised (that one becomes a call to
   it): Sessions page, exactly that group's filter on.
9a. Home's 「AI セッションを始める」 button is removed: the toolbar's
   「新しいセッション」 is the one primary action (DESIGN.md: never a second one),
   as in the mock's Home. The Home redesign itself stays change 3.
9b. The Projects list page's 「フォルダを選択…」 becomes a secondary button, so no
   page draws two primary buttons once the toolbar has one.
10. The launch form's dependency list shows `title (group word)` from the same
    status view as the sidebar, not `SessionStatus::label`'s English token.
10a. `SessionStatus::label` and Home's `active_count`, `queued_count`,
    `completed_count` lose their callers and are deleted.
11. No English status token (`IDLE`, `RUNNING`, `QUEUED`, `DONE`, …) is drawn
    anywhere a person reads a status.

## Behaviour

See `./screen.md`. New message ids: 「新しいセッション」 (already used, gains
rows), 「新しいセッション  {chord}」, 「検索・操作」, 「検索・操作  {chord}」,
「検索・操作を開く」, 「履歴」, 「履歴を開く」, 「待機中」; each through `tr`/`tf!`.
Ids left unused are removed from every table: 「クイック操作  ⌘K」,
「プロジェクトを追加  ⌘O」, 「クイック操作を開く」, 「空き」, 「セッションツール」, and
「検索」 if no other caller remains.

## Design

- `src/app/keymap.rs`: one `KeymapAction` row.
- `src/app.rs`: rename to `open_new_session`; `session.new` chord beside the
  page chords; a helper `open_command_palette(&mut self)` used by the chord and
  the toolbar so the two cannot drift; a helper `show_status_group(&mut self,
  group)` used by the Home tiles (filter set to exactly that group, page
  Sessions).
- `src/app/screens.rs`: toolbar, sidebar header, library heading, palette
  actions (`[_; 8]`), Home tiles, launch-form dependency label.
- `src/ui/widgets.rs`: `SessionStatusKind::label` delegates to the group;
  `SessionStatusGroup::message_id` for `Free` becomes 「待機中」; the view's label
  is stored translated.
- `src/i18n_tables.rs`: rows in every table for each new id; 「空き」 and
  「セッションツール」 rows removed when unused.
- `README.md`, `README.ja.md`, `README.ko.md`: the keyboard paragraph names
  "new session" among the actions and ⌘N; the status chip wording (空き / 유휴 /
  Free → 待機中 and its translations) at README.md:72, README.ja.md:67,
  README.ko.md:66.
- `src/app/keymap.rs` module doc: the action count.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | yes | No new role; tiles use `warning`, `success`, `calm`, `text_faint`, which exist. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | yes | Existing constants only (`ICON_ADD`, `ICON_SEARCH`, `ICON_ATTENTION`, `ICON_STATUS_*`). |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | The action id `session.new` is written once in `KEYMAP_ACTIONS` and read by the chord, toolbar, and palette; a test asserts the palette row draws the chord the handler listens for (existing round-trip test extended). |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | No persisted field changes. `keybindings.json` gains an optional id; a file without it gets the default. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | No new child or launch path; the button calls the existing launch-form entry. |
| Documentation — user-facing docs change in all three languages together | yes | The README keyboard paragraph and the status chip words change in all three. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | no | UI only. |
| Budgets — any new scan or output path states its byte and item ceiling | no | No new scan or output. |

## Flagged concerns

- **A person's keymap already binds ⌘N** — a conflict with the new default
  refuses the whole file and names both actions, which is the keymap's existing
  rule; the fix is one line in their file. Resolved: accepted, stated here.
- **Fourteen states read as four words** — detail moves to the hover sentence
  and the colour; a failed session is still red. Resolved: the owner approved
  the four words in the checklist.
- **Toolbar tests pin coordinates** — `test_titlebar_adversarial_all_controls_immunity`
  and the rate-limit tests around src/tests.rs:34900, 35284, 35605 click fixed
  x positions. They are rewritten to find each control's rect from the drawn
  frame (or recomputed for the new widths) so each still hits the control it
  names; a point that lands on a different control is a failure, not a pass.
- **Keymap exception to "keybindings.json keeps working"** — a file that already
  binds ⌘N is refused whole, as above; that is the one exception to the
  intent's constraint.
- **Tests that name English tokens** — `every_session_status_kind_belongs_to_exactly_one_group`
  keys uniqueness and reachability on `label()`; `reports_what_a_session_is_actually_doing`
  (src/tests.rs:10844) and the test at src/tests.rs:11417 assert WORKING,
  WAITING, STOPPING, BLOCKED, UNKNOWN. All move to asserting `.kind` (reachability
  keyed on `format!("{kind:?}")`), so the per-state distinctions they guard stay
  guarded.

## Acceptance

- `cargo test --locked` passes, including new tests
  `the_new_session_chord_is_bound_and_listed_first_in_the_palette` (rows 0 and
  1 are `session.new` and 履歴を開く by id, and activating each changes state:
  the launch form's project is selected / the library is open),
  `every_status_label_is_one_of_the_four_group_words` (every kind's label is its
  group's message id, no label is ASCII, and every group word and new id has a
  row in `translation_for(Language::En, _)` and `(Language::Ko, _)`),
  `a_group_tile_filters_the_session_list_to_that_group` (Home drawn headless,
  the 実行中 tile clicked, only that filter on, and the tile count equals
  `status_group_counts`), the rewritten toolbar tests, and the i18n tests.
  The sidebar row's hover sentence (req 8) has no test; it is checked on screen.
- `cargo clippy --locked -- -D warnings` and `cargo fmt --check` are clean.
- In the installed app, the toolbar and sidebar match `./screen.md`; ⌘N opens
  the launch form; 「履歴」 opens the library.

## Rejected alternatives

- Keeping English status tokens beside the Japanese group word — two
  vocabularies again, which is the problem.
- A Japanese word per state (fourteen words) — the owner approved four.
- A second keymap action for 「履歴」 — no one asked for a key; the palette row
  and the button reach it.
