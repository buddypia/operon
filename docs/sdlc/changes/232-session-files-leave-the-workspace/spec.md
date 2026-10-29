# Spec: Keep sessions and files in the central workspace

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Opening a file from a session keeps Page::Sessions and adds/selects a document
   tab without discarding its buffer; reopening selects the existing tab.
   Proof: `session_file_opens_in_central_workspace` and `workspace_tabs_preserve_mixed_tabs_and_document_identity`.
2. Sessions use central tabs, switching preserves other open sessions and files,
   and the session page left sidebar lists projects rather than session rows.
   Proof: `workspace_tabs_preserve_mixed_tabs_and_document_identity` plus rendered inspection.
3. Both tab kinds can move between groups or create left/right and top/bottom
   splits, initially half-size, with draggable dividers and accessible tabs.
   Proof: `workspace_tabs_split_both_axes_and_resize`,
   `workspace_tabs_reorder_and_retain_active_identity`,
   `workspace_tabs_refuse_invalid_or_too_small_drop_atomically` and rendered drag inspection.
4. Closing a session tab never stops its process; closing the last tab of a
   group collapses that group and keeps a valid focus. Dirty files retain the
   existing close refusal and save errors. Proof: `workspace_tabs_move_merge_and_close_collapse_groups`
   and `workspace_dirty_document_close_is_refused`.
5. Only visible session tabs count as on-screen for activity/unread and terminal
   polling. Stale sessions/documents are removed from groups without invalid focus.
   Proof: `workspace_tabs_reorder_and_retain_active_identity`.

## Behaviour

The left column is headed プロジェクト. Choosing a project makes its sessions
available in central tabs; the right session inspector continues to provide
ファイル and the other existing views. A file click adds its tab to the focused
group and leaves the session page visible. Tab selection changes group content;
tab dragging reorders tabs, moves them to another group, or splits at an edge.
Tab close uses タブを閉じる. Existing drop labels 左に分割, 右に分割, 上に分割,
下に分割 and これ以上分割できません describe landing and size refusal.

An empty workspace offers セッションを選択. Loading files retain 読み込んでいます…;
read denial, excessive size, and save errors use the existing editor states.
Closing dirty files retains 未保存の変更があります。保存するか、ディスクから読み直してから閉じてください。
Escape or dropping outside a group cancels movement. Long tab labels truncate
with full session/file identity available on hover; overflow tabs remain scrollable.
Closing every tab leaves an empty workspace rather than silently reopening one.

## Design

Placement: EXTEND. `PaneLayout` in `src/ui/session_tree.rs` already implements
recursive horizontal/vertical pane geometry, edge drop targets, splitter shares,
minimum dimensions and PANE_MAX. `open_document_at` in `src/app.rs` and
`ui_editor` in `src/app/screens.rs` already implement buffers, async file reads,
preview, saving and dirty-close refusal. Reuse those behaviors rather than
inventing another editor or terminal implementation.

Extend the existing session-tree concern with a WorkspaceTabs model. Each group
has a stable UUID, ordered WorkspaceTab values (Session UUID or DocumentId), and
an active tab. Reuse PaneLayout for group geometry. App state tracks focus and
drag payload; the workspace draws one tab strip and active content per group.
Use existing terminal and editor renderers, suppressing their redundant outer
navigation strips in workspace groups. Left project selection opens sessions
centrally; navigation never changes pages for an internal file click.

No persisted shape, schema, cancellation, restore/archive, new subprocess, new
scan, or new output budget changes. File reads/saves keep their existing bounded
background tasks and exact DocumentId root/worktree identity. Runtime-only group
state does not affect the durable session arrangement.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | Yes | Reuse Palette roles and DESIGN.md surfaces; no new literals or roles. |
| Icons | Yes | Reuse ICON_CLOSE, file and agent glyphs in ICON_VOCABULARY. |
| Identifier SSOT | Yes | Reuse DocumentId identity and existing constants for pane limits. |
| Durability | No shape change | Workspace state is in memory; existing reads/saves and session store remain authoritative. |
| Subprocess safety | No new child | Existing terminal renderer and background operations remain in use. |
| Documentation | Yes if user docs move | Update all three README language versions together if describing navigation. |
| Local-first | Yes | No telemetry, cloud, new dependency, or unsafe. |
| Budgets | Existing budgets | Reuse pane count/minimums and bounded file operations; no new scan path. |

## Flagged concerns

None unresolved. The user explicitly chose the changed layout; exact spacing,
model structure and focus defaults are reversible implementation decisions.

## Acceptance

- `make q.fast` passes and test/ignore counts do not regress against main.
- Named workspace tests in Requirements pass; each new guard is observed failing
  before implementation or under an assertion-backed bounded mutation.
- Inspect a running workspace with two session tabs and a document tab, exercise
  switching, reopening, dirty-close refusal, reordering, moving between groups,
  left/right split, top/bottom split, and divider resizing.
- Verify project-only left sidebar and file opens retaining the session page.
- `bash scripts/check-bands.sh` passes; review verdict is fresh for landing.
- After CI and merge, package/install and verify installed build identity.

## Rejected alternatives

- Keep opening the project page: preserves the navigation interruption.
- Add files only to a fixed side preview: cannot provide mixed central tab groups.
- Generalize persisted session arrangement: unnecessary migration for view state.
- New editor implementation: duplicates dirty-buffer and file safety behavior.
