# Spec: follow a path or an address out of the output

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. A run of path characters containing a name, a dot, and an extension is a
   candidate path, optionally followed by a line and a column. The column is
   read and dropped: the editor opens at a line.
2. An extension made only of digits is not an extension. Output is full of
   version numbers and none of them is a file.
3. Trailing sentence punctuation belongs to the sentence. A panic prints the
   location followed by a colon; a link at the end of a sentence carries its
   full stop; a link in brackets carries the bracket.
4. `http://` and `https://` are recognised only when what precedes them is not a
   path character, so a scheme inside another token is not a link.
5. A candidate path is resolved against the session's own folder, must be a
   file, and must be inside that folder. A trace naming somewhere else cannot
   turn a click into a way of reading it.
6. Resolution never happens on a draw path. The pane records the names it has
   not seen; they are resolved once, outside the frame, and remembered — both
   hits and misses, because output repeats itself.
7. Only a resolved path is drawn as openable. Nothing is underlined that a
   click could not follow.
8. The underline appears on hover only. A pane of permanently underlined words
   is harder to read than the plain text it replaced.
9. A click on a target opens it: a path in the editor, scrolled to its line; an
   address in the browser. While the pane has focus a plain click still only
   goes to the pane, and `⌥` is what says "open this" — an agent being talked to
   must not lose the keyboard because a trace was under the pointer.
10. The right button offers copying the path and revealing it in Finder.
11. Only the rows the viewport shows are read for targets.

## Behaviour

The screen was agreed before this was written:

```
┌─ セッション ───────────────────── 1180px ─┐
│ thread 'tests::x' panicked at              │
│   src/app.rs:120:8                          │
│   ‾‾‾‾‾‾‾‾‾‾‾‾‾‾  ← ホバーで下線        │
│   → Local:  http://localhost:5173/          │
└────────────────────────────────────────┘
```

The states that are not the happy one:

- **A path that is not there.** No underline, and no click. This is most
  candidates: a word with a dot in it is common.
- **A path outside the project.** It resolves, so it underlines, and the click
  reports 「{p0} はこのプロジェクトの外にあります。」 rather than opening the
  editor at a name it would resolve somewhere else.
- **No worktree on the session.** Nothing resolves; nothing underlines.
- **A file with no line number.** It opens at the top.
- **The pane has focus.** A plain click goes to the pane, as it always has.

## Design

`src/ui/terminal.rs`:

```rust
pub(crate) enum TerminalTargetKind { Path { path: String, line: Option<usize> }, Url(String) }
pub(crate) struct TerminalTarget { column: usize, columns: usize, kind: TerminalTargetKind }
pub(crate) fn terminal_targets(text: &str) -> Vec<TerminalTarget>
pub(crate) fn resolve_terminal_path(worktree: &Path, path: &str) -> Option<PathBuf>
pub(crate) struct TerminalLinkOverlay<'a> { resolved, unresolved, palette }
pub(crate) fn terminal_row_links(...) -> Option<TerminalTargetKind>
```

Positions are display cells, from `terminal_display_columns` — the same
arithmetic the search highlight uses, and for the same reason.

`terminal_rows` calls `terminal_row_links` for the rows it is already drawing
and returns whatever is under the pointer, so the caller decides what a click
means. Names the cache has not seen go into `unresolved`, which the frame loop
drains into `resolve_terminal_paths` — outside the borrow, outside the draw.

`src/app.rs` holds `resolved_paths`, `unresolved_paths`, `editor_jump_line`, and
`terminal_path_menu`. The editor's scroll area takes an offset when a jump is
pending, and the jump is spent by the frame that uses it so scrolling by hand
afterwards stands.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | The underline is `palette.accent_text`, the ink already used for a link. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No new mark. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | The extension ceiling and the probe ceiling are `src/config.rs` constants. The two shapes are recognised in one function. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes | Nothing persisted. A resolution is true while the session is running. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | The only outward acts are opening a URL and revealing a path, both through the existing system-action path, both requiring a click. |
| Documentation — user-facing docs change in all three languages together | Yes | One bullet in each README. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Reading text already on screen, and one `canonicalize` per name never seen before. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | `TERMINAL_PATH_PROBE_LIMIT` per frame, `TERMINAL_PATH_MAX_EXTENSION` per candidate, and only visible rows are read. |

## Flagged concerns

- **A click that opens something is a click that could open the wrong thing.**
  Resolved by requirement 5: inside the session's folder, and a file. A trace
  naming a home directory resolves to nothing.
- **Stealing the click from the pane.** Resolved by requirement 9. The gesture
  when the pane has focus is an `⌥`-click, which is what the agreed screen says.

## Acceptance

- `cargo test --locked` passes, including:
  - `finds_a_path_and_its_line_in_a_stack_trace`
  - `does_not_call_ordinary_words_paths`
  - `finds_a_url_without_the_punctuation_after_it`
  - `reports_a_target_position_in_display_cells`
  - `refuses_a_path_that_escapes_the_session_folder`
- In the running app: a failing `cargo test` in a session, hover the path in the
  panic line, click it, and the editor opens at that line.

## Rejected alternatives

- **A popover with three items.** The other option on the agreed screen. Opening
  is what a click on a link means everywhere else, and a menu covers the row
  under the one being read.
- **A pattern crate.** A dependency is a paused surface, and the two shapes are
  narrow enough to say out loud.
- **Underline everything that looks like a path.** Then a click does nothing on
  most of them, and an underline stops meaning anything.
- **Probe the filesystem while drawing.** One metadata call per candidate per
  frame is exactly what the Rust rule forbids, and a stack trace repeats the
  same name on twenty rows.
- **Take every click.** An agent being talked to would lose the keyboard because
  a trace scrolled under the pointer.
