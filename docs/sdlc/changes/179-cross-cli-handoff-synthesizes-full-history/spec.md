# Spec: Replace cross-CLI full-history store synthesis with a compact Handoff bundle

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. **No destination store synthesis**: `import_full_cli_history` must not spawn
   `codex app-server`, inject turns into a Codex rollout file, write an
   Antigravity SQLite trajectory database or `history.jsonl` entry, or write a
   synthetic Claude Code JSONL file under `~/.claude/projects`. The production
   functions `import_full_history_into_codex`,
   `import_full_history_into_antigravity`, `import_full_history_into_claude`,
   and `CodexAppServer` are retired from the handoff pipeline.
2. **Local source snapshot and manifest preserved**: `import_full_cli_history`
   continues to resolve the source transcript via
   `full_history_restore_transcript`, snapshot it byte-for-byte via
   `snapshot_native_transcript` (`0700` directory, `0600` file permissions) into
   `shared-sessions/<import_id>/`, classify every record via
   `read_transcript_conversation`, and atomically write `manifest.json`
   (`schema_version: 5`, `"restore": "compact handoff bundle"`) and
   `handoff-bundle.md` beside the snapshot.
3. **Bounded, compact Handoff bundle structure**: `build_compact_handoff_bundle`
   builds a deterministic Markdown/text bundle containing:
   - Opening envelope:
     `Continue this coding task in a new CLI session.\n\nTask to continue:\n<task>\n\nLatest terminal context from the source session:`
     where `<task>` is extracted from the source conversation's user turns (and
     passed through `original_task_from_session_goal` so repeated handoffs never
     nest handoff envelopes).
   - Optional user handoff note (`Handoff note:\n<note>`) when `note.trim()` is
     non-empty.
   - Workspace and provenance block: source provider label, source conversation
     ID, project path, git branch (when present on `CliSession`), and the local
     `handoff-bundle.md` and snapshot paths.
   - Recent conversational context: at most the last
     `HANDOFF_BUNDLE_MAX_TURNS` (`8`) conversational turns (`user` and
     `assistant`), skipping raw `[Tool call:` and `[Tool result]` blocks when
     dialogue turns exist, truncating each turn at
     `HANDOFF_BUNDLE_TURN_MAX_CHARS` (`1_200` chars), and capping total recent
     context bytes at `HANDOFF_BUNDLE_MAX_BYTES` (`6_000` bytes).
4. **Fresh native destination launch**: `import_full_cli_history` prepares the
   destination launch via `command_with_managed_native_session_id(target, destination.executable().to_owned(), import_id)`
   so `"claude"` gets `claude --session-id <import_id>` (`native_session_id = Some(<import_id>)`),
   `"codex"` gets `"codex"` (`native_session_id = None`), and `"gemini"` gets
   `"agy"` (`native_session_id = None`). `finish_full_history_import` stores
   `CliHandoff` with `full_history: false`, stores `Session.goal` via
   `goal_with_native_tracking_token(target, imported.handoff_bundle, id)` when
   `handoff_bundle` is non-empty, sets `Session.native_session_path` to `None`
   when launching a fresh handoff bundle session (so later resume/handoff
   resolves the destination CLI's own real transcript rather than
   `handoff-bundle.md`), and starts the session in `tmux` via
   `self.start_session(id)`.
5. **Envelope peeling and UI copy**: `original_task_from_session_goal` and
   `session_display_goal` peel the compact Handoff bundle back to `<task>` even
   across multiple consecutive handoffs. `ui_cli_sessions`,
   `render_restore_options`, `restore_progress_title`, and
   `restore_progress_footer` present the compact Handoff bundle strings in
   `./screen.md`, with matching English and Korean rows in `src/i18n_tables.rs`
   and updated documentation in `README.md`, `README.ja.md`, and `README.ko.md`.

## Behaviour

