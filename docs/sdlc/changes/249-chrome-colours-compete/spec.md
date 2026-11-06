# Spec: neutral dark application chrome

- **Intent**: `./intent.md`
- **Status**: draft

## Requirements

1. Dark theme structural surfaces, selection, borders, text, accent fills/inks and branch metadata are achromatic; primary and selection controls remain distinct and readable. Test: `dark_chrome_uses_neutral_roles` and `every_theme_stays_readable`.
2. Semantic status, links, diffs, agent identity, terminal ANSI colours, Light and High Contrast retain their existing roles and values. Test: `neutral_chrome_preserves_semantic_colours` and `ansi_numbers_map_to_the_colours_they_name`.
3. DESIGN.md matches the implemented palette and explains neutral dark hierarchy. Test: `design_md_documents_exactly_what_the_app_paints`.

## Behaviour

The existing 「新しいセッション」 button uses a light grey fill and dark label; 「セッション」 and 「ファイル」 selections use dark grey fills with light text and focus borders. App name and branch metadata lose orange/blue tint. No layout, labels or navigation changes. Empty/loading/interrupted/error/too-large states keep current content and status colours. CLI output inside terminals remains controlled by the CLI.

## Design

EXTEND: CLAUDE.md maps `src/theme.rs` to Palette and its three tables; DARK_PALETTE currently assigns orange accent and blue selection. Change only existing dark roles and their DESIGN.md values. No persisted changes, new modules, children, background work or draw-path computation. Keep light and high contrast palettes unchanged to preserve their dedicated accessibility behaviour. Use #383838 for selection, #D0D0D0 for primary fill, #202020 on-accent, #E0E0E0 accent text, #C8C8C8 accent soft/branch, #DCDCDC selection border. Neutralize dark regular text channel differences as well.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | yes | Existing roles only; synchronize DESIGN.md; existing AA tests check all inks and surfaces. |
| Icons | no | No glyph or vocabulary changes. |
| Identifier SSOT | no | No identifiers introduced or duplicated. |
| Durability | no | No store, migration, cancellation or budget changes. |
| Subprocess safety | no | No process code changes. |
| Documentation | yes | DESIGN.md is the technical palette owner; no user README text changes. |
| Local-first | yes | No dependencies, network code, unsafe or telemetry. |
| Budgets | no | No new scan, output or computation. |

## Flagged concerns

None. The user explicitly requested neutral dark appearance. Neutralising terminal ANSI or error colours would remove useful meaning and is outside this request.

## Acceptance

- Named requirement tests pass; make q.fast and bands check pass; mutation sweep catches orange accent and blue selection returning.
- Render existing toolbar and selected controls at desktop width, inspect neutral selection, primary button, label readability and focus outline.
- CI passes before merge; package from main and verify installed build (record an install blocker explicitly if running app cannot be closed within authorization).

## Rejected alternatives

- One saturated accent throughout: does not satisfy black/grey chrome request.
- Greyscale terminal/status/diff colours: loses semantic information.
- Rearrange UI: unrelated to the requested palette.
