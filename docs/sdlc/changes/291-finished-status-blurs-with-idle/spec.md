# Spec: retire the Finished status group, auto-restore killed sessions in place, and close clean exits to History

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `SessionStatusGroup::GROUPS` contains exactly three groups in order —
   `NeedsYou` (`"要対応"`), `Busy` (`"実行中"`), and `Free` (`"アイドル"`) — with
   no `Finished` group; Home's metric row, the session sidebar's status tiles,
   and the session library's filter chips draw only those three groups, and individual
   clean-exit / cancelled states name themselves `"正常終了"` (`Done`) and
   `"停止済み"` (`Stopped`) rather than `"終了"`.
2. When a live managed session (`Starting`, `Active`, or `Unknown`, excluding
   `setup_sessions`) transitions to `SessionStatus::Lost` or
   `SessionStatus::Failed` during an active Operon run — via `apply_session_poll`
   or via `BackgroundResult::TerminalResized` observing `TmuxState::Gone` after
   `self.persist()` succeeds — and `relaunch_kind(&session)` is `Some`, Operon
   automatically relaunches it in the same session record and tab
   (`LaunchFocus::Keep`) from its saved conversation ID or plain-terminal shell
   and shows a brief toast
   `「強制終了されたセッションを {count} 件自動で復元しました。」` (while preserving
   `「前回開いていたセッションを {count} 件再開しました。」` when the session was in
   `relaunch_candidates` at startup).
3. Automatic restore enforces a per-session cooldown of
   `AUTO_RESTORE_COOLDOWN_SECONDS` (15 seconds) from the last automatic restore
   of that session so an immediate repeat crash stops looping; manual launch or
   manual resume (`start_session` / `resume_managed_native_session`) clears that
   session's cooldown.
4. A session in `SessionStatusKind::Failed` or `SessionStatusKind::Lost` that is
   not automatically restored (no saved conversation/transcript, setup failure,
   store write failure, or repeat crash within the cooldown) keeps its workspace
   tab open and belongs to `SessionStatusGroup::NeedsYou` for as long as it has
   an open workspace tab or is unread; once its tab is closed and it is read, it
   no longer counts under `NeedsYou`, `attention_count()`, the Home warning
   banner, or `⌘J`, while remaining visible in History under `すべて`.
5. When a session transitions to `SessionStatus::Exited` or
   `SessionStatus::Cancelled` (on poll, stop completion, resize-after-cancel, or
   queued cancellation), Operon automatically closes its workspace tab if the
   session is not pinned (recording it in `closed_tabs`, preserving unseen
   neighbour tabs' `unread_sessions` state when off-screen, clearing stale
   `selected_session` / background project selection, clearing `unread_sessions`
   and `unread_hold` on explicit `Cancelled` while keeping `Exited`'s unread
   mark until viewed, and emitting no refusal toast when pinned); opening a past
   exited or cancelled session manually from History keeps its tab open.

## Behaviour

### Three status groups everywhere

- **Home (`ui_home`)**: Draws three equal columns for `要対応`, `実行中`, and
  `アイドル` using each group's `icon()`, `message_id()`, count from
  `status_group_counts()`, and `tone()`. No fourth `終了` tile is drawn.
- **Session sidebar (`ui_sidebar_session_filters`)**: Draws the three tiles
  responsively across the available sidebar width (`gap = SPACE_XS`):
  - If `available_width >= 3.0 * STATUS_TILE_MIN_W + 2.0 * gap`: 1 row of 3
    equal-width tiles (`[要対応] [実行中] [アイドル]`).
  - Else if `available_width >= 2.0 * STATUS_TILE_MIN_W + gap` (including the
    default 268px sidebar): row 1 has `[要対応] [実行中]` at half width, and row 2
    has `[アイドル]` spanning the full row width
    (`(available - gap * (row.len() - 1) as f32) / row.len() as f32`), leaving
    no empty slot on the right.
  - Else (down to the 180px minimum): 3 full-width rows stacked vertically.
  Hovering `要対応` shows `"回答待ち・異常終了・前提が壊れて止まっているセッション"`
  plus `"承認・回答待ち {count} 件"`.
