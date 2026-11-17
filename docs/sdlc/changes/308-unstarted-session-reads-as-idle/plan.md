# Plan: a session that has not been given its first request reads 未開始

- **Spec**: `./spec.md`
- **Approved**: 2026-11-17
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/tmux/hooks.rs` | `HookReading.new_conversation: bool`; `fn starts_a_new_conversation(payload) -> bool` (`source` is `startup` or `clear`); set from Claude's and Codex's `SessionStart` arms in `normalise_hook_event`, `false` everywhere else. |
| `src/app.rs` | Field `awaiting_first_request: HashSet<Uuid>` (initialised empty); maintained in `apply_hook_event` (insert on `new_conversation`, else remove) and `session_activity_notifications` (remove when the tracker settles on `Working`); removed beside `hook_status.remove` in the launch's hook environment and the session teardown, not in the stale sweep; passed from `session_status_kind_for` and `status_view`. |
| `src/app/relaunch.rs` | Remove the session from `awaiting_first_request` beside its `hook_status.remove`. |
| `src/app/screens.rs` | The two hover `session_status_view` calls pass `self.awaiting_first_request.contains(..)`. |
| `src/app/broadcast.rs` | `broadcast_targets` keeps kinds whose group is `Some(SessionStatusGroup::Free)`; its doc comment names not-started agents. |
| `src/ui/widgets.rs` | `SessionStatusKind::NotStarted` and its arms (`ALL`, `icon`, `status_mark`, `label`, `tone`, `group`); `session_status_kind` / `session_status_view` take `awaiting_first_request: bool`; NotStarted hint; `SessionStatusGroup::Free.hint_id()` widened. |
| `src/glyphs.rs` | `ICON_STATUS_NOT_STARTED = icons::RECORD` with its `ICON_VOCABULARY` row; `StatusMark::Dot` (filled centre dot, radius 2.2 grid units). |
| `src/i18n_tables.rs` | Sorted EN and KO rows for 「未開始」, 「まだ最初の依頼を受けていません」, 「未開始、またはターンを終えて次の依頼を出せるセッション」; remove 「ターンを終え、次の依頼を出せるセッション」 from both tables. |
| `DESIGN.md` | `calm` tone line and the `StatusMark` centre list name the dot for not started. |
| `docs/MANUAL.md`, `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md` | The status table names Not started / 未開始 / 시작 전 under Idle for Claude Code and Codex; the broadcast paragraph says it reaches not-started agents. |
| `src/tests.rs` | Six `unstarted_308_*` tests; `claude_tool_reading`'s `HookReading` literal gains the field; every `session_status_kind` / `session_status_view` call gains the argument (`false` keeps today's meaning); `every_status_label_is_one_of_the_four_group_words` and the kind → mark table widened in place for `NotStarted`. No test removed, no `#[ignore]`. |
| `docs/sdlc/changes/308-…/` | `mutations.py`, `verification.yaml`, `state.yaml`, `approvals.log`, `review.yaml` as the gates ask. |

## Order of work

1. Commit `state.yaml`, `intent.md` (alone first), then `spec.md`, `screen.md`,
   `plan.md`, `approvals.log` after `check-readiness.sh` and
   `check-approvals.sh` pass.
2. Write the six `unstarted_308_*` tests against the names the spec fixes;
   the tree does not compile until step 3 adds `NotStarted`, the field, and the
   argument — that is the watched failure (compile errors naming the missing
   symbols), then each test is run red once the symbols exist as stubs.
3. `src/glyphs.rs`, `src/ui/widgets.rs`, `src/tmux/hooks.rs`, then
   `src/app.rs`, `src/app/relaunch.rs`, `src/app/screens.rs`,
   `src/app/broadcast.rs`; update the existing test call sites.
4. `src/i18n_tables.rs`, `DESIGN.md`, the three manuals.
5. `cargo test --locked unstarted_308_` and the existing tests the spec's
   Acceptance names; `make q.fast`; `bash scripts/check-bands.sh`.
6. `mutations.py` (claim mutations included), `verification.yaml`, the
   `empirical-verifier` falsification pass, `rust-reviewer`; commit, push,
   CI, land, package.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A resumed session reads 未開始 | Operon's own relaunch or `--resume` shows the dot on a conversation with history | R1's resume/compact/fork rows and R2's resume case; mutation accepting `resume` |
| The state survives a turn | A session that worked shows the dot again when idle | R2 (each turn event ends it); mutation dropping the removal in `apply_hook_event` |
| The stale sweep or a launch leaks or drops it | Tab turns to アイドル after 30 quiet minutes, or a relaunch inherits the dot | R3; mutations moving the removal into the sweep and out of the launch reset |
| Broadcast skips not-started agents, or a group count drops them | 「アイドル中のエージェントへ送信…」 lists fewer sessions than the アイドル tile counts | R4; mutation restoring `== Idle` and moving `NotStarted` to `Busy` |
| The dot reads as another state's shape | Two states share one mark, told apart by colour alone | R5's distinctness check over `SessionStatusKind::ALL`; mutation mapping `NotStarted` to `Pause` |
| A sentence promises what the code does not do | Hover or manual says something false | Claim mutations on the hint, the label, and the manual's broadcast sentence |
| `src/app.rs` or `src/app/screens.rs` crosses 10 000 lines | `check-bands.sh` reports `propose` | Additions held under 30 lines each; `bash scripts/check-bands.sh` in the contract |
| Orphan or unsorted catalog row | Commit gate fails | `every_message_row_is_one_the_code_shows`, `every_translation_table_is_sorted_by_message_id`, `every_message_id_has_a_row_in_every_table` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked unstarted_308_` — `6 passed; 0 failed`.
- `cargo test --locked every_status_label_is_one_of_the_four_group_words`,
  `hook_events_normalise_per_provider`,
  `broadcast_282_reaches_only_the_projects_idle_running_agents`,
  `every_icon_resolves_from_the_bundled_icon_font`, and the three catalog
  tests — `ok`.
- `bash scripts/check-bands.sh` — nothing at `propose`.
- `python3 docs/sdlc/changes/308-unstarted-session-reads-as-idle/mutations.py`
  — every mutation caught.
- Packaged app: a Claude Code tab launched without a prompt shows the dot and
  hovers 未開始; after one request it shows the pause and hovers アイドル;
  `/clear` brings the dot back.

## Departures from the plan
