# Intent: UI interactions, drag-and-drop operations, and rich content panes rebuild, clone, and re-shape cached work on every frame

- **Status**: approved
- **Opened**: 2026-11-26

## Problem

The person's verbatim request:

> 絶対にパフォーマンスをあげる必要がある。できればUI/UXは既存のまま。レスポンスやレンダリングは最速でなるべくキャッシュ。D&Dとかもよくやっているから再レンダリングしないやり方がよい。コア機能を直す必要が合っても構わないから直して。

Across Operon, the existing visual design and workflow are already settled, yet frame latency and drag smoothness degrade rapidly as projects, sessions, diffs, Markdown specs, and open files grow because core draw and update loops redo expensive work on every frame instead of caching derived artifacts and deferring non-visual work during drag-and-drop:

1. **Drag-and-drop stalls and spurious terminal resizes**: Dragging dock borders, panel split rules, dock panels, or workspace/column items either triggers live terminal resize subprocesses mid-drag (because dock rule drags are not included in layout-drag tracking and pending resize checks), deep-clones the entire workspace tab/pane tree on every drag pointer frame just to test whether a drop fits, or keeps firing periodic background polling timers mid-drag.
2. **Per-frame workspace synchronization, cloning, and O(N²) session scans**: On every UI frame—including every frame of a D&D gesture—the session workspace re-runs live-session tab synchronization, sorts and searches projects and sessions repeatedly, writes the visible-session list twice per frame, and clones project/session records and status strings (`O(S²)` / `O(S×T)`) for the sidebar and filter chips even when nothing changed.
3. **Un-cached Diff, Editor, Viewer, and Markdown shaping**: The unified/side-by-side diff pane rebuilds the flattened row and height-offset tables from scratch on every frame and shapes a fresh text layout per visible row through font locks every frame; the code editor clones full syntax layouts (up to 128 KiB), line-number gutter strings, and cursor prefix strings every frame while re-parsing 17 syntax scope selectors per highlight pass; the Markdown preview rebuilds and submits widgets for all paragraphs, headings, bullets, quotes, tables, and code fences on every frame with no block-level shaped cache or off-screen skip.
4. **Redundant per-frame palette, file-tree, path, and icon work**: Opening or navigating `⌘K` computes ranked entries over up to 12,000 file paths and open-document symbols twice per frame (even when moving selection with arrow keys); the file tree clones path buffers for every directory check; home-directory lookup queries user directories per row; and bundled PNG icons look up shared context storage on every row and badge.

## Who feels it, and when

- Anyone dragging split pane dividers, dock panel borders, dock panels, tabs between split panes, or project/session rows in the session sidebar while agents are active.
- Anyone switching tabs, scrolling, or keeping multiple projects and dozens of sessions open in `Page::Sessions`.
- Anyone scrolling large Git diffs, long source files in the built-in editor/viewer, or long design/spec Markdown previews, or filtering quickly in `⌘K`.

## Desired outcome

- **Existing UI/UX and behaviour intact**: Every screen, layout proportion, colour, icon, shortcut, and Japanese string remains visually and functionally unchanged.
- **Smooth, zero-rerender-churn D&D**: Dragging tabs, split dividers, dock edges, dock panels, or sidebar rows triggers zero mid-drag terminal resize subprocesses (resizing once on drop release), zero full-workspace clones per pointer move, zero periodic background poll spawns mid-drag, and reuses already-cached sidebar, tab, diff, editor, and Markdown outputs during the drag.
- **Aggressive derived-state caching and O(1)/O(N) steady frames**: Unchanged frames perform zero workspace-session re-syncs, zero sidebar re-sorts/clones, zero diff row-table rebuilds, zero `Galley` re-shaping across diff/editor/viewer/Markdown, and single-pass cached `⌘K` palette results—with strict cache invalidation whenever underlying data, filters, fold state, theme, DPI, or layout width actually change.
- **Maintainable module sizes**: Relocating cohesive sidebar/terminal-search helpers into existing sibling modules keeps the largest source files well below control-band limits while touching zero high-risk paused surfaces.

## Constraints this change inherits

- macOS only; `eframe`/`egui` immediate-mode GUI.
- Local-first: zero new external crates, zero new `unsafe` blocks, zero raw child spawns outside `src/exec.rs`.
- Paused high-risk surfaces (`src/store.rs`, `src/models.rs`, `src/main.rs`, `src/sys.rs`, `Cargo.toml`, `Cargo.lock`, `.claude/`, `.githooks/`, `scripts/`) remain untouched.
- User-facing copy remains Japanese; code, comments, and SDLC/spec docs in English.

## Systems likely affected

- `app` (`screens`, `session_order`, `terminal_output`) — unified drag state & deferred terminal resize, content-fingerprinted workspace sync, deduplicated visible-session/stream sync, `SidebarViewCache`, relocated search/sidebar helpers, `FileTreeRowCache`, and `⌘K` palette cache.
- `ui` (`session_tree`, `diff`, `syntax`, `markdown_view`, `files`, `widgets`) — allocation-free `can_move_tab`, `DiffRenderCache` (flat rows + bounded row `Galley`s), `ScopeSelectors` `OnceLock` + `CachedEditorTextLayout` / `ViewerRowLayout` `Galley` caching, `MarkdownPreviewCache` per-block `Galley` & large-doc viewport skip, zero-clone `file_tree_rows`, `OnceLock` home-dir lookup in `friendly_path`, and once-decoded bundled PNG `ColorImage` cache.
- `config` — bounded diff row galley cache ceiling (`MAX_CACHED_DIFF_ROW_GALLEYS`).

## Open questions

None. Code inspection established the exact hot paths and existing test source-pin boundaries in `src/tests.rs` and `docs/sdlc/bands.yaml`.

## Not in scope

- Redesigning visuals, typography, spacing, colours, or interaction workflows.
- Changing persisted store (`store.json`), `workspace-layout.json`, or `session-order.json` serialization formats (`src/models.rs` and `src/store.rs` are untouched).
- Adding new external dependencies or background threads beyond existing channels.
