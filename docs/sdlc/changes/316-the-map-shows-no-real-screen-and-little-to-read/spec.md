# Spec: the product map shows no real screen, and too little to read or do

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

Words used below: a *requirement line* is a bullet of a feature spec's
`## Behaviour` section, and it is *pinned* when its `Pinned by` names at least
one test that exists as `fn <name>` in `src/tests.rs` — the rule of change
315's requirement board, read by the same code. A *way's words* are its `press`
when it has one, else its `label`. A control *answers* to words by the runner's
rule (`plain` names; an exact name, else the only name containing the words),
read from `scripts/verify-app.py`, not restated.

1. Every screen of `docs/features/flow.yaml` carries exactly one of `reach`
   (the words to press, `;`-separated, from a fresh launch with the demo
   project; `-` for none) or `noshot` (a Japanese reason). `python3
   scripts/capture-screens.py --plan` prints, as JSON, every `reach` screen
   with its words and starts nothing; every word is text the application's
   source draws or the demo project's name. Fails without it:
   `capture_plan_reaches_every_screen_with_words_the_app_draws`.
2. `docs/images/screens/screens.json` holds an entry for exactly the `reach`
   screens: its picture's file name, the window's size in points, the scale
   (picture pixels per point), the controls with names and bounds in points
   relative to the window's top-left corner, all inside the window, and the
   words on the window; with the time and commit taken. Each picture exists,
   is a JPEG of at most 600 KiB, and measures size × scale. Fails without it:
   `every_screen_of_the_flow_has_its_picture_or_says_why`.
3. A capture with stand-ins for the binary, the window tool and the picture
   tool launches the binary apart from the person's home once per screen,
   presses the screen's words in order, writes each picture and entry, exits
   0, and leaves no process it started and no scratch folder. Fails without
   it: `capture_writes_each_picture_and_leaves_nothing_running`.
4. When a press fails, that screen keeps its earlier picture and entry, the
   others are still taken, and the run exits 1 naming the screen and the
   word. Fails without it: `capture_keeps_the_old_picture_of_a_screen_it_could_not_reach`.
5. A capture sent `SIGTERM` stops the application it started and removes the
   scratch folder before it exits, and writes no half picture. Fails without
   it: `capture_interrupted_leaves_nothing_behind`.
6. For a photographed screen, a way out whose words answer to exactly one
   control becomes a pressable area at that control's bounds; no way's words
   answer to two controls; and every way out of a photographed screen that
   has words and no key answers to one control. Every way, with an area or
   not, is listed beside the picture and can be taken from there. Fails
   without it: `every_way_with_words_lands_on_one_control_of_its_picture`.
7. The page's logic runs under node without a browser: each link
   (`#overview`, `#features`, `#feature/<id>`, `#screen/<id>`, `#canvas`,
   `#domains`, `#domain/<id>`, `#glossary`, `#glossary/screens`, `#term/<id>`,
   `#requirements`, `#changes`, `#change/<NNN>`) parses back to the view it
   was made from, and an unknown link opens 概要; a pressable area scales to
   the width the picture is drawn at; a walk (press, press, back, back,
   forward) lands where a browser's history would; search over all kinds ranks
   a title equal to the query above one containing it and returns every kind;
   and the key table holds `/`, `j`, `k`, `Enter`, `Escape`, `?`, `ArrowLeft`,
   `ArrowRight`. Fails without it:
   `map_page_logic_routes_walks_searches_and_scales_areas`.
8. The overview's counts are the model's: features, domains, screens and the
   photographed ones, requirement lines and the pinned ones, terms, changes by
   group; per domain its features and its pinned share. Fails without it:
   `map_overview_counts_what_the_model_holds`.
9. A feature carries what its page shows: the flow screens whose `features`
   name it, the terms whose `features` name it, the modules its domain owns,
   the changes linked to it newest first, and its requirement lines with
   their tests. A domain carries its features, modules and screens. Fails
   without it: `map_feature_and_domain_gather_screens_terms_code_and_changes`.
10. A photographed screen lists the glossary terms whose term or on-screen
    word appears among the words on its picture. Fails without it:
    `map_glossary_by_screen_lists_the_terms_its_picture_shows`.
11. The map is still written, and says why per screen, when `screens.json` is
    missing or not JSON, names a picture that is gone, a picture whose size
    differs from its entry, or a picture past 600 KiB, or holds more than the
    ceilings below (the excess is left out and the summary says how many);
    the summary line counts screens with no picture. Fails without it:
    `change_map_survives_missing_or_broken_pictures`.
