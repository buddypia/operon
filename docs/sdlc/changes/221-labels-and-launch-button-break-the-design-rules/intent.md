# Intent: the screen names things in ways its own documents rule out

- **Status**: approved
- **Opened**: 2026-10-24

## Problem

A review of the shipped screens (README screenshots, DESIGN.md, the source)
asked "what makes this feel inconvenient?" and the person answered "introduce
everything that is worth it". Three of the findings are small enough to fix on
their own and are wrong by the product's own rules:

- In the Japanese window the Git sub-tabs under a project read `Changes`,
  `Diff`, `History` — English that stays English when the language is switched
  to Japanese or Korean, while every neighbouring tab is translated.
- The group of sessions that are running and waiting for the person's next
  request is called 「待機中」. The same word is already how the notices speak of
  a session queued behind a prerequisite ("キャンセルできるのは待機中のセッション
  だけです"), and it sits beside 「要対応」 and 「応答待ち」, so a person cannot tell
  "idle, ready for my next instruction" from "stuck waiting". (A queued
  session's own status word is a different one; the confusion is between the
  filter and those sentences.)
- The button that starts a session is painted in the colour of whichever agent
  is selected (teal for Codex, orange-brown for Claude). DESIGN.md says the
  button that starts work is always the accent colour and a vendor's colour
  never paints a verb.

## Who feels it, and when

Anyone starting or filtering sessions in Japanese, English or Korean: opening a
project's changes tab and reading its sub-tabs; turning on the 「待機中」 filter
expecting the queued work its notices call 待機中 and getting idle sessions; picking a different agent in
the launch sheet and watching the one button on the sheet change colour.

## Desired outcome

- Every sub-tab under a project's changes reads in the chosen language, and
  the words say what is behind them.
- The word for an idle session is not the word for a queued one, in the
  window and in the documents that name the filter.
- The launch button looks the same whichever agent is selected.

## Constraints this change inherits

- User-facing text is Japanese; every new string has a row in the English and
  Korean tables, in sorted order.
- The three README files and the three manuals change together.
- Colour comes from a palette role, never from a literal.

## Systems likely affected

`src/app.rs` (sub-tab labels), `src/app/screens.rs` (launch button),
`src/ui/widgets.rs` (group words), `src/i18n_tables.rs`, the READMEs and the
manuals, `src/tests.rs`.

## Open questions

None. The wording is proposed in the spec for the evaluator to judge.

## Not in scope

Merging the Changes and Diff sub-views (its own change), renaming the loanwords
`worktree` and `PR`, which neighbouring sentences use throughout, and any other
status word.
