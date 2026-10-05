# Spec: narrow the session list by what a person can do with each session

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. One function decides which state a session is in. The icon, the English
   token, the colour tone, and the group all derive from that decision by an
   exhaustive match, so a new `SessionStatus` variant, or a new state kind, does
   not compile until its group is named.
2. The fourteen states map onto exactly four groups, and the mapping is total
   and disjoint over state kinds:

   | Group | Chip | States |
   |---|---|---|
   | `NeedsYou` | 要対応 | `WAITING`, `BLOCKED` |
   | `Free` | 空き | `IDLE` |
   | `Busy` | 実行中 | `WORKING`, `RUNNING`, `STARTING`, `CHECKING`, `STOPPING`, `QUEUED`, `READY` |
   | `Finished` | 終了 | `DONE`, `FAILED`, `STOPPED`, `UNKNOWN` |

3. The session list's header carries four toggle controls, one per group, each
   showing that group's mark and how many sessions are in it right now.
4. The four toggles are independent. With none selected every session is listed,
   which is today's behaviour. With one or more selected, only sessions in a
   selected group are listed.
5. A project all of whose sessions are filtered out contributes nothing to the
   list — not its name, not its path, not its "セッションはまだありません" line.
6. While any toggle is selected the list says how many sessions it is hiding. If
   the selection matches nothing at all, it says that instead, and offers to
   clear the selection in one click.
7. The selection is held on `OperonApp` and is not written to the store, so
   launching the app always shows the whole list.
8. 要対応 counts a session that is unread as well as one whose state is
   `WAITING` or `BLOCKED` — the definition `attention_count` already uses. The
   separate `⚠ n` badge in the header is removed, because the 要対応 toggle now
   carries that number and two differently-defined counts side by side is worse
   than one.
9. Counting the four groups every frame allocates no `String` and runs no
   command, no file read, and no directory scan.

## Behaviour

The header above the list gains two rows of two toggles, below the existing
title row and above the gesture hint. Each reads as a mark, a Japanese word, and
a number: `⚠ 要対応 2`, `○ 空き 2`, `● 実行中 9`, `✓ 終了 7`. The number is how
many sessions are in that group at that moment, whether or not the toggle is on,
so a person can see there is something worth switching to before switching.

Clicking a toggle turns it on; clicking it again turns it off. Several can be on
at once, and the list then shows the union — 要対応 and 空き together is the
combination the intent is about, and it is one click each.

- **Nothing selected.** Every project, every session, exactly as today. No
  hidden-count line, because nothing is hidden.
- **Something selected, something matches.** Only the matching sessions, still
  grouped under their project, newest first. Projects with no match are absent
  entirely. Under the last one: `12 件を非表示中`.
- **Something selected, nothing matches.** `選んだ状態のセッションはありません`
  and a button `フィルタを解除`. The toggles stay visible with their counts, so
  the reason is legible: 空き is on and reads `空き 0`.
- **No sessions at all.** The existing
  `セッションはまだありません。プロジェクトから起動してください。` wins over the
  filter message; a person with no sessions is not being filtered.
- **A group's count is zero.** Its toggle still draws, greyed by its own count
  rather than removed. A control that disappears when it would be useful to know
  it is empty is a control a person hunts for.
- **The selected session is filtered out.** It stays open in the terminal pane
  to the right. Hiding a row does not close what a person is looking at, and
  nothing about the pane changes.

## Design

`src/ui/widgets.rs` splits the existing thirteen-arm decision in
`session_status_view` in two. The match now yields a new
`SessionStatusKind` — one variant per state, no strings — and `icon`, `label`,
`tone`, and `group` become methods on it, each an exhaustive match. The existing
`SessionStatusView` gains a `kind` field; its `icon`, `label`, `tone`, and
`hint` are built from the kind as they are now, so every current caller of
`session_status_view` and `status_chip` is unchanged. `hint` stays outside the
kind: `BLOCKED`'s hint names the dependency, which is data, not a constant.

`SessionStatusGroup` is the four-variant enum, with a `GROUPS` array in
display order and, per group, the `ICON_*` constant and message id its chip
draws.

`src/app.rs` gains `session_status_kind_for(&Session) -> SessionStatusKind`
beside the existing `status_view`, a `status_group_counts() -> [usize; 4]` that
walks the sessions once, and `session_matches_status_filter(&Session) -> bool`.
The selected groups live in a new `status_filter: [bool; 4]` field on
`OperonApp`. `attention_count` keeps its body and becomes what the 要対応 count
calls, so the two cannot drift.

`src/app/screens.rs` draws the chips in `ui_terminal_session_tabs`, filters each
project's sessions through `session_matches_status_filter`, skips a project
whose filtered list is empty, and draws the hidden-count or no-match line at the
foot of the scroll area.

The per-frame cost is one pass over the sessions calling
`session_status_kind_for`, which is a `HashMap` lookup, an `Instant::now()`, and
a match. The only scan inside it is the existing dependency lookup, which only
`Queued` sessions reach. No `String` is built to count; the strings are built
only for the rows actually drawn, as now.