- **Session library / History (`ui_session_library` / list view)**: Shows
  `すべて {count}` followed by the three chips `要対応`, `実行中`, and `アイドル`.
  With no status chip selected (`status_filter == [false; 3]`), `すべて` shows
  every session including `Done` (`正常終了`), `Stopped` (`停止済み`), and closed
  `Failed` / `Lost` records.
- **Individual session labels and hover hints (`session_status_view`)**:
  - `Failed` and `Lost` report `label = tr("要対応")` and keep their specific
    hints (`"異常終了。出力を確認してください"` and
    `"ターミナルが残っておらず、結果を確認できません"`).
  - `Done` reports `label = tr("正常終了")` and
    `hint = tr("ターミナルは正常に終了しました")` so hovering a completed row on
    Home or in History shows a distinct title and detail line instead of
    repeating `"正常終了"` twice.
  - `Stopped` reports `label = tr("停止済み")` and
    `hint = tr("停止操作による終了")`.

### Automatic restore on unexpected exit, cooldown, and fallback

- **Startup first look (`first_look == true`)**: Preserves Change 270's startup
  toast `「前回開いていたセッションを {count} 件再開しました。」` so startup
  recovery keeps naming the previous run, and records `Instant::now()` in
  `last_auto_restore` for each relaunched session.
- **Mid-run crash or killed terminal (`first_look == false`)**:
  - Eligible in `apply_session_poll` when a `Starting | Active | Unknown`
    session transitions to `SessionStatus::Lost` or `SessionStatus::Failed`,
    `!self.setup_sessions.contains(&id)`, `relaunch_kind(&session).is_some()`,
    and `self.auto_restore_ready(id, now)` is `true` (no entry in
    `last_auto_restore` younger than
    `Duration::from_secs(AUTO_RESTORE_COOLDOWN_SECONDS)`), after `self.persist()`
    commits the transition.
  - Also eligible in `BackgroundResult::TerminalResized` when
    `tmux_error_state(&error) == TmuxState::Gone` and `!cancellation_was_pending`
    for a `Starting | Active | Unknown` session:
    - If `!self.persist()` fails (`store_retry_pending = true` + retry warning
      notice), Operon leaves `relaunch_candidates` untouched and skips
      `relaunch_lost_session` so the retry warning notice is not clobbered and
      live state is not mutated over an unpersisted `Lost` transition.
    - When `self.persist()` succeeds, Operon consumes
      `let first_look = self.relaunch_candidates.remove(&session_id);` and runs
      the same `setup_sessions` / `relaunch_kind` / `auto_restore_ready` check,
      showing the startup toast if `first_look` or the mid-run toast otherwise.
  - Calls `relaunch_lost_session(id)` (`LaunchFocus::Keep`: same `Uuid`, same
    tab, same split/focus, repaired torn JSONL tail if needed via
    `prepare_native_resume`).
  - When `relaunch_lost_session(id)` succeeds: prunes `last_auto_restore` to
    current `store.sessions` IDs, records `last_auto_restore.insert(id, now)`,
    suppresses the `"セッションが失敗しました: {name}"` notification for that
    transient crash, and announces
    `「強制終了されたセッションを {count} 件自動で復元しました。」` (for mid-run).
  - **Empty / missing transcript (`NativeResume::NoTranscript`), worktree busy,
    store write error, or cooldown active**: `relaunch_lost_session` returns
    `false` (or is skipped by cooldown). The session stays `Failed` or `Lost` in
    its open tab so the person can read the pane output, enters `要対応` (because
    its tab is open), and if `next == SessionStatus::Failed` with
    `notifications_enabled`, sends `"セッションが失敗しました: {name}"`.
  - **Manual retry / resume**: Calling `start_session(id)` or
    `resume_managed_native_session(id)` calls `clear_auto_restore_cooldown(id)`
    so a user-initiated restart is never throttled by an earlier crash.

