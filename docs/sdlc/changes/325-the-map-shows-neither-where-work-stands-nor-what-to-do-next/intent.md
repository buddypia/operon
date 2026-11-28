# Intent: the product map shows neither where work stands nor what to do next

- **Status**: approved
- **Opened**: 2026-11-28

## Problem

The person, on 2026-11-28, with Operon's product map (changes 312–316) beside
learn_korean's feature map:

> ~/dev/buddypia/learn_korean <- 의 기능지도처럼 확실하게 요구사항, 스펙, 문서 상세 내용을 볼 수 있도 전체적인 UI프레임과 어느방식으로 진행되는지 가시화 등 전반적으로 learn_korean처럼 기능지도의 기능을 대부분만들어줘. flutter라서 모바일이 아닌 웹과 내 프로젝트(operon)에 맞게 변형해서 최대한 기능을 알맞게 이식하고 싶어.

learn_korean's map was opened on this machine the same day and set beside
Operon's, view by view. Operon's map already has an overview of counts, a page
per feature, real pictures of the screens that can be walked, domains, a
glossary, a requirement table, the change list and a basket. What
learn_korean's has and Operon's does not:

- **Where things stand, on one screen.** learn_korean opens on 현황: how many
  features are done, moving, waiting for a person or stuck; a row of stages
  with how many sit in each; cards of what a person must handle, most urgent
  first; what is under way with its next step; progress by domain; who is
  working on which branch. Operon's overview counts things but does not say
  where the open work is or what the person has to do.
- **Finding by state.** learn_korean's feature tiles carry chips (stage,
  review, release) with a count on each; pressing one leaves only those
  features, several can be combined, and the choice is in the link. Operon's
  feature list cannot be narrowed at all.
- **A frame to move in.** learn_korean keeps a list of every feature down the
  left, grouped by domain with its progress, so a person steps from feature to
  feature without going back to a list. Operon has no such list.
- **A work board.** learn_korean shows what is left to do, what is being
  done, what waits for the person's check and what was finished this week, as
  columns of cards. Operon keeps the same facts (the backlog of change 323,
  the changes under way, the ones waiting for a person) and shows none of them
  as a board.
- **Depth on a feature's page.** learn_korean's page has a rail of which
  documents exist, the requirements as cards, the design (states, errors), a
  code map by layer where a file can be picked, the review state and the
  decisions. Operon's page shows the brief, the requirement lines and a list
  of modules; a spec's states, invariants, decisions and named code are
  reachable only by opening the whole document.
- **How work proceeds.** The person asks for 「어느방식으로 진행되는지 가시화」.
  Operon's pipeline (which kinds of request pass which stages, how many
  attempts, what evidence, which files make a session stop and ask) is written
  in three tables nobody sees on the map.
- **Handing a selection to a CLI session.** learn_korean's basket keeps what
  was picked across reloads, takes a sentence of what to do, offers kinds of
  work, lists the files to read and shows the text before it is copied; a
  verdict (fine, fix, hold) and a note per feature are collected and copied as
  one packet. Operon's basket forgets on reload, takes no sentence, and there
  is no verdict.
- **The design system.** learn_korean has a component and an icon catalogue.
  Operon's colours, marks and keys are tables in the source that the map does
  not show.
- **Always current.** learn_korean's map is served by a local server that
  rebuilds it from the documents on every load. Operon's is a file that is as
  old as the last time a session ran the script.

## Who feels it, and when

- The person, opening the map to decide what happens next: they ask a session
  what is in flight, what waits for them and what is left, because the map
  does not say.
- The person reviewing the product feature by feature: they lose their place
  on every reload and have nowhere to put a verdict.
- A session handed a copied instruction: it gets the feature's name and two
  documents, and searches for the files itself.

## Desired outcome

