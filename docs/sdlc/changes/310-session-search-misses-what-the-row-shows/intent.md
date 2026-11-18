# Intent: the session search says nothing matches words that are on the row, and the history search says so before it has looked

- **Status**: draft
- **Opened**: 2026-11-18

## Problem

The person's request, verbatim:

「デザイン(UI/UX)や利便性など改善できるポイントを見つけて改善を行って」

The request leaves finding the points to the session. A read of the screens
found two places where a search answers "nothing matches" and the answer is
false, and this change is about those two:

1. **「セッションを検索」 does not look at the title the list shows.** A session
   started without a name or a request is listed under the first line of the
   first thing typed into it. Typing a word of that line into the search hides
   the row and reads 「一致するセッションはありません。」. The same is true of
   the project's name, the branch, and the agent's name as the screen writes it
   (「Antigravity」 is on the row; only its internal word finds it). In the grid
   view a trailing space after the word hides every session.
2. **「ローカル履歴」 reads 「一致する履歴はありませんでした。」 before any search
   has run.** The search starts on Enter or the button, but the sentence appears
   at the first character typed and stays while the search is running.

## Who feels it, and when

- Someone with a dozen sessions who remembers what they asked — the words on
  the row — types them, and is told there is no such session, so they scroll
  the list by eye instead.
- Someone typing a query into 「ローカル履歴」: before they press Enter the panel
  already says there was nothing, and while a slow scan runs it still says so,
  so they stop waiting for a result that was on its way.

## Desired outcome

- Typing words that are written on a session's row — its title as listed, its
  project, its branch, its agent's name — keeps that row and hides the others,
  in the list and in the grid alike, whatever the letter case and with spaces
  around the words.
- 「一致する履歴はありませんでした。」 appears only when a search for the text
  now in the field has finished and found nothing. While a search runs the
  panel says it is searching; before one has run for that text it says nothing.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.37 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese, with English and Korean catalog rows.
- The persisted store is on real machines; this change must not alter its shape.
- The two largest modules are near the 10 000-line band and must stay under it.

## Systems likely affected

`src/app.rs`, `src/app/screens.rs`, `src/i18n_tables.rs`, `src/tests.rs`.

## Open questions

None.

## Not in scope

- The other points the same read found — one-click deletions with no way back,
  dialogs that ignore Esc, copy buttons that say nothing, shortcuts shown
  nowhere. Each is its own change.
- The command palette (⌘K), which already searches titles, projects, and
  branches.
- Results of an earlier history query staying on screen under edited text.
- Clearing a search field with Esc or a ✕.
