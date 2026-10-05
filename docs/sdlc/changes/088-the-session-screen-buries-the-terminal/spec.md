# Spec: a session screen that gives its width to the terminal

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. A notice set through the new `notice_briefly` path is drawn as a toast at the
   bottom-right of the window, not as the top banner, and is cleared
   `NOTICE_TOAST_SECONDS` (4) seconds after it was set. Any other notice keeps
   today's banner and stays until closed or replaced.
2. A notice that replaces a brief one is never cleared by the brief one's timer.
3. These successes use `notice_briefly`: a system action's `…: 完了`, 「ターミナルのログをコピーしました。」,
   「コミットしました。」, 「push しました。」, 「AIレビューが完了しました。」,
   「ローカルツールの検出結果を更新しました。」. Their failures do not.
4. The sidebar's status filter is one row: a 「すべて」 chip (selected when no
   filter is on; clicking it clears the filter) followed by the four groups,
   each showing its count only when it is above zero. The toggles behave as today.
5. A project is one row: icon, name, count, "+". Its full path is the name's
   hover text and no longer a line of its own.
6. A session is two lines: (a) a status dot in the status colour, the row
   title, and at the right the relative time — replaced by `···` and `✕` while
   the row is hovered or selected; (b) the agent logo, agent name, the status
   label, and the branch when the session runs in a worktree. The `···` menu,
   the context menu, rename, double-click rename, and the removal confirmation
   are unchanged.
7. The row title is `session_title`, except that a session whose name is the
   launch fallback (`<workspace> · <agent>`) and whose goal is empty shows the
   first line of its first recorded prompt turn, when there is one.
8. The session header is one line: agent logo, title (goal on hover), status
   chip, then project name and branch, then ports; at the right `···` and the
   state verb. The meta line under it is gone.
9. The red hover wash (`danger`) of the 「停止」 verb of an `Active` session is
   dropped while that session's agent activity is `AgentActivity::Idle`. Every
   other verb keeps its `danger` exactly as today — in particular 「停止中…」 and
   the resume verb, whose red marks a dangerous launch flag being carried. The
   verb's word, slot, and action are unchanged. Decided by a pure
   `verb_is_loud(&SessionVerb, Option<AgentActivity>) -> bool`.
10. The two side columns become one side panel with tabs 「ファイル」, 「会話 N」
    (N = turns, omitted at 0), 「変更 N」 (N = changed files, omitted at 0) and a
    fold button. Folded, an icon button in the header reopens it. Without a
    project, only 会話 is offered.
11. The panel is drawn when it is open and the workspace is at least 640px
    wide; its width is `clamp(28% of the width, 240, 300)` and the terminal gets
    the rest, at least 300.
12. 「最新へ」 is a button at the terminal's bottom-right and does what 「最新へ移動」
    did; the conversation tab no longer has its own header.
13. The files tab shows, top to bottom: a filter field; when the filter is
    non-empty, only matching paths (case-insensitive substring, at most
    `FILE_FILTER_RESULT_LIMIT` = 200) instead of the tree; otherwise
    「ターミナルに出たファイル」 (requirement 15, drawn only when non-empty) then the tree
    with top-level entries whose name starts with `.` folded into one
    「隠しフォルダ N 件」 row; and a one-line 「一部のファイルを省略しています」 at the
    bottom when the scan warned, with the full warning on hover.
14. The changes tab lists the project's changed files with their git letter,
    read from the existing `git_changes_cache` (never git in the draw path),
    each opening the file like the tree does. States: not loaded yet →
    「変更を読み込んでいます…」 and a load is requested; `Err` →
    「変更を読み取れませんでした: {error}」; empty → 「変更はありません」;
    worktree session → the worktree sentence in `./screen.md`. The tab's count
    is shown only when the cache holds `Ok` with at least one file.
15. 「ターミナルに出たファイル」 counts only entries of `resolved_paths` that
    resolved to a file (`Some`) under the session's root, ordered by path,
    at most `TOUCHED_FILES_LIMIT` (8). The cached list is rebuilt when a
    generation counter bumped on every insert into or prune of
    `resolved_paths`, or the session, changes.
16. Closing the toast with ✕ clears both the notice and the brief-notice record.
17. Everything reachable from the header `···` today (Terminal.app で開く,
    表示中のログをすべてコピー, 今すぐ同期) and from the session row's `···` and
    context menu stays reachable.

## Behaviour

