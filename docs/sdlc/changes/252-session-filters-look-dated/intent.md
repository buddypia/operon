# Intent: The sidebar's session filters look dated and crowd the list

- **Status**: approved
- **Opened**: 2026-11-07

## Problem

The person said, of the session page's left column: 「左のソートのUI/UXを改善したい。プロデザイナー視点でどのようにUI/UXをしたらより使いやすくモダンなデザインと利便性を高めることができるか考えて進めて。機能自体は良いと思うけどダサい」.

The CLI and status filters that change 251 added work, but they are eleven framed grey buttons on four rows under two headings. They sit above every session, take roughly the height of four session rows, show no counts, carry two separate 「すべて」 choices, and repeat 「承認・回答待ち」 inside 「要対応」. Selected and unselected choices differ only by a slightly different grey.

## Who feels it, and when

Every time the person looks at the session page to find which session needs them, or to narrow the column to one CLI: the filters are read before the sessions, and nothing in them says how many sessions a choice would leave.

## Desired outcome

After comparing three rendered proposals, the person chose proposal A and folding 「承認・回答待ち」 into 「要対応」. Afterwards the column shows the four status choices as a two-by-two block, each with its state's mark, colour and current count; the CLI choices are one row of official marks; and a single clear action with the number of shown sessions appears only while something is filtered. Filtering keeps working exactly as it does today otherwise.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.
- Colours, glyphs, sizes and controls come from `DESIGN.md` roles and helpers.

## Systems likely affected

`src/app/screens.rs` (sidebar filters), `src/app.rs` (filter state), `src/ui/widgets.rs` (filter controls), `src/i18n_tables.rs`.

## Open questions

None: the proposal and the folded waiting choice were chosen by the person on 2026-11-07.

## Not in scope

The history list's own status chips, the project rows below the filters, persistence of the filter, and any new filter dimension.
