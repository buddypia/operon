# Intent: the feature map names no domain, no term, and no path between screens

- **Status**: approved
- **Opened**: 2026-11-23

## Problem

The person's request (2026-11-23), about the feature map of another project of
theirs, learn_korean:

> ~/dev/buddypia/learn_korean <- ここの機能地図でSPEC/BRIEF/PRD/CONTEXTなど文章からドメイン、用語集、機能別、画面の流れなど全て実装してSSOTとして活用できるように時間をかけて完璧に作る必要がある。

and, when that was first read as work inside learn_korean:

> learn_koreanの機能地図を今のプロジェクトへ移行してってことだよ

Operon's product map (changes 312 and 313) shows the product by screen, each
feature with a Japanese brief, an English spec and its change history. Set
beside learn_korean's map, four things a person and a session lean on there
are missing here:

- **Domains.** Features are grouped only by the screen they appear on. Nothing
  says which areas of responsibility the product is made of, where one ends and
  the next begins, or which features and code belong to each — so "is this the
  session side or the git side" is answered by reading code.
- **Terms.** Operon's words — 要対応, セッション, worktree, ネイティブ再開,
  チェックポイント, ハンドオフ and dozens more — are defined nowhere. A person
  reading a brief, and a session writing a new label, each guess what a word
  means and which word the product already uses for a thing; nothing says which
  words the product avoids.
- **The path between screens.** Each feature names its screen, but nothing shows
  how a person moves from one screen to another — which key, which button,
  which page or sheet comes next — so the product's whole shape cannot be
  followed, and a change that adds a page or a way in has nothing to keep true.
- **Requirements across features.** Each spec numbers what its feature does now,
  but they can be read only one feature at a time; "which behaviours does the
  test suite pin, and which does nothing pin" cannot be asked of the whole
  product, and picking a few of them to hand to a session means copying paths
  by hand.

None of it is checked either: a word, a screen or a way between screens could
be written once and silently go stale.

## Who feels it, and when

- The person, opening the map to understand or review the product: they want to
  move from the whole (domains, the screen flow) to a part (a feature, a screen,
  a term) and to its documents without leaving the page.
- A session starting a change: before naming a new button or placing new
  behaviour it needs the domain the work belongs to, the words already in use,
  and the screens and ways in it will touch.
- The person handing work to a session: today they name files and documents by
  hand; in learn_korean they pick items on the map and copy an instruction.

## Desired outcome

- On the same page, a person can see the product's domains, each with what it
  owns and does not, its neighbours, its features and code, and the changes that
  touched it.
- A glossary lists every product term with its meaning, the screen words and
  code names it goes by, the feature it belongs to and the words to avoid; a
  feature and a screen show the terms they use.
- A screen-flow view draws every screen, sheet and overlay a person reaches and
  every way between them (key, button), and choosing a screen shows its
  features, its ways in and out, its terms and its picture.
- A requirement board lists every numbered behaviour of every feature with the
  tests that pin it, filterable, so what nothing pins is visible.
- A person can pick features, screens, behaviours or terms and copy one
  instruction for a CLI session that carries their documents and paths.
- These are the source of truth the pipeline reads: a session starting a change
  reads the domain and the terms; a change that adds a screen, a way between
  screens, or a user-facing word the glossary forbids is caught by the suite.
- Every claim these documents make that a machine can check — a feature, a code
  name, a screen word, a key — is checked, so they cannot silently drift.

## Constraints this change inherits

- User-facing text is Japanese; code, comments, and docs are English. What the
  person reads on the map is Japanese.
- Change 312 and 313's rules: the page is generated and not committed, and no
  second record of a change's position is kept — progress stays derived.
- Local-first: the page loads nothing from outside.

## Systems likely affected

`docs/features/`, `scripts/change-map.py`, `src/tests.rs`, the feature-pilot and
sdlc skills, `docs/sdlc/README.md`.

## Open questions

None put to the person. Two readings of the request are decided by the managing
session and recorded in `state.yaml`: learn_korean's per-feature state file is
not ported (Operon derives state from change records), and there is no separate
PRD (learn_korean merged its PRD into SPEC and CONTEXT; the brief is the
person's document here).

## Not in scope

- learn_korean's Flutter-specific parts: ARB keys, widget mock-ups drawn from a
  component catalogue, GoRouter route extraction, RICE priorities.
- A review-freshness ledger per feature (learn_korean's spec-hash review packets):
  approvals stay per change in `approvals.log`.
- Changing the application, its manuals, or any user-facing string.
- Translating the briefs or the glossary into English or Korean.
