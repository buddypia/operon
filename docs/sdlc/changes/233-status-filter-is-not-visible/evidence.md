# History and project-navigation verification

## Cause and integration

`show_status_group` set the page and chip selection but did not open the library.
The normal workspace intentionally has project navigation and central tabs, so
it did not draw the selected filter or its session rows. The history branch also
called two complete session renderers sequentially: `ui_sessions` followed by
`ui_terminal_session_tabs`. The first could consume the available height before
the second exposed the advanced actions.

The corrected history calls `ui_sessions` once. Its List mode uses the advanced
project/session renderer, with pinned sessions, status filters, rename, stop,
remove, unread and handoff actions. Grid mode keeps the existing grid. Transcript
search and external CLI history discovery/restore remain above the chosen mode.
The obsolete simple history list/card were removed; Home's existing production
row and details menu retain managed native-resume command evidence, launch
options, worktree path and dependencies.

A history title defers its single-click navigation for egui's configured double
click interval, measured with egui input time. A double click cancels the pending
navigation and starts rename. Body navigation stays immediate; explicit opens,
filter changes, menus, mode changes and leaving history cancel pending title
navigation. Selecting a project returns to central tabs, and the project-column
new-session action targets that selected project even when another project's
session tab remains open.

## Named checks

The model/tests agent reproduced all 28 failures from the original CI run before
migration and preserved their behavioral guarantees at the new render location.
Its reported 50-test pass included those 28, workspace guards and new Home,
single-render and production Home-row native-command checks. Additional focused
interaction and mutation checks are recorded in this directory's mutation/test
artifacts rather than inferred from that earlier pass.

This implementation session also ran:

- `cargo clippy --locked -- -D warnings`: passed before the final small grid and
  sidebar navigation fixes; the final gate must cover the completed source.
- `cargo test --locked an_empty_project_in_the_history`: both List and Grid
  tests passed after the bounded grid-name fix.

## Actual native rendering

Scratch-only harness directory, created by `mktemp -d`:

`/var/folders/29/1fj7pwg51hb3g8s6lyy_rldw945t4y/T/tmp.VJdxAlVCYO`

From the 233 worktree:

```sh
cargo run --locked \
  --manifest-path /var/folders/29/1fj7pwg51hb3g8s6lyy_rldw945t4y/T/tmp.VJdxAlVCYO/Cargo.toml \
  --bin operon-workspace-preview --target-dir target
```

The scratch manifest reuses the worktree dependencies. Source modules and assets
are symlinked to the 233 worktree; the harness main replaces application startup
with a synthetic scratch store/project and Exited sessions. Notifications are off.
The wrapper renders production `ui_terminal_workspace` directly, without running
`OperonApp::update`, acquiring live-store locks or operating any real session.
The window is named Operon Workspace Preview. Screenshots come from eframe's
native `ViewportCommand::Screenshot` / `Event::Screenshot` path. The wrapper
captures four states, then closes only its own viewport and exits 0.

Images:

- `/var/folders/29/1fj7pwg51hb3g8s6lyy_rldw945t4y/T/tmp.VJdxAlVCYO/history-list-1280.png`
- `/var/folders/29/1fj7pwg51hb3g8s6lyy_rldw945t4y/T/tmp.VJdxAlVCYO/history-grid-1280.png`
- `/var/folders/29/1fj7pwg51hb3g8s6lyy_rldw945t4y/T/tmp.VJdxAlVCYO/history-list-900.png`
- `/var/folders/29/1fj7pwg51hb3g8s6lyy_rldw945t4y/T/tmp.VJdxAlVCYO/history-grid-900.png`

All four images were opened with `view_image`. The left column remains projects
only. In List mode one advanced list, its filters, project counts/add controls
and session-row actions are visible below search; no second full-height list
follows it. In Grid mode the grid is visible, with its project add/open controls.
A deliberately excessive project name/path first exposed a real grid-header
width problem: the name consumed the control space. After bounding/truncating
that name, recaptured native images show both project controls inside the center
at 1280 and 900 logical pixels. The same long name truncates in the left column.

The screenshots prove representative geometry and visibility. They do not prove
pointer actions, delayed-click timing, real agent input, saving or restoration;
those are separate named interaction/model checks. The final sidebar navigation
target fixes changed click handlers after capture and require their pointer
tests, not a claim based on these static images. The harness touched no live
Operon data or account and left no preview process running.
