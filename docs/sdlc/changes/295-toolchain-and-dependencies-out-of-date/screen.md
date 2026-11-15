# Screen: unchanged

- **Status**: approved

Window 1440×900 points, dark theme (the app's default), same store.

No layout, string, or role changes. The screen this change agrees to is the one
`main` draws today, and the proof is a comparison, not a drawing:

| View | Before (egui 0.31) | After (egui 0.37) |
|---|---|---|
| main screen, one project, session column | to capture (plan step 4) | to capture (plan step 7) |
| terminal tab with output | | |
| diff view | | |
| knowledge tab | | |
| launch sheet | | |
| tab strip `+` menu open | | |
| a context menu (session row) open | | |
| a tooltip shown (toolbar button hover) | | |
| a popup/combo open (launch sheet project picker) | | |

Any difference visible at normal viewing distance is corrected toward the
before image; what remains is listed here with the reason, and goes to the
person, not the evaluator.

## Departures from the plan

The comparison ran at the app's default window, 1280×820 points, not 1440×900:
the harness launches the debug build fresh per view with a scratch `HOME` and
tmux socket, and the window comes up at its default size. Both columns are
the same size, the same theme, and the same seeded store (one project, `demo`).
Before is `main` at e1ecdab (egui 0.31); after is this branch's head.

| View | Result |
|---|---|
| home | Same, except the notice banner (below). |
| sessions, empty workspace | Same layout. Before clipped the session column's left edge ("ロジェクト", "いているセッション"); after draws it whole. |
| projects | Same, except the banner. |
| ⌘K palette | Same rows and chords. The search field's hint sits a few pixels further in (egui 0.37's text-field margin). Focus lands in the field and typing reaches it (checked headless); no caret in the shot because egui draws it only while the window has focus, and blinking. |
| launch sheet (home and `+`) | Same controls and order. The request box is about 4 points taller (egui 0.37's text-field margin), so the sheet is too. |
| history | Same. |
| settings | Same. |
| project context menu | Not compared: the harness's right click opens no menu on either build. |

The notice banner changed on purpose. On egui 0.37 a long path in it was drawn
outside the frame on the left: a label that starts after the mark is laid out
with a first-row indent, and 0.37 subtracts that indent again on a later row
that has no space to break at. The message now wraps in a column of its own
beside the mark, so every row starts at the same edge — which also removes 0.31's
own glitch, where the path's second row was indented and its first was not.
`a_banner_keeps_a_long_path_inside_its_frame` holds it.

Also not visible in a still: egui 0.37 raises the default animation time from
1/12 s to 0.2 s; `apply_interface_metrics` sets it back to 1/12 s so disclosures
and scroll jumps keep their pace. Text is rasterised with Skrifa and hinting now;
at normal viewing distance the glyphs in both columns read the same. Menus, tooltips and popups were exercised by the UI tests
(`session_273_plus_opens_launch_menu`, `session_283_mark_names_cli_and_state_on_hover`,
`a_truncated_session_tab_names_its_title_once_on_hover`), not by the screenshots.
