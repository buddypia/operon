# Spec: the editor keeps up with what agents write

- **Intent**: `./intent.md`
- **Status**: draft

## Requirements

1. While a document is the front tab, the editor notices within about two
   seconds that the bytes on disk stopped matching the bytes it read, and it
   does so without any filesystem call in a draw path.
2. When it notices and the buffer holds no unsaved edits, the editor adopts the
   new bytes and says in the one-line notice that it did. The rendered view, the
   source view, and the editor's own diff all show the new bytes afterwards.
3. When it notices and the buffer holds unsaved edits, the editor changes
   nothing and shows a bar naming the situation, offering exactly two ways on:
   reload from disk, and look at the diff. Neither is taken automatically.
4. The bar goes away when the document is reloaded, and when a save succeeds.
   A save that fails because the file changed underneath keeps its existing
   error and the bar together.
5. Rendering a Markdown document parses and highlights it at most once per
   distinct (text, theme) pair, not once per frame. Two consecutive frames over
   an unchanged document build nothing.
6. `- [ ]` and `- [x]` at the head of a list item are a checkbox in the rendered
   view — an unticked box and a ticked one — rather than literal brackets. The
   marker is an `ICON_*` constant, and the case of the `x` does not matter.
7. A fenced block whose info string names a language the editor already colours
   is coloured in the rendered view with the same grammar and the same palette
   roles the source view uses for that language. A fence with no language, or
   one nothing is known about, keeps the plain monospace it has today.
8. `~~struck out~~` renders struck out.
9. Nothing here reads more of a file than `EDITOR_FILE_MAX_BYTES`, which is the
   ceiling the editor already enforces on the same path.

## Behaviour

The Files tab, right pane, at 900px. The tab strip, the view switch, and the
content are as they are today; the bar between them is new.

**The bar — the file changed and there are unsaved edits.**

```
┌─ Files ─ operon ─────────────────────────────────────────────── 900px ─┐
│  plan.md ●     REVIEW.md                                               │
│                                                                        │
│   編集   プレビュー   差分                       ⟳   ⧉   [  保存  ]    │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ ⚠  このファイルはエディタの外で変更されました。保存すると、      │  │
│  │    その変更を上書きします。      [ 読み直す ]  [ 差分を見る ]    │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                        │
│  ## 手順                                                               │
│  ...                                                                   │
└────────────────────────────────────────────────────────────────────────┘
```

The bar sits under the view switch and above the content, in the same place and
the same frame the save error already uses, because they are the same kind of
statement about the same document — and when both are true, both are shown, the
save error first. It carries `ICON_ATTENTION` in `palette.warning` on
`palette.card`; no new colour role.

「読み直す」 is the existing reload. 「差分を見る」 switches to the Diff view,
which is the git diff of the working tree against the last commit — so it shows
the agent's write, which is the question the bar raises.

**No bar — the file changed and there are no unsaved edits.** The document is
reloaded, and the one-line notice at the top of the window reads
「plan.md はエディタの外で変更されたため、読み直しました。」. Nothing is lost:
there was nothing to lose, and a prompt whose only sensible answer is yes is a
prompt people learn to click through.

**The rendered view, with the marks agents actually write.**

```
│  ## 手順                                                               │
│                                                                        │
│    ☑  スキーマを読む                                                   │
│    ☐  移行を書く                                                       │
│    ☐  移行をロールバックして確かめる                                    │
│                                                                        │
│   ┌ rust ───────────────────────────────────────────────────────────┐  │
│   │ let plan = Plan::new(store);        // let, Plan, // each in     │  │
│   │ plan.write("移行")?;                //   their own role's ink    │  │
│   └─────────────────────────────────────────────────────────────────┘  │
│                                                                        │
│   これは ~~取り消し線~~ です。                                          │
```

The ticked box is `palette.success`, the unticked one `palette.text_faint`, and
the text beside either is body ink in both cases — a finished item is still an
item, and dimming it would make a finished checklist unreadable.

**The states that are not the happy one.** A watch that errors — the file was
deleted or became unreadable — changes nothing on screen and raises no bar: the
document a person is reading is still the document they were reading, and a
banner that appears every two seconds for a file that is gone is worse than the
save error they will get if they try to write it. A file that grew past
`EDITOR_FILE_MAX_BYTES` or became binary while open is treated as a change like
any other: unmodified buffers adopt it and show the "cannot open" reason,
modified buffers get the bar. Loading, empty, and unopenable are unchanged.

