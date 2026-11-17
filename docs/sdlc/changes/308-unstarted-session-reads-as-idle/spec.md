# Spec: a session that has not been given its first request reads 未開始, not アイドル

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. **A new conversation is read from the CLI's own word.** A Claude Code or
   Codex `SessionStart` hook whose `source` is `startup` or `clear` yields a
   reading marked as the start of a new conversation. `resume`, `compact`,
   `fork`, a missing `source`, every other event, and every Antigravity event
   are not. Test: `unstarted_308_a_new_conversation_is_read_from_session_start`.
2. **The session reads 未開始 until its first request.** A live (`Active`)
   session whose latest hook reading was a new conversation, and whose activity
   is idle, has the state `NotStarted`: label 「未開始」, hint 「まだ最初の依頼を
   受けていません」. Any later reading that is not a new conversation —
   `UserPromptSubmit`, a tool event, a permission request, a `Stop`, a resume —
   ends it, and it does not come back until another new conversation. While
   the activity is working or awaiting input, the usual state shows. A plain
   `SessionStart` from a resume still reads アイドル. Test:
   `unstarted_308_a_session_is_not_started_until_its_first_request`.
3. **It outlives a quiet hook, not a turn.** After the hook reading is dropped
   as stale (`HOOK_STALE_AFTER_SECONDS`) and the screen settles on idle, the
   session still reads 未開始. A screen reading that settles on working ends
   it. A new launch of the session (start, retry, relaunch) and removing the
   session end it. Test:
   `unstarted_308_the_mark_outlives_a_stale_hook_but_not_a_turn_or_a_launch`.
4. **It is still a session that can take work.** `NotStarted` belongs to the
   アイドル group: it is counted in the アイドル tile and chip, shown by the
   アイドル filter, and is a target of 「アイドル中のエージェントへ送信…」. Test:
   `unstarted_308_counts_as_idle_and_receives_a_broadcast`.
5. **It is told apart without colour.** `NotStarted` has its own `StatusMark`
   (`Dot`: a filled dot at the centre) and its own Phosphor status glyph
   (`ICON_STATUS_NOT_STARTED`), distinct from every other state's mark, and
   shares アイドル's `StatusTone::Idle`. The pinned label rule allows
   exactly this one live state a word of its own. Tests:
   `unstarted_308_has_its_own_mark_word_and_hint`, and the existing
   `every_status_label_is_one_of_the_four_group_words` and the kind → mark
   table test widened in place.
6. **The manuals say it.** `docs/MANUAL.md`, `docs/MANUAL.ja.md`, and
   `docs/MANUAL.ko.md` name the state (Not started / 未開始 / 시작 전) beside
   アイドル, say it applies to Claude Code and Codex, and say the broadcast
   reaches it. Test: `unstarted_308_the_manuals_name_the_state`.

## Behaviour

- **Tab and sidebar row.** A Claude Code or Codex session launched without a
  prompt reads 実行中 while it starts (`Starting`, unchanged), then, once the
  CLI reports `SessionStart(startup)`, its mark's badge shows a centred dot in
  the アイドル colour instead of the pause bars. A session launched with a
  prompt goes straight from starting to working: the prompt's
  `UserPromptSubmit` ends the state as it arrives.
- **Hover** on the tab or row: 「Claude Code」 / 「未開始」 / 「まだ最初の依頼を
  受けていません」. The working-time suffix never appears: no turn has run.
- **Status chip** (History rows, Home rows, palette): the Phosphor glyph
  `RECORD` (a ring holding a dot) and 「未開始」.
- **アイドル tile and chip hover** reads 「未開始、またはターンを終えて次の依頼を
  出せるセッション」 instead of 「ターンを終え、次の依頼を出せるセッション」.
