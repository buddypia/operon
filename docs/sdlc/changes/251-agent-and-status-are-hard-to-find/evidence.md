# Verification evidence

## Failing guards before implementation or correction

- The initial real workspace render failed with `sidebar CLI choices absent` before sidebar controls were wired.
- The history no-match assertion failed because the agent-narrowed candidate vector was empty and was incorrectly treated as an empty store; the empty-store check now reads the actual store.
- The independent verifier proposed a long legacy-agent name attack. It failed in the author tree with `a legacy CLI fallback must leave its session title readable in the sidebar`; the bounded fallback corrected it and the same guard passed.

## Executed checks

- Six `sidebar_251_` tests pass, including 6,144 agent/status/waiting/ended combinations, real pointer clicks, official texture presence, accessible icon names, stale hooks, independent dimension reset, no-match and ended rows, unknown/custom/terminal names, and scrolling the 120px stacked column.
- Twenty-seven relevant existing and new tests passed before adding the final three fallback/short-column tests; the final six-test run passed after those additions.
- `CARGO_INCREMENTAL=0 make q.fast` passed after the fallback correction.
- An independent empirical verifier inspected the source and proposed three attacks from an isolated tree; its isolated linking failed with disk exhaustion. The author ran all three attacks, corrected the reproduced long-label failure, and observed all passing. No isolated pass is claimed.

## Native render

`sidebar.png` is an eframe/egui native framebuffer capture from a disposable three-session fixture using this change's actual sources, theme, fonts, textures, and `ui_terminal_workspace`. The fixture uses no real store or agent process. The official marks, wrapped controls, project grouping and readable session titles were inspected at the normal sidebar width. It is evidence of the rendering, not an installed-application verification.

## Storage and limits

Local linking repeatedly exhausted the nearly full disk. Only files created by this session were removed; no pre-existing user cache or another session's files were deleted. Both reviewer branches and its generated worktree were cleaned and checked. Ignored live-CLI restore tests were not run because restore/archive code did not change.

## Mutation evidence

All five mutations were caught by behavioral guards: disconnected sidebar controls, OR instead of AND across dimensions, ended rows still hidden, waiting label replaced with unrelated text, and vendor marks removed. The runner restored source byte-for-byte. The native screenshot fixture captured its frame successfully but did not exit on the close command; its bounded subprocess timeout terminated that disposable process, and no preview process remained.

## First CI test failure

Run 42263483921 passed fmt/clippy/bands and found three stale UI hit targets: `a_session_rows_menus_offer_stop_and_remove`, `test_change_181_hovers_say_rows_and_projects_can_be_dragged`, and `test_change_181_session_row_drag_reorders_within_its_project`. All three reproduced locally because they searched for the deliberately removed Claude/Codex word on the second row. Their click/hover/menu/order assertions remain; their hit target now uses the existing second-row separator.

The band check reports existing warnings for steering_bytes and approval_escapes, and a pre-existing largest_module_lines diagnostic: app.rs was 9,529 lines on main and is 9,573 here (+44 lines of filter state and predicates). It was already above the 9,400 diagnostic boundary; this change introduces no new concern requiring a module split.

All nine targeted tests passed after updating those hit targets, and make q.fast passed again. The final head must still pass CI before merging; waiving a duplicated state-file result does not waive this gate.
