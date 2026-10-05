# Plan: the editor keeps up with what agents write

- **Spec**: `./spec.md`
- **Approved**: 2026-09-01
- **Status**: done

This is the plan produced in plan mode and accepted before any file was edited.
If the implementation departs from it, update this file — an abandoned plan is
worse than no plan, because the next reader trusts it.

## Files that change

| File | Change |
|---|---|
| `src/markdown.rs` | `MarkdownBlock::Bullet` gains `task: Option<bool>`; new `split_task_marker`; `MarkdownSpan::Strike`; `~~` handled in `parse_markdown_spans`; the comment on `parse_list_marker` rewritten, since it currently argues for the behaviour this change reverses |
| `src/glyphs.rs` | `ICON_TASK_DONE`, `ICON_TASK_TODO`, both with `ICON_VOCABULARY` rows |
| `DESIGN.md` | one paragraph in *Iconography* saying why a checkbox is a mark |
| `src/files.rs` | `fence_language`, mapping a fence's info string to `EditorLanguage` |
| `src/ui/syntax.rs` | `SyntaxColours` becomes `pub(crate)`, so both caches key on one struct |
| `src/ui/markdown_view.rs` | `MarkdownPreviewCache` and `RenderedMarkdown`; `markdown_view` takes the rendered document; fences drawn from their cached layout; checkbox and strikethrough drawing |
| `src/git.rs` | `read_file_for_editing_if_changed` |
| `src/app.rs` | `BackgroundKey::FileWatch`, `BackgroundResult::FileWatched`, `last_editor_file_watch`, `watch_active_document`, `apply_watched_file`; `OpenDocument.external_change`; the bar in `ui_editor`; the preview branch reads the cache |
| `src/tests.rs` | the seven tests named in the spec's **Acceptance** |

## Order of work

Bottom-up, so the tree compiles at every step and the only red is the new test
that is supposed to be red.

1. `src/markdown.rs` — `task`, `split_task_marker`, `Strike`. The tree does not
   compile between this step and step 2: widening `Bullet` breaks its one
   construction site in `markdown_view.rs` and its four assertions in
   `tests.rs`. Fix `tests.rs` in this step so the break is confined to drawing.
2. `src/glyphs.rs`, `DESIGN.md` — the two marks. Compiles.
   `design_md_documents_exactly_what_the_app_paints` and
   `every_icon_resolves_from_the_bundled_icon_font` should pass at the end of
   this step; run them alone before going on, because a Phosphor name that does
   not exist is a blank box at runtime and nothing at compile time.
3. `src/files.rs` — `fence_language` with its test. Compiles, green.
4. `src/ui/syntax.rs` — widen `SyntaxColours`. Compiles, green.
5. `src/ui/markdown_view.rs` — the cache, the rendered document, and the three
   new marks drawn. Restores the compile broken in step 1.
6. `src/app.rs` — the preview branch reads the cache. Green: step 5's test can
   be written against the cache directly and does not need the app.
7. `src/git.rs` — `read_file_for_editing_if_changed` with its test. Green.
8. `src/app.rs` — the watch: key, result, map, poll, apply, and
   `external_change`. Then the bar. Then the three watch tests.
