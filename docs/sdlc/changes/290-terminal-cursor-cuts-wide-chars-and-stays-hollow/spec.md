# Spec: industry-standard block cursor width, theme fill, and focus state in the terminal

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `terminal_rows` sizes the caret rectangle to `1 * cell.x` on a narrow
   character or empty cell and `2 * cell.x` (anchored at the character's lead
   display column) on a full-width character — including three-byte Japanese
   characters, four-byte UTF-8 CJK extensions and emoji, base characters
   followed by zero-width combining or variation-selector marks, and a cursor
   coordinate landing on the trailing column of a two-column glyph — while
   still reporting `None` when the widened caret would extend past the pane's
   inner right edge.
   Tests: `cursor_290_wide_and_four_byte_glyphs_span_two_cells`,
   `model_check_cursor_290_cell_span_partitions_every_line`.
2. When an active terminal pane holds keyboard focus and has no active IME
   preedit, `draw_terminal_cursor` paints a solid `palette.terminal_fg` block
   over the caret rectangle (light on `Dark` and `HighContrast`, dark ink on
   `Light` — never literal white) and, when the cell holds a non-whitespace
   character, redraws that character cluster at `caret.min` in
   `palette.terminal_bg`.
   Test: `cursor_290_focused_pane_draws_filled_theme_block_with_inverted_glyph`.
3. When an active terminal pane does not hold keyboard focus and has no active
   IME preedit, the pane draws a hollow `1.0`px inside-stroked outline in
   `palette.terminal_fg` over the same `1 * cell.x` or `2 * cell.x` caret
   rectangle without redrawing the character in `palette.terminal_bg`; a
   non-active session (`Exited` or `Failed`) draws no cursor.
   Test: `cursor_290_unfocused_pane_draws_hollow_outline_and_inactive_session_draws_none`.
4. When a focused terminal pane's cursor is visible on a two-column glyph, the
   claimed `IMEOutput` (`rect` and `cursor_rect`) carries that two-column
   caret rectangle, and moving the cursor or toggling focus between narrow and
   wide glyphs triggers zero rebuilds in `TerminalPaneRenderCache`.
   Test: `cursor_290_ime_anchor_and_galley_cache_follow_wide_caret`.

## Behaviour

- No new user-facing Japanese strings, controls, or notices are introduced; the
  existing empty-output placeholder `ターミナル出力を待っています…` is unchanged
  and draws no cursor because `lines` is `None`.
- **Focused active pane** (`session.status == SessionStatus::Active`,
  `terminal_response.has_focus()`, `preedit.is_empty()`):
  - Cursor cell fill is `palette.terminal_fg` (`#CED4DE` on `Dark`, `#F5F7FA`
    on `HighContrast`, `#24282C` on `Light`), zero corner rounding.
  - Non-whitespace glyph cluster under the cursor (`a`, `あ`, `𠮷`, `が`,
    `葛󠄀`, `🎉`) is painted on top at `caret.min` in `palette.terminal_bg`
    (`#0A0C10`, `#040507`, `#FCFBF8`), giving >= 13.1:1 contrast on every
    theme.
  - Whitespace (`' '`, `'　'`) or an empty cell at/past end-of-line paints the
    filled `palette.terminal_fg` block alone (`1 * cell.x` past EOL or on
    ASCII space, `2 * cell.x` on `U+3000` ideographic space).
- **Unfocused active pane** (`!terminal_response.has_focus()`,
  `preedit.is_empty()`):
  - Paints an unfilled `1.0`px `StrokeKind::Inside` outline in
    `palette.terminal_fg` over the `1 * cell.x` or `2 * cell.x` caret
    rectangle; the underlying row galley keeps its own colour.
- **IME preedit active** (`!preedit.is_empty()`): unchanged — draws
  `draw_terminal_preedit` over `anchor` and suppresses the block/outline
  cursor.
