# Plan: a launch sheet over any page, a real 概要, restore in 履歴

- **Spec**: `./spec.md`
- **Approved**: 2026-09-27
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/app.rs` | `launch_sheet_open`, `history_project` fields and init; `open_project_session_setup` opens the sheet; `launch_ready(&Project)`; `ui_launch_sheet` called in `update` before the two modals; Esc / ⌘↩ handling; `launch_detail_title` prefix 「詳しい設定（任意）」 |
| `src/app/screens.rs` | `ui_launch_sheet` (focus on open, notice in footer); `ui_launch_form` from the launch half of `ui_overview`; `ui_agent_choice` as a compact selectable frame; new `ui_overview` summary; `ui_cli_sessions` moved out of `ui_overview`; 履歴 section under `ui_sessions`; the worktree tab's 「この場所で新規セッション」 calls `open_project_session_setup` |
| `src/i18n_tables.rs` | EN and KO rows for new ids; unused ids removed |
| `src/tests.rs` | `overview_labels` → `sheet_labels`; the four launch-screen tests re-pointed; source anchor updated; 091 project-start test; six new tests |
| `README.md`, `README.ja.md`, `README.ko.md` | the launch paragraph: a sheet from anywhere, ⌘↩; restore in 履歴 |

## Order of work

1. Fields (`launch_sheet_open`, `launch_sheet_needs_focus`,
   `history_project`), `launch_ready`, `open_project_session_setup`, the
   worktree tab's button. Compiles.
2. `ui_launch_form` + `ui_launch_sheet` + keys; closing on launch.
3. `ui_overview` summary; `ui_cli_sessions` into 履歴.
4. i18n; tests moved and added; README ×3.
5. Gates, mutation check, rust-reviewer and subprocess-safety-reviewer (the
   launch entry moved), commit, package, install, look at the screen.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A launch safeguard is lost in the move | dangerous switch launches unasked | `a_dangerous_switch_cannot_be_launched_without_the_acknowledgement`, now against the sheet; ⌘↩ shares `launch_ready` |
| ⌘↩ bypasses the disabled button | a dangerous switch launches without the acknowledgement (the one refusal `launch_session` does not make itself) | `cmd_enter_and_the_button_share_one_readiness`, on exactly that case |
| The worktree tab's new-session button lands on a summary | the worktree 作業場所 cannot be chosen | `the_worktree_tabs_new_session_opens_the_sheet_there` |
| A refusal is drawn under the scrim | the sheet stays open and says nothing | footer repeats `self.notice`; reviewer |
| The sheet cannot be closed | stuck modal | Esc / ✕ test |
| Entry points still navigate away | same as before | `new_session_opens_the_sheet_over_the_current_page` |
| Restore becomes unreachable | no way to import a CLI conversation | `cli_restore_is_in_history_not_in_the_overview` |
| 概要 does I/O per frame | stutter | counts read caches; requests deduplicated by `BackgroundKey`; reviewer |
| Sheet over the launch-progress modal | progress hidden | the sheet is drawn before the modals, which are `Foreground` after it |
| Keys typed in the sheet reach the terminal | stray input to an agent | req 1a moves focus to the goal field; `opening_the_sheet_takes_focus_from_the_terminal` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- New tests watched failing with their rule inverted.
- Installed app: sheet, 概要, 履歴 as in `./screen.md`.

## Departures from the plan

Carried from the round-2 evaluation, recorded here rather than moving the
approved digests:

- The spec's "Account row vs the mock" concern says "the sheet's one
  departure"; screen.md lists three, and screen.md is right.
- The tests row says "six new tests"; the spec's Acceptance names eight.

Found while building:

- 履歴's session list ends in a scroll area that fills the rest of the pane,
  so a section drawn under it gets no height. The CLI section is drawn above
  the list instead, as a folded 「CLI セッション（過去の会話の検出と復元）」 with
  its project picker (screen.md said "under today's session grid").
- `cmd_enter_and_the_button_share_one_readiness` does not assert the button
  is drawn disabled; the shared `launch_ready` is what the button's enabled
  state reads, and the ⌘↩ mutation (readiness dropped) went red.
- `ui_history_cli_section` is `pub(crate)` so the suite can draw it.
- README ×3: one paragraph after the toolbar one — the sheet, ⌘↩, 概要, and
  restore in 履歴.

Found in review (each an Important finding, each with a test watched red):

- The sheet's project picker goes through `switch_launch_project`, which
  keeps the request, the name, and the session behind the sheet.
- The app chords moved from `update` into `handle_app_shortcuts`; ⌘K, ⌘N,
  ⌘1–3, ⌘, and ⌘O are not live while the sheet is open.
- `launch_sheet_is_git` holds the footer's git answer, asked when the sheet
  opens or its project changes, not every frame.
- The sheet is not drawn and takes no keys while a progress modal shows.
