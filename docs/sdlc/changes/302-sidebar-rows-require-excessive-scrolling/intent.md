# Intent: session sidebar rows and headers require excessive vertical scrolling

- **Status**: approved
- **Opened**: 2026-11-17

## Problem

The person's request, verbatim:
「サイドバーのUIなんだけど、上下をもっと詰めて欲しい。ならならスクロールをたくさんしないといけないからそこを改善したく文字を少し小さくしてもいい。そしてピン留めと中央トップにサイドバーと同じタブの一覧を表示していたはずだが、なんで消えた？ピン留めとかもなくなっているし、どういうことだ。直して」

On the session page, the left sidebar holds the filter band, project headings, and open session rows. Because vertical spacing between rows, inside project frames, around dividers, and between projects is wide and sidebar headings and titles are drawn at `12.5px`, only a small number of sessions fit in the viewport without scrolling.

Separately, the person noticed that session pinning and the center-top tab strip matching the sidebar appeared missing on screen because the running app instance on the machine was an older bundle (`/Applications/Operon Stable.app` from 2026-10-24) rather than the current `/Applications/Operon.app` build; shipping the updated build restores those existing behaviours while keeping them verified under the compact sidebar layout.

## Who feels it, and when

Anyone working across ten or more open sessions or multiple projects on the session page: scanning the sidebar and switching sessions forces frequent vertical scrolling because each row, divider, and project group spends unnecessary vertical height.

## Desired outcome

- Session rows, project headings, filter controls, and gaps between projects in the left sidebar are vertically tighter so noticeably more sessions fit without scrolling.
- Sidebar text labels step down cleanly on the existing type scale (12px for project headings and session titles, 11.5px for column/summary metadata) while staying readable and preserving all hover tooltips, drag-and-drop reordering, and click targets.
- Session pinning (`ピン留め` / `ピン留めを外す`) and the unified center-top workspace tab strip continue to stay in sync with the compact sidebar and are live in the shipped `/Applications/Operon.app` build.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.37 immediate-mode GUI; nothing new per frame beyond painting.
- Every text size and button height stays on `DESIGN.md`'s documented scale and `CONTROL_HEIGHT` / `CONTROL_HEIGHT_SMALL`.
- Colours and icons come strictly from existing `Palette` roles and `ICON_*` constants; no new role, glyph, or persisted store change.
- User-facing text is Japanese; code, comments, and docs stay English.

## Systems likely affected

- `src/app/screens.rs` (session sidebar drawing and filter header)
- `src/ui/widgets.rs` (per-project status count chip sizing)
- `src/config.rs` (sidebar row gap constant)
- `src/tests.rs` (layout density, type-scale, and pin/sync guards)

## Open questions

None.

## Not in scope

- Rewriting or deleting `/Applications/Operon Stable.app` without the user's confirmation.
- Changing the design system (`DESIGN.md`) palette, introducing non-scale font sizes, or redesigning the center-top tab strip or right inspector dock.