- **Empty, scrolled-away, too-wide, or interrupted/exited states**:
  - Output not yet arrived (`lines` is `None`), `cursor` is `None`, cursor row
    outside the viewport (above or on egui's spare bottom row), or cursor cell
    whose right edge (`column + columns`) exceeds the pane's inner width plus
    `0.5`px float slack: reports `caret: None` and paints nothing.
  - Session `Exited` or `Failed`: paints neither focused nor unfocused cursor.
  - Pathological cluster with dozens of combining marks (> 32 UTF-8 bytes):
    clamped cleanly at the last UTF-8 character boundary `<= 32` bytes without
    panicking or allocating on the row pass.

## Design

Placement: `EXTEND` in `src/ui/terminal.rs` and its single caller in
`src/app/screens.rs` (`git grep -n "draw_terminal_cursor"` names only those two
files).

- **`src/ui/terminal.rs`**:
  - Refine `terminal_character_columns(character: char) -> usize` so control
    codepoints, combining and conjoining marks (`U+0300..=U+036F`,
    `U+1160..=U+11FF`, `U+1AB0..=U+1AFF`, `U+1DC0..=U+1DFF`, `U+20D0..=U+20FF`,
    `U+302A..=U+302D`, `U+3099..=U+309A`, `U+FE20..=U+FE2F`), zero-width
    joiners/spaces (`U+200B..=U+200D`, `U+FEFF`), and variation selectors
    (`U+FE00..=U+FE0F`, `U+E0100..=U+E01EF`) return `0` columns, and
    additional wide kana/ideographic/emoji blocks including four-byte SMP
    ranges (`U+231A..=U+231B`, `U+2329..=U+232A`, `U+FE10..=U+FE19`,
    `U+1B000..=U+1B16F`, `U+1F004`, `U+1F0CF`, `U+1F18E`, `U+1F200..=U+1F2FF`,
    `U+1FA00..=U+1FAFF`) join the existing wide ranges in returning `2`
    columns.
  - Add pure helper `terminal_cursor_cell(line: &str, cursor_x: u16) -> TerminalCursorCell<'_>`
    returning `{ column: usize, columns: usize, text: &str }`: groups each
    non-zero-width base character with any trailing zero-width codepoints; when
    `usize::from(cursor_x)` falls inside `col..col + width`, snaps `column` to
    `col` and returns `columns = width` (`1` or `2`) and the borrowed slice;
    past the end of `line` (or on an empty line), returns
    `{ column: usize::from(cursor_x), columns: 1, text: "" }`.
  - Add `Copy` stack value `TerminalCaretGlyph` (`[u8; 32]` + `u8` length) and
    `pub(crate) caret_glyph: TerminalCaretGlyph` on `TerminalRowsResult`
    (keeping `pub(crate) caret: Option<egui::Rect>` intact for existing
    callers). Inside `terminal_rows`, resolve `terminal_cursor_cell` once on
    `lines[row].text` when `row` is visible, build `rect` with width
    `span.columns as f32 * cell.x`, check containment against the clip and
    `inner_right + 0.5`, and populate `caret` and `caret_glyph` without
    touching `TerminalPaneRenderCache`.
  - Update `draw_terminal_cursor(ui: &egui::Ui, caret: egui::Rect, glyph: &str, focused: bool, palette: &Palette)`:
    when `focused`, paint `rect_filled(caret, 0.0, palette.terminal_fg)` and,
    if `glyph.chars().any(|c| !c.is_whitespace())`, lay out `glyph` via
    `ui.fonts(|fonts| fonts.layout_job(...))` (memoized in egui's font layout
    cache, identical `TextFormat` metrics to `append_terminal_text`) in
    `palette.terminal_bg` at `caret.min`; when `!focused`, paint
    `rect_stroke(caret, 0.0, egui::Stroke::new(1.0, palette.terminal_fg), egui::StrokeKind::Inside)`.
- **`src/app/screens.rs`**:
  - Unpack `caret_glyph` beside `caret` from `terminal_rows` in
    `ui_terminal_pane` and, when `session.status == SessionStatus::Active` and
    `preedit.is_empty()`, call
    `draw_terminal_cursor(ui, caret, caret_glyph.as_str(), terminal_response.has_focus(), palette)`
    for any `Some(caret)` (both focused and unfocused).
- No persisted-shape change, no external command, no background task.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | yes | Uses existing `palette.terminal_fg` and `palette.terminal_bg` roles across `Dark`, `Light`, and `HighContrast` (>= 13.1:1 contrast; `DESIGN.md` and `.claude/rules/palette-and-glyphs.md` read, no colour literal added) |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | No icon or glyph constant is added (`src/glyphs.rs` read) |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | no | No shared identifier string is introduced; column widths reuse `terminal_character_columns` in one place (`src/config.rs` read) |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | No persisted state, sidecar, or tmux lifecycle changes |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | No child process or command construction changes |
| Documentation — user-facing docs change in all three languages together | no | No README prose describes cursor geometry or fill (`CONTRIBUTING.md` read) |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | Pure immediate-mode painting in Rust with zero network calls, zero new dependencies, and zero `unsafe` (`CONTRIBUTING.md` read) |
| Budgets — any new scan or output path states its byte and item ceiling | yes | `TerminalCaretGlyph` is stack-bounded at 32 UTF-8 bytes; `terminal_cursor_cell` scans only the single visible cursor line already bounded by `COMMAND_OUTPUT_MAX_BYTES` |

## Flagged concerns

None.

## Acceptance

- `cargo test --locked cursor_290_` passes: four empirical tests plus one
  `model_check_cursor_290_cell_span_partitions_every_line` exhaustive model
  check.
- `make q.fast` passes.
- `python3 docs/sdlc/changes/290-terminal-cursor-cuts-wide-chars-and-stays-hollow/mutations.py`
  catches every mutation.
- In the packaged app, moving the terminal cursor across English (`abc`) and
  Japanese / four-byte characters (`あ𠮷🎉`) covers one cell on narrow glyphs
  and all two cells on wide glyphs, fills solidly in `terminal_fg` with
  inverted `terminal_bg` text when focused (light on Dark, dark on Light), and
  switches to a 1px hollow outline when unfocused.

## Rejected alternatives

- Hardcode `Color32::WHITE` for the cursor block: violates `DESIGN.md` (no
  colour literals at drawing call sites) and disappears against `Light`'s
  `#FCFBF8` paper terminal background.
- Paint a translucent wash over the character instead of inverting the glyph:
  dims contrast on both Dark and Light themes and departs from xterm /
  Terminal.app / Ghostty / WezTerm standard reverse-video block cursors.
- Keep hiding the cursor completely when unfocused: in Operon's split terminal
  workspace, hiding the unfocused cursor removes the visual cue showing where
  typing will resume and makes a hollow outline redundant.
- Add `unicode-width` to `Cargo.toml`: `Cargo.toml` is a `paused` high-risk
  surface and `terminal_character_columns` in `src/ui/terminal.rs` already owns
  this pane's 1:2 Sarasa column table without an external crate.