### Clean exit and explicit stop close unpinned tabs to History

- **Unread state on `Cancelled` vs `Exited`**:
  - Explicit cancellation (`SessionStatus::Cancelled` across `apply_session_poll`,
    `BackgroundResult::SessionStopped`, `BackgroundResult::TerminalResized` with
    `cancellation_was_pending`, and `cancel_queued_session`, when the session is
    not being deleted via `remove_requested`) clears
    `self.unread_sessions.remove(&id)` and clears `self.unread_hold` if equal to
    `Some(id)` so an explicitly stopped session never remains stuck in `要対応` /
    `attention_count()` / Dock badge / `⌘J` after its tab closes.
  - Clean exit (`SessionStatus::Exited`) keeps any existing `unread_sessions`
    mark so a turn that finished off-screen still alerts the person in `要対応`
    until viewed (`an_unread_session_is_counted_as_needing_a_person`).
- **Tab closing (`close_finished_session_tab(id)`)**:
  - If `is_session_pinned(id)` is `true`, returns immediately without closing
    the tab and without showing `"ピン留め中のセッションは閉じられません…"`.
  - If `self.workspace` holds `WorkspaceTab::Session(id)`:
    - Closes `&tab` and records `remember_closed_tab(&tab)`.
    - When the terminal workspace is visible (`self.page == Page::Sessions && !self.session_library_open`):
      calls `activate_workspace_tab(&next)` if `workspace.active()` is `Some(next)`,
      and if `self.selected_session == Some(id)` afterwards (no active tab or a
      `WorkspaceTab::Document` became active), calls
      `set_terminal_session_context(None)`.
    - When the terminal workspace is off-screen (`self.page != Page::Sessions || self.session_library_open`):
      does **not** call `select_session` (preserving `unread_sessions` /
      `unread_hold` on unseen neighbour tabs); instead updates terminal context
      via `set_terminal_session_context(Some(next_id))` when the active tab is
      `WorkspaceTab::Session(next_id)`, or `set_terminal_session_context(None)`
      whenever `self.selected_session == Some(id)`.
  - Else if `self.project_workspaces` holds `WorkspaceTab::Session(id)`:
    - Closes `&tab` in each `(tabs, selected)` entry, records
      `remember_closed_tab(&tab)` if closed, updates `*selected` whenever
      `*selected == Some(id)` to `tabs.active()`'s session ID (or `None` if empty
      or a `Document` tab), and calls `set_terminal_session_context(None)` if
      `self.selected_session == Some(id)`.
- Opening a finished session later from History, Recent Sessions on Home, `⌘K`,
  or `⇧⌘T` calls `select_session` / `open_session_in_terminal` without triggering
  a status transition, so its tab stays open while the person reads it.

## Design

### Placement — `EXTEND`

- `src/config.rs`: **EXTEND** with `pub(crate) const AUTO_RESTORE_COOLDOWN_SECONDS: u64 = 15;`.
- `src/ui/widgets.rs`: **EXTEND** `SessionStatusGroup` (shrink `GROUPS` from 4
  to 3 by removing `Finished`, update `NeedsYou::hint_id`), change
  `SessionStatusKind::group(self) -> Option<SessionStatusGroup>` (`Failed | Lost => Some(NeedsYou)`,
  `Done | Stopped => None`), update `SessionStatusKind::label(self)` (`Done => "正常終了"`,
  `Stopped => "停止済み"`, otherwise `group.message_id()`), and update `Done`'s
  `hint` in `session_status_view` to `"ターミナルは正常に終了しました"`.
- `src/app/session_order.rs`: **EXTEND** `OperonApp` with
  `clear_cancelled_unread_and_close_tab(&mut self, id: Uuid)` and
  `close_finished_session_tab(&mut self, id: Uuid)` implementing the pinned
  skip, off-screen unread preservation, `Document` active-tab context cleanup,
  and `project_workspaces` `selected` cleanup without growing `src/app.rs`.
