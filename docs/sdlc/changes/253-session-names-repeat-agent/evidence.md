# Verification evidence

- Before implementation, `session_253_rename_from_sidebar_and_tab` failed on the sidebar double click: `renaming_session` remained None.
- Final author tree: `cargo test --locked session_253_` passes all three tests, including numbering in three languages, persisted rewrite, repeat startup, failed-write rollback, pointer entry from sidebar/tab using both gestures, Enter, Escape and blur.
- Existing checks passed: rename tests, translation sort/placeholders, session-list titles, Japanese literals, sidebar 251 marks/filters and live state in tabs. `make q.fast` passes.
- Independent empirical verifier reran all six recorded obligations, including the original eight-mutation sweep: all passed in 213 seconds with no surviving mutation. The final editor refactor only removes a per-frame clone and returns the same save/cancel decision; its updated Escape mutation and R3 are checked separately.
- Tests grew from 1,179 to 1,182; ignored annotations remain eight. No restore or shared-session archive path changed, so live CLI restore tests were not needed.
- Band reading: steering_bytes and approval_escapes warn; largest_module_lines diagnoses a pre-existing breach (9,596 lines on main, about 63 added here). No new scan, subprocess or IO in drawing.

Final-shape R3 and the updated Escape mutation passed (1/1 caught, source restored). Independent attacks also passed: queued real Antigravity launch with a request gets a numbered saved name, explicit launch name remains; empty inline names preserve old names in both locations; existing English Session 1 reserves its number, and migration works with only a project path. The real queued-launch case is retained in R1 instead of a source-text assertion.

`numbered.png`, `sidebar-edit.png`, `tab-edit.png`: actual egui tessellation and bundled Japanese fonts/vendor textures, rasterized at 2x by a scratch software renderer. These are disposable fixture views, not installed-application captures. Inspected all three: names and agent/status marks fit, the sidebar editor stays inside its column, the tab editor takes its label slot, and only one editor appears. Fixture CPU rendering averaged 0.375 ms/frame across 100 frames; this excludes GPU submission and is not a whole-application performance benchmark.

The retained real queued-launch guard was watched failing after replacing the automatic numbered name with the workspace name: new-name-repeats-project caught, 1/1; all source restored. Verifier worktree and local branch were removed and verified absent.
