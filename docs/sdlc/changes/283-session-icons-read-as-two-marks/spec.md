# Spec: a session leads with one mark, its state a dot on it

- **Intent**: `./intent.md`
- **Status**: draft

## Requirements

1. A session tab whose CLI has a mark draws that mark alone before its title:
   no circled state mark beside it. Its state is a filled dot centred near the
   mark's lower right corner, in `Palette::status` of the state's tone, inside
   a ring of the tab's own fill so it stays separate from the artwork. —
   `session_283_tab_leads_with_one_mark`
2. A plain terminal (`terminal`, `existing terminal`) in the state `Running`
   draws no dot; in any other state it does. An agent draws a dot in every
   state. — `session_283_running_terminal_says_nothing`
3. A session that needs the person (`Waiting`, `Blocked`) draws a larger dot
   with an exclamation mark in it; no other state draws the exclamation. —
   `session_283_waiting_dot_keeps_a_shape`
4. A row in the left column draws the same mark and dot as the tab — the
   terminal mark for a terminal instead of the word 「ターミナル」 — and no
   state glyph of its own. A session with no mark keeps its state glyph and
   its label. — `session_283_column_row_matches_its_tab`
5. Hovering a tab's mark names the CLI, the state, and the state's
   explanation. — `session_283_mark_names_cli_and_state_on_hover`
6. A session with no mark (a custom command) keeps today's circled state mark
   in its tab. — covered in `session_tabs_show_the_cli_mark_after_the_state`.

## Behaviour

- Tab: `[mark with dot] タイトル ×`. The tab is one mark plus its gap (18px)
  narrower than before.
- Left column row: `[mark with dot] タイトル`, the mark at 16px as the vendor
  marks there are today.
- Dot: 5px across (at 14px), 6px at 16px; the waiting dot 7px / 8px with the
  exclamation in the surface colour. A 1.5px ring in the surface colour.
- Running terminal: the mark alone. Its state is still in the header line and
  in the column's counters.
- Hover on the tab mark: 「ターミナル」/「Claude Code」…, then the state word
  (「実行中」…) and its explanation, as the tab's state mark said before.
- Hover on the column mark: the CLI name and the state word.
- Empty, error, too-large, interrupted: none — this is painting only.

## Design

Placement: **EXTEND** — `session_status_icon` and `session_mark`
(`src/ui/widgets.rs`) draw the two marks; their callers are the tab strip
(`src/app/screens.rs`, `workspace tabs`) and the column row (`src/app/screens.rs`,
the session rows under each project). Route `modify`.

- `src/ui/widgets.rs`: `session_mark_speaks(agent, kind) -> bool` (R2);
  `session_badge(ui, palette, agent, kind, size, surface) -> Response`, which
  falls back to `session_status_icon` when `session_mark_texture` has none,
  otherwise paints the texture and, when it speaks, `paint_status_dot`.
  `session_mark` is removed (its one caller goes).
- `src/glyphs.rs`: `COLUMN_SESSION_MARK_SIZE = 16.0`, and the dot's
  proportions `STATUS_DOT_RATIO`, `STATUS_DOT_ATTENTION_RATIO`,
  `STATUS_DOT_RING`.
- `src/app/screens.rs`: the tab draws `session_badge` with the tab's fill as
  the surface and one hover; the column row draws `session_badge` with
  `row_selected` or `panel` as the surface when the session has a mark, and
  today's glyph and label otherwise.
- `DESIGN.md`: the session-tab and terminal-mark paragraphs describe the dot.
- `README.md`, `README.ja.md`, `README.ko.md`, `docs/MANUAL.md`,
  `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md`: "starts with its state mark"
  becomes "starts with its CLI's mark, its state a dot on it".
- No persisted shape, no child process, no background work; `session_mark_texture`
  is the cached texture the tab already reads each frame.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | Yes | The dot reads `Palette::status(tone)` and the surface roles `card`, `row_selected`, `panel`; no new role. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | Yes | No glyph: the dot and its exclamation are geometry, like `StatusMark`; `DESIGN.md` describes them. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | The terminal agent names are matched where `session_mark_texture` already matches them, beside it. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | Nothing is stored. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No child, no `unsafe`. |
| Documentation — user-facing docs change in all three languages together | Yes | README and manual in English, Japanese and Korean. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | No | Painting only. |
| Budgets — any new scan or output path states its byte and item ceiling | No | None. |

## Flagged concerns

- **Colour carries most of the state at dot size** — `DESIGN.md` says colour
  may not carry a meaning alone. Resolved: the state that asks for action keeps
  a shape (R3); the rest are named on hover, in the header line, and counted in
  the column's four counters, and a session with no mark keeps its full mark.

## Acceptance

- `cargo test --locked session_283_` passes (five tests), and the rewritten
  `session_273_tabs_are_compact`, `session_tabs_show_the_cli_mark_after_the_state`,
  `session_tab_mark_names_its_cli_on_hover`,
  `workspace_session_tab_shows_its_status_glyph` and the tab state vocabulary
  test pass.
- `make q.fast` passes.
- `python3 docs/sdlc/changes/283-session-icons-read-as-two-marks/mutations.py`
  catches every mutation.
- In the packaged app the tab strip and the left column show one mark per
  session, as `screen.md` draws.

## Rejected alternatives

- Tabs carry only the state — the tab would no longer say which CLI it is.
- Keep both marks and dim the state one — still two marks competing, at a
  contrast below what the state needs.