- `src/app/relaunch.rs`: **EXTEND** `OperonApp` with
  `auto_restore_ready(&self, id: Uuid, now: Instant) -> bool`,
  `record_auto_restore(&mut self, id: Uuid, now: Instant)`,
  `clear_auto_restore_cooldown(&mut self, id: Uuid)`,
  `relaunch_interrupted_sessions(&mut self, startup_ids: Vec<Uuid>, midrun_ids: Vec<(Uuid, Option<String>)>, now: Instant) -> Vec<String>`,
  and `auto_restore_after_resize_gone(&mut self, session_id: Uuid, now: Instant)`.
- `src/app.rs`: **EXTEND** `OperonApp` with `status_filter: [bool; 3]` and
  `last_auto_restore: HashMap<Uuid, Instant>`; update `kind_in_status_group` and
  `status_group_counts*` (`[usize; 3]`) so `Failed | Lost` count under
  `NeedsYou` only while `session_has_a_tab(session.id) || is_session_unread(session.id)`;
  widen `TerminalResized` (`TmuxState::Gone`) to `Active | Starting | Unknown`
  and gate auto-restore on `self.persist()`; wire `close_finished_session_tab` /
  `clear_cancelled_unread_and_close_tab` into the four `Exited` / `Cancelled`
  transition sites; clear cooldown in `start_session` and
  `resume_managed_native_session`.
- `src/app/screens.rs`: **EXTEND** `ui_home` (`ui.columns(SessionStatusGroup::GROUPS.len(), ...)`
  using `group.icon()`), `ui_sidebar_session_filters` (3/2/1 column responsive
  chunking with per-row width `(available - gap * (row.len() - 1) as f32) / row.len() as f32`),
  `ui_workspace_projects` (`Option<SessionStatusGroup>` for project header chips),
  and session library `group_counts: [0usize; 3]`.
- `src/i18n_tables.rs`, `README.md`, `README.ja.md`, `README.ko.md`,
  `docs/MANUAL.md`, `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md`: **EXTEND** `EN_TABLE`
  and `KO_TABLE` with the four new sorted rows while removing the three orphaned
  rows (`"終了"`, `"ターミナルが終了したセッション"`,
  `"回答待ち・前提が壊れて止まっているセッション"`), and update 3-language docs.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No — existing roles only | Reuses existing `StatusTone` (`Attention`, `Running`, `Idle`, `Failed`, `Neutral`, `Recovered`) and `Palette` roles with no new colour constants. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No — existing marks only | Reuses `ICON_ATTENTION`, `ICON_STATUS_RUNNING`, `ICON_STATUS_IDLE`, `ICON_STATUS_DONE`, and existing `StatusMark` shapes; `ICON_STATUS_DONE` remains referenced by `SessionStatusKind::Done` and Home's all-clear card. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | `AUTO_RESTORE_COOLDOWN_SECONDS` lives in `src/config.rs` and is shared by `src/app/relaunch.rs` and `src/tests.rs`. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes | `SessionStatus` on-disk variants and `STORE_SCHEMA_VERSION` (`5`) are untouched; both `apply_session_poll` and `TerminalResized` skip candidate removal, tab closing, and auto-restore when `!self.persist()` fails, preserving rollback and retry warnings. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No — no new subprocess call sites | Mid-run restore delegates to the existing `relaunch_lost_session` -> `launch_session_record` -> `launch_stored_session` path without adding `Command::new` or shell arguments. |
| Documentation — user-facing docs change in all three languages together | Yes | Updates `README.md`, `README.ja.md`, `README.ko.md` and `docs/MANUAL.md`, `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md` together in the same commit. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Purely local state, workspace tab management, and local transcript resume; zero network calls and zero `unsafe`. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes — bounded in-memory map | Adds no file scan or output buffer; `last_auto_restore` is pruned to existing `self.store.sessions` IDs whenever recorded so deleted sessions leave no entries behind. |

