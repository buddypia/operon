# Spec: say how long a working agent has been working

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. A session's working time is measured from the first frame that finds its
   state 実行中 with the agent working (`SessionStatusKind::Working`) and is
   forgotten on the first frame that finds it in any other state, or gone. —
   `working_278_clock_runs_only_while_the_agent_works`; that every frame
   keeps the clocks — `working_278_frame_keeps_the_clocks`
2. A session that stops working and works again is measured from zero. —
   `working_278_a_new_turn_starts_from_zero`
3. The time is worded like the waiting time, without its first step: nothing
   under one minute, 「{n}分」 from one to fifty-nine minutes, 「{n}時間」 from
   sixty. — `working_278_time_is_said_in_minutes_then_hours`
4. In the per-project session list (履歴's list and the docked session
   panel), a working session's row reads 「実行中 · 12分」 where it read
   「実行中」, once there is a time to say; a session that is not working, or
   has worked under a minute, reads as before. —
   `working_278_row_says_how_long_the_agent_has_worked`
5. The heading of an open pane reads 「実行中 · 12分」 beside its state mark
   in the same way. —
   `working_278_open_pane_says_how_long_the_agent_has_worked`
6. A hover that explains a working session's state ends with the same time:
   「応答を生成中 · 12分」, 「Bash を実行中 · 12分」. That is the hover of every
   state chip and row word drawn from `status_view` —
   `working_278_hover_says_how_long_too` — and the hovers of a session's row
   in the project column and of its tab's state mark —
   `working_278_column_and_tab_hovers_say_how_long`.

## Behaviour

Three places say it. The per-project session list's row, whose second line
is `agent mark · state word`, becomes `agent mark · 実行中 · 12分` in the same
size and colour as the state word. The open pane's heading chip, `● 実行中`,
becomes `● 実行中 · 12分`. And the hovers: the project column's rows and the
tabs print no state word (changes 250 and 253 took it off them), so there the
time is in the hover, after the sentence that says what the state is. The time
appears at one minute and counts in minutes, then in hours from one hour
(「1時間」, 「2時間」). It refreshes with the frame, which wakes at least every
two seconds.

- **Empty** (no sessions) and **not working** (起動中, 順番待ち, アイドル, 要対応,
  終了): unchanged.
- **Just started** (under a minute): 「実行中」 alone — a count that reads
  「0分」 says nothing.
- **Too long**: 「実行中 · 12時間」 is the widest ordinary form; the row
  already truncates its line, so a narrow column cuts the branch name after it
  first, as today.
- **Interrupted**: the agent stops (waits, finishes, the terminal ends) — the
  time goes with the state. Operon restarted while an agent works: the count
  starts from when Operon first sees it working again; nothing is stored.
- **Error**: none of its own; it reads state the app already holds.

No new string: 「{p0}分」, 「{p0}時間」 and 「{p0} · {p1}」 are already in
every table of `src/i18n_tables.rs`.

## Design

Placement: **NEW_IN_MODULE** — session state tracking and its wording live in
`src/app.rs` beside `track_attention` / `attention_wait_label`
(`git grep -n attention_since`); the code goes in a new file under that
module, `src/app/working_time.rs`, because `src/app.rs` and
`src/app/screens.rs` stand at the `largest_module_lines` diagnose tier.

- `src/app/working_time.rs`: `impl OperonApp { track_working(now),
  working_time(session_id, now) -> Option<Duration>,
  working_time_text(session_id, now) -> Option<String> }` and
  `working_time_label(Duration) -> Option<String>`. `track_working` is the
  shape of `track_attention`: the map is taken out of `self`, entries whose
  session is no longer `Working` are dropped, sessions newly `Working` are
  stamped `now`; no allocation when nothing changes.
- `src/app.rs`: field `working_since: HashMap<Uuid, Instant>` (not persisted);
  `update` calls `track_working` after `track_attention`; `status_view`
  appends the time to `hint` with `tf!("{p0} · {p1}")`, after the tool a
  hook names has replaced it; a closed session's
  entry goes with the next `track_working`. `mod working_time`.
- `src/app/screens.rs`: `with_working_time` wraps the state word of the
  per-project list's row and of the open pane's heading, and the hint line of
  the project column row's hover and of the tab mark's hover (both build
  their view with `session_status_view`, not `status_view`).
- `CLAUDE.md`: the module map names the new file.
- `README.md`, `README.ja.md`, `README.ko.md`, `docs/MANUAL.md`,
  `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md`: one sentence each where the
  session list's states are described.
- No persisted shape, no external command, no background thread. The draw
  path gains one map lookup per drawn row and, for a working row with a time,
  one small `String`.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | The time is part of the state word's own label: same role, same ink. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No glyph added. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | No identifier added; the tests build their expected text through `tr`/`tf!` and the label function, not literals of the separator. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | Nothing is stored; `src/models.rs` and `src/store.rs` are untouched. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No child process. |
| Documentation — user-facing docs change in all three languages together | Yes | The README and the manual, in English, Japanese and Korean, each gain the sentence. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | No | None added. |
| Budgets — any new scan or output path states its byte and item ceiling | No | No scan or output path. |

## Flagged concerns

- **A state that flickers** — an agent whose state is read off its screen can
  be seen as not working for one reading and working again, which restarts the
  count. Resolved: accepted; `ActivityTracker` already settles a state before
  reporting it, and a restarted count errs towards "recently active", which is
  the safe side for "is it stuck".

## Acceptance

- `cargo test --locked working_278_` passes (eight tests).
- `make q.fast` passes; `cargo test --locked claude_md harness` style guards
  for the module map and the three-language documents pass.
- `python3 docs/sdlc/changes/278-working-agent-does-not-say-how-long/mutations.py`
  catches every mutation.
- In the running app: an agent given a long task reads 「実行中」, then
  「実行中 · 1分」 after a minute, and loses the time when its turn ends.

## Rejected alternatives

- The time on the session tab — change 250 took words off the tabs.
- Seconds — a number that changes every frame draws the eye to every row.
- A stored start time — it would survive a restart and then claim a duration
  Operon did not observe.