- **Happy path (from `履歴` or session row context menu)**:
  1. In `履歴` (`ui_cli_sessions`), the user optionally enters a note in
     `引き継ぎメモ（任意）`, picks a target in `引き継ぎ先`, and clicks
     `[ ⟲ Handoff バンドルで引き継ぐ ]`; or in the session list right-click /
     `···` menu (`render_restore_options`), clicks `{p0} へ Handoff`.
  2. `handoff_cli_session` opens the 520px progress modal (`ui_restore_modal`)
     with title `{p0} の会話を {p1} に Handoff しています…` (or
     `{p0} の会話を同じ CLI に複製しています…` when source == destination) and
     footer `ローカル履歴から Handoff バンドルを組み立てています · 経過 {elapsed_seconds} 秒`.
  3. On background completion, `finish_full_history_import` records a
     `CliHandoff` (`full_history: false`), creates the destination `Session`
     (`SessionOrigin::Restored { from: source.provider }`), sets notice
     `{source} の会話 {source_conversation} から Handoff バンドル（直近 {records} ターン）を作成し、{destination} セッションを起動しています。{breakdown}`,
     and calls `self.start_session(id)`, which opens the destination CLI in
     `tmux` with the compact Handoff bundle as its initial interactive prompt.
- **Empty or unreadable source transcript**:
  - Missing file returns `{p0} には、会話 {native_id}（{p2}）の読み取り可能な履歴がありません。`
    before spawning a background task.
  - A transcript with zero conversational turns returns
    `選択した元履歴には、復元できる記録がありません。`, closes the progress modal, and
    shows `会話全履歴を復元できませんでした。元の CLI セッションは変更されていません: {error}`
    in the notice bar without creating a session or `CliHandoff`.
- **Too-large source transcript**:
  - `read_transcript_conversation` remains bounded by
    `TRANSCRIPT_FILE_MAX_BYTES` (8 MiB) and `TRANSCRIPT_FILE_LINE_LIMIT`
    (20,000 lines).
  - `build_compact_handoff_bundle` bounds the inline prompt context to the most
    recent `HANDOFF_BUNDLE_MAX_TURNS` (`8`) dialogue turns,
    `HANDOFF_BUNDLE_TURN_MAX_CHARS` (`1_200`) chars per turn, and
    `HANDOFF_BUNDLE_MAX_BYTES` (`6_000`) bytes total, noting how many earlier
    turns were omitted and pointing to the local `handoff-bundle.md` and
    snapshot path.
- **Interrupted / dismissed modal**:
  - Pressing `Escape` or clicking `バックグラウンドで続ける` dismisses the modal
    (`self.restore_progress = None`) while the background task finishes and
    surfaces its result in `self.notice`.
- **Denied / missing tools or concurrent worktree mutation**:
  - If `self.worktree_mutation_running(project.id)` is true, refuses with
    `worktree 操作の完了を待ってください。`.
  - If `tmux` or the destination CLI executable is missing from `PATH`, refuses
    with `{destination} へ会話全履歴を復元するには {tools} が PATH 上に必要です。`.
  - While a `BackgroundKey::FullHistoryImport` is in flight, the session menu's
    `{p0} へ Handoff` buttons are disabled (`!is_importing`).

## Design

### 1. Constants (`src/config.rs`)

Add three constants to `src/config.rs`:
- `pub(crate) const HANDOFF_BUNDLE_MAX_TURNS: usize = 8;`
- `pub(crate) const HANDOFF_BUNDLE_TURN_MAX_CHARS: usize = 1_200;`
- `pub(crate) const HANDOFF_BUNDLE_MAX_BYTES: usize = 6_000;`
- `pub(crate) const HANDOFF_BUNDLE_FILE_NAME: &str = "handoff-bundle.md";`

### 2. Data carrier (`src/cli.rs`)

Extend `ImportedNativeSession` with `#[derive(Default)]` compatibility on the new
field:
- `pub(crate) handoff_bundle: String` — the compact Handoff bundle prompt text
  to pass to the destination CLI on launch (empty only when a synthetic test
  constructs `ImportedNativeSession` with an explicit `resume` command).

### 3. Compact Handoff bundle builder (`src/history.rs`)

