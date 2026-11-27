# Intent: work a session names as left over is lost once its completion report scrolls away

- **Status**: approved
- **Opened**: 2026-11-27

## Problem

The request, as the person wrote it (2026-11-27):

> AI가 생물을 생성하고 완료 출력을 할때 남은 일이 뭐라고 출력하는데 그걸
> backlog같 써서 SSOT로 github issue를 연결해서 다중 worktree로 작업을 해도
> 문제가 없는 방식으로 만들고 싶은데. 1. 그걸 실행하는 CLI와 래퍼스킬이 있어?
> AI한테 바로 스킬만 실행시키면 바로 실행되어야 해. 2. AI가 작업완료 후에 남은
> 작업/개선점/문제점 등을 ICE프레임워크 같은 방식으로 저장하고 그걸 스킬을
> 사용해서 우선순위가 높거나 가치가 높은걸 우선으로 실행되게 해야 해.

The first question has a plain answer: no. Nothing in this repository or among
the person's installed skills records leftover work, and the repository has no
issue at all.

When a session finishes a change it says what remains: work it did not get to,
an improvement it noticed, a problem it found and left. That sentence exists
only in the final message of one conversation. The next session does not see
it, the person has to remember it or scroll for it, and nothing says which of
twenty such remarks is worth doing first.

Sessions also run side by side, each in its own working copy. A list kept as a
file in the repository would differ between copies until each lands, two
sessions would add to it and collide, and two sessions asked for "the next
thing" would both start the same item.

## Who feels it, and when

- The person, after several changes have landed, wanting to know what was left
  undone and which of it matters most, without rereading old conversations.
- A session told to do the most valuable thing next, which today has nothing
  to choose from.
- Two sessions working side by side, each told to take the next item.

## Desired outcome

- What a session leaves undone at the end of a change is kept as an issue on
  the repository's GitHub page, with a score for impact, confidence and ease.
- One request to a session lists what is kept, most valuable first.
- One request to a session takes the most valuable item nobody is working on
  and carries it out; a second session asked at the same moment gets a
  different item, never the same one.
- A finished item no longer appears as waiting. (Not in the request's words;
  it follows from asking for a list that says what to do next.)
- It works from the main copy and from any working copy, with nothing to set
  up first, and no copy holds a list of its own that could disagree.

## Constraints this change inherits

- User-facing text is Japanese; code, comments, and docs are English.
- A session starts on the main copy, which runs only a fixed list of
  commands, and the request is that asking once is enough. Whether that list
  grows is the person's decision, not a session's.
- `CLAUDE.md` and `AGENTS.md` together stay within their character ceiling: a
  line added is paid for by a cut.
- The application itself stays local-first. This is a tool for the sessions
  that develop it, and it talks to GitHub through the `gh` the person already
  signed in to.

## Systems likely affected

None of the application's modules: the development scripts, the skills a
session is steered by, the policy for the main copy, and the test suite.

## Open questions

None open. The one that needed the person — whether the main copy's command
list may grow for this — was asked on 2026-11-27 and answered yes.

## Not in scope

- Showing the backlog inside the Operon application.
- Refusing a session's finish until it has recorded its leftovers; the skill
  asks for it, no hook enforces it.
- Importing the leftovers of the 322 changes already landed.
- Running several items in one request. One request takes one item; repeating
  the request, or a loop, takes the next.