Nothing is added to `src/models.rs` or `src/store.rs`.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | Each chip takes `palette.status(tone)` for its group's representative tone — `Attention`, `Idle`, `Running`, `Neutral` — all four already roles on all three tables. Selected state comes from `ui.selectable_label`, which reads the theme's widget styling, so no colour is named at the call site and `every_theme_stays_readable` still covers every ink used. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | The four chips reuse `ICON_ATTENTION`, `ICON_STATUS_IDLE`, `ICON_STATUS_RUNNING`, `ICON_STATUS_DONE`, which the entries beside them already draw for the same states. No new glyph, so no `ICON_VOCABULARY` or `DESIGN.md` entry. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes, and it is the point | The group a state belongs to is decided once, by `SessionStatusKind::group`, and nothing matches on the English token to find out. Requirement 8 keeps the 要対応 count as a call into `attention_count` rather than a second copy of its rule. No new string is shared across modules, so nothing new belongs in `src/config.rs`. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | Requirement 7: the selection is `OperonApp` state and is never persisted. `src/store.rs` and `src/models.rs` are not edited, so there is no schema change and no version bump. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No child process, no launch path, no change to either module. Requirement 9 forbids a command in the counting path. |
| Documentation — user-facing docs change in all three languages together | Yes | `README.md` describes the session sidebar; the toggles get a sentence in all three of its language sections in the same commit. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | No | Reads state the app already computes; adds no `unsafe`. |
| Budgets — any new scan or output path states its byte and item ceiling | No | No new scan or output path. The counting pass is bounded by the session list already in memory, which the store's own limits bound. |

## Flagged concerns

- **The unread overlay makes the groups non-partitioning at the filter level.**
  Requirement 2 is disjoint over state kinds, and requirement 8 then adds unread
  sessions to 要対応 whatever their state, so an unread `DONE` session appears
  under both 要対応 and 終了 and the four counts can sum to more than the total.
  *Resolved, and deliberate:* the chips are independent OR-filters, not a
  segmented control, so overlap changes nothing about how they behave, and an
  unread finished session genuinely does want a person. The test asserts the
  partition over `SessionStatusKind`, which is the property the compiler is
  being asked to protect; the overlay is asserted separately as its own rule.
  The counts are not claimed anywhere to sum to the total.

## Acceptance

- `cargo test --locked` passes, including:
  - `every_session_status_kind_belongs_to_exactly_one_group` — every
    `SessionStatusKind` variant maps to a group, and the four groups' state
    lists are disjoint and cover all of them.
  - `every_status_group_has_a_chip_icon_and_a_message_id` — each of the four
    names an `ICON_*` constant that is in `ICON_VOCABULARY` and a message id
    that has a row in every table in `src/i18n_tables.rs`.
  - `the_status_filter_shows_everything_when_nothing_is_selected` — with
    `status_filter` all false, `session_matches_status_filter` is true for a
    session in each of the four groups.
  - `an_unread_session_is_counted_as_needing_a_person` — an unread session whose
    state is `DONE` matches the 要対応 filter, and does not match 空き.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-bands.sh` — `bands: N metrics within their bands`.
- In the running app, with sessions in more than one group: the four toggles
  show counts that add up to the visible entries; turning on 空き leaves only
  the idle sessions and drops the projects that have none; the foot says how
  many are hidden; turning on a group with a count of `0` alongside it shows the
  no-match message and the clear button; quitting and relaunching shows the
  whole list again.

## Rejected alternatives

- **Move the list to a strip across the top.** The original request. An entry is
  five lines tall and would have to lose its purpose, its branch, and its
  restore control to fit a row; and a long horizontal scroll is harder to scan
  than the long vertical one it replaces. Recorded in `intent.md`'s Not in scope
  with the reasoning.
- **A single 「すぐ使える」 toggle.** Fewer controls and it answers the stated
  goal directly, but it cannot answer "which of these finished", which is the
  other question a person brings to this list. The four toggles include it as
  one click on 要対応 plus one on 空き.
- **Thirteen checkboxes in a dropdown.** Most expressive, least usable: folded
  into a menu it stops being glanceable, and it requires knowing what `READY`
  means as distinct from `QUEUED`.
- **Sort by group instead of hiding.** Keeps everything reachable, but the list
  stays as long as it is, and the intent's problem is its length. Doing both at
  once would also make it impossible to say which of the two helped.
- **Persist the selection.** A restart would then open on a list that is
  missing sessions with no memory of why. Requirement 7 is the opposite.
- **Derive the group from the existing `StatusTone`.** Would have needed no new
  enum, but the tones do not partition this way — `QUEUED` shares `Attention`
  with `WAITING`, `CHECKING` shares `Neutral` with `DONE` — so it would have
  meant either wrong groups or new tones, and a tone is a colour role.
- **Compute the counts by calling `status_view` for each session.** The obvious
  shape, and it builds two `String`s per session per frame plus a dependency
  scan, in the draw path. Splitting the kind out of the view removes the reason
  to do it.
- **Also filter the セッションツール page and the grid.** Same control, second
  screen, and neither is where the intent's problem is felt. Left out to keep
  one screen's worth of judgement in one change.
- **Make the sidebar collapsible in the same change.** Serves the terminal's
  width, not finding a session, and argues against itself here — a list you
  cannot see is a list you cannot search. Better judged once the list is short.