## Design

**Watching.** `src/git.rs` gains `read_file_for_editing_if_changed(root,
relative, known)`, which is `read_file_for_editing` plus one comparison: it
returns nothing when the outcome is text equal to `known`, and the outcome
itself otherwise. Reusing that function is the point — the size ceiling, the
binary check, the UTF-8 check, and the path resolution are all already there and
must not be restated.

`src/app.rs` gains `BackgroundKey::FileWatch` and `BackgroundResult::FileWatched`
beside the existing `FileOpen` and `FileSave`, one `last_editor_file_watch` map
keyed by project and path, and `watch_active_document`, called from `ui_files`
next to `refresh_editor_git_views` and on the same two-second tick. It watches
the front tab only: the other tabs are not being read, and the one that comes to
the front is watched two seconds later.

`OpenDocument` gains `external_change: bool`. It is set when a watch reports a
difference and the buffer is modified; it is cleared in `apply_opened_file` and
on a successful `apply_saved_file`. `src/models.rs` is not touched:
`FileOpenOutcome` keeps its shape, and `OpenDocument` lives in `src/app.rs`.

**Rendering once.** `src/ui/markdown_view.rs` gains `MarkdownPreviewCache`,
shaped like `EditorTextLayoutCache` next door and for the same reason: one
bounded cache for the one document on screen, keyed by the text and the colours,
counting its builds under `cfg(test)` so a test can assert a second frame builds
nothing. Its value is a `RenderedMarkdown` holding the parsed blocks and, in a
vector indexed the same way, the highlighted layout of each block that is a
fence — parallel to the blocks rather than inside `MarkdownBlock`, so
`src/markdown.rs` stays a parser with no drawing types in it.

`SyntaxColours` in `src/ui/syntax.rs` becomes visible to the crate so both
caches key on the same struct rather than each writing out its own list of
roles. `src/files.rs` gains `fence_language`, which maps a fence's info string
to the `EditorLanguage` the editor already knows: the words agents write
(`rust`, `bash`, `python`) explicitly, and anything else tried as an extension
through `editor_language`, so a fence marked `rs` or `py` needs no second table.

**The two new marks.** `src/markdown.rs` gains `task: Option<bool>` on
`MarkdownBlock::Bullet`, filled by a new `split_task_marker` that takes the
`[ ]`, `[x]`, or `[X]` off the front of an item's text. The comment on
`parse_list_marker` saying a checkbox is content rather than syntax is reversed
here and must be rewritten rather than left contradicting the code.
`src/glyphs.rs` gains `ICON_TASK_DONE` and `ICON_TASK_TODO` with their
`ICON_VOCABULARY` rows, and `DESIGN.md` gains the sentence saying why a checkbox
is a mark at all. `MarkdownSpan` gains `Strike`.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | Every ink used is an existing role: `warning`, `card`, `success`, `text_faint`, and the roles `syntax_layout` already maps grammar scopes onto. No literal reaches a call site, so `design_md_documents_exactly_what_the_app_paints` and `every_theme_stays_readable` stay green untouched |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | Yes | `ICON_TASK_DONE` and `ICON_TASK_TODO`, both Phosphor names, both in `ICON_VOCABULARY`, and both described in `DESIGN.md` beside the existing account of what a mark is for. `every_icon_resolves_from_the_bundled_icon_font` checks they resolve |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | No new identifier crosses a boundary. The two facts that could have been written twice — the editor's size and binary rules, and the list of palette roles the syntax theme reads — are shared by calling `read_file_for_editing` and by widening `SyntaxColours` rather than by copying either |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | Nothing persisted changes. `OpenDocument` is in-memory only, and the one write path, `save_project_file`, keeps its compare-and-swap and its atomic rename exactly as they are |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No new child process. The new `src/git.rs` function spawns nothing; it is an `fs::read` behind `resolve_project_file`, which is what keeps it inside the project root |
| Documentation — user-facing docs change in all three languages together | No | `README.md` describes the editor at a level this does not change. `DESIGN.md` is English-only by design and is where the two marks are recorded |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Nothing leaves the machine and no `unsafe` is added. The watch reads a file inside the project the person opened, on a two-second tick, only while its tab is in front |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | The watch reads at most `EDITOR_FILE_MAX_BYTES` per poll because it goes through `read_file_for_editing`, at most once per two seconds, for at most one document. The preview cache holds one document, which is the same ceiling again |