- **`/clear`** in a session that had turns: it reads 未開始 again.
- **Empty**: no session — nothing changes. **Error**: a CLI that never sends
  `SessionStart` (hooks not installed, Antigravity, a custom command) reads
  アイドル when idle, as today. **Interrupted**: a session whose terminal dies
  leaves the state with its launch; the relaunch reads whatever its CLI then
  reports (a resume reads アイドル). **Restart of Operon**: the state is not
  remembered; running sessions read アイドル until their next new conversation.
- No notification, unread mark, or Dock badge is raised by entering or leaving
  the state: the boundary already never notifies, and the first request's turn
  notifies as any turn does.

Strings a person reads, with their catalog rows:

| Japanese | English | Korean |
|---|---|---|
| 未開始 | Not started | 시작 전 |
| まだ最初の依頼を受けていません | No request yet | 아직 첫 요청을 받지 않았습니다 |
| 未開始、またはターンを終えて次の依頼を出せるセッション | Sessions not started yet or that finished their turn | 시작 전이거나 턴을 마치고 다음 요청을 보낼 수 있는 세션 |

The row 「ターンを終え、次の依頼を出せるセッション」 loses its last user and is
removed from all three tables.

## Design

Placement verdict: **EXTEND**. The state machine of a session's live status
exists — `SessionStatusKind` and `session_status_kind` in `src/ui/widgets.rs`,
fed by `normalise_hook_event` / `HookReading` in `src/tmux/hooks.rs` and by the
screen tracker in `OperonApp::session_activity_notifications` — and grows one
state. Evidence: `git grep -n "SessionStatusKind::Idle"` lists the five
`widgets.rs` match arms and the one `broadcast.rs` filter that change.

- **`src/tmux/hooks.rs`.** `HookReading` gains `new_conversation: bool`, set by
  `normalise_hook_event` through one helper,
  `starts_a_new_conversation(payload)` (`source` is `startup` or `clear`), for
  Claude's and Codex's `SessionStart` arms only. Codex's arm keeps reading every
  `SessionStart` as an idle boundary; only the new flag reads `source`.
  `HookStatus` and `apply_hook_reading` do not change.