9. Mutate each new guard and watch it fail. Then the three gates and the bands.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| `layout_for` borrows `&mut self.markdown_preview_cache` while reading `self.open_documents` | Compile error at the one call site in `ui_editor` | The compiler. `ui_editor_text` already does exactly this with `editor_text_layout_cache`, so the shape is known to work; if it does not, the fallback is to take the text out first, which costs one clone per frame and is therefore not acceptable — the real fallback is to pass the cache in as an argument |
| The watch polls a document that was closed, or whose project was removed, while the read was in flight | Nothing visible; a stale result applied to the wrong index | `apply_watched_file` looks the index up by project and path, like `apply_opened_file`, and returns when it is gone |
| Auto-adopting a file an agent is halfway through writing | The rendered view flickers to half a document for one poll | Nothing catches it and nothing should: recorded as the second flagged concern in `spec.md`. The invariant that matters — a modified buffer is never adopted — is what `an_agents_write_never_overwrites_what_a_person_typed` pins |
| The watch and `refresh_editor_git_views` both fire every two seconds, so the git diff and the buffer can disagree for one tick | The Diff view shows the pre-write diff for up to two seconds after the bar appears | Accepted. Both converge on the next tick, and `invalidate_project_diff_views` is called when a document is adopted so the diff is re-read rather than left stale |
| `fence_language` falling through to `editor_language` gives a fence a language nobody meant, because the info string happened to look like an extension | A fence marked `text` or `output` gets coloured as something | The fallback only reaches `editor_language`, which returns `Plain` for anything it does not know, and `Plain` is what `syntax_layout` treats as "do not colour". Pinned by the negative case in `a_fence_is_coloured_by_the_language_it_names` |
| `src/app.rs` crosses the `largest_module_lines` warn threshold | `bash scripts/check-bands.sh` names a warn breach | Read the metric before and after and state both. 8 539 against a warn of 8 800 leaves 261 lines; the watch is budgeted at well under that, and if it is not, the watch moves to a module of its own rather than the band being widened |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`, with
  N at 317 or more against the 310 this change started from.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-bands.sh` — `bands: N metrics within their bands`, and
  `largest_module_lines` stated before and after.
- The seven new tests, each watched failing first:
  - `an_agents_write_reaches_an_unmodified_buffer_by_itself`
  - `an_agents_write_never_overwrites_what_a_person_typed`
  - `the_bar_clears_when_the_document_is_reloaded`
  - `a_rendered_document_is_built_once_however_many_frames_read_it`
  - `a_checklist_renders_as_checkboxes`
  - `a_fence_is_coloured_by_the_language_it_names`
  - `a_changed_file_is_reported_and_an_unchanged_one_is_not`
  and four more added answering the review:
  - `a_watch_answer_about_an_older_document_is_dropped`
  - `a_watch_does_not_answer_a_failed_save_on_the_persons_behalf`
  - the ordered-checklist case in `a_checklist_renders_as_checkboxes`
  - the hoisted-runs case in `a_rendered_document_is_built_once_however_many_frames_read_it`
- In the running app, the observation in the spec's **Acceptance**: open
  `docs/sdlc/lessons.md`, have an agent append to it, and watch the notice, then
  repeat with a character typed first and watch the bar.

## Departures from the plan

Six, none of them large, all recorded because the departure is the interesting
part.

- **Step 1 did not fix `src/tests.rs` in place.** The plan said to widen the
  four `Bullet` assertions in the same step that widened the type, to confine
  the compile break to drawing. In practice the whole implementation was written
  first and the assertions fixed at the end, so the tree did not compile between
  steps 1 and 5. Nothing was lost — the compiler named every site — but the plan
  claimed a property the work did not have, and saying so is cheaper than
  pretending otherwise.
- **`read_file_for_editing_if_changed` compares with `matches!` rather than
  against a constructed value.** `outcome == FileOpenOutcome::Text(known.to_owned())`
  reads better and copies the whole buffer on every poll — every two seconds,
  for nothing. The `matches!` form borrows.
- **The watch stands down while a save is in flight.** Not in the plan and not
  in the spec. A `FileSave` running against the same path is about to rewrite
  the file itself, and a watch racing it would report this app's own write as
  somebody else's — a bar raised over the save the person just asked for.
- **`document_for`, not `layout_for`.** It returns a parsed document with its
  fences coloured, not a layout, and naming it after its neighbour would have
  made the two look interchangeable when only one of them is a layout.
- **`README.md`, `README.ja.md`, and `README.ko.md` changed after all.** The
  spec answered the documentation policy with "does not apply", which was wrong:
  the README already carries a bullet promising a save never overwrites an
  agent's work, and that promise is now half of a pair. All three languages
  gained the other half together, plus the checklist and coloured-fence line.
