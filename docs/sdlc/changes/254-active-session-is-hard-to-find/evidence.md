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
