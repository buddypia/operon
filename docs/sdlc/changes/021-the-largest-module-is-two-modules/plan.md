# Plan: the page drawing moves out of `src/app.rs`

- **Route**: refactor — behaviour must be provably unchanged, which is what the
  reviewer is for. No intent or spec: the band is the report, and inventing a
  spec for a file move is the ceremony that gets pipelines abandoned.
- **Status**: in progress

## Why now

`largest_module_lines` has been recorded as breached in three commits in a row:
8862, then 9078, then 9269, against a warn of 8800. `docs/sdlc/bands.yaml` calls
warn "recorded, no action", and three records in a row is the point at which the
record stops being information and starts being a habit. The last commit message
said splitting should come before the next feature; this is that.

## What moves

`CLAUDE.md` already says what `src/app.rs` is: "all app state, background tasks,
and the top-level UI wiring". The page drawing is none of those three. Lines
4490 to 8326 are one contiguous run inside a single `impl OperonApp` block, and
every item in it paints a page: the toolbar, the palette window, home, projects,
overview, the session card, git, worktrees, pull requests, files, the editor and
its tabs, skills, rules, the three session lists, settings, the terminal tabs and
panel, and the terminal workspace.

They move verbatim into `src/app/screens.rs`, a child module declared inside
`src/app.rs` — a child, because a new top-level module is a line in
`src/main.rs` and `docs/sdlc/risk.yaml` holds that at `paused`.

Nothing is rewritten. A refactor whose tests changed is not a refactor, and a
refactor whose bodies changed is not one either.

## Files that change

| File | Change |
|---|---|
| `src/app.rs` | `pub(crate) mod screens;`; the run of drawing methods removed; the impl block closed where they began. |
| `src/app/screens.rs` (new) | The same methods, in an `impl OperonApp` block of their own. |
| `CLAUDE.md` | The module map gains the child file. |
| `src/git.rs` | Its `mod` declarations move below `use crate::*;`, matching `src/tmux.rs`. Behaviour-neutral, and listed because on a refactor route the file list is the contract. |
| `src/tests.rs` | One guard rewritten — see the departures. |
| `docs/sdlc/lessons.md` | Entry 014. |

## Order of work

1. Move the run, declare the module, compile.
2. Widen only the visibility the split requires, and only where the compiler
   says so.
3. `cargo fmt --check`, the suite, clippy, bands.
4. The reviewer for the refactor route, then commit.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A method's body changes while it is being moved | A behaviour change dressed as a move | The move is done by line range, not by retyping. `git diff -M --stat` should show the moved lines as a rename-like pair, and the test count must not move. |
| A private method becomes reachable from further away than it was | Encapsulation quietly lost | Only the names the compiler names are widened, and each is listed in the departures. |
| The band does not actually improve | The whole change was for nothing | `bash scripts/check-bands.sh` before and after, both recorded. |

## Proof of completion

- `cargo fmt --check`, `cargo test --locked` with the **same** count as before
  (371 passed, 6 ignored), `cargo clippy --locked -- -D warnings`.
- `bash scripts/check-bands.sh` shows `largest_module_lines` back inside its band.
- `.claude/agents/rust-reviewer.md`, which the refactor route requires.

## Departures from the plan

- **No visibility was widened at all.** Step 2 turned out to be empty: every
  method in the moved run was already `pub(crate)`, and the four that were not —
  `ui_sessions`, `ui_project_session_list`, `ui_session_grid`, `session_grid_node`
  — are only called from inside the run, so they moved with their callers.
- **One test changed, and it took two attempts.**
  `the_brand_is_drawn_as_artwork_and_not_from_the_glyph_vocabulary` read
  `src/app.rs` by name and counted `brand_mark_image(` in it. Its subject is
  Operon's mark, not that file, so moving the drawing made it fail for a reason
  that had nothing to do with the brand.

  The first rewrite walked `src/` and counted across all of it. It passed, and
  the reviewer for this route caught that it is **weaker** than what it
  replaced: a third call site anywhere keeps a tree-wide total at two while one
  of the two required surfaces goes back to text, and the old form could not be
  fooled that way because a site in `src/ui/` never counted toward its total.

  The guard now names the two surfaces the assertion was always about,
  `ui_topbar` and `ui_projects`, finds each body wherever it is defined, and
  asks each separately. Watched failing twice, including the exact case the
  tree-wide count would have passed. Entry 014 in `docs/sdlc/lessons.md`
  records both the lesson and the wrong cure, because the wrong cure is the
  part worth remembering.
- **The module doc and the `CLAUDE.md` line were both overclaiming**, and the
  reviewer was right that a rule a reader would follow to the wrong file is
  worse than no rule. Nine items in the moved run decide rather than draw — the
  palette's ranking and key handling among them — and three drawing methods
  stayed in `src/app.rs`: `ui_restore_modal`, `ui_setup_prompt`,
  `ui_diff_comment_footer`. Both now say "one contiguous run moved" and name the
  three that did not.
- **`mod screens;` is not `pub(crate)`.** The module holds one inherent `impl`
  and no free items, so nothing needs to name it.
- **`steering_bytes` rose by roughly the size of that lesson**, from 144891 to
  146929, because `docs/sdlc/*.md` is inside what the metric counts. It was
  already past `diagnose` before this session. The band's response at that tier
  is "read-only investigation: what moved, and when", and the answer is:
  `lessons.md`, which is the mechanism working — a mistake found twice becomes a
  written check. `always_loaded_bytes`, the number that is a tax on every turn,
  is inside its band.
- **`scripts/check-readiness.sh` blocks a refactor for a missing `intent.md`.**
  `docs/sdlc/routes.yaml` gives the refactor route the stages `build,test`, and
  the checker's own comment says a bugfix or refactor "skips stages 1-2 by
  design" — but it applies that only to `spec.md`. A missing `intent.md` is
  blocking on every route. This change is the first one to have neither, so it
  is the first to find it. An `intent.md` was written rather than the checker
  changed: correcting a gate in order to pass it is the wrong order, and the
  inconsistency is recorded here for whoever touches that script next.