- **`MarkdownSpan::Strike` is drawn in `text_faint`, not body ink.** The spec
  did not say which ink. Struck-out text is text the author withdrew, and a page
  of revisions drawn at full weight is a page of noise.
- **`RenderedMarkdown` holds rendered blocks, not parsed blocks beside a
  parallel vector of layouts.** The spec described the parallel shape. Both
  reviews named the same defect in it — the two vectors had to be indexed in
  step and nothing in the types said so — and hoisting the inline runs needed a
  per-block home anyway. See the review section below.
- **A tab brought to the front is watched at once, not two seconds later.** The
  spec said the delay was the price of watching only the front tab. It was the
  price of keeping the last-watched time in a map keyed by every file opened;
  with one slot the price is not paid.

## What the review found, and what was done

`.claude/agents/rust-reviewer.md`, run twice in its own context because the first
run's report was lost in transit and the second was cut short by a spend limit —
both delivered, and they converged on the same two Important findings.
**Verdict: do not approve.** Both are fixed here; the anchors are given as files
rather than `file:line` because `harness_documents_only_name_paths_that_exist`
reads every backticked path in `docs/` and a line suffix is not a path.

**Important 1 — a watch answer could be applied to a later generation of the
same document.** `apply_watched_file` decided by the buffer's state on arrival
and never asked whether the answer was still about the document that asked.
Reachable three ways: the person clicks 「読み直す」 while a read is in flight and
the reload is rolled back to the older bytes and announced as an agent's write;
⌘S lands first and the app reports its own write as somebody else's; the tab is
closed and the same file reopened, and the stale answer is adopted into the
fresh document, leaving `disk` holding bytes the file does not have — so the next
save fails its compare-and-swap over a change nobody made. Nothing typed was
ever lost, because `modified()` still refused, but each of those is a wrong
result on screen.

Fixed the way this repository already solves it for `BackgroundResult::GitDiff`,
which carries a `generation`: `BackgroundResult::FileWatched` now carries the
`disk` snapshot the read was issued against, and the answer is dropped unless
that snapshot is still what the document holds. The same check also drops an
answer that arrives while a `FileOpen` or `FileSave` for that path is in flight,
which is the same fact one moment earlier — `editor_file_work_in_flight` is that
question, asked at both ends of the watch. Pinned by
`a_watch_answer_about_an_older_document_is_dropped`, which covers all three.

**Important 2 — the preview still parsed the whole document every frame.** The
cache hoisted the block parse and the syntect pass, and `markdown_line` went on
calling `parse_markdown_spans` for every heading, paragraph, bullet, quote, and
table cell on every repaint, allocating a `String` per run. `spec.md`
requirement 5 says two consecutive frames over an unchanged document build
nothing, and that was true of the blocks and the fences and false of the runs;
`a_rendered_document_is_built_once_however_many_frames_read_it` counted cache
builds and could not see it. Of the two ways out the review offered — narrow the
sentence, or hoist the runs — the second is the one that keeps the document
honest, so `RenderedMarkdown` now holds the runs, `markdown_line` takes them,
and the guard asserts they are there and are the runs the markers imply. What is
left in the draw path genuinely needs the frame: assembling a `LayoutJob` takes
`ui.style()` and wrapping takes `ui.available_width()`.

Four of the nits taken, one dissolved by the fix above:

- **An ordered checklist lost its number.** `1. [x] …` drew the box *instead of*
  the `1.`, so a numbered checklist rendered unnumbered. Both are drawn now, and
  `a_checklist_renders_as_checkboxes` covers it.
- **`lines.join("\n")` ran every frame** for a fence with no language. Gone:
  every fence now carries one `LayoutJob`, coloured or plain, since `Plain` is
  what `syntax_layout` reads as "leave this alone" and what it returns for it is
  the monospace body an unlabelled fence already had.
- **`last_editor_file_watch` was never pruned.** It is one slot rather than a
  map now — one document is watched, so a map was a map of the answer to a
  question nobody asked, plus a pruning path on every route a tab can close by.
