# Spec: CLI Launch Progress Modal and Folder Visibility

- **Intent**: ./intent.md
- **Status**: approved

## Requirements

1. When starting an agent session or terminal session, `OperonApp` must record an active `LaunchProgress` state containing session ID, target project name, target folder path, agent name, command preview, goal text, start instant, and status phase.
2. An active `LaunchProgress` must render as an elevated modal (`ui_launch_modal`) with a background scrim, an animated spinner, the target folder's friendly path, the selected AI agent and command summary, and the elapsed seconds.
3. The launch modal must provide a "run in background" dismiss action (`バックグラウンドで続ける`), allowing the user to close the modal while the background launch continues undisturbed.
4. When a session launch succeeds, the modal must automatically close and transition the user to the active terminal workspace (`Page::Sessions`) with the newly launched session selected.
5. When a session launch fails, the modal must enter an error state displaying the failure reason in clear, human-readable Japanese, providing direct action buttons to retry, open settings, or dismiss.
6. The project overview screen's workspace folder section must visually emphasize the active folder with path, branch badge, and a button to reveal the folder in macOS Finder.
7. All user-facing strings must use `tr()` or `tf!()` and have corresponding entries in `src/i18n_tables.rs` in alphabetical sorted order.

## Behaviour

- **Starting a launch**:
  - The user clicks the launch button on the overview page or starts an empty terminal.
  - The modal pops up immediately in the center of the screen with a semi-transparent scrim behind it.
  - The modal displays:
    - Title: 「AI セッションを起動中…」 (or 「ターミナルを起動中…」)
    - Target directory: folder icon + friendly folder path (e.g. `~/dev/playground/operon`)
    - Agent & options: agent name badge (Codex / Claude Code / Antigravity / Custom) and model or command
    - Request summary: preview of user's task prompt if entered
    - Progress indicator: animated spinner and text 「tmux セッションを準備しています…」
    - Elapsed time: 「経過 X 秒」
    - Dismiss button: 「バックグラウンドで続ける」
- **Background continuation**:
  - Clicking 「バックグラウンドで続ける」 or pressing Escape dismisses the modal overlay without killing the launch thread.
  - When the background launch finishes, the application smoothly switches to the terminal page.
- **On Failure**:
  - If the background launch fails (e.g. missing executable, tmux error, invalid folder), the modal changes tone to warning/danger with an attention mark.
  - Title becomes 「セッションの起動に失敗しました」.
  - The exact failure reason is displayed clearly with explanatory advice (e.g. tmux missing or PATH missing).
  - Action buttons: 「設定を開く」 (if a tool was missing), 「再試行」 (to re-attempt), and 「閉じる」.
- **Folder Visibility on Overview**:
  - In the Overview screen's detail section, the folder row shows an explicit indicator whether it is the project root or a Git worktree, with a 「Finder で表示」 button to inspect the targeted folder on macOS before launching.

## Design

- `src/app.rs`:
  - Define `LaunchProgress` struct:
    - `session_id: Uuid`
    - `project_name: String`
    - `worktree_path: PathBuf`
    - `agent: String`
    - `agent_command: String`
    - `goal: String`
    - `started_at: Instant`
    - `failure: Option<String>`
  - Add `launch_progress: Option<LaunchProgress>` to `OperonApp`.
  - In `launch_session()` and `launch_empty_session()`, initialize `self.launch_progress = Some(...)`.
  - In `start_session(session_id)`, ensure `self.launch_progress` is updated if retrying or resuming.
  - In `BackgroundResult::SessionStarted` and `BackgroundResult::EmptySessionStarted`:
    - On success: clear `self.launch_progress = None`, switch to `Page::Sessions`.
    - On error: set `launch_progress.failure = Some(error.clone())` so the modal displays the diagnostic error card rather than disappearing.
  - Implement `ui_launch_modal(&mut self, ctx: &egui::Context)` following the proven structure of `ui_restore_modal`:
    - Draw scrim layer on `Order::Middle`.
    - Draw elevated modal window on `Order::Foreground`.
    - Render spinner, title, target folder with icon, agent badge, command, elapsed seconds, and actions.
- `src/ui/widgets.rs`:
  - Helper functions for launch progress display: `launch_progress_header`, `launch_progress_footer`.
- `src/app/screens.rs`:
  - Enhance the overview folder selector UI with folder status and a Finder reveal button.
- `src/i18n_tables.rs`:
  - Add Japanese string IDs to `EN_TABLE` and `KO_TABLE` in strictly sorted order.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | Reuses existing palette tokens (`raised`, `scrim`, `accent`, `text_strong`, `text_muted`, `warning`, `danger`). |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | Uses existing icons from `src/glyphs.rs` (`ICON_PROJECT`, `ICON_TREE_WORKTREE`, `ICON_OPEN_EXTERNAL`, `ICON_ATTENTION`, `ICON_RETRY`). |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | No shared wire configuration or cross-process identifier constants introduced. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | `LaunchProgress` is ephemeral UI state and does not alter the persisted store schema. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | Launches continue using existing `launch_stored_session` and `start_empty_tmux_session`. |
| Documentation — user-facing docs change in all three languages together | No | UI text only; no Markdown documentation modifications required. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Runs entirely on-device with zero network requests or unsafe blocks. |
| Budgets — any new scan or output path states its byte and item ceiling | No | No new directory traversal or unbounded byte buffers added. |

## Flagged concerns

- **Modal interference during rapid batch starts** — If multiple sessions are started in fast succession or via scripting, having a modal could feel intrusive if not dismissable. The modal solves this by allowing one-click background dismissal, Escape key dismissal, and only tracking the most recently initiated active launch.
- **Preserving terminal transition upon background completion** — If the user dismisses the modal to continue browsing, they might be startled if the page suddenly switches to terminal while they are reading files. The completion handler checks whether the modal was actively dismissed; if so, it posts a success notice and keeps the current page, only switching automatically if the user was still waiting on the modal.

## Acceptance

- `cargo test --locked` passes, including new tests:
  - `the_launch_modal_names_what_it_is_doing`
  - `the_launch_modal_shows_folder_and_agent_details`
  - `the_launch_modal_draws_error_state_on_failure`
  - `translation table verification tests for new message IDs`
- Launching an AI agent from the Overview screen displays the modal with accurate folder path, agent name, elapsed time, and spinner.
- Closing the modal continues background execution without blocking.
- Intentionally failing a launch (e.g. unconfigured agent) shows the clear failure card with actionable buttons.

## Rejected alternatives

- Toast banner only: Rejected because a small single-line notice at the bottom edge lacks visibility and cannot display multi-attribute progress (folder, agent, command, elapsed time).
- Separate terminal window: Rejected because Operon is designed to manage agent interactions inside its own embedded terminal workspace.
- Blocking modal without background option: Rejected because users should never be trapped waiting for a background CLI startup if they want to navigate elsewhere.
