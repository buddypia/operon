# Intent: The tab-strip + starts only a plain terminal, and the tabs take too much room

- **Status**: approved
- **Opened**: 2026-11-09

## Problem

The person asked:

> - セッションのタブ名の右側の+ボタンを選択したらターミナル、Claude Code, Codex CLI, Antigravity CLIのアイコンが出て選択したら起動するようにして。
> - セッションタブ名の一覧がちょっと大きい気がする。

The + at the end of the session tabs always opens a plain terminal. To start
Claude Code, Codex CLI or Antigravity next to the tabs already open, the person
has to leave the tab strip for the launch sheet, fill it in, and come back. The
place where a new tab obviously goes offers only one of the four things the
person starts there.

The session tabs themselves are roomier than their content needs: each one is
about 170pt wide and 32pt tall for a state mark, a CLI mark, a short name and a
close mark, so three or four tabs already fill much of a pane and the strip
reads heavier than the terminal under it.

## Who feels it, and when

The person working in the Sessions workspace who wants one more agent or
terminal beside the ones already open in a pane — several times a session —
and who reads the tab names to switch between them.

## Desired outcome

- Pressing the + after the last tab shows the four choices — ターミナル,
  Claude Code, Codex CLI, Antigravity — each with its mark. Choosing one starts
  it straight away as a new tab in that pane, in the same project and folder as
  the pane's current tab, without opening the launch sheet.
- A CLI that is not installed on this Mac is visibly not available, and choosing
  it starts nothing.
- The session tabs are visibly smaller — less height and less width per tab —
  while the state, the CLI, the name and the close mark all stay readable and
  clickable.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.
- A launch whose options need the danger acknowledgement keeps needing it.

## Systems likely affected

The terminal workspace drawing in `src/app/screens.rs`, session launch in
`src/app.rs`, the status and CLI marks in `src/glyphs.rs` and `src/ui/widgets.rs`,
strings in `src/i18n_tables.rs`, `DESIGN.md`, the three READMEs, tests in
`src/tests.rs`.

## Open questions

Put to the person on 2026-11-09 with previews; no answer within ten minutes, so
the recommended defaults were adopted (recoverable, recorded in `state.yaml`):

- The menu's shape — a vertical list with each CLI's mark and name
  (recommended, adopted) or a single row of marks named on hover.
- How much smaller — margins and gaps tightened plus both marks at 14pt
  (recommended, adopted) or margins and gaps only.

## Not in scope

- The launch sheet, its options, and the toolbar's 新しいセッション.
- The sidebar's per-project + and the document tabs.
- A custom CLI in the menu: it has no launch without its typed command.
