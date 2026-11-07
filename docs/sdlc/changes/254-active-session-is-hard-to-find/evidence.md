# Verification evidence

Artifact evaluator independently approved intent, specification, existing-screen refinements, and implementation plan. The readiness script initially named a missing exact heading (Desired outcome); the heading was corrected without changing the evaluated content, and readiness returned Go.

The first regression tests were drafted while the artifact evaluator was working and before the plan commit; no production implementation was edited until after readiness and the plan commit. This departs from the strict source-file order (tests are source too) without changing the implementation decision.

Production changes reuse existing plain-terminal background work. No restore, archive, cancellation, persistence schema, process command or byte ceiling is changed; the ignored authenticated CLI restore tests are not applicable.

The unmodified implementation failed all four initial regression tests: missing selected-row fill, oversized tab text, missing strip plus, and missing split-pane plus. A test-only type name was corrected before this observed red run.

The first green-path run exposed two fixture assumptions: the existing resolver canonicalizes macOS /var paths to /private/var; egui 0.31 treats any two primary clicks within 300ms as a double click irrespective of their widget. The selection test had sent independent navigation gestures only a few synthetic frames apart, triggering the existing rename action. Inspection of input_state/mod.rs lines 970-984 and the row response CLICKED flag established the cause. The test now separates independent clicks by 24 simulated frames; the production double-click rename behavior stays intact.

The five session_254_ tests pass, including all three themes, selection through actual pointer events, fixed plus with 20 tabs, no-project and empty-project states, missing tmux, real plain-terminal process launch in the clicked split pane worktree, canonical CWD, and preservation of a stale launch-sheet path draft.

`make q.fast` passed. Existing rename, tab selection rule, split-header geometry, type-scale and control-height tests passed (7 tests). The translation sort guard caught the new Japanese 開く entry placed after 開始; moved it before 開始 in both language tables and re-ran the guard. Test annotations increased 1182 to 1187; ignore annotations remain 8.

`check-bands.sh` exits 0 with existing steering_bytes and approval_escapes warnings and the pre-existing largest_module_lines diagnosis. Main src/app.rs has 9659 lines; this extension adds 50, reaching 9709. No policy or budget was relaxed.

Independent verification reproduced F1: the original mutation run caught 7/8; moving the sidebar rule to the row's opposite edge survived. The verifier strengthened the existing selection test to anchor the line to its selected row fill, watched that mutation fail (1/1), and restored the source. Lesson 108 records the concrete guard.

Rendered Japanese fixture images exposed F2: the active tab's orange bottom rule existed as a shape at y=37 but the newly fixed child clip ended at y=32. The independent `session_254_selected_tab_rule_is_visible` reproduction failed on that exact points-versus-clip condition. The correction preserves the horizontal bounds while allowing the tab strip's natural vertical extent.

Final corrected source: `cargo test --locked session_254_` passes all six tests (0.91s), and fresh `make q.fast` passes (clippy7.19s). Final test annotations1188 versus1182 on main; ignored annotations remain8.

Independent Japanese fixture geometry covers640px and1100px, long names, split panes, dark and light. Final software-rendered `normal.png`, `narrow.png`, `split.png`, `light.png` were inspected by verifier and author: selected sidebar row and left rule are distinct, short titles use less space, the plus stays at the pane's right edge, and the orange tab underline is visible. These are egui tessellation fixture images, not installed-application captures.

Final100-frame CPU draw means in the unoptimized fixture: normal3.525ms, narrow1.010ms, split1.346ms, light0.574ms. This excludes GPU submission, terminal IO and whole-app startup; shared-machine variability prevents treating the difference between fixture runs as a regression measurement.

Final independent mutation proof: original seven caught, repaired left-edge mutation caught1/1, and new clipping mutation caught1/1. All nine guards were watched failing; source bytes restored and matched the author tree. F1 is the sweep obligation and F2 the visible-tab-rule regression. CI re-runs the complete nine-mutation sweep on the final head.

Final restored independent source re-run: six session_254_ tests passed, zero failed,0.51s; all production and test source bytes match the author checkout.

First final-head CI test run reproduced four failures (1178 passed,4failed,6ignored). Named causes: lesson108 omitted the exact admission markers **The rule:** and watched failing despite actual mutation evidence (F3); the existing selection-role count allowed two screen sites before the two newly requested selections were added (F4); a fixed400-character source window ended before the expanded conditional title ink (F5); and a first-circle-within160px helper chose the preceding tab's status icon once tabs became compact (F6). Corrected the lesson markers, enumerated all four actual selection sites, bounded the title guard by its label truncation instead of arbitrary character count, and selected the nearest preceding status mark. No product behavior changed in this correction. All four old tests are retained and recorded as reproductions.

Corrected-head local check: all10 named tests (four CI reproductions plus six UI guards) pass in14.14s; fresh make q.fast passes (clippy1.72s). All original tests and ignored annotations are retained. Production sources and verified fixture images remain unchanged.

The first run's verification job also reproduced F7: the worktree-context mutation survived on CI, where tmux is absent. The test returned before every context assertion when tool_available(tmux) was false. A local child process restricted to /usr/bin:/bin:/usr/sbin:/sbin confirms no tmux and the old guard returns success in0.01s. Removed the early return: project, persisted worktree and selected tab are always checked; actual live-process CWD is additionally checked when tmux exists, and the existing failed-launch notice is checked otherwise.

F7 correction proof: bounded runner compiled the unmutated guard and ran it with a restricted child PATH without tmux (green), then changed the pane context to project root and ran the same restricted test. The worktree assertion failed, so the previously CI-surviving mutation is caught1/1 without tmux. Source was restored by mutation_sweep. Live tmux remains additionally exercised on this Mac.

Final F7-restored source: all10 affected tests pass in10.28s; fresh make q.fast passes (clippy4.27s). Lesson109 records the dependency-independent context guard.

Final-head CI reproduced F8:1181 passed,1failed,6ignored; the no-tmux context guard asserted the global notice still contained tmux. Deterministic reproduction queued a SessionOutput error for another pane after the launch failure: shared notice became the output-read error while launch_progress.failure retained the tmux diagnosis. This was watched failing with exit101 under the restricted child PATH. The guard now checks the matching session's retained, undismissed launch diagnostic; the concurrent output-read event remains in the test. Production behavior is unchanged.

F8-stable missing-tool guard passed20/20 child processes with no tmux and an explicitly queued unrelated output failure (four concurrent children,2.48s total). The earlier F7 targeted run had stopped at the incidental shared-notice assertion; it was replaced as worktree proof by a new green-baseline/reversion run after F8: the mutated test fails at the actual path comparison (project root versus worktree, line69743),1/1 caught, source restored.

Final F8-restored check: all10 affected tests pass in11.71s; make q.fast passes (clippy4.69s). No product source changes followed the inspected clipping correction.
