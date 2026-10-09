# Spec: waiting sessions first, one key to the next, three columns

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

Each names the test in `src/tests.rs` that fails without it.

1. **A queue of the sessions that need a person, longest-waiting first.** The
   app keeps, outside the draw code, the moment each session entered 要対応
   (the `SessionStatusGroup::NeedsYou` membership `attention_count` already
   counts: waiting for input, blocked, or unread). A session that leaves 要対応
   leaves the queue; one that comes back is timed afresh. The order is oldest
   entry first, ties by newest launch.
   — `the_attention_queue_orders_waiting_sessions_by_how_long_they_have_waited`
   — `a_session_that_stops_needing_a_person_leaves_the_attention_queue`
2. **The session list pins the queue above the projects.** Above the project
   groups, a section headed 要対応 with its count lists the queued sessions that
   pass the status chips. Each row shows the title, the project and branch, how
   long it has waited, and — when known — the gist of what the agent said (see
   Behaviour). A session drawn there is not drawn again in its project group.
   With the status chips set so that 要対応 is excluded, there is no pinned
   section. Opening a session that was pinned only because it was unread clears
   the mark, and it leaves the section for its project group.
   — `the_session_list_pins_waiting_sessions_above_the_projects`
   — `a_pinned_session_is_not_drawn_again_in_its_project`
   — `chips_that_exclude_needs_you_hide_the_pinned_section`
   — `opening_an_unread_pinned_session_returns_it_to_its_project`
   — `a_pinned_session_shows_what_its_agent_last_said`
3. **⌘J opens the next waiting session and puts the keyboard in its terminal.**
   A new keymap action `attention.next`, default `Mod+J`, live on every page and
   while a terminal has focus, opens the first queued session after the selected
   one (wrapping), switches to the session page, and focuses its terminal when
   the session is running; a session that is not running is opened without a
   focus request. With nothing queued it says so briefly and changes nothing.
   — `the_next_waiting_chord_opens_the_next_session_in_the_queue`
   — `opening_the_next_waiting_session_focuses_its_terminal`
   — `opening_a_waiting_session_that_is_not_running_asks_for_no_focus`
   — `the_next_waiting_chord_with_an_empty_queue_changes_nothing`
4. **The title bar offers the same move.** While the queue is not empty the
   title bar shows a button 「要対応 N」 with the chord beside it; pressing it
   does what ⌘J does. It is a title-bar control: clicking it neither drags nor
   maximises the window.
   — `the_title_bar_offers_the_next_waiting_session_while_one_waits`
   — `test_titlebar_adversarial_all_controls_immunity` (extended to the new button)
5. **⌘K finds a session by what its agent last said, and ranks a waiting one
   first among equals.** The palette's session haystack gains the agent's last
   words; within one score, a session in 要対応 sorts before one that is not.
   — `the_palette_finds_a_session_by_what_its_agent_last_said`
   — `the_palette_ranks_a_waiting_session_above_an_equal_match`
6. **Three columns by default.** The file / conversation / changes panel opens
   docked on the right, so the session list has the left column to itself.
   Docking it left still stacks it under the list (change 127). A new keymap
   action `panel.toggle`, default `Mod+Alt+B`, folds and reopens the panel.
   — `test_sidebar_defaults_left_files_and_right_conversation`, renamed
     `the_side_panel_opens_on_the_right_by_default` and inverted
   — `the_panel_chord_folds_and_reopens_the_side_panel`
7. **The terminal starts right under the session header.** The spacer, rule,
   spacer run between the session header and the terminal frame becomes one
   hairline, so the gap between the header row and the terminal frame is the
   hairline plus the panel's item spacing and no more.
   — `the_terminal_starts_directly_under_the_session_header`

## Behaviour

Strings, all through `tr`/`tf!` with rows in every table of `src/i18n_tables.rs`:

| Where | Japanese |
|---|---|
| pinned section header | 「要対応」 (existing id) and the count |
| wait time, under a minute | 「たった今」 |
| wait time | 「{p0}分」 (minutes, floor) ; 60 and over 「{p0}時間」 |
| row line when the agent is waiting for a permission or an answer | 「許可または回答を待っています」 |
| title-bar button | 「要対応 {p0}」 and the chord label from the keymap |
| ⌘J with an empty queue (`notice_briefly`) | 「返事を待っているセッションはありません」 |
| keymap labels | 「次の要対応セッションへ」, 「サイドパネルを開閉」 |

**What the row shows as the agent's words.** The status hooks already capture
the agent's last message (`HookStatus.last_message`, at most 200 characters,
from payloads that carry `last_assistant_message`). It is shown, clamped to two
lines, only when the hook status is fresh and its activity is `Idle` — the
message that ended the turn. While the activity is `AwaitingInput`, the carried
message belongs to an earlier turn, so the row says
「許可または回答を待っています」 instead. With no message, or a CLI whose
hooks send none, the row has no third line. Nothing is read from the pane or
from transcripts to guess it.

**This narrows intent outcome 1.** "The gist of what it asked" is met as "the
agent's last words at the end of its turn, or a generic line while it waits
for a permission or an answer". The words of a pending permission request or
question are not captured by the hooks today (`PermissionRequest` details are
not parsed); capturing them is a later change, not something this one guesses
at.

**States.**

- Empty queue: no pinned section, no title-bar button. The project groups are the
  list as before.
- Status chips that exclude 要対応: no pinned section.
- Opening a pinned session that was there only because it was unread clears the
  unread mark, so it leaves the section and returns to its project group; the
  terminal stays open on it.
