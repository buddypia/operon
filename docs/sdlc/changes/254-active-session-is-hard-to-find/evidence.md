# Verification evidence

Artifact evaluator independently approved intent, specification, existing-screen refinements, and implementation plan. The readiness script initially named a missing exact heading (Desired outcome); the heading was corrected without changing the evaluated content, and readiness returned Go.

The first regression tests were drafted while the artifact evaluator was working and before the plan commit; no production implementation was edited until after readiness and the plan commit. This departs from the strict source-file order (tests are source too) without changing the implementation decision.

Production changes reuse existing plain-terminal background work. No restore, archive, cancellation, persistence schema, process command or byte ceiling is changed; the ignored authenticated CLI restore tests are not applicable.

The unmodified implementation failed all four initial regression tests: missing selected-row fill, oversized tab text, missing strip plus, and missing split-pane plus. A test-only type name was corrected before this observed red run.

The first green-path run exposed two fixture assumptions: the existing resolver canonicalizes macOS /var paths to /private/var; egui 0.31 treats any two primary clicks within 300ms as a double click irrespective of their widget. The selection test had sent independent navigation gestures only a few synthetic frames apart, triggering the existing rename action. Inspection of input_state/mod.rs lines 970-984 and the row response CLICKED flag established the cause. The test now separates independent clicks by 24 simulated frames; the production double-click rename behavior stays intact.

The five session_254_ tests pass, including all three themes, selection through actual pointer events, fixed plus with 20 tabs, no-project and empty-project states, missing tmux, real plain-terminal process launch in the clicked split pane worktree, canonical CWD, and preservation of a stale launch-sheet path draft.

`make q.fast` passed. Existing rename, tab selection rule, split-header geometry, type-scale and control-height tests passed (7 tests). The translation sort guard caught the new Japanese 開く entry placed after 開始; moved it before 開始 in both language tables and re-ran the guard. Test annotations increased 1182 to 1187; ignore annotations remain 8.

`check-bands.sh` exits 0 with existing steering_bytes and approval_escapes warnings and the pre-existing largest_module_lines diagnosis. Main src/app.rs has 9659 lines; this extension adds 50, reaching 9709. No policy or budget was relaxed.
