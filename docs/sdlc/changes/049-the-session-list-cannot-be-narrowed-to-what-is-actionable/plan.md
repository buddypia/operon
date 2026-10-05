# Plan: narrow the session list by what a person can do with each session

- **Spec**: `./spec.md`
- **Approved**: 2026-09-17
- **Status**: approved

This is the plan produced in plan mode and accepted before any file was edited.
If the implementation departs from it, update this file — an abandoned plan is
worse than no plan, because the next reader trusts it.

## Files that change

| File | Change |
|---|---|
| `src/ui/widgets.rs` | New `SessionStatusKind` (fourteen variants) and `SessionStatusGroup` (four). `session_status_view` keeps its signature and its match, but the match now yields a kind; `icon`, `label`, `tone`, `group` become exhaustive-match methods on the kind, and `SessionStatusView` gains a `kind` field built from them. |
| `src/app.rs` | `status_filter: [bool; 4]` on `OperonApp` and its initialiser. `session_status_kind_for`, `status_group_counts`, `session_matches_status_filter`, `clear_status_filter` beside the existing `status_view`. |
| `src/app/screens.rs` | `ui_terminal_session_tabs`: draw the four chips, filter each project's sessions, skip a project with no match, draw the hidden-count or no-match line. Remove the `⚠ n` header badge. |
| `src/i18n_tables.rs` | Seven new message ids in all three tables: the four chip labels, `{p0} 件を非表示中`, `選んだ状態のセッションはありません`, `フィルタを解除`. |
| `src/tests.rs` | The four tests named in `spec.md`'s Acceptance. |
| `README.md` | One sentence on the toggles in each of the three language sections. |

Nothing is added to `src/models.rs`, `src/store.rs`, `src/glyphs.rs`, or
`src/theme.rs`: no persisted shape, no new glyph, no new colour role.

## Order of work

Numbered, so that the tree compiles between steps wherever it can.

1. **`src/ui/widgets.rs` — the kind.** Add `SessionStatusKind` with one variant
   per row of the spec's table, and give it `icon`, `label`, `tone` methods that
   are the exact arms `session_status_view` returns today. Rewrite
   `session_status_view` so its match produces a kind plus the hint, and build
   `SessionStatusView { kind, icon: kind.icon(), label: kind.label().to_owned(),
   hint, tone: kind.tone() }`. **The tree compiles here and the suite is
   unchanged** — every existing caller reads the same four fields, and the only
   new one is `kind`. This step is a pure refactor and is verified as one: the
   suite must be green before step 2 begins.
2. **`src/ui/widgets.rs` — the group.** Add `SessionStatusGroup` with its four
   variants, `SessionStatusGroup::GROUPS` in display order, `icon` and
   `message_id` methods, and `SessionStatusKind::group` as an exhaustive match.
   Nothing calls it yet; the tree compiles.
3. **`src/tests.rs` — the two tests that need no app.**
   `every_session_status_kind_belongs_to_exactly_one_group` and
   `every_status_group_has_a_chip_icon_and_a_message_id`. They pass on step 2's
   code; they are here so that step 4's ids exist before step 5 uses them.
4. **`src/i18n_tables.rs`** — the seven ids, three tables each. Step 3's second
   test fails until this lands, which is the order the test is for.
5. **`src/app.rs`** — the field and the four methods.
   `session_status_kind_for` is `session_status_view(...).kind` reduced to the
   kind alone: it calls the same match with the same three inputs and drops the
   hint, so no `String` is built. `status_group_counts` walks
   `self.store.sessions` once, and adds an unread session to `NeedsYou`
   whatever its kind — the rule `attention_count` already states.
   `session_matches_status_filter` returns `true` when no bit is set.
6. **`src/tests.rs`** — `the_status_filter_shows_everything_when_nothing_is_selected`
   and `an_unread_session_is_counted_as_needing_a_person`.
7. **`src/app/screens.rs`** — the drawing. Chips first (two rows of two, below
   the title row, above the gesture hint), then the per-project filter and the
   empty-project skip, then the foot line. The `⚠ n` badge comes out in the same
   step that puts the 要対応 count on its chip, so the header is never carrying
   two counts at once.