- A waiting session that is not running (ended, lost) is opened without focusing
  a terminal, because a dead pane takes no input.
- Wait time is in-memory: after a restart every queued session is timed from
  launch of the app. It is not persisted.

## Design

Placement: **EXTEND** — the session list, the title bar, the palette, and the
keymap already exist (`ui_terminal_session_tabs`, `ui_topbar`, `palette_entries`,
`KEYMAP_ACTIONS`); nothing here is a new concern. Route stays `feature` because
it adds behaviour a user can see.

- `src/app.rs` — `OperonApp` gains
  `attention_since: HashMap<Uuid, Instant>`, `attention_order: Vec<Uuid>`,
  `terminal_focus_request: Option<Uuid>`.
  `track_attention(now)` runs once per frame from `update` before any panel:
  one non-allocating pass over the sessions (the same shape as
  `status_group_counts`) inserting entrants, then `retain` dropping leavers;
  `attention_order` is rebuilt and sorted only when membership changed.
  `open_next_waiting_session()` implements requirement 3; `session_last_words`
  returns the requirement-2 line. `handle_app_shortcuts` gains the two chords;
  `session_inspector_side` defaults to `Right`.
- `src/app/screens.rs` — `ui_topbar` draws the requirement-4 button (a
  `quiet_button`) and adds its rect to `control_rects`;
  `ui_terminal_session_tabs` draws the pinned section;
  `ui_terminal_panel` drops the spacer/rule/spacer to one `hairline` and honours
  `terminal_focus_request`; `palette_entries` adds the words to the haystack and
  the tie-break.
- `src/app/keymap.rs` — two `KeymapAction` rows.
- `src/i18n_tables.rs` — the rows in the Behaviour table.
- No persisted shape changes; no new subprocess, file read, or background task.
  The draw path reads only in-memory state.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | yes, no new role | the pinned section uses `text_muted`, `warning` (the 要対応 hue the chips already use), `row_selected`; read against `.claude/rules/palette-and-glyphs.md` |
| Icons | no new mark | no glyph literals; the pinned rows reuse the status dot |
| Identifier SSOT | yes | keymap ids `attention.next` and `panel.toggle` live once in `KEYMAP_ACTIONS`; drawing and handler both read them through `chord_for`/`label_for`, as `the_palette_draws_the_chord_the_handler_listens_for` checks |
| Durability | no | nothing persisted; attention times are in-memory by design |
| Subprocess safety | no | no new child process |
| Documentation | yes | `README.md`, `README.ja.md`, `README.ko.md` gain the 要対応 section and ⌘J / ⌘⌥B together; the "⌘K is the only search" sentence stays true |
| Local-first | yes | no network, no `unsafe` |
| Budgets | yes | the agent's words are already capped at 200 characters (`config.rs`); no new scan |

Also read: `DESIGN.md` controls (the title-bar button is a `quiet_button`, one
filled `primary_button` per screen stays 新しいセッション), the type scale (row
excerpt 12, wait time 11.5, header 11.5), and `.claude/rules/rust.md` (the
tracker is the per-frame pass the rule allows: no allocation unless membership
changes).

## Flagged concerns

- **Does ⌘J reach the app while a terminal has focus?** The terminal input
  path must not swallow ⌘-chords. Resolved in the build by
  `opening_the_next_waiting_session_focuses_its_terminal` driving the chord with
  the terminal focused; if the pane consumes it, `chord_consumed` before the
  pane draws, as `terminal.find` already does. Checked by the evaluator:
  `terminal_key_binding` drops every `command` chord, so the pane does not take
  it. The comment above `handle_app_shortcuts` says ⌘ chords are "deliberately
  not live while a terminal has focus"; the code does not gate on terminal focus,
  and this change rewrites that comment to say what the code does — ⌘ is the
  modifier macOS keeps for the app, so app chords stay live over a pane.
- **Changing the default dock side reverses change 122's default.** The person
  chose mockup A, which shows the panel on the right; docking left stays one
  click away, so this is recoverable.

## Acceptance

- `cargo test --locked` passes, including every test named above.
- `bash .claude/hooks/gate-commit.sh` admits the commit; `bash scripts/check-bands.sh`
  no worse than `main`.
- In the running app with three sessions where two finish while a third is read:
  the two appear under 要対応 at the top of the list, oldest first, each with its
  last words when its CLI sends them; ⌘J opens the first with the cursor in its
  terminal; ⌘J again opens the second; the title-bar button disappears when none
  are left; ⌘⌥B folds the right panel.

## Rejected alternatives

- **A text filter above the session list** (shown in mockup A) — `README.md`
  states "検索・操作 (`⌘K`) is the only search"; requirement 5 makes ⌘K find a
  session by what its agent said and rank waiting ones first instead.

- **Mockup B (activity bar + tabs + ⌘P)** — hides the session list while files
  or changes are open, which change 127 already ruled out.
- **Mockup C (reply inbox with answer buttons)** — the buttons depend on reading
  three CLIs' prompts correctly; a misread sends the wrong keys to an agent.
- **Reading the agent's question from the pane or the transcript** — the pane is
  captured only for the selected session, and inferring a terminal's
  conversation is what `.claude/rules/transcripts.md` forbids.
- **Moving the page navigation out of the title bar and adding a status bar**
  (shown in mockup A) — not needed for any outcome in the intent; left for a
  later change so this one stays reviewable.
- **⌘1–9 to jump to the Nth session** — ⌘1–3 open pages today; reassigning them
  is its own decision.
