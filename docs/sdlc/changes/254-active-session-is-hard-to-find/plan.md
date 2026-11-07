# Plan: Clear current-session navigation

- **Spec**: `./spec.md`
- **Approved**: 2026-11-07
- **Status**: done

## Files that change

| File | Change |
|---|---|
| src/app/screens.rs | Sidebar selection, compact left-aligned tab labels, persistent right-edge add control. |
| src/app.rs | Launch in the clicked pane context through existing plain-terminal path. |
| src/i18n_tables.rs | Japanese action's English and Korean translations. |
| src/tests.rs | Rendered geometry, pointer navigation and real bounded terminal creation tests. |
| change records | Verification and mutation evidence. |

## Order of work

1. Commit intent, evaluate spec/screen/plan independently, pass readiness and commit plan before source edits.
2. Add failing rendered tests for selection/tab density/plus and launch context.
3. Apply UI refinements and reuse plain-terminal creation with the pane's context.
4. Run named tests, fmt/clippy, mutation sweep, and independent empirical verification.
5. Push, wait for CI, merge on main, package, clean up and verify repository state.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Plus scrolls away or overlaps close | Hidden/overlapping control on narrow strip | Rendered geometry with overflowing tabs and pointer clicks. |
| Wrong project or worktree | New terminal opens elsewhere | Stale form context and active pane tests. |
| Highlight marks a different session | Sidebar disagrees with terminal | Real pointer navigation in both directions. |
| Smaller tabs lose rename | Editor clips or takes drag | Existing session_253_ tests. |

## Proof of completion

`make q.fast`, named session_254_ and session_253_ tests, caught mutations, and CI green. Inspect rendered fixture images. Package from main; installation waits for permission to quit if Operon is running.

## Departures from the plan

Regression tests were drafted while the artifact evaluator was working, before the plan commit; production implementation still followed the independent evaluation, readiness Go and plan commit. See evidence.md. The existing plain-terminal creation body was factored into launch_empty_session_at so the plus can pass explicit context without overwriting a saved launch-sheet draft.

Independent verification found a missing horizontal-position assertion and a vertical clip hiding the tab underline. Anchored the sidebar guard to its own row, added an actual shape-versus-clip regression, and retained the tab row's natural vertical clip while still clipping horizontally before the plus. Lesson 108 covers the guard gap.

The first full CI run exposed three existing source/layout test assumptions and missing lesson admission markers. Corrected these retained guards, recorded F3-F6, ran all10 affected checks, and incremented the closing-gate attempt count to1/3.

The first CI verification job exposed an early return in the context test when tmux was missing. Split mandatory context assertions from the optional live-process probe, caught the mutation without tmux, and recorded F7 and lesson109.