Each observable on the map, opened in a browser. The person's own phrases
are 「요구사항, 스펙, 문서 상세 내용을 볼 수 있도」 (the fourth line below),
「전체적인 UI프레임」 (the second) and 「어느방식으로 진행되는지 가시화」 (the first
and the fifth). Every other line is the session's reading of 「기능지도의
기능을 대부분」, taken from setting the two maps side by side; the person can
strike any of them.

- The first view says, without scrolling far, how many features are steady,
  changing and waiting for the person; where every open change is among the
  pipeline's stages; what the person has to handle, with each item one press
  from its record; what is under way and its next step; each domain's state;
  and which branch holds which change.
- The feature list is tiles with their pictures, narrowed by chips whose
  counts follow the other chips, with the choice kept in the link; a list of
  every feature stays down the left of every view.
- A work board shows the backlog, what a session holds, the changes under
  way, what waits for the person and what was finished lately, each card in
  one column, narrowed to the person's or the sessions' cards.
- A feature's page shows which of its documents, pictures, tests and
  in-application checks exist as a rail; its requirements as cards; its
  states, invariants and decisions; its code by layer with every file
  pickable; and what evidence its latest changes landed on.
- A view draws the pipeline: the stages, each kind of request across them
  with its ceilings and evidence, and the files that make a session stop and
  ask, with how many changes took each route.
- A view lists every colour role in the three themes, every mark and every
  key the application defines.
- The basket survives a reload, takes a sentence, offers kinds of work, shows
  the text and its size before copying, and names the files to read; a
  verdict and note per feature survive a reload and are copied as one text
  for a CLI session.
- A session can start the map as a local page that is rebuilt on every load.

## Constraints this change inherits

- Local-first: the page loads nothing from the network; a served map listens
  on this machine only. The backlog is on GitHub since change 323 and is read
  through that change's own script, only when asked.
- User-facing text is Japanese; code, comments, and docs are English.
- The map writes nothing back: no ledger, no second record of a change's
  position (`.claude/skills/feature-pilot/SKILL.md`).
- Every scan and every child the map starts has a byte, item and time ceiling.

## Systems likely affected

`scripts/change-map.py` and its page under `scripts/change-map/`,
`docs/features/README.md`, `src/tests.rs`. The application itself is not
changed.

## Open questions

None put to the person. Decided by the session, each undone by a revert, and
reported to the person when the change is offered:

- 「대부분」 is read as the list above. Left out, as what 「flutter라서 모바일이
  아닌 웹과 내 프로젝트(operon)에 맞게 변형」 excludes: phone frames and drawn
  screen models (Operon's map shows real pictures since change 316), Flutter
  component goldens, localisation files, and the server-side sections (edge
  functions, tables, row policies) Operon has no counterpart for.
- learn_korean's stages of a feature (briefing … released) have no
  counterpart: an Operon feature is described as it is on main, and what moves
  is a change. The stage row therefore counts changes, and a feature's state
  stays 安定 / 変更中 / 判断待ち with its tested share.
- learn_korean's board plans by week through GitHub milestones and writes to
  GitHub from the page. Operon's backlog is ordered by score and has no weeks,
  and the page stays read-only: a card offers the instruction to copy for a
  session, which does the writing through the backlog script.
- What the person picks on the page — the basket, a verdict and its note, the
  chosen theme — is kept in the browser only, as learn_korean keeps it. It is
  never written to the repository or to a change record, says nothing about a
  change's position, and is lost with the browser's profile. A revert removes
  the code that reads it and leaves a few unused entries in the browser.
- Change 316 held the map to one static file with no server. The file stays,
  and works as before, opened from disk. Beside it a session may start the
  same page from a process that listens on this machine only and stops when
  the session stops it; nothing starts it unasked. A revert removes the
  option and leaves the file.
- The marks are listed by name and use; drawing them needs the icon font,
  which is not in the repository.

## Not in scope

- Any change to the application, to the backlog script, or to a gate.
- Writing to GitHub, to a change record or to a feature document from the map.
- Weekly planning, due dates and milestones for backlog items.
- Drawing a screen that has no picture.