- **`src/app.rs`.** `OperonApp` gains `awaiting_first_request: HashSet<Uuid>`,
  in memory only.
  - `apply_hook_event`: after `normalise_hook_event`, insert the session when
    `reading.new_conversation`, otherwise remove it.
  - `session_activity_notifications`: after a screen observation, remove the
    session when its tracker has settled on `Working`.
  - Removed beside `hook_status` where a launch or a removal resets it (the
    launch's hook environment, the session's teardown,
    `src/app/relaunch.rs`'s relaunch) — not in the stale sweep, which is the
    case requirement 3 keeps it through.
  - `session_status_kind_for` and `status_view` pass
    `self.awaiting_first_request.contains(&session.id)` on.
- **`src/ui/widgets.rs`.** `SessionStatusKind::NotStarted`, in `ALL` after
  `Idle`; `icon` → `ICON_STATUS_NOT_STARTED`, `status_mark` → `StatusMark::Dot`,
  `tone` → `StatusTone::Idle`, `group` → `Some(Free)`, `label` → 「未開始」.
  `session_status_kind` and `session_status_view` take
  `awaiting_first_request: bool`; `Active` with `Some(Idle)` and the flag set
  is `NotStarted`. Hint 「まだ最初の依頼を受けていません」.
  `SessionStatusGroup::Free.hint_id()` becomes the new group hint.
- **`src/glyphs.rs`.** `ICON_STATUS_NOT_STARTED = icons::RECORD` and its
  `ICON_VOCABULARY` row. `StatusMark::Dot` paints a filled circle of 2.2 grid
  units at the centre (so `paint` shows a ring holding a dot and `paint_badge`
  a dot in the badge's surface colour).
- **`src/app/broadcast.rs`.** `broadcast_targets` keeps a session whose kind's
  group is `Some(SessionStatusGroup::Free)` (Idle or NotStarted) instead of
  `== Idle`.
- **`src/app/screens.rs`.** The two hover calls to `session_status_view` pass the
  flag the same way.
- **`DESIGN.md`.** The `calm` tone line and the `StatusMark` centre list name
  the dot for not started.
- No persisted shape, no external command, no new background task. The set is
  read by the draw path as a `HashSet::contains`, no allocation.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | Yes, no new role | Reuses `StatusTone::Idle` (`calm`); `DESIGN.md`'s calm line is widened to name the state. No colour literal. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | Yes | `ICON_STATUS_NOT_STARTED` with its vocabulary row; `every_icon_resolves_from_the_bundled_icon_font` checks it. `StatusMark::Dot` is vector, and `DESIGN.md`'s centre list names it. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | The two `source` words are another CLI's vocabulary, spelled once in `starts_a_new_conversation` and used by both arms; requirement 1's test feeds real payload shapes through `normalise_hook_event` and requirement 2's through `apply_hook_event` (the round trip). |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | Store, cancellation, migration, and byte limits are untouched; the set is in memory only (read against the skill's surface list). |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No child process or launch path is added. |
| Documentation — user-facing docs change in all three languages together | Yes | `docs/MANUAL.md`, `.ja.md`, `.ko.md` change together (requirement 6); `README*.md` do not describe per-state words and are unchanged. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | None added. |
| Budgets — any new scan or output path states its byte and item ceiling | No | No scan or output path; the set holds at most one entry per stored session and is cleared with it. |
| Draw path (`.claude/rules/rust.md`) | Yes | One `HashSet::contains` per status query; no allocation, I/O, or command. |
| Module size band (`largest_module_lines`, propose at 10 000) | Yes | `src/app.rs` (9 861) and `src/app/screens.rs` (9 905) each grow by under 30 lines; `bash scripts/check-bands.sh` is in the contract. |

## Flagged concerns

- **Codex's `source` field.** Answered: Codex's published hook schema
  (`codex-rs/hooks/schema/generated/session-start.command.input.schema.json`)
  requires `source` ∈ `startup resume clear compact fork`. A Codex build that
  omits it reads アイドル, today's behaviour.
- **Hook order.** Claude's tool events run async (change 301), but
  `SessionStart` and `UserPromptSubmit` are the synchronous lifecycle hooks that
  bracket a turn; a reordering there would already mislabel today's activity.
  Not made worse.

## Acceptance

- `cargo test --locked unstarted_308_` — 6 passed.
- `cargo test --locked every_status_label_is_one_of_the_four_group_words`,
  `hook_events_normalise_per_provider`,
  `broadcast_282_reaches_only_the_projects_idle_running_agents`,
  `every_icon_resolves_from_the_bundled_icon_font`,
  `every_message_row_is_one_the_code_shows`,
  `every_message_id_has_a_row_in_every_table`,
  `every_translation_table_is_sorted_by_message_id` — ok.
- `make q.fast` and `bash scripts/check-bands.sh` — no new breach.
- In the packaged app: a Claude Code tab opened without a prompt shows the dot
  badge and hovers 未開始; after one request and its reply it shows the pause
  badge and hovers アイドル; `/clear` brings the dot back.

## Rejected alternatives

- **A flag on `HookStatus`** — the stale sweep drops `HookStatus` after 30
  minutes, so a tab left alone would turn into アイドル with nothing having run.
- **Keeping a fresh boundary from going stale** — the sweep is what lets the
  screen take back a session whose hooks went quiet; exempting it would pin a
  wrong state forever when hooks break.
- **Persisting the state in the store** — a store shape change (paused surface)
  for a state that a restart of Operon costs only until the next `/clear`.
- **A new group or chip for not started** — the person's question is "which
  idle ones have a result", answered by the mark; the chip set stays three
  actions (respond, wait, hand work), and a not-started session takes work.
- **Reusing `StatusMark::Play`** — `Ready` already wears it; two live states
  with one shape would be told apart by colour alone.
- **Inferring "not started" from the screen or the transcript** — the screen
  has no stable marker for an empty conversation, and the transcript's
  existence differs per CLI; only `SessionStart`'s `source` is the CLI's own
  word.