12. `.claude/skills/feature-pilot/SKILL.md` says a change that alters what a
    screen shows, or adds a screen or way, updates `reach` or `press` in
    `flow.yaml` and runs `scripts/capture-screens.py`; `docs/features/README.md`
    says how. Fails without it:
    `the_pipeline_retakes_pictures_when_a_screen_changes`.

What no test here can check — that the page draws these, that a press on a
picture opens the next one, that back works in a real browser — is the
`page-exercised-in-a-browser` contract item, done in headless Chrome.

## Behaviour

**Taking pictures** — `python3 scripts/capture-screens.py [--plan] [--only
SCREEN ...] [--no-build]`, run by a session on this machine, prints English.
It builds with `cargo build --locked` unless `--no-build`, then for each
`reach` screen in flow order: launches the application through
`scripts/verify-app.py` (seed `project`; the person's home and tmux
untouched), presses each word with that runner's `press`, reads the tree with
`snapshot --window-id W --compact --include-bounds --max-depth 40`, waits until
two reads 300 ms apart agree (at most 3 s), photographs the window alone
with `screencapture -x -o -l N`, converts it to JPEG at quality 72 with `sips`,
and stops the application. Each screen is bounded at 120 s and the run, after the build, at
60 min. The window appears on the person's screen while it runs (about 10 s a
screen), as with `verify-app.py run`; the pointer is not moved and nothing is
typed. A screen whose press fails keeps its earlier picture and entry, and
the run ends with exit 1 naming each screen and word. Interrupted, the
application it started is stopped and the scratch folder removed; a picture
is written to a temporary name and renamed only when whole.

**The map** (`python3 scripts/change-map.py`, one HTML file, dark by default):

- Top bar: 「Operon 製品地図」, views 概要 · 機能 · 画面 · ドメイン · 用語集 · 要件 ·
  変更, a search box 「検索（/）」 whose results list features, screens, domains,
  terms, requirement lines and changes grouped by kind, 作業かご, and
  「ライト」/「ダーク」 (kept in the browser). Every view and item has its own
  link; reload keeps it; back and forward move through what was opened. `?`
  shows the keys: `/` search, `j`/`k` next and previous in a list, `Enter`
  open, `Esc` close, `←`/`→` back and forward on 画面.
- **概要**: tiles 機能 · ドメイン · 画面（写真 n）· 要件（テストあり n · p%）· 用語 ·
  変更; then あなたの判断を待っています, 進めている変更, ドメイン別 (features and
  pinned share as a bar), テストで守られていない要件が多い機能 (top 5), 最近入った変更
  (latest 10, with date), 写真がない画面.
- **機能** page: a header (name, domain, screen, 安定/変更中/判断待ち, counts), a
  section index that follows the scroll, and 概要 (the brief), 画面 (a strip of
  its pictures; pressing one opens it on 画面), 要件 (each line and its tests;
  「テストなしだけ」 narrows), 実アプリで確認 (the steps, when the spec has them),
  用語, コード (modules its domain owns, paths its spec names), 変更履歴 (newest
  first, with state), 文書 (brief and spec, opened in the side panel).
- **画面**: the window as the application draws it, as wide as the page
  allows. Moving over it outlines each pressable area with its words; pressing
  one shows the next screen; 「戻る」「進む」 and the trail of screens walked sit
  above it; 「押せる場所を表示」 outlines all areas. Beside it: name and kind,
  「撮影 YYYY-MM-DD（commit）」, its features, ways in and out (key, words, or
  「行を押す」 for a `click` way), each one pressable, the terms on it, its note.
  Ways with a key appear under the picture as keys that can be pressed. A
  screen with no picture is an empty window with its name and 「写真がありません:」
  and the reason. A list of every screen with a small picture runs down the
  left, grouped by catalogue screen; 「全体図」 switches to the canvas.
- **全体図**: every screen as a small picture in lanes by catalogue screen,
  arrows between them; drag moves, ⌘ with the wheel or −/＋/「全体」 zoom;
  moving over a screen keeps only its arrows bright; pressing it opens it on
  画面.
- **ドメイン** page: what it owns and does not, adjacent domains as links, its
  features with state and pinned share, its modules, its screens.
- **用語集**: 「用語順」 as before and 「画面別」: each photographed screen with
  the terms its picture shows.
- Empty and broken states: 「写真がまだありません。scripts/capture-screens.py で
  撮れます」 when `screens.json` is missing; 「写真を読めませんでした:」 with the
  reason (gone, size differs, too large) per screen; 「一致するものはありません」
  for a search with no result; 「この機能の画面はまだ流れにありません」.

## Design

Placement verdict: `EXTEND` of the product map (changes 312, 313, 315), which
no product feature owns (`features: none`). Evidence: `scripts/change-map.py`
draws the flow from `docs/features/flow.yaml`; `scripts/verify-app.py`
(change 314) launches and presses. Nothing in `src/` but `src/tests.rs`
changes.

