# Intent: the product map shows no real screen, and too little to read or do

- **Status**: approved
- **Opened**: 2026-11-24

## Problem

The person, on the product map after change 315 landed (2026-11-24):

> これだけでは物足りないよ。画面の流れは本物に近いUI/UXにしたいとわかんないよ。他も色々機能不足で使い勝手やそもそも情報量が少ない

Set beside learn_korean's feature map, which the person asked Operon's map to
match (change 315's intent), the map falls short in three ways:

- **The screen flow is boxes, not screens.** Each screen is a rectangle with
  its name and a line to the next. Nobody can tell from it what the screen
  looks like, where on it the button that leads on sits, or what a person sees
  after pressing it. learn_korean draws every screen inside a device frame and
  lets a person press its buttons to walk to the next screen. Operon is a
  desktop application that can be launched apart from the person's data and
  photographed (change 314), so its map can show the real screens rather than
  drawings of them.
- **Too little to read.** A feature is a card and a document; there is no
  overview of the product's health (how many requirements a test pins, which
  screens have no picture, what changed lately), no page per feature that
  gathers its screens, requirements, terms, code and history in one place, no
  page per domain, and no way to see which terms appear on which screen.
- **Hard to use.** Nothing can be linked to (every view resets on reload), the
  browser's back button does nothing, search covers one view at a time, and
  there is no keyboard path through the lists.

## Who feels it, and when

- The person, opening the map to understand or decide on the product: they
  cannot picture a screen, and they ask a session to describe it instead.
- A session asked to change a screen: it reads code to learn what is on the
  screen and where a button leads, because the map cannot show it.

## Desired outcome

What the person's words ask for — 「画面の流れは本物に近いUI/UX」:

- Opening the screen flow, the person sees each screen as the application
  draws it, inside its window, and pressing a button on that picture shows the
  screen the application would show next; 「戻る」 returns. Every way out of a
  screen can be taken from the page — on the picture where its button is on
  the picture, otherwise from the list of ways beside it — so the whole
  product can be walked from ホーム without launching it.
- Every screen of the flow has a real picture, or the page says why it has
  none. Pictures are retaken by one command, and the page says when each was
  taken.

What the session reads 「機能不足で使い勝手やそもそも情報量が少ない」 as, by setting
the map beside learn_korean's (change 315's request asked for that map), each
observable on the page:

- Views that exist afterwards and not now: 概要 (the product's state in counts),
  a page per feature (its pictures, requirements and their tests, terms, code,
  changes), a page per domain, 用語集 by screen, and a canvas of the flow drawn
  with the pictures.
- Every view and item has a link that reopens it; the browser's back and
  forward move through what was opened; one search box finds features,
  screens, domains, terms, requirements and changes; lists move with `j`/`k`.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.37 immediate-mode GUI.
- Local-first: the map is one static file; no server, no network.
- User-facing text is Japanese; code, comments, and docs are English.
- Pictures are taken apart from the person's data (change 314's isolation).

## Systems likely affected

`scripts/change-map.py` and its page, a new capture script beside
`scripts/verify-app.py`, `docs/features/flow.yaml`, pictures under
`docs/images/`, `src/tests.rs`. The application itself is not changed.

## Open questions

None put to the person. Decided by the session, each undone by a revert, and
reported to the person when the change is offered for landing:

- Real pictures rather than drawn wireframes: a picture cannot drift from the
  application the way a drawing does, and the application can be photographed
  apart from the person's data (change 314).
- The pictures are committed (about 27 JPEGs, a few MiB in git history, more
  each time they are retaken) so the map works from a fresh checkout; the map
  file grows by their size.
- The session's reading of 「機能不足」「情報量が少ない」 above; the person can
  strike any of it.

## Not in scope

- Naming icon-only buttons in the accessibility tree. Photographing showed that
  icon buttons (「プロジェクトを開く」 in the history, a project row's marks)
  carry their name only as a tooltip, so VoiceOver and the runner see a
  nameless button. That is a change to the application the person did not ask
  for; it is reported to them as a candidate, not made here.

- Drawing a screen that cannot be reached by pressing controls alone (one that
  needs typed input, a running agent, or a second window): it gets a reason in
  place of a picture.
- Recording a video or an animated walkthrough.
- Editing documents from the map (it stays read-only, like learn_korean's
  except its review packet, which change 315's work basket already covers).
