# Spec: the session column's project becomes a heading

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. In `ui_workspace_projects`, a project is drawn as a group heading, not a
   card: no `card_frame` fill for the group, the name at 12.5 in
   `text_muted` (strong only when it is the selected project, still in
   `text_muted` so requirement 3 holds in that group too), and a
   `hairline` under the heading row. Pinned by
   `session_column_project_is_a_heading_not_a_card`, which reads the roles
   on the name label itself, not anywhere in the column.
2. The heading keeps every operation it has today: fold/unfold, drag to
   reorder (the name stays the grip), click to select the workspace project,
   the status count chips, and the small `+` that starts a session. Pinned by
   `session_column_project_heading_keeps_its_operations`.
3. Session rows under a heading are the strongest text in the group: the
   title at 12.5 `strong` in `text`. Pinned by the same test as requirement 1
   reading the row draw.
4. The terminal pane names its place once, small: a crumb line
   `"{project} / {title}"` at 11.5 in `text_muted` above the pane header,
   from `session_crumb`. The pane header no longer repeats the project name
   beside the title (the branch stays). An unregistered project reads
   「登録されていないプロジェクト」 in the crumb. The format is pinned by
   `session_crumb_names_project_then_title`; the draw (crumb row present at
   that role, header project label gone, in both single and split panes,
   which share this one header draw) is pinned by
   `terminal_pane_draws_the_crumb_once`.
5. Nothing else moves: the Projects page, the top navigation, the history
   grouping, folding state, column order, and the store are unchanged. No new
   string is added; the crumb reuses the project name, the session title, and
   the existing unregistered-project sentence.

## Behaviour

- Happy path: the column shows, per project, a muted heading
  (disclosure, name, status chips, count is not drawn — the chips already
  count), a hairline, then its live sessions with strong titles. The open
  terminal shows the crumb above its header.
- Empty: a project with no live session still shows its heading and
  「起動中のセッションはありません」, as today. No projects at all still
  shows 「プロジェクトがありません」.
- Folded: only the heading shows, as today.
- Long names: the heading name truncates as today; hover still shows name,
  path, and the drag hint.
- The person approved this screen in the before/after HTML rev 2
  (`.tmp/session-project-dedup/`): three changes only, listed there.

## Design

Placement verdict: `EXTEND`. The behaviour exists in
`src/app/screens.rs` (`ui_workspace_projects`, `ui_terminal_pane`); the
column grouping was added by change 163 and the status chips by 237. No new
module, type, or persisted shape.

- `ui_workspace_projects`: replace the per-project `card_frame` with a flat
  group (transparent frame, no fill, small margin). The heading row keeps its
  right-to-left chips and `+`, drawn at heading strength instead of hero
  strength. A `hairline` follows the heading. The drag target records the
  group's rect, as the card's rect was recorded. The column's top
  「プロジェクト」 heading is demoted to the same muted heading size so the
  column has one hero level: its sessions.
- `session_crumb(project_name, title) -> String`: a pure formatting helper
  beside the pane, so the claim is testable without drawing.
- `ui_terminal_pane`: draw the crumb as its own small row before the header
  row, and drop the project-name label from the non-split header (the crumb
  now says it). Split panes keep their bounded header; the crumb is drawn
  there too, so a split pane also names its project.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No new role | Only existing `Palette` roles (`text`, `text_muted`, `text_strong`, `accent_text` no longer used here) and `hairline`. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No new mark | Reuses `ICON_PROJECT` removal only; disclosure, add, and status icons unchanged. Actually: the heading drops `ICON_PROJECT`; no icon is added. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Does not apply | No identifier is introduced; the crumb joins two values already shown. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Does not apply | No store, migration, cancellation, or byte-limit surface is touched. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Does not apply | No child process or launch path changes; the `+` still calls `open_project_session_setup`. |
| Documentation — user-facing docs change in all three languages together | Does not apply | No manual or README sentence changes; no new user-facing string. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Applies | None added; drawing only. |
| Budgets — any new scan or output path states its byte and item ceiling | Does not apply | The draw path already clones the project and session lists per frame; the crumb formats two existing strings. No new scan. |

## Flagged concerns

None.

## Acceptance

- `cargo test --locked session_column_project_is_a_heading_not_a_card`,
  `cargo test --locked session_column_project_heading_keeps_its_operations`,
  `cargo test --locked session_crumb_names_project_then_title`, and
  `cargo test --locked terminal_pane_draws_the_crumb_once` pass, each
  watched failing first (one filter per invocation).
- `make q.fast` passes.
- In the running app: the session column's project names read as muted
  headings, session titles are the strongest text, and the terminal shows
  the small crumb. The Projects page is unchanged.

## Rejected alternatives

- Merge the Projects and Sessions tabs: a bigger move the person did not
  ask for; the pages still do different jobs (manage vs work).
- Remove the project name from the column entirely: two projects' sessions
  could no longer be told apart, which change 163 fixed.
- Keep the card and only shrink the name: the card fill is what makes it
  read as a destination; size alone was the pre-163 failure.