- Update `import_full_cli_history`:
  - Add `pub(crate) fn import_full_cli_history_with_note(project: &Project, source: &CliSession, target: &str, note: &str, archive_root: &Path, import_id: Uuid) -> UiResult<ImportedNativeSession>`,
    and keep `import_full_cli_history(project, source, target, archive_root, import_id)`
    delegating to `import_full_cli_history_with_note(..., "", ...)` so existing
    callers and tests keep their five-argument signature.
  - Snapshot the source transcript via `snapshot_native_transcript` and parse
    `(turns, classification) = read_transcript_conversation(&archive_path, source.provider)?`.
  - Call `build_compact_handoff_bundle(project, source, destination, note, &turns, &archive_path, &bundle_path)`
    to construct `(handoff_bundle, included_turns)`.
  - Write `bundle_path` (`archive_dir.join(HANDOFF_BUNDLE_FILE_NAME)`) and
    `manifest.json` (`"restore": "compact handoff bundle"`) atomically via
    `write_file_atomically`.
  - Build `(agent_command, native_session_id)` via
    `command_with_managed_native_session_id(destination.agent(), destination.executable().to_owned(), import_id)`.
  - Return `ImportedNativeSession` with `managed_session_id: import_id`,
    `provider: destination`, `native_session_id`,
    `native_session_path: bundle_path`, `archive_path`,
    `handoff_bundle`, `imported_records: included_turns`, `classification`, and
    `agent_command`.
- Implement `pub(crate) fn build_compact_handoff_bundle(...) -> (String, usize)`:
  - Determines the task (`<task>`): takes the first user turn (or
    `source.last_user_message`), strips any nested handoff envelope via
    `original_task_from_session_goal`, and if the latest user turn differs from
    the opening user turn, includes the latest user request under
    `Latest user request:` while keeping `<task>` in `Task to continue:` so
    `original_task_from_session_goal` always recovers the root task.
  - Filters `turns` for conversational turns (excluding turns that start with
    `"[Tool call:"` or `"[Tool result]"` unless no other turns exist), selects up
    to the last `HANDOFF_BUNDLE_MAX_TURNS` turns within
    `HANDOFF_BUNDLE_MAX_BYTES`, truncating each turn at
    `HANDOFF_BUNDLE_TURN_MAX_CHARS`, and formats the bundle with the envelope:
    ```text
    Continue this coding task in a new CLI session.

    Task to continue:
    <task>

    Latest terminal context from the source session:
    - Source: <source_provider> (<source_native_id>)
    - Workspace: <project_path> [branch if present]
    - Local snapshot: <archive_path>
    [Handoff note if non-empty]
    ---
    [User / Assistant recent turns]
    ---
    ```
- Retire `import_full_history_into_codex`,
  `import_full_history_into_antigravity`, `import_full_history_into_claude`,
  `CodexAppServer`, and `delete_codex_thread` from production code; gate
  remaining low-level format unit-test helpers with `#[cfg(test)]` so zero dead
  code remains in the non-test binary.

### 4. App wiring and UI (`src/app.rs`, `src/app/screens.rs`, `src/agents.rs`, `src/ui/widgets.rs`, `src/i18n_tables.rs`)

- `handoff_cli_session` (`src/app.rs`) calls
  `import_full_cli_history_with_note(&project, &source, &target, &note, &archive_root, import_id)`.
- `finish_full_history_import` (`src/app.rs`):
  - Records `CliHandoff` with `full_history: imported.handoff_bundle.is_empty()`.
  - When `!imported.handoff_bundle.is_empty()`, sets `Session.goal` to
    `goal_with_native_tracking_token(target, imported.handoff_bundle.clone(), id)`
    and `Session.native_session_path` to `None` (leaving `Some(imported.native_session_path)`
    only when `handoff_bundle` is empty for synthetic resume-command test
    fixtures).
  - Sets `self.notice` to report the compact Handoff bundle creation and starts
    the session via `self.start_session(id)`.
- Update UI labels in `src/app/screens.rs`, `src/agents.rs`, and
  `src/ui/widgets.rs` per `./screen.md`, and add sorted `EN_TABLE` and
  `KO_TABLE` entries in `src/i18n_tables.rs`.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | Does not apply | No new colour role or palette entry is added; existing `text`, `text_muted`, `text_faint`, `text_strong`, and `accent` roles are reused. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | Does not apply | Reuses existing `ICON_RESTORE`, `ICON_RESUME`, `ICON_SEARCH`, `ICON_OPEN_EXTERNAL`, and `ICON_BRANCH` constants. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Applies | `HANDOFF_BUNDLE_MAX_TURNS`, `HANDOFF_BUNDLE_TURN_MAX_CHARS`, `HANDOFF_BUNDLE_MAX_BYTES`, and `HANDOFF_BUNDLE_FILE_NAME` live in `src/config.rs`; envelope markers match `original_task_from_session_goal` in `src/agents.rs` and round-trip is tested in `src/tests.rs`. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Applies | `Store`, `Session`, and `CliHandoff` schemas in `src/models.rs` are unchanged (`full_history: bool` already exists with `#[serde(default)]`); `handoff-bundle.md` and `manifest.json` are written through `write_file_atomically` beside `snapshot_native_transcript` (`0700`/`0600`). |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Applies | Retires `CodexAppServer` (`codex app-server --stdio` child spawn) from `src/history.rs`; destination sessions launch through `command_with_managed_native_session_id`, `agent_shell_command` (`shell_quote`), and `launch_stored_session` in `src/agents.rs` and `src/tmux.rs`. |
