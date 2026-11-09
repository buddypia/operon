# Plan: a relaunch brings back the sessions the person had, and a terminal takes the kind of the CLI running in it

- **Spec**: `./spec.md`
- **Approved**: 2026-11-09
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/app.rs` | `terminal_provenance`; the kind switch in `adopt_native_identity`; `diff_comment_target` without the origin clause; `start_session` split into `launch_session_record(id, LaunchFocus)`; `prepare_native_resume` taken out of `resume_managed_native_session` and its `EmptyOrMissing` arm fixed; `relaunch_candidates`, `quiet_launches`, `relaunch_remembered_sessions`, `relaunch_lost_session`; the `SessionStarted` handler's quiet branch; the candidates' `failed` notification suppressed in `apply_session_poll`; `saved_workspace_layout`, `workspace_layout_save_failed`, `restore_workspace_layout`, `workspace_layout_snapshot`, `save_workspace_layout_if_changed`, called from `from_state`, the two-second tick, and `on_exit` |
| `src/app/screens.rs` | the error-explain `is_agent` test without the origin clause |
| `src/ui/session_tree.rs` | `PaneNode` serde; `WorkspaceLayout`, `SavedWorkspace`, `SavedGroup`; `WorkspaceTabs::to_saved` / `from_saved`; `workspace_layout_path`, `load_workspace_layout`, `save_workspace_layout`; `PaneLayout`'s doc comment |
| `src/config.rs` | `WORKSPACE_LAYOUT_FILE_NAME`, `WORKSPACE_LAYOUT_FILE_MAX_BYTES` |
| `src/i18n_tables.rs` | English and Korean rows for 「前回開いていたセッションを {count} 件再開しました。」 and 「タブの配置を保存できませんでした: {error}」 |
| `README.md`, `README.ja.md`, `README.ko.md` | one bullet after the `session-order.json` one |
| `src/tests.rs` | the eight tests below |
| `docs/sdlc/changes/270-…/` | state, plan, verification, mutations, approvals |

## Order of work

1. **Failing tests first**, each run by name and watched failing on its
   assertion (stubs only where a name must exist to compile: empty
   `WorkspaceLayout` types and functions that do nothing, and
   `relaunch_candidates` filled but never read):
   - `a_terminal_takes_the_kind_of_the_cli_running_in_it` — a plain terminal
     (`origin PlainTerminal`, kind `terminal`, plain goal) receives a Claude
     `SessionStart` with `session_id`: kind is `CliProvider::Claude.agent()`
     (asserted through `cli_provider_for_agent`), goal empty, origin kept, and
     the reloaded store says the same. Sending the same ID again after setting
     the kind back to `terminal` switches it again (the early return does not
     skip it). A later Codex report moves it to `codex`. A legacy record
     (`origin None`, kind `terminal`) switches and gains `origin
     PlainTerminal`; an `Adopted` one keeps `Adopted`. A session launched as
     `codex` receiving a Claude report stays `codex`.
   - `a_terminal_running_claude_receives_review_notes` — the switched session
     is `diff_comment_target`; a plain terminal is not.
   - `the_workspace_comes_back_after_a_relaunch` — two projects; project A:
     three sessions in tabs, one moved into a right-hand split with
     `move_tab`, share set to 0.3, the second tab in front, the third tab
     closed (its session still `Active`); project B's workspace in
     `project_workspaces` with one tab. `save_workspace_layout_if_changed`, then
     `OperonApp::from_state` with the same store and data file: the same
     groups, tabs, front tabs, focused group, layout and share, selected
     session, workspace project, and project B's workspace; after
     `sync_workspace_live_sessions`, the closed session is still not a tab,
     and a session added to the store before the second start is.
     A save with nothing changed does not rewrite the file (its modification
     time and bytes are unchanged), and an app whose workspace was never laid
     out writes nothing.
   - `a_damaged_workspace_file_restores_only_what_is_whole` — garbage bytes, a
     file over the ceiling, and an empty object each give today's start
     (`workspace_initialized` false, no groups); a file naming a deleted
     session drops it and closes its emptied group's pane; a pane naming a
     group that is not there, a pane tree with a repeated leaf, and more than
     `PANE_MAX` panes each drop that project's layout (no groups) while
     another project in the same file is restored; a session in two groups
     stays only in the first; a front-tab index of 9 and a share of 7.0 come
     back inside their bounds; an unknown project is skipped.
   - `sessions_running_at_the_quit_come_back_when_their_terminal_is_gone` —
     the store holds: a plain terminal `Active`; a Codex session `Active` with
     a conversation ID and a one-line transcript file; a Claude session
     `Active` with no ID; a Codex session `Exited`; a Codex session already
     `Lost`; a terminal `Active` that will be observed alive; a Codex session
     `Active` observed dead with status 1. Built with `from_state` (so the
     candidates are taken from the store), tools `tmux` and `codex` set, the
     worktrees pointing at a folder that does not exist (the launch then fails
     before tmux is asked). `apply_session_poll` with the observations: the
     plain terminal, the Codex with an ID, and the crashed Codex are
     `Starting` with `SessionStart` background keys, the Codex records carry
     `codex resume <id>` in `agent_command` under the same id and tmux name,
     no record was added, the selected session and the page are unchanged;
     the Claude without an ID, the `Exited`, the earlier `Lost`, and the alive
     one are untouched. A second poll with the same observations starts
     nothing more.
   - `the_relaunch_says_how_many_sessions_it_reopened` — the same set: right
     after the poll the notice is 「前回開いていたセッションを 3 件再開しました。」;
     no 「セッションが失敗しました」 notification body was produced for the
     crashed candidate; after the background launches finish (they fail on the
     missing folder) each is `Failed`, `launch_progress` is `None`, and the
     selected session and page are still unchanged. A quiet launch that
     succeeds (an `Ok` `SessionStarted` sent on `background_sender` and drained by `process_background_results`) keeps the
     notice that was showing and the selection.
   - `resuming_a_session_with_no_transcript_starts_it_fresh_once` — a `Lost`
     Codex session with an ID and no transcript: `start_session` returns
     (before the fix this overflows the stack, so it is watched failing on its
     own), the store still holds one record, and its `SessionStart` key is
     present.
   - `the_manual_resume_still_makes_a_new_record` — a `Lost` Codex session with
     an ID and a transcript: `start_session` adds one `Resumed` record carrying
     `codex resume <id>`, as before the split.
2. **Code**, in the order the tests above: kind switch; agent consumers;
   `config.rs` constants and the saved types; restore and snapshot; the
   `start_session` split and `prepare_native_resume`; candidates, relaunch,
   quiet completion, the notification suppression; the notice.
3. **Wiring**: the two strings in every table; the README bullet in three
   languages.
4. **Tidy** the diff: no per-frame work added, no new `unwrap`.
5. **Verify**: `make q.fast`; `cargo test --locked` for the eight tests and for
   the existing guards this touches — `a_fresh_hook_state_outranks_the_screen_and_a_stale_one_yields`,
   `every_message_the_code_shows_has_a_row_in_every_table`, the `workspace_`
   tab tests, and the resume tests (`rg -n "resume_managed_native_session|start_session\(" src/tests.rs`).
   `cargo test --locked -- --ignored` only if restore code moved (it does not:
   `src/history.rs` and the archive are untouched).

## Risks

- **Splitting `start_session`** can change the manual start or retry. Guarded by
  the manual-resume test above and the existing launch tests; the split is a
  move of the tail into a function with one added parameter.
- **A relaunch storm**: every candidate starts at once. Bounded by the sessions
  that were running at the quit, each on its own background key, the same
  number the person had running.
- **A restored layout that does not fit** a smaller window: `fits` is consulted
  only on drop; drawing a layout under its minimum is what today's window
  shrink already does (panes clamp). No new failure.
- **Starting a CLI by itself** after a restart: resume commands never carry a
  prompt (`agent_shell_command` drops the goal for a native resume), so the CLI
  waits at its prompt. Dangerous launch options travel with the resume as they
  do on the manual 「再開」, which the person acknowledged at launch.
- **The kind switch on a terminal that has left the CLI**: the session stays
  Claude after `/exit` back to the shell — the person's choice; the screen
  reader then finds no agent activity and the row reads idle.

## Proof of completion

The eight tests pass; each was watched failing first; `make q.fast` passes;
the mutation sweep (`mutations.py`) catches every mutation it lists; CI passes on
the branch head. After landing, the app is packaged from `main` and the person's
relaunch is observed: tabs come back, and a terminal running Claude shows the
Claude mark.

## Departures from the plan
