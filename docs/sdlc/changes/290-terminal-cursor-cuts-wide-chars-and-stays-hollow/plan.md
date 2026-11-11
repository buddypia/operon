# Plan: industry-standard block cursor width, theme fill, and focus state in the terminal

- **Spec**: `./spec.md`
- **Approved**: 2026-11-12
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/ui/terminal.rs` | Zero-width combining/variation ranges and four-byte wide ranges in `terminal_character_columns`; `TerminalCursorCell`, `terminal_cursor_cell`, and `TerminalCaretGlyph`; `caret_glyph` on `TerminalRowsResult` and two-column `caret` sizing in `terminal_rows`; filled+inverted focused block and 1px unfocused outline in `draw_terminal_cursor` |
| `src/app/screens.rs` | `ui_terminal_pane` unpacks `caret_glyph` and calls `draw_terminal_cursor` for active sessions in both focused and unfocused states |
| `src/tests.rs` | `cursor_290_wide_and_four_byte_glyphs_span_two_cells`, `model_check_cursor_290_cell_span_partitions_every_line`, `cursor_290_focused_pane_draws_filled_theme_block_with_inverted_glyph`, `cursor_290_unfocused_pane_draws_hollow_outline_and_inactive_session_draws_none`, `cursor_290_ime_anchor_and_galley_cache_follow_wide_caret` |
| `docs/sdlc/changes/290-terminal-cursor-cuts-wide-chars-and-stays-hollow/` | `mutations.py`, `verification.yaml`, `state.yaml` |

## Order of work

1. Add the four `cursor_290_*` tests and `model_check_cursor_290_cell_span_partitions_every_line` (routing through `exhaustive_check` as enforced by `every_model_test_goes_through_exhaustive_check`) in `src/tests.rs` against the new `terminal_cursor_cell` / `draw_terminal_cursor` signatures, stubbing only enough for compilation if needed, and watch every new requirement test fail against the current 1-cell hollow-only behaviour.
2. Implement `terminal_character_columns` zero-width and wide range updates, `terminal_cursor_cell`, `TerminalCaretGlyph`, two-column `caret` sizing in `terminal_rows`, and focused/unfocused `draw_terminal_cursor` in `src/ui/terminal.rs`, plus the `ui_terminal_pane` call site in `src/app/screens.rs`.
3. Run `cargo test --locked cursor_290_`, `cargo test --locked model_check_cursor_290_`, existing caret/IME tests (`a_caret_`, `counts_a_cjk_`, `ime_`, `adversarial_varying_content_and_cursor_positions_focus_invariance`), and `make q.fast`.
4. Write `mutations.py` and `verification.yaml`, run the mutation sweep and `empirical-verifier` / `formal-verifier` checks, run `bash scripts/check-bands.sh` and `bash scripts/check-review.sh --index`, and close `state.yaml`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| macOS NFD Japanese (`か` + `U+3099`) or Kanji + IVS (`葛` + `U+E0100`) counts the mark as extra columns and shifts every cursor cell after it | Cursor lands 1–2 cells right of the character after NFD text | `cursor_290_wide_and_four_byte_glyphs_span_two_cells` and `model_check_cursor_290_cell_span_partitions_every_line` |
| Widened 2-cell cursor near the right edge of a resized pane paints 1 cell outside the terminal border | Cursor box pokes past `terminal_border` | Right-edge containment check in `cursor_290_wide_and_four_byte_glyphs_span_two_cells` and mutation of `span.columns` |
| Focused solid block covers the character without redrawing it in `palette.terminal_bg`, or uses a hardcoded white that vanishes on `Light` | Character under cursor becomes unreadable on Dark or invisible on Light | `cursor_290_focused_pane_draws_filled_theme_block_with_inverted_glyph` across all three `AppTheme` palettes |
| Unfocused pane either draws a solid block (hiding which split pane has focus) or draws nothing | Split panes look identical or lose their resting cursor | `cursor_290_unfocused_pane_draws_hollow_outline_and_inactive_session_draws_none` |
| Per-frame cursor move invalidates `TerminalPaneRenderCache` | Extra galley rebuilds on arrow-key navigation | `cursor_290_ime_anchor_and_galley_cache_follow_wide_caret` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked cursor_290_` — 5 passed (`4` empirical + `1` model check).
- `python3 docs/sdlc/changes/290-terminal-cursor-cuts-wide-chars-and-stays-hollow/mutations.py` — every mutation caught.
- In the packaged app: focused cursor is a solid `terminal_fg` block with inverted `terminal_bg` glyph (`1` cell on ASCII, `2` cells on Japanese / 4-byte glyphs; light on Dark, dark ink on Light), and unfocused cursor is a `1`px hollow outline of the same width.

## Departures from the plan
