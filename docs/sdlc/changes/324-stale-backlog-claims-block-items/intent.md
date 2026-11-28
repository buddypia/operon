# Intent: an item held by a session that is gone is never handed out again, and the backlog command alone does not start work

- **Status**: approved
- **Opened**: 2026-11-27

## Problem

Two requests, both about the backlog change 323 introduced.

The person's request, as written (2026-11-27), after a bare `/backlog` only
listed the items:

> 次をやって。次からは/backlogだけで優先度の高いイシューを対応するようにして

The item the session then took, issue #2 of the repository, as filed by
change 323:

> 着手したセッションが終了しても着手コメントは残り、誰かが release するまで
> 項目は配られない。着手から一定時間たち、対応するブランチも作業ツリーも無い
> 項目を戻す手段が要る。

Today the backlog command given alone is read as "show what is left": the
session lists the items and stops, and the person has to ask a second time for
the work to begin. And a session that took an item and then ended — closed,
crashed, ran out of context — leaves the item marked as taken for good. Nobody
is working on it, but every later request for the next item skips it, until a
person notices and gives it back by hand.

The two meet in one situation: the person runs the backlog command again and
again, or in a loop, to work through the list unattended. The first makes each
run do nothing; the second makes the list shrink by every item a dead session
touched.

## Who feels it, and when

- The person, typing the backlog command alone and expecting the most valuable
  item to be worked on, who instead gets a list and must ask again.
- The person or a loop asking for the next item a day after a session died
  holding one: that item is skipped every time, though nobody works on it.

## Desired outcome

- The backlog command given alone takes the highest-scored item nobody holds
  and carries it out. The list alone is shown only when the person asks what
  is left.
- An item taken more than a day ago by a session that left no working copy and
  no commit for it is handed out again on the next request for the next item,
  and the item says that it was given back and why.
- An item someone is still working on — a working copy exists for it, or a
  commit names it — is never given back this way. (Not in the request's words;
  the request's "no branch, no working copy" condition stated as what must not
  break.)

## Constraints this change inherits

- User-facing text is Japanese; code, comments, and docs are English.
- The backlog command runs on the main copy too, which runs only a fixed list
  of commands. That list does not change here.
- The repository is public: nothing written to an item names a host, a person
  or a session.

## Systems likely affected

The backlog script and skill from change 323, and the test suite. None of the
application's modules.

## Open questions

None. How long "a day" is was already the skill's manual rule (a claim past a
day whose working copy is gone); this change applies it without a person.

## Not in scope

- Giving back an item whose working copy still exists but whose session is
  gone. A working copy is visible to the person and may hold work.
- A heartbeat or any record of which session is alive.
- Showing the backlog inside the Operon application.
