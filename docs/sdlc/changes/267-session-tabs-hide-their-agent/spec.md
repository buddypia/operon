# Spec: session tabs show which CLI runs in them

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. R1: a session tab draws its CLI's mark between the state mark and the title, on the same row: the vendor mark for `claude`, `codex`, and `gemini` (Antigravity), the terminal mark for `terminal` and `existing terminal`. A session whose agent has neither (a custom command, an older record naming another binary) draws no mark and reserves no space, so its title follows the state mark as today. `session_tabs_show_the_cli_mark_after_the_state` renders the workspace and finds each mark's texture between the state mark and the title of its own tab, and none in the custom tab.
2. R2: the terminal mark is a bundled 128x128 PNG drawn as the same rounded square as the vendor marks — opaque at its centre, transparent at its corners — and the session mark lookup maps exactly the two terminal kinds to it, the three vendors to their own marks, and everything else to nothing. `the_terminal_mark_is_a_rounded_square_for_terminal_sessions_only` pins the asset's shape and the mapping.
3. R3: hovering a tab's mark names the CLI with the same words the left column uses (`Antigravity`, `Claude`, `Codex`, `ターミナル`, `復旧ターミナル`). `session_tab_mark_names_its_cli_on_hover` hovers an Antigravity and a terminal mark and reads the tooltip.
4. R4: everything a tab already did keeps working — state mark and its hover, truncated title, click to activate, double-click rename, drag to move, close, document tabs. `workspace_session_tab_shows_its_status_glyph` and the `workspace_pointer_` tests pin it.

## Behaviour

A tab reads `[state mark][CLI mark] セッション2 [×]`. The CLI mark is 16 points square, the size the left column draws a vendor mark beside 12-point text, and vertically centred on the tab row. Hover shows the CLI name only; the state's own hover is unchanged. No Japanese sentence is added: the hover words are the existing `agent_short_label` strings (`ターミナル`, `復旧ターミナル` already have rows in every table).

Empty: no sessions, no tabs. A session removed while its tab is open keeps the existing blank fallback, with no mark. Too many tabs: the strip scrolls horizontally as today, and the mark scrolls with its tab. An image that fails to decode draws no mark, the same fallback the vendor marks already use. Nothing here can error, so no notice is added. The mark follows the session's stored agent kind, which does not change while the session lives.

## Design

Placement: EXTEND. The tab strip in `ui_terminal_workspace` (`src/app/screens.rs`) already draws `session_status_icon` before the title; the vendor marks already exist (`agent_icon_texture` in `src/agents.rs`, uploaded once through `png_texture` in `src/ui/widgets.rs`).

- `assets/terminal-mark-128.png` — new, Operon's own artwork (a light plate holding a dark screen with a green `>` and a light `_`), rendered from `assets/terminal-mark.svg`, which is committed beside it so it can be regenerated. It lives outside `assets/agent-icons/`, whose PNGs `THIRD_PARTY_NOTICES.md` declares third-party; it is covered by the MIT License like `operon-mark-128.png`.
- `src/glyphs.rs` — `TERMINAL_MARK` (the `include_bytes!`) and `SESSION_MARK_SIZE` (16.0), documented beside `BRAND_MARK`.
- `src/ui/widgets.rs` — `session_mark_texture(ctx, agent)`: the vendor mark from `agent_icon_texture`, else the terminal mark for `terminal` / `existing terminal`, else `None`; and `session_mark(ui, agent)`, which draws it at `SESSION_MARK_SIZE` with the `agent_short_label` hover, drawing and reserving nothing when there is no mark.
- `src/app/screens.rs` — the tab calls `session_mark` right after `session_status_icon`.
- `src/agents.rs` is not edited: `agent_icon_asset` keeps meaning "vendor mark", so the launch-gate surface and its formal-proof obligation are not touched, and the left column, the CLI filter, and the launch sheet keep drawing exactly what they draw today.
- Draw path: one `ctx.data` lookup and an `Arc` clone per tab per frame, the cost the left column's rows already pay; the decode and upload happen once per process. No I/O, subprocess, background task, persisted shape, or dependency.
- `DESIGN.md` gains a sentence on the terminal mark next to the vendor marks; `assets/agent-icons/SOURCES.md` is unchanged.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — `DESIGN.md` | Yes | No colour is painted from code; the new mark is artwork, like the vendor marks and `BRAND_MARK`, and the tab keeps its palette roles. |
| Icons — `src/glyphs.rs` | Yes | No Phosphor glyph is added; the image constant lives in `src/glyphs.rs` beside the other marks and is documented in `DESIGN.md`. |
| Controls and type scale — `DESIGN.md` | Yes | No button or `.size(…)` is added; the image size is a named constant. |
| Identifier SSOT — `.claude/rules/identifiers.md` | Yes | The agent kinds are matched by the same literals `agent_short_label` matches; no new identifier is introduced and no file in that rule's scope is edited. |
| Durability | No | No store, cancellation path, migration, or byte limit. |
| Subprocess safety | No | No child process or launch path; `src/agents.rs` is not edited. |
| Licensing — `THIRD_PARTY_NOTICES.md` | Yes | The new artwork is Operon's own and sits outside `assets/agent-icons/`, so the third-party row stays accurate. |
| Local-first — `CONTRIBUTING.md` | Yes | Bundled asset; no network, telemetry, or account. |
| Draw path — `.claude/rules/rust.md` | Yes | Cached texture, no per-frame decode, allocation, or I/O. |

## Flagged concerns

None.

## Acceptance

R1–R4 tests pass and each new guard is watched failing; a mutation sweep catches a removed tab mark, a terminal mapped to nothing, a custom command given the terminal mark, and a hover that names nothing. `make q.fast` passes. CI passes before merging into `main`. After landing, package from `main` and swap the installed app per `.claude/skills/ship/SKILL.md`.

## Rejected alternatives

- Adding the terminal to `agent_icon_asset`: one lookup, but it edits `src/agents.rs` (launch-gate surface) and silently changes the left column, the CLI filter, and the launch sheet, which the person did not ask for.
- A Phosphor terminal glyph: one stroke weight of outline next to three full-colour app icons reads as a different kind of thing; the person asked for an icon and authorised generating one.
- Drawing the CLI name as text in the tab: change 253 removed exactly that repetition for width.
- Overlaying the state as a badge on the CLI mark: smaller state marks and a new compositing shape for a gain of 18 points.
