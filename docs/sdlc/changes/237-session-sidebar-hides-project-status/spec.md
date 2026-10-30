# Spec: project column groups each project's session states and folds

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. The project column draws no column-wide 「新しいセッション」 button; each
   project's `+` still opens the launch sheet for that project, and 「履歴」
   still toggles the library.
   Test: `workspace_project_selection_leaves_history_and_new_session_uses_selected_project`
   (rewritten to click the project's `+`) and
   `workspace_project_column_has_one_start_per_project`.
2. Each project header shows one count per non-empty group among 要対応, 実行中,
   指示待ち, in that order, as `<group glyph> <n>` in the group's tone; finished
   sessions are not counted. The header keeps its counts when folded.
   Test: `workspace_project_header_counts_sessions_by_status_group`
   (supersedes `workspace_project_picker_keeps_saved_order_counts_and_per_project_launch`'s
   「N 件」 assertions; that test keeps its order and launch assertions).
3. Clicking a count opens the first session of that group in that project:
   it becomes the active central tab, the project becomes selected.
   Test: `workspace_project_count_click_opens_first_session_of_its_group`.
4. An unfolded project lists its unfinished sessions (and unread finished ones,
   as 要対応) ordered 要対応 → 実行中 → 指示待ち, then by column order; clicking a
   row opens that session. An unfolded project with none reads
   「起動中のセッションはありません」.
   Test: `workspace_project_rows_order_by_status_and_open_on_click`.
5. The disclosure control folds and unfolds a project; folded projects draw no
   rows; the folded set is written to the arrangement file and read back;
   an arrangement file without the field loads with every project unfolded.
   Test: `workspace_project_fold_hides_rows_and_survives_reload`,
   `session_arrangement_without_folded_projects_still_loads`.
6. Dragging a project header still reorders projects through the existing writer.
   Test: `workspace_project_picker_drag_saves_order_in_the_real_sidebar` (kept).
7. Every central session tab draws its state glyph in its state colour before
   its title; hovering the tab shows the state's label and hint.
   Test: `workspace_session_tab_shows_its_status_glyph`.

## Behaviour

- Header: `▾/▸` (tooltip 「折りたたむ」/「展開する」), folder glyph, project
  name (truncated; hover name, path, 「ドラッグで並べ替え」), counts, `+`
  (tooltip 「セッションを開始」). Count hover: group label, then 「{n} 件」.
- Row: state glyph + session title, truncated; selected session row highlighted;
  hover: state label, then hint (e.g. 「ターミナルを開いて回答してください」).
- No projects at all: 「プロジェクトがありません」.
- Fold state write failure: notice 「並び順を保存できませんでした: …」 (existing
  string), the fold still applies in memory.
- Unchanged: project click selects its workspace; drag reorders; history toggle.

## Design

Placement verdict: `EXTEND` — `ui_workspace_projects` and the tab strip in
`src/app/screens.rs`, `SessionArrangement` in `src/ui/session_tree.rs`.

- `SessionArrangement` gains `#[serde(default)] folded: Vec<Uuid>`; the toggle
  saves through `save_session_arrangement`, pruned to existing projects like
  the other two lists. Old files load (serde default); old builds ignore the
  unknown field (no `deny_unknown_fields`). Bound: one id per project, inside
  the existing 1 MiB `SESSION_ARRANGEMENT_FILE_MAX_BYTES`.
- Per frame: one pass over `store.sessions` computing
  `session_status_kind_for` (allocation-free) and the unread overlay; a small
  `Vec` per frame of (project, group, session id); titles via
  `session_list_title` as the tabs already do. No file I/O except on the
  toggle click.
- A count or row click calls `open_waiting_session`, which already switches
  the workspace project, opens the tab, and requests terminal focus.
- Tab glyph: `session_status_kind_for` → `kind.icon()`, `palette.status(kind.tone())`;
  hint built only inside the hover closure.
- No background task, no external command, no store change.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | yes, no new role | `palette.status(tone)` and existing roles only |
| Icons | yes, no new mark | `ICON_DISCLOSURE_OPEN/CLOSED`, `SessionStatusKind::icon()`, `ICON_PROJECT`, `ICON_ADD` |
| Identifier SSOT | no | no string shared between two places |
| Durability | read `.claude/skills/durability-invariants/SKILL.md` | arrangement file is not the store; additive serde-default field, atomic writer reused; no schema bump needed |
| Subprocess safety | no | no child process |
| Documentation | checked | README does not describe the column's controls |
| Local-first | no | nothing leaves the machine; no `unsafe` |
| Budgets | yes | file stays under its existing ceiling; one id per project |

## Flagged concerns

- **Screen decided without a reply** — the layout question went unanswered for
  ten minutes; the user's request spells out each element, and the
  recommended option was adopted. The person confirms on the built app.

## Acceptance

- `make q.fast` passes; the named tests pass in `cargo test --locked`.
- In the app: counts per project, folding, row/count click switches tab,
  tabs show glyphs, one start control per project.

## Rejected alternatives

- Fold state in memory only — the user asked for a persistent list; a restart
  unfolding everything is the behaviour being complained about.
- Showing finished sessions in rows — history already lists them; rows are for
  what needs attention.
