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

## Central divider acceptance probe

A bounded scratch probe ran the production workspace through egui pointer
press, move and release for both central split dividers, without calling the
model setter directly. `completion_probe_workspace_dividers_resize_with_real_pointer_input`
passed 1/1: horizontal groups changed 500/500 to 650/350 pixels; vertical groups
changed 407/407 to 529.1/284.9 pixels. Both shares became 0.6500, neighbors stayed
bounded, above their minimums and non-overlapping. The temporary probe test was
removed by restoring exact original src/tests.rs bytes; git diff --exit-code
confirmed no test source change and HEAD stayed 0d51d9b96da3db774a8dfa967f3cd92434899923.

Scratch runner, original-byte backup and result.log are at
`/var/folders/29/1fj7pwg51hb3g8s6lyy_rldw945t4y/T/operon-divider-proof-ev1wkkso/`.
This proves egui interaction wiring, not live agent typing or installed-app identity.

## CI follow-up on unchanged application behavior

Run40903014789 reached the full suite on image20261021.0445.8:
1074 passed,2 failed,6 ignored. The two failures were unused EN/KO translation
rows after removing the old simple list and lesson103 missing the factual
`watched failing` wording. Both named guards were reproduced red, corrected
minimally, and passed; q.fast passed. Commit fb9a652 records those corrections.

The checks job instead used image20261026.0455.2, restored generated cache
9420855145 (`refs/heads/main`, key
`v0-rust-checks-Darwin-arm64-489ab134-a449688a`), and reused native CMake paths
for OpenSSL@4/4.0.3 while that image supplies4.1.0. CMake then failed to find
OpenSSL::SSL. Source code and lockfiles did not cause this environment mismatch.
Read-only inventory and the cache action's exact/fallback prefix semantics
identified exactly one applicable checks cache. That single reproducible build
artifact was invalidated; a REST query then confirmed zero matching main-cache
candidates. The other ten cache entries were kept, and no workflow or validation
check was changed. Run40907460785 tests corrected source fb9a652 with fresh checks
build state. The proposed image-key workflow change was not made; its unused
worktree has no unique commits and remains for normal post-merge cleanup.

CI40907460785 completed successfully on fb9a652. The pre-merge check then
refused pending installation, even though ship must run from main after merge.
The source records now explicitly defer only pre-merge release execution and
retain installed-app as a mandatory pending human contract. The goal remains
incomplete until packaging, authorized replacement, launch and installed-build
verification really succeed. No source check or workflow was removed.

## Installed application verification

The user explicitly approved quitting, replacing and restarting the running
Operon (「終了して更新・再起動する」). From main at
3eadf483c56d2409754dd6992b80b607e71edc15, package-macos.sh completed fmt, clippy,
release compilation, signing and plist checks. codesign --verify --deep --strict
also passed. After a clean quit (both Operon and operon process lookups empty),
replace-macos-bundle.sh installed /Applications/Operon.app and open launched it.
check-installed-build.sh reports CURRENT: stamp 3eadf48 includes latest
release source fb9a652. The installed native window reached Home. Clicking
Sessions, Files and CONTRIBUTING.md opened that file as a central document tab
beside the retained session tab, with Sessions still active and projects in the
left column. No file was edited and no command was sent to a live agent.

Native automation confirmed that installed file-opening path and rendered state.
Its drag attempts did not establish a split, so they are not counted as native
drag evidence. Both split orientations are covered by the rendered native
harness states and production egui pointer tests; the divider probe above proves
resizing through actual egui input on both axes.

The local native CMake configuration initially referenced OpenSSL4.0.3 after the
machine moved to4.1.0. Only three generated CMakeCache.txt files were moved
unchanged into a mktemp backup directory; source and compiled objects were kept.
Regeneration made the same main-checkout packaging command succeed. Backups and
their verified hashes are in
/var/folders/29/1fj7pwg51hb3g8s6lyy_rldw945t4y/T/tmp.NBFAgSFgsr/.
The installed app remains CURRENT. The obsolete dist/Operon.previous.app was
moved to Trash, and no rollback bundle was requested.