## Flagged concerns

- **Re-reading the whole file every two seconds rather than comparing a stamp.**
  A modification time and a length are cheaper than reading the bytes, and every
  file watcher works that way. This one does not, for two reasons. Comparing
  content is exact where a stamp is a guess: a stamp says a write happened, and
  an agent that rewrites a file with the same bytes would raise a bar over
  nothing. And a stamp needs somewhere to live — the moment the editor's copy
  was read — which means the open path and the save path both start carrying a
  timestamp, which means `FileOpenOutcome` changes shape, which is
  `src/models.rs`, which is a high-risk paused surface this change has no other
  reason to touch. The cost being paid is one read of at most 128 KiB every two
  seconds for one file, from the page cache, on a background thread. That is
  the right trade here and would not be at a hundred files.

- **A watch that fires while an agent is mid-write.** An agent writing a file
  non-atomically can be observed halfway, and an unmodified buffer would then
  adopt half a document and say so in the notice. The next poll two seconds
  later corrects it, so the failure is transient rather than lost work — nothing
  is written back, and the person's own edits are never the thing overwritten,
  because a modified buffer never auto-adopts. It is not worth a debounce: a
  debounce trades a rare flicker for a permanent delay on the common case, which
  is the case this change exists to make fast. Recorded so the next reader does
  not mistake it for something nobody thought about.

## Acceptance

- `cargo test --locked` passes at 310 or more, `0 failed`, and exactly
  `6 ignored`, including these new tests:
  - `an_agents_write_reaches_an_unmodified_buffer_by_itself` — a document is
    opened, the file is rewritten underneath it, the watch runs, and the buffer
    holds the new bytes with no bar raised.
  - `an_agents_write_never_overwrites_what_a_person_typed` — the same, with the
    buffer modified first: the buffer is untouched, and `external_change` is set.
  - `the_bar_clears_when_the_document_is_reloaded` — and stays clear.
  - `a_rendered_document_is_built_once_however_many_frames_read_it` — two
    `layout_for` calls over the same text and palette build once, and changing
    either builds again.
  - `a_checklist_renders_as_checkboxes` — `- [ ]` and `- [x]` parse to
    `task: Some(false)` and `task: Some(true)` with the brackets off the text,
    and `- [z]` stays literal text.
  - `a_fence_is_coloured_by_the_language_it_names` — a `rust` fence gets more
    than one section; a fence with no language and one naming nothing known get
    the plain layout.
  - `markdown_renders_what_agents_write` keeps passing with the widened
    `Bullet`, and gains the strikethrough case.
- `cargo clippy --locked -- -D warnings` and `cargo fmt --check` are silent.
- `bash scripts/check-bands.sh` reports no new breach; `largest_module_lines`
  is read before and after and stated, because `src/app.rs` is at 8 539 against
  a warn threshold of 8 800 and this change adds to it.
- In the running app: open this repository, open `docs/sdlc/lessons.md` in the
  editor, and run an agent that appends to it. Within two seconds the rendered
  view shows the new text and the notice says it reloaded. Type a character
  first and repeat: the bar appears, the buffer keeps the character, 「差分を見る」
  lands on the agent's diff, and 「読み直す」 replaces the buffer.

## Rejected alternatives

- **A filesystem notification API instead of a poll.** A dependency, a platform
  API, and a whole class of missed-event bugs, to save one bounded read every
  two seconds for one file.
- **Prompting before adopting an unmodified change.** A question with one
  sensible answer trains people to dismiss questions.
- **Watching every open tab.** Nine tabs is nine reads for eight documents
  nobody is looking at; the tab that comes to the front is watched two seconds
  later, which is the same delay the front tab already has.
- **Holding the highlighted fence inside `MarkdownBlock`.** It would put an
  `egui` layout type into the parser and make `src/markdown.rs` untestable
  without a font.
- **A full CommonMark crate.** `src/markdown.rs` covers what agents write and
  says so; swapping it out is a change about dependencies, not about this one.
- **Rendering a done checklist item in muted ink.** A finished plan would be a
  page of grey, and the checkbox already carries the state.
