# Project selection correction evidence

## Reproduction and focused checks

Before implementation, all five new whole-workspace regression tests failed for
the named causes: saved order ignored, only one live tab initialized, empty
project displaying a foreign terminal, stale selected project on session focus,
and project switches sharing the same tab/layout state. After the correction,
all five passed. Two further real-pointer drag tests cover the project sidebar
and central History session list. The History guard was observed failing when
the left renderer consumed its release; giving the project navigator its own
runtime drag state made both pass. Final project-filter run:7/7 passed, including
rendered hover wording and a live count excluding finished history.

Before the last guard-only additions, workspace-filter24/24 and history-filter
26/26 passed. One startup integration error initially closed History while
initializing its workspace; using the context-only switch at startup retained
History and all26 passed. q.fast passed fmt/clippy on the completed application
source. The final verification and mutation results are recorded below after
completion, rather than inferred from the commands starting.

## Actual native rendering and project roundtrip

Independent artifacts evaluator read this change's bug/screen/plan, compiled a
scratch harness against the production source, and captured native eframe output
at1280x820logical pixels. Scratch-only synthetic store, projects and files;
notifications off; unique fake tmuxnames with existing output/resize background
keys blocked. The wrapper calls ui_terminal_workspace only, not OperonApp.update,
and opens no live store, account or real agent session.

Command from the234worktree:

```sh
cargo run --locked --manifest-path /var/folders/29/1fj7pwg51hb3g8s6lyy_rldw945t4y/T/tmp.LtuZ87vCwD/Cargo.toml --bin operon-project-preview --target-dir target
```

Run70777 exited0 and closed its own viewport. eframe ViewportCommand::Screenshot
captured three images, each opened with view_image by the evaluator; root also
viewed the A and B images:

- /var/folders/29/1fj7pwg51hb3g8s6lyy_rldw945t4y/T/tmp.LtuZ87vCwD/project-a-dirty-split.png
- /var/folders/29/1fj7pwg51hb3g8s6lyy_rldw945t4y/T/tmp.LtuZ87vCwD/project-b-scoped-live-tab.png
- /var/folders/29/1fj7pwg51hb3g8s6lyy_rldw945t4y/T/tmp.LtuZ87vCwD/project-a-restored-dirty-split.png

Fixture store order B,A and saved arrangement A,B produced A,B cards. A2件 and
B1件 reflect live counts; both project-local+ controls remain visible, the long
B name truncates before them, and selected styling follows A→B→A. A shows its
two live tabs and dirty workspace.rs in a split. B shows only its own live tab.
Returning to A restores identical WorkspaceTabs, tab order/active/focus/layout,
and the unsaved buffer. Archive/Queued IDs never appear automatically. The
harness held restored state for12native frames and logged VERIFIED.

The evaluator approved these observed project-navigation contracts. Transitions
used production select_workspace_project calls; screenshots do not prove native
pointer drags, plus-button clicks or live CLI typing. Separate whole-workspace
egui pointer tests prove launch and both drag callers. No live command was sent.

## Independent code review corrections

Round1 at staged f60ef673bf1b893b found two reachable context regressions.
The common project-launch handler reset selected_session but retained the prior
central workspace, so invoking it before a workspace frame could restore the
old project's launch target. On an already loaded terminal, its renderer could
instead restore selected_session in the release frame, preserving the launcher
but leaving a foreign workspace behind it. Both cases now have guards: the
project-local pointer test checks matching workspace and three later frames;
the common-handler test checks the next frame. Both were observed failing.
The launcher now preserves the old workspace terminal context across the
existing draft reset and switches the central project with it.

The second issue cleared automatic unread state on a hidden terminal while
History was visible. The guard creates automatic unread state (not the held
manual-unread override), renders three actual History frames, and was observed
failing. Active-session reading synchronization now runs only when History is
closed. Independent rust_review approved the corrected staged digest
 a9598ba09f0fb5f0 in round2 with no remaining findings. All9 project guards pass.
Final q.fast passed fmt/clippy.

The first bounded sweep caught7/8. Its survivor was an ineffective ordering
mutation: reversing fixture P,Q accidentally produced its desired saved Q,P
order. The corrected mutation omits sorting instead; no assertion was weakened.
The completed-source sweep adds both review regressions for10 cases total.

## Completed source verification

The corrected official bounded sweep detected10/10 cases, including omission of
saved project ordering, dead launch controls, automatic archive flooding,
discarded project layouts, stale highlighting, missing drag grip, resurrected
closed tabs, false live-count wording, launcher/workspace divergence and hidden
History read marking. Every catch was an assertion failure, not a compiler error
or timeout. The runner restored source, and git diff --exit-code against the
approved index confirmed exact restored bytes. Workspace-filter28/28 and
History-filter28/28 passed on completed source; final q.fast passed.
check-bands.sh exited0 with four warn-only metrics (steering bytes, largest
module, approval escape and subagent count); no threshold or validation changed.

The implementation is ready to push. Full CI cannot exist before that push;
independent rust_review approved waiving only pre-push execution while keeping
scripts/ci-verified.sh mandatory on the exact branch head before main merge.
Installation remains mandatory after main-checkout packaging. No guard or
requested deliverable is waived by this phase record.