Measured on this machine before writing (2026-11-24, agent-desktop 2.5):

- `agent-desktop list-windows` gives the window id `w-107943`; `screencapture
  -x -o -l107943` wrote a 2560 × 1640 PNG for a 1280 × 820-point window, so
  the digits are the window number `screencapture` takes, and the picture is
  the window alone at scale 2. The runner's own `picture` (the window tool's
  `screenshot`) gave 2696 × 1776 with the shadow and a margin, which would
  misplace every area.
- `snapshot --include-bounds` gives each node's bounds in screen points; the
  window node's bounds are the window's. Area = node bounds − window origin,
  kept in points; the page multiplies by drawn width / window width.
- Project rows are not controls in the tree, and icon-only buttons have empty
  names, so neither can be pressed; the six project tabs are reached through
  the palette's 「プロジェクトを開く」 after a terminal is opened in the demo
  project, and a screen nothing reaches gets `noshot`.

Parts:

- `scripts/capture-screens.py` — standard library; loads `verify-app.py` with
  `importlib` for launch, press, tree, stop and its stand-in driver.
  `CAPTURE_PICTURE` names a stand-in for `screencapture` and `sips`, for the
  suite. Children: the window tool (30 s each, as the runner), `screencapture`
  (30 s), `sips` (30 s), `cargo build --locked` (1800 s).
- `docs/features/flow.yaml` — `reach` / `noshot` per screen, `press` per way
  (optional). `docs/images/screens/` — `<id>.jpg` and `screens.json`.
- `scripts/change-map.py` — reads `screens.json` (4 MiB at most; 80 screens,
  300 controls and 400 words a screen; pictures 600 KiB each and 16 MiB in
  all), joins pictures and controls to the flow, and derives the overview,
  the feature, domain and per-screen term links. The page moves out of the
  Python string into `scripts/change-map/page.html`, `page.css` and `page.js`,
  inlined at build. `page.js` keeps its logic in plain functions exported to
  node and its drawing behind `typeof document`. Pictures are inlined as data
  URLs so the map stays one file; at 27 pictures of about 200 KiB the file
  grows about 7 MiB.
- `src/tests.rs` — the twelve tests above.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | the application is not changed; the page has its own sheet, as before |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | no mark is added to the application |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | a way's words and a control's name are matched by the runner's own `plain` and `control_for`, loaded from `verify-app.py`, not retyped |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | the store is untouched; the seed goes through change 314's path |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | no child in the application; each script child is bounded as listed |
| Documentation — user-facing docs change in all three languages together | no | no user-facing document changes; `docs/features/README.md` is for sessions |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | pictures are taken and kept on this machine; the page loads nothing from the network |
| Budgets — any new scan or output path states its byte and item ceiling | yes | `screens.json` 4 MiB, 80 screens, 300 controls, 400 words; a picture 600 KiB, 16 MiB in all; 120 s a screen, 60 min a run |

## Flagged concerns

- **CI cannot take pictures** — a hosted runner has no window permission.
  Answer: requirements 1–12 run without a window (stand-ins for 3–5); taking
  the pictures is the `pictures-taken` human item, done here as in change 314.
- **Pictures go stale** — a later change alters a screen. Answer: the page
  shows when and at which commit each was taken; requirement 12 puts retaking
  in the pipeline; requirement 6 fails when a way's words leave the picture.
- **Repository size** — about 27 JPEGs of about 200 KiB enter git history,
  and every retake adds its changed pictures again. Answer: the session's
  decision, recorded in the intent and reported to the person; bounded by
  600 KiB a picture and 16 MiB in all.

## Acceptance

- `cargo test --locked` passes, including the twelve tests above and
  `change_map_page_script_parses`.
- `python3 scripts/capture-screens.py` on this machine exits 0 and leaves no
  `operon` process of this checkout running.
- In headless Chrome: 概要 shows the counts; a feature page shows its pictures;
  on 画面, pressing ホーム's 「すべて表示」 area shows セッション and 「戻る」
  returns; 全体図 shows pictures; 用語集 「画面別」 lists terms; a copied link
  reopens the same view; no console error.

## Rejected alternatives

- Drawn wireframes from a per-screen model, as learn_korean — a second
  description of every screen to keep true by hand.
- Pictures made only locally and not committed — a fresh checkout's map
  would have no screens.
- Pressing by pointer coordinates — takes the mouse from the person.
- Naming icon buttons in the application in this change — not asked for;
  reported as a candidate (intent, Not in scope).
- PNG pictures — three to five times a JPEG's size at no visible gain.
- The runner's `screenshot` — includes the shadow and margin (measured above).
