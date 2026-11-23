# Plan: real screens, and more to read and do, on the product map

- **Spec**: `./spec.md`
- **Approved**: 2026-11-24
- **Status**: approved

Interrogated before acceptance. What breaks: nothing in the application — no
`src/` file but `src/tests.rs` changes. What can break is the page (a script
error blanks a view, which `change_map_page_script_parses` and the node logic
test catch only in part, so the browser item is real work), the existing
change-map tests when the page moves out of the Python string, and the
capture on this machine (a press that the window ignores, a screen that needs
a running agent). What was assumed: the window tool's id digits are the
window number (measured), the palette's 「プロジェクトを開く」 opens the
selected project's overview once a terminal runs in the demo project (to be
seen on the first capture; if not, those tabs get `noshot` and the departure is
recorded), and JPEG at quality 72 keeps text readable at scale 2. The
alternative for the page was to keep it in the Python string; it is past
1,200 lines and will roughly double, so it moves to files the editor can read.
What proves it done: the twelve tests, a sweep that watches each go red, the
pictures taken here, and the page walked in headless Chrome.

## Files that change

| File | Change |
|---|---|
| `scripts/capture-screens.py` | new: plan, launch per screen, press, tree with bounds, picture, entry, stop |
| `docs/features/flow.yaml` | `reach` / `noshot` per screen, `press` per way where the words differ; header documents both |
| `docs/images/screens/*.jpg`, `screens.json` | new: the pictures and their entries |
| `scripts/change-map.py` | read `screens.json` with its ceilings; join to the flow; areas; overview; feature, domain and per-screen term links; inline the page files and pictures |
| `scripts/change-map/page.html`, `page.css`, `page.js` | new: the page, moved out of the Python string and grown — router, views, 画面 walk, 全体図, search, keys, theme |
| `src/tests.rs` | the twelve tests and their fixtures; existing change-map tests follow the page's move where they read its text |
| `.claude/skills/feature-pilot/SKILL.md` | close: a screen change updates `reach`/`press` and retakes pictures |
| `docs/features/README.md` | how to take pictures and what `reach`, `noshot`, `press` mean |
| `docs/sdlc/changes/316-…/` | `plan.md`, `mutations.py`, `verification.yaml`, `approvals.log`, `state.yaml` |

## Order of work

The tree compiles after every step; the suite is red only between a test and
the code it asks for.

1. `flow.yaml`: `reach`, `noshot`, `press` from the measured tour. Tests R1
   (plan) watched failing, then `capture-screens.py --plan`.
2. `capture-screens.py` whole, with `CAPTURE_PICTURE`. Tests R3, R4, R5 on
   stand-ins (the 314 runner's stand-in driver, a stand-in binary and a
   stand-in picture tool), each watched failing first.
3. Take the pictures here: `python3 scripts/capture-screens.py`. Fix `reach`
   and `press` until it exits 0; a screen nothing reaches gets `noshot`.
   Test R2 then holds against the committed files.
4. `change-map.py` model: read `screens.json` (R11 first, on a fixture with
   each broken case), areas (R6, against the real files), overview (R8),
   feature and domain links (R9), per-screen terms (R10). `--json` carries
   all of it.
5. Move the page into `scripts/change-map/`; `change_map_page_script_parses`
   and the existing tests pass unchanged in meaning.
6. `page.js`: logic functions first with R7 under node, then the drawing:
   概要, 機能 page, ドメイン page, 画面 with areas and the walk, 全体図,
   用語集 画面別, search, keys, theme. Opened in headless Chrome after each
   view.
7. Skill and README lines; R12.
8. `mutations.py` (a mutant per guard), `verification.yaml`, the
   empirical verifier; fixes recorded as `Fn`.
9. The browser item, the sweep, bands; push, CI.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A view throws and draws nothing | blank panel | node `--check`, R7, and the browser item with the console read |
| Areas misplaced (points vs pixels, origin) | outline beside its button | R6 checks bounds inside the window; the browser item compares by eye on three screens |
| The capture leaves an `operon` or tmux behind | process after the run | R3/R5; `pgrep` after the real run |
| A word answers two controls after a later change | an area on the wrong button | R6 refuses ambiguity |
| The map grows past what a browser opens quickly | slow first paint | pictures decoded lazily; the 16 MiB ceiling; measured in the browser item |
| Existing change-map tests read the page text | a test fails on the move | step 5 runs them before anything else moves |
| The flow tests of change 315 refuse the new keys | `screen_flow_reaches_every_page_and_every_way_resolves` or the flat-reader test fails | step 1 runs them first and widens the accepted keys where they are listed |
| No accessibility or screen-recording permission here | the capture exits 2 | the change records `awaiting-user` with what to grant; pictures are never stubbed |

## Proof of completion

- `cargo fmt --check` — no output; `cargo clippy --locked -- -D warnings` — clean.
- `cargo test --locked capture_ map_ change_map every_way every_screen the_pipeline_retakes` — all pass; the full suite in CI.
- `python3 docs/sdlc/changes/316-…/mutations.py` — every mutant caught.
- `python3 scripts/capture-screens.py` — exit 0; `pgrep -f <worktree>/target/debug/operon` prints nothing.
- Headless Chrome: the acceptance list in the spec, with no console error.

## Departures from the plan

Filled in during implementation.
