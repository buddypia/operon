# Plan: waiting sessions first, one key to the next, three columns

- **Spec**: `./spec.md`
- **Approved**: 2026-10-09
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/app.rs` | `attention_since`, `attention_order`, `terminal_focus_request` fields and defaults; `track_attention` called from `update` before the panels; `open_next_waiting_session`; `session_last_words`; two chords in `handle_app_shortcuts`; `session_inspector_side` default `Right` |
| `src/app/screens.rs` | title-bar 要対応 button and its `control_rects` entry; pinned 要対応 section in `ui_terminal_session_tabs`, and its members skipped in the project groups; one hairline in place of spacer/rule/spacer in `ui_terminal_panel`; the terminal focus request honoured there; palette haystack and tie-break |
| `src/app/keymap.rs` | `attention.next` (`Mod+J`) and `panel.toggle` (`Mod+Alt+B`) |
| `src/i18n_tables.rs` | rows for every new message id in every table |
| `src/tests.rs` | the tests the spec names; `test_sidebar_defaults_left_files_and_right_conversation` renamed and inverted; tests that assumed the left dock set it explicitly |
| `README.md`, `README.ja.md`, `README.ko.md` | the 要対応 section, ⌘J and ⌘⌥B, and the keymap sentence's list of actions |

## Order of work

1. Keymap rows and i18n rows; the build compiles.
2. Requirement 1: write the two queue tests, watch them fail, then the fields and
   `track_attention`.
3. Requirement 3: the four ⌘J tests failing (including the not-running case),
   then `open_next_waiting_session`, the chord, the focus request in
   `ui_terminal_panel`, and the corrected comment above `handle_app_shortcuts`.
   If the focused pane swallows ⌘J, switch to `chord_consumed`.
4. Requirement 2: the five pinned-section tests failing (pinned above projects,
   not drawn twice, hidden by chips, unread returns to its project, last words),
   then the section and `session_last_words`.
5. Requirement 4: title-bar button test failing, then the button; extend the
   title-bar adversarial test to it.
6. Requirement 5: palette tests failing, then haystack and tie-break.
7. Requirement 6: default side and `panel.toggle`; fix the tests that assumed
   the left dock by setting it explicitly where their subject is the left dock.
8. Requirement 7: the gap test failing, then one hairline.
9. README in three languages.
10. `make q.fast`; the named tests; mutation of each new guard; screenshot of
    the running app against `screen.md`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| `track_attention` allocates every frame | stutter with many sessions | the pass mirrors `status_group_counts`; `attention_order` rebuilt only when membership changes; `ranking_the_palette_over_a_full_store_stays_inside_a_frame`-style timing not needed beyond review |
| Changing the default dock side breaks tests that silently relied on Left | red tests in the sidebar group | the suite; each fixed by stating the side the test is about |
| The title-bar button lands in the drag region | clicking it drags the window | `test_titlebar_adversarial_all_controls_immunity` extended |
| ⌘J swallowed by a focused terminal | chord does nothing while typing in a pane | `opening_the_next_waiting_session_focuses_its_terminal` drives it with focus in the pane |
| Showing a stale agent message as the question | wrong text under a waiting row | message shown only for fresh `Idle` hook status; tested in `a_pinned_session_shows_what_its_agent_last_said` |
| A text size or button height off the documented scale | design tests fail | `every_text_size_is_a_level_on_the_type_scale`, `every_button_keeps_a_documented_height` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked <name>` for each test the spec names — `ok`; the full
  suite runs in CI on the pushed branch: `test result: ok. N passed; 0 failed; 6 ignored`.
- Each new test watched failing before its code, and mutated after.
- `bash scripts/check-bands.sh` — no new breach against `main`.
- The packaged app shows the After frame of `screen.md` with waiting sessions.

## Departures from the plan

- **The title-bar test is its own test.** Requirement 4 was to extend the
  adversarial title-bar matrix; the matrix iterates fixed controls with no
  waiting session, so `the_title_bar_offers_the_next_waiting_session_while_one_waits`
  carries the click, double-click, and drag checks for the new button instead.
- **Some tests were written after their code.** The ⌘J handler, the focus
  request, and the ⌘⌥B toggle landed before their tests were watched failing.
  Each was then checked by mutation (handler removed, focus forced, toggle
  removed, tie-break removed, button forced on): every mutant failed its
  named test. The ⌘J mutant first survived, because the test called the method
  rather than pressing the key; the test now presses the chord through
  `handle_app_shortcuts`.
- **The old sidebar-default test was renamed, not replaced.** Requirement 6
  named `the_side_panel_opens_on_the_right_by_default` as the rename of
  `test_sidebar_defaults_left_files_and_right_conversation`; the new name was
  taken by a narrower test, so the old one became
  `test_sidebar_defaults_right_files_from_either_side` and asserts the right
  default. The two stacked-sidebar tests now set the left dock themselves.
- **Pinned rows have no rename or menu.** They are a way to the session, not a
  second copy of its row; renaming stays in the project group after it is
  answered.
- **`zero_match_transcript_search_obeys_shared_byte_and_line_budgets` fails
  locally on `main` as well** (an export of `main` fails at the same line), so
  it is outside this change; CI decides whether it is local to this machine.
- **Review round 1 sent it back once.** A pinned row opened only from its
  padding (its labels were selectable and took the click), now covered by
  `clicking_a_pinned_rows_title_opens_the_session`; a project whose sessions were
  all pinned said 「セッションはまだありません」 and 「0 件」, now covered by
  `a_project_emptied_by_pinning_does_not_claim_it_has_no_sessions`. Both tests
  were watched failing against the old code. The palette now matches only a
  fresh hook status, as the row shows it; a stale focus request is dropped
  when another session is selected; the 要対応 button gained its hover text;
  `docs/MANUAL.md` documents ⌘J, ⌥⌘B and the pinned section.
- **A keymap file that already gives ⌘J or ⌥⌘B to another action is now
  refused whole**, as change 090 recorded for ⌘N: that is the existing
  conflict rule, and the refusal names both actions, so the person can move one.
- **Review round 3 exists only because `main` moved.** Change 138 landed while
  139 was in review. Merging `main` into the branch kept 139's code as it was,
  but shifted its hunks, so the diff `main` would take had a new digest
  (5a585926538e8328) and the landing merge was refused. The refused merge was
  aborted on `main` with nothing committed, and the three reviewers confirmed
  the landing diff. The first time this has cost a round; if it happens again
  it belongs in `docs/sdlc/lessons.md` with a guard, for example review against
  `main` at the moment of landing.