- **A watch could clear a save error nobody had read.** A reload answers a
  failed save because a reload is the person deciding; a watch is not.
  `a_watch_does_not_answer_a_failed_save_on_the_persons_behalf` holds it.
- **`blocks` and `fences` had to be indexed in step and nothing said so** — the
  fragility both reviews named. Dissolved: `RenderedBlock` carries each block's
  runs and its fence layout, so the pairing cannot come apart.

Not taken: caching an `Arc<Galley>` per fence instead of cloning the `LayoutJob`.
Both reviews called the clone defensible and said it is worth doing only after
the span parse, which is now done; it needs `Ui` inside `document_for` and is a
change about egui's galley cache rather than about this one.

Two answers worth keeping. `split_task_marker`'s `&rest[2..]` **cannot** panic:
reaching it requires two one-byte ASCII chars, so byte 2 is a char boundary and
`rest.len() >= 2`, and a multi-byte character takes the early return instead. And
no `clone()` here exists to escape a borrow — the disjoint field borrow the plan's
risk table worried about compiles, so the fallback it named was never needed.

## What the numbers did

Read before and after, as the plan asked, because `src/app.rs` was the module
with the least headroom:

| Metric | Before | After | Band |
|---|---|---|---|
| `tests_in_ci` | 310 | 319 | min 271, no breach |
| `tests_ignored` | 6 | 6 | max 6, unchanged, which is the number that matters |
| `largest_module_lines` | 8 539 | 8 760 | max, warn at 8 800 — 40 lines of headroom left |
| `steering_bytes` | 141 120 | 141 780 | already at `diagnose`; this change added ~660 bytes of `DESIGN.md` |
| `modules`, `unsafe_blocks`, `unwrapped_spawns` | 20, 2, 2 | 20, 2, 2 | unchanged |

`steering_bytes` was at `diagnose` before this change and stays there. It is
change 008's recorded waiver and change 009's subject — a defect in the metric's
own formula — not something this change caused or is asked to fix. The 660 bytes
it did add are the `DESIGN.md` paragraph the icon policy requires, which is the
cheapest tier that policy allows.

`largest_module_lines` is the one to watch, and it is now close. 40 lines of
headroom means the next thing that lands in `src/app.rs` — this change's own
review fixes ate half of what was left — breaches the warn tier. The answer at
that point is a module, not a wider band, and the watch is the obvious candidate:
`watch_active_document`, `apply_watched_file`, and `editor_file_work_in_flight`
are one concern with one entry point and no dependency on the rest of the file.

## Each guard, watched failing

Run one at a time, each against a mutation of the thing it guards, each restored
afterwards. A guard nobody has seen fail is a guess about what it guards.

| Guard | Mutation | Result |
|---|---|---|
| `a_checklist_renders_as_checkboxes` | `split_task_marker` returns `(None, text)` always | FAILED |
| `a_fence_is_coloured_by_the_language_it_names` | `fence_language` returns `Plain` always | FAILED |
| `a_rendered_document_is_built_once_however_many_frames_read_it` | the cache never hits | FAILED |
| `a_changed_file_is_reported_and_an_unchanged_one_is_not` | the equality check dropped, so every read is a change | FAILED |
| `an_agents_write_reaches_an_unmodified_buffer_by_itself` | `apply_watched_file` returns before acting | FAILED |
| `an_agents_write_never_overwrites_what_a_person_typed` | the `modified()` branch dropped, so every buffer is adopted | FAILED |
| `the_bar_clears_when_the_document_is_reloaded` | `apply_opened_file` no longer clears `external_change` | FAILED |
| `a_watch_answer_about_an_older_document_is_dropped` | the `known`-and-in-flight check dropped | FAILED |
| `a_rendered_document_is_built_once_…` (hoisted runs) | a block's runs left empty for the draw path to find | FAILED |
| `a_checklist_renders_as_checkboxes` (ordered case) | `parse_list_marker` stops recognising a number | FAILED |
| `a_watch_does_not_answer_a_failed_save_on_the_persons_behalf` | the save error is not carried across the adoption | FAILED |