See `./screen.md` for the layout and the table of non-happy states; its
strings are the ones listed there. Additional strings: 「すべて」,
「サイドパネルを表示」, 「サイドパネルを畳む」, 「ファイル」, 「会話」, 「変更」,
「最新へ」, 「ターミナルの最新行へ移動」, 「ファイルを絞り込む」, 「隠しフォルダ {count} 件」.

## Design

- `src/config.rs`: `NOTICE_TOAST_SECONDS`, `FILE_FILTER_RESULT_LIMIT`,
  `TOUCHED_FILES_LIMIT`.
- `src/app.rs`: `brief_notice: Option<(String, Instant)>`,
  `fn notice_briefly(&mut self, text)`, pure `brief_notice_expired(&Option<String>,
  &Option<(String, Instant)>, Instant) -> bool`; the update loop clears an
  expired brief notice and requests a repaint for the remaining time; the banner
  block skips a notice that is the brief one and a toast is drawn in an
  `egui::Area` anchored `RIGHT_BOTTOM`. Fields `show_session_inspector`,
  `session_inspector_tab: InspectorTab`, `session_file_filter: String`,
  `hovered_session_row: Option<Uuid>` replace `show_session_file_tree` and
  `show_prompt_timeline`.
- `src/agents.rs`: pure `session_list_title(&Session, Option<&str>) -> String`.
- `src/app/screens.rs`: sidebar and header redrawn as above.
- `src/ui/session_tree.rs`: `SessionColumnsLayout` becomes a two-column
  `SessionColumnsLayout { show_inspector, inspector_w, terminal_w }`;
  `ui_session_inspector` draws the tabs and dispatches to the files tab
  (the old `ui_session_file_tree` body, rearranged), the conversation list, and
  a changes list. The top-level split of the tree into visible and hidden is
  a pure `split_hidden_entries(&[FileTreeNode])`, done when the tree cache is
  rebuilt, not per frame.
- The mentioned-files list is derived from `resolved_paths` per requirement 15;
  `resolved_paths_generation: u64` is bumped wherever that map is inserted
  into, pruned, or cleared.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | yes | No new role. Dot uses `palette.status(tone)`, toast uses `raised`/`border_subtle`/`text`, existing roles only. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | yes | Uses existing constants (`ICON_MORE`, `ICON_CLOSE`, `ICON_NOTICE`, `ICON_FOLDER_OPEN`, `ICON_SEARCH`); if a fold/arrow glyph is missing it is added with its vocabulary entry. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | Toast duration and list ceilings live in `src/config.rs`; no string is shared across processes. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | No persisted field is added or changed; all new state is in-memory on `OperonApp`. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | No new child process; the existing Terminal.app action is reused unchanged. |
| Documentation — user-facing docs change in all three languages together | checked | README describes no session-screen layout detail that changes; every new string gets a row in every table in `src/i18n_tables.rs`. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | no | Drawing only. |
| Budgets — any new scan or output path states its byte and item ceiling | yes | No new scan; the filter results are capped at 200 and the mentioned files at 8, both in `src/config.rs`. |

## Flagged concerns

- **Brief notices set from many call sites** — only the six successes in
  requirement 3 move; every other `self.notice = Some(..)` is left as a banner.
  Resolved: a missed success stays a banner, which is today's behaviour.
- **Existing tests that toggle `show_session_file_tree` / `show_prompt_timeline`**
  — they are rewritten to the inspector fields with the same assertions of
  default-open and toggle; resolved in the plan.

## Acceptance

- `cargo test --locked` passes, including new tests
  `brief_notice_expires_only_for_its_own_text`,
  `session_list_title_prefers_the_first_prompt_over_the_launch_fallback`,
  `only_an_idle_stop_loses_its_red` (a dangerous resume and 「停止中…」 stay red),
  `session_columns_layout_gives_the_inspector_one_clamped_column`,
  `hidden_entries_fold_out_of_the_top_level`, and the i18n table tests.
- `cargo clippy --locked -- -D warnings` and `cargo fmt --check` are clean.
- In the installed app, the session screen matches `./screen.md`: a ⌘-click on
  a terminal path shows the toast, which is gone after about four seconds.

## Rejected alternatives

- Timer on every notice — failures would vanish unread.
- Deciding success from the text (`: 完了` suffix) — a string-shape rule is the
  thing `identifiers.md` forbids; the call site says what it is instead.
- Keeping two side columns with better defaults — the terminal still loses
  ~480px for panels that are empty most of the time.
- Persisting the first prompt as the session name — a store change for a
  display rule; the fallback title is good enough after a restart.