8. **`README.md`**, three languages.
9. The three gates, then `scripts/check-bands.sh`, then the app.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The refactor in step 1 changes a label or a tone by transcription error | A session shows the wrong word or the wrong colour; nothing fails | Step 1 is separated from every other step precisely so the suite runs against a no-op diff. The existing status tests must pass unchanged before step 2. |
| `status_group_counts` becomes a second definition of 要対応 and drifts from `attention_count` | The chip and any other attention reading disagree | `attention_count` keeps its body; the count for `NeedsYou` is the same predicate, and `an_unread_session_is_counted_as_needing_a_person` asserts the overlap rule directly. |
| A `String` sneaks into the counting pass — `label()` returns `&'static str` but a caller writes `.to_owned()` | A stutter with many sessions; no test failure | `label()` returns `&'static str` by signature, and `session_status_kind_for` never touches `hint`. `.claude/rules/rust.md` is the rule; `rust-reviewer` is the second read. |
| A project header is drawn before its sessions are known to be empty | Requirement 5 silently unmet: an empty project still takes four lines | The filtered `Vec` is built before the header is drawn, and the `continue` moves above it. Visible immediately in the app with 空き on. |
| The filter hides the session currently open in the terminal pane | A person loses the row for what they are looking at | Deliberate, and stated in `spec.md`'s Behaviour: the pane does not change. Checked by hand in step 9. |
| A new `SessionStatus` variant is added later and lands in no group | A session that cannot be filtered to | It cannot compile: `group` is an exhaustive match over `SessionStatusKind`, and `session_status_view`'s match is exhaustive over `SessionStatus`. |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
  Six, not seven.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-bands.sh` — `bands: N metrics within their bands`.
- The four new tests, each watched failing before its implementation lands:
  `every_session_status_kind_belongs_to_exactly_one_group`,
  `every_status_group_has_a_chip_icon_and_a_message_id`,
  `the_status_filter_shows_everything_when_nothing_is_selected`,
  `an_unread_session_is_counted_as_needing_a_person`.
- In the running app, with sessions in more than one group: the four chips read
  counts that match the visible entries; 空き alone leaves only the idle
  sessions and drops the projects that have none; the foot says how many are
  hidden; a group whose count is `0`, selected alone, shows
  `選んだ状態のセッションはありません` and a `フィルタを解除` button that
  restores the full list in one click; quitting and relaunching shows
  everything.

## Departures from the plan

- **`src/models.rs` did change, and the plan said it would not.** Deciding
  `BLOCKED` means asking whether a prerequisite will never finish, and the only
  function that answered was `blocking_dependency_label`, which clones the
  prerequisite's name to say so. Calling it once per queued session per frame
  to fill in a count is an allocation in the draw path. So the judgement was
  split: `blocking_dependency` returns a new `BlockingDependency` enum and
  allocates nothing, and `blocking_dependency_label` is that with a name
  attached. One rule still, in one place, and the existing three callers are
  unchanged. Nothing persisted moved, so the durability row of `spec.md`'s
  policy table still reads No.
- **`attention_count` was rewritten rather than left alone.** `spec.md` said it
  would keep its body and be called by the chip. That would have left two
  definitions of 要対応, because the chip's group includes `BLOCKED` and
  `attention_count` did not. It now counts the 要対応 group, which is the same
  predicate the chip filters by — and the home screen's banner therefore counts
  blocked sessions too. That is a small user-visible change beyond the spec,
  and the right one: a session stopped on a prerequisite that failed wants a
  person exactly as much as one stopped on a question.
- **`every_session_status_kind_belongs_to_exactly_one_group` is stronger than
  specified.** As `spec.md` described it — every kind maps to a group, the
  groups are disjoint — it could not fail: a Rust match returns one value, so
  the partition holds by construction. What can actually go wrong is
  `SessionStatusKind::ALL` going stale against the enum, so the test drives
  `session_status_kind` over every `SessionStatus`, every activity, both
  cancellation states, and the two dependency shapes, and asserts the set it
  reaches equals `ALL` exactly. Watched failing by removing `Checking` from
  `ALL`.
- **`SessionStatusGroup` also carries `hint_id`.** Four Japanese words across
  268px cannot say that 実行中 includes 順番待ち, so each chip has hover text.
  Four more message ids than `spec.md` counted, three tables each.
- **`SessionStatusKind::ALL` is `#[cfg(test)]`.** Nothing the app draws
  iterates the states, so in a release build it is dead code and
  `cargo clippy -- -D warnings` says so.
- **The home screen's attention banner changed too, and the plan did not name
  `ui_home`.** Widening `attention_count` to the 要対応 group left its one
  remaining caller saying `{n} 件のセッションが応答を待っています` about a set
  that now includes sessions with no terminal to answer in — a blocked queued
  session has no tmux session at all. The banner reads
  `{n} 件のセッションがあなたを待っています` now, and its button says
  `要対応のセッションを見る` and opens the session list with the 要対応 chip
  already on, which is the shortest path the intent describes. Two more message
  ids in three tables; the id it replaced was removed rather than left orphaned.
  Found by `rust-reviewer`, not by a gate — no test asserts that a count and the
  sentence beside it mean the same thing.
- **A filter toggle clears `pending_session_close`.** The removal confirmation
  is drawn inside the card it belongs to, so narrowing the list can take the
  question off screen with the answer still pending, and un-narrowing would
  bring back a 削除しますか nobody remembers asking for. No work can be lost
  either way — the removal only happens on the confirm click — but a dialogue
  that reappears is a dialogue that gets clicked through. Found by
  `durability-reviewer`.
- **The project header's `{p0} 件` now counts what is shown, not what exists.**
  A consequence of filtering the list before the header is drawn, and the right
  reading — the number is beside the rows it describes — but it is an existing
  label whose meaning changed, which neither `spec.md` nor this plan said.

## Costs this change adds, named

- `attention_count` is drawn on the home screen every frame and was a status
  comparison plus a `HashMap` lookup per session. As the 要対応 group it also
  decides each session's kind, which for a session with prerequisites walks the
  session list to check them. Sessions without prerequisites — nearly all of
  them — still cost a `find_map` over an empty slice, so the added cost is
  bounded by the number of sessions that actually have dependencies. No
  allocation and no syscall either way.
- `session_matches_status_filter` decides the kind once per session rather than
  once per selected chip. `rust-reviewer` measured the unfixed version at four
  full list walks per queued session per frame; it is one now.
- The one scan this change knowingly leaves doubled: a drawn `BLOCKED` row
  walks its dependencies twice, once to decide the kind and once for the hint
  that names the prerequisite. Only rows that are on screen *and* blocked pay
  it, and `depends_on` is normally zero or one entry long.

  There is a third way to collapse it that is better than the two this plan
  weighed, and `rust-reviewer` named it: give `blocking_dependency_label` a
  `BlockingDependency<'_>` instead of the pair it re-derives from, so the call
  becomes `blocking_dependency(sessions, session).map(blocking_dependency_label)`.
  No rule is duplicated and no label moves into `SessionStatusKind`. It changes
  the signature of a function with three existing callers, which is more than
  this change should carry for a cost that is not measurable — but it is the
  shape to reach for next time `src/models.rs` is open.