## Flagged concerns

- **Historical `Failed` and `Lost` records flooding `要対応` and the Dock badge** —
  Resolved in Design: `SessionStatusKind::Failed | SessionStatusKind::Lost` maps
  to `Some(SessionStatusGroup::NeedsYou)`, and `kind_in_status_group` gates those
  two kinds on `self.session_has_a_tab(session.id) || self.is_session_unread(session.id)`.
  An unexpected failure whose tab is still open (or unread) demands attention on
  Home, the sidebar, the Dock badge, and `⌘J`; once the person closes its tab
  and has read it, it retires to History (`すべて`) without haunting `要対応`.
- **Crash loop when a CLI binary or resume transcript fails immediately on every launch** —
  Resolved in Design: `last_auto_restore` records the `Instant` of each
  automatic restore per session and refuses another automatic restore within
  `AUTO_RESTORE_COOLDOWN_SECONDS` (15s). The second crash within 15s leaves the
  session `Failed` / `Lost` in its open tab under `要対応` with its error output
  intact and sends the failure notification if enabled.
- **Background auto-close clearing an unseen neighbour tab's unread mark or leaving stale `selected_session`** —
  Resolved in Design: `close_finished_session_tab` calls `activate_workspace_tab`
  only when the terminal workspace is currently visible; off-screen it updates
  `set_terminal_session_context` without calling `select_session`, clears
  `selected_session` when a `Document` tab or no tab remains active, and clears
  stale `selected` entries in `project_workspaces`.
- **`src/app.rs` line budget (`largest_module_lines` is 9,906, near the 10,000 `propose` ceiling)** —
  Resolved in Design: new tab-close and cooldown/relaunch helpers live in
  `src/app/session_order.rs` and `src/app/relaunch.rs`, keeping `src/app.rs` at
  or below its current line count, checked by `bash scripts/check-bands.sh`.

## Acceptance

- `cargo test --locked status_291_` passes all new requirement tests (`status_291_groups_are_three_and_done_stopped_avoid_finished_word`, `status_291_midrun_crash_auto_restores_in_same_tab_with_toast`, `status_291_auto_restore_cooldown_stops_crash_loop_until_manual_retry`, `status_291_unrestorable_failure_stays_in_needs_you_only_while_tab_open_or_unread`, `status_291_clean_exit_and_stop_close_unpinned_tab_but_keep_pinned_and_history_open`).
- `cargo test --locked model_check_a_session_is_relaunched_only_at_its_first_stored_look` passes the bounded state-machine check updated for startup look, mid-run auto-restore, cooldown suppression, and manual cooldown reset.
- `cargo test --locked every_message_row_is_one_the_code_shows` and `cargo test --locked the_documents_name_the_idle_filter_with_the_windows_word` pass.
- `make q.fast`, `bash scripts/check-bands.sh`, and the mutation sweep (`mutations.py` in this change directory) pass with zero surviving mutations.
- In the packaged app: Home and the session sidebar show only 要対応, 実行中, and アイドル; killing a session's tmux pane mid-run reopens its conversation in the same tab; `/exit` closes an unpinned session tab to History while leaving a pinned tab open.

## Rejected alternatives

- **Keep `終了` as a fourth tile and rename it to `プロセス終了`** — Rejected because the person explicitly questioned keeping `終了` at all ("終了とか要る？") and approved retiring the fourth bucket in favour of automatic restore plus History (`進めて`).
- **Auto-start a fresh CLI session when `native_session_id` has no transcript on disk** — Rejected because launching fresh would silently re-send or lose the person's initial prompt; leaving the tab open in `要対応` preserves the error output and lets the person click `再試行` deliberately.
- **Count every historical `Failed` / `Lost` record in `store.sessions` under `要対応` forever** — Rejected because months-old closed failures in `store.json` would permanently inflate Home's warning banner, the Dock badge, and `⌘J`.
