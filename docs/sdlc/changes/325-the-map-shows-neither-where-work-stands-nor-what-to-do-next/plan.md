# Plan: the product map shows neither where work stands nor what to do next

- **Spec**: `./spec.md`
- **Approved**: 2026-11-28
- **Status**: approved

What breaks, what was assumed, and the alternatives, before the order:

- **Breaks**: change 316's page tests read the page as text (two `script`
  blocks, one `img`, the bar's order, strings such as `作業かご` and
  `指示をコピー`) and `map_page_logic_routes_walks_searches_and_scales_areas`
  compares `parseRoute` results exactly, so a `query` key may appear only when
  the link has one. `--json` consumers gain keys and lose none.
- **Assumed**: a `Palette` table is one `Color32` call per field apart from
  `dark` and the `ansi` array; every `ICON_*` constant is `icons::NAME`; the
  backlog's JSON is the list `scripts/backlog.py` prints for `list --json`.
  Requirement 6's test fails on this repository if any of the three is wrong.
- **Alternatives**: in `spec.md`, "Rejected alternatives".

## Files that change

| File | Change |
|---|---|
| `src/tests.rs` | The twelve tests of the spec, written first and seen failing; fixtures for the three pipeline tables, a palette, marks, keys and a stand-in backlog script. |
| `scripts/change-map.py` | `status_of`, `feature_depth`, `work_of`, `read_backlog`, `read_process`, `read_catalog`, `serve`; `build` takes the backlog choice; `main` takes `--backlog`, `--backlog-seconds`, `--serve`, `--port`, `--requests`. |
| `scripts/change-map/page.js` | `MapLogic`: links with a choice, lenses, the work order, the review packet, board and requirement filters, the queue, storage, theme, keys. `main`: the left column, 現況, tiles and chips, a feature's rail, sections and verdict bar, 作業, 要件 cards, 進め方, カタログ, the kept basket. |
| `scripts/change-map/page.html` | The bar's ten entries, the left column, panes for 作業, 進め方 and カタログ. |
| `scripts/change-map/page.css` | Layout of the column, tiles, board, rail, pipeline and swatches; the `auto` theme. |
| `docs/features/README.md` | The views, `--backlog` and `--serve`. |
| `docs/sdlc/changes/325-the-map-shows-neither-where-work-stands-nor-what-to-do-next/` | `verification.yaml`, `mutations.py`, `state.yaml`. |

## Order of work

The application is not touched, so the tree compiles throughout; a step is
done when its tests pass and the earlier map tests still do.

1. Tests for requirements 1–6 and 11 in `src/tests.rs`, failing on a missing
   key or an unknown option.
2. `scripts/change-map.py`: the model (`status`, feature depth, `process`,
   `catalog`, `work` without a backlog), then `--backlog`, then `--serve`.
3. Tests for requirements 7–9 (node over `page.js`), then the `MapLogic`
   functions.
4. Tests for requirements 10 and 12, then `page.html`, `page.css`, the views
   in `page.js` in this order: the column and 現況; tiles and chips; the
   feature page; 作業; 要件; 進め方; カタログ; the basket, verdicts and theme.
   Then `docs/features/README.md`.
5. The page in headless Chrome against this repository: every view, the
   acceptance list of the spec, the console.
6. `mutations.py` and `verification.yaml`; the empirical verifier in its own
   worktree; `scripts/check-review.sh` on the index; `scripts/check-bands.sh`.
7. `state.yaml` to done, what is left over to the backlog; the branch goes to
   the remote, CI runs, and it is merged on `main`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A value from a document or from GitHub becomes markup | A title with a tag in it draws as one | Every model value passes `esc()`; the hostile-title tests of change 316 run on the new views' strings; the backlog card's link is taken only when it begins `https://github.com/` |
| The served page answers something other than this machine | A request from another host is answered | The socket is bound to `127.0.0.1`; requirement 11's test reads the bound address from the printed line and a mutant binding all addresses is in the sweep |
| A slow or signed-out `gh` stalls every map | The script hangs | The backlog is read only with `--backlog`, under a timeout; requirement 4's test runs a stand-in that sleeps |
| Stored basket entries outlive the records they name | A copied text names a feature that is gone | `pruneBasket` on load; requirement 8 |
| The page grows past what a browser opens comfortably | Slow first paint | The new model keys are counts and names, not documents; the page's size is compared before and after in step 5 |
| An earlier link stops working | `#features`, `#requirements/home`, `#316` open the wrong view | The earlier logic test's exact comparisons stay as they are |

## Proof of completion

- `cargo fmt --check` — no output; `cargo clippy --locked -- -D warnings` — clean.
- `cargo test --locked map_` and `cargo test --locked change_map_` — all pass,
  the twelve new names among them; the full suite in CI.
- `python3 docs/sdlc/changes/325-the-map-shows-neither-where-work-stands-nor-what-to-do-next/mutations.py` — every mutant caught.
- `python3 scripts/change-map.py` — exit 0, the summary as before; with
  `--serve --port 0 --requests 1` — the address, one page, exit 0.
- Headless Chrome: the acceptance list of the spec, with no console error.

## Departures from the plan

- **The model was written before the tests of requirements 1-6 saw red.** The
  functions of `scripts/change-map.py` came first in one sitting and the tests
  after; each test was then watched failing against a broken model by the
  mutation sweep instead (this change's `mutations.py`), which is the evidence the plan's
  "see them fail" asked for.
- **A holder the backlog cannot name.** `scripts/backlog.py` says
  `held_known: false` when it could not read who holds an item; the board then
  says 「着手中かどうか不明」 and keeps the item in the backlog column rather than guessing
  a holder (spec requirement 4's "unknown holder").
- **The 作業 view keeps who and which columns in its link** (`#work?who=me`),
  like the lenses over the features, so a narrowed board survives a reload.
- **A request line cut at 8 KiB is answered 414.** The first version handed a
  line with no end to `parse_request`, which read the cut line as a request for
  `/` and served the map; the server test caught it.
- **The browser's own `/favicon.ico` request is answered 404**, as every path
  but `/` is (requirement 11), so a served page shows one 404 line in the
  console. Nothing in the page itself fails.
- **A whole-request limit on the server (F1).** The empirical verifier found
  that the 5-second timeout bounded each read, not the request: a client
  sending a byte every 3 s held the one-at-a-time server for minutes. A timer
  now shuts the connection's reading side at 5 s and the request is answered
  408.
- **The backlog reader against hostile output (F2-F5).** The second pass found
  that an integer past a float's range raised out of `finite` and stopped the
  map; that a grandchild in its own session kept the pipe open past the
  deadline (the output is now read against the deadline with `select`); that a
  repeated or fractional number gave two cards one id (each issue number is
  one card, the first); and that a basket kept as `null` stopped the page
  (`MapLogic.basketOf` now shapes whatever was kept).
- **The sweep runs test by test.** Judging all 44 mutations by every map test
  took about 17 minutes, past the 900 seconds CI gives an obligation; the first
  CI run failed on it. Each mutation is now judged by the one test that guards
  it, as change 323's sweep is grouped, and the whole sweep takes two minutes.