| Documentation — user-facing docs change in all three languages together | Applies | `README.md`, `README.ja.md`, and `README.ko.md` are updated together to describe the compact Handoff bundle instead of full-history store synthesis, and all new UI strings are added to `EN_TABLE` and `KO_TABLE` in `src/i18n_tables.rs`. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Applies | All transcript reading, bundle generation, and archiving happen on local files with zero network calls and zero new `unsafe` blocks. |
| Budgets — any new scan or output path states its byte and item ceiling | Applies | Source transcript reads stay bounded by `TRANSCRIPT_FILE_MAX_BYTES` (8 MiB) and `TRANSCRIPT_FILE_LINE_LIMIT` (20,000 lines); the compact Handoff bundle is bounded by `HANDOFF_BUNDLE_MAX_TURNS` (8 turns), `HANDOFF_BUNDLE_TURN_MAX_CHARS` (1,200 chars/turn), and `HANDOFF_BUNDLE_MAX_BYTES` (6,000 bytes). |

## Flagged concerns

- **Preserving `original_task_from_session_goal` and `session_title` across repeated handoffs** — If the compact Handoff bundle placed variable metadata before `Task to continue:\n` or omitted the `\n\nLatest terminal context` delimiter, `original_task_from_session_goal` in `src/agents.rs` would fail to strip the envelope, causing session titles to display the raw bundle header and causing subsequent handoffs to nest bundles. Resolved by formatting every bundle with the exact `Task to continue:\n<task>\n\nLatest terminal context` envelope that `original_task_from_session_goal` already parses, and verifying multi-hop handoff peeling in `src/tests.rs`.
- **Avoiding mistaking `handoff-bundle.md` for a native CLI transcript on later resume or handoff** — Previously `finish_full_history_import` set `Session.native_session_path = Some(imported.native_session_path)` because `imported.native_session_path` was a synthesized destination store file. With a compact Handoff bundle, the destination CLI creates its own transcript only after `start_session` runs. Resolved by setting `Session.native_session_path = None` when `!imported.handoff_bundle.is_empty()` so `resume_managed_native_session`, `handoff_managed_session`, and `request_native_session_resolution` resolve the real transcript written by the destination CLI.

## Acceptance

- `cargo fmt --check` and `cargo clippy --locked -- -D warnings` pass with no output.
- `cargo test --locked` passes, including new tests:
  - `compact_handoff_bundle_carries_task_note_and_bounded_recent_turns_without_tool_spam`
  - `import_full_cli_history_writes_compact_bundle_without_synthesizing_destination_store`
  - `finish_full_history_import_launches_fresh_session_with_unwrapped_display_goal`
- `bash scripts/check-bands.sh` reports all metrics within their bands.
- In the running app (`履歴` → `CLI セッション` and session context menu), cross-CLI
  handoff shows the compact Handoff bundle labels from `./screen.md` and starts
  the target CLI with the compact bundle prompt rather than writing into the
  destination CLI's store before launch.

## Rejected alternatives

- **Keep full-history store injection alongside the compact Handoff bundle as a toggle**: Rejected because the user explicitly directed abolishing the full conversation history import feature (`現状「会話履歴の完全インポート機能」は廃止で`) to eliminate the maintenance and operational burden of reverse-engineering three private CLI store formats.
- **Pass only a file path `@handoff-bundle.md` in the initial prompt with no inline summary**: Rejected because not all CLIs expand `@path` references automatically without a tool call, and including the bounded task + recent turns inline (while also naming the local snapshot/bundle path) lets the receiving CLI begin immediately with zero extra file-read round-trips for normal handoffs.
