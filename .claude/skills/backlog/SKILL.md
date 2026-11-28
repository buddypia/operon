---
name: backlog
description: Keeps the work a session leaves undone as GitHub issues scored by ICE (impact, confidence, ease), and hands out the highest-scored item to one session at a time. Use when a change closes with remaining work, an improvement noticed, or a problem found and left; when a person asks what is left, what to do next, or for the most valuable thing; and when asked to run, take, or work the backlog. Works from the main checkout and from any worktree with no setup. Do not use to plan the change in hand — that is the sdlc skill.
---

# The backlog

The repository's GitHub issues are the only record. `scripts/backlog.py` keeps
no file, so the main checkout and every worktree read the same list, and two
sessions asked for the next item never receive the same one. It runs anywhere
in the repository, main included, exactly as written below — no `cd`, no
variables, no setup.

Decide which of the two uses this is, then run it without asking: both are
undone by one command (`release`, or closing an issue). `/backlog` given alone,
with nothing after it, is the Run use: take the next item and carry it out, do
not stop at a list.

## Record — when a change closes

Before the final report, go through what this session knows and did not do:

- **remaining** — work the request implied that this change left out.
- **improvement** — something that would make the product or the process
  better, noticed on the way.
- **problem** — a defect, a risk, or a failing check found and left.

One item per independent piece of work; split what could land separately.
Leave out what is already done, what a `person:` line in `review.yaml` already
carries to the person as a decision, and anything you would not do yourself if
asked tomorrow.

```sh
python3 scripts/backlog.py add --title "計測なしでは再描画の改善を確かめられない" --kind improvement --impact 6 --confidence 7 --ease 8 --source 322-ui-and-drag-rerender-caches --body "何が起きるか、どこで見つけたか、何をもって完了とするか。Impact 6: …。Confidence 7: …。Ease 8: …。"
```

- **Title**: the problem in one line, in Japanese, as the person would say it —
  not the fix. The same title is never filed twice; the command answers with
  the existing number instead.
- **Body**: what goes wrong or is missing, where it was seen (file, change,
  command), what done looks like, and one sentence arguing each score. A
  session that has never seen this conversation must be able to start from it.
- **The repository is public, so every item is readable by anyone.** Never
  put a secret, a token, a personal detail, a host or user name, or another
  repository's private content in an item.

### Scoring

Whole numbers, 1 to 10. The score is their product.

| | 1–3 | 4–6 | 7–10 |
|---|---|---|---|
| **impact** — what the person gains when it is done | cosmetic, rare | a real annoyance or a recurring cost | data, safety, or something used every day |
| **confidence** — how sure the gain and the approach are | a guess | reasoned, not measured | reproduced, measured, or the person asked for it |
| **ease** — how little work it is | days, several modules, a paused surface | a change with its own spec | minutes to an hour, one place |

Score what you know, not what you hope: an unmeasured speed-up is confidence 3
however plausible. When a later session learns more, it corrects the scores:

```sh
python3 scripts/backlog.py score 12 --impact 8 --confidence 9 --ease 5
```

In the final report, list each item filed as its number, score and title — the
line `add` printed — in place of a paragraph about what remains.

## Run — when the next thing is asked for

`/backlog` alone, "do the next thing", or "the most valuable thing" starts at
step 1; a person asking what is left gets the list instead (the end of this
section).

1. Take the best item nobody holds:

   ```sh
   python3 scripts/backlog.py claim
   ```

   It prints the item's number, score, title and text; the item is now held in
   this session's name. 「着手できる項目はありません」 means there is nothing to
   do: say so and stop. When the person named an item, take that one:

   ```sh
   python3 scripts/backlog.py claim 12
   ```

   Exit 3 means another session holds it; say who, and do not work on it.
   A claim a day old whose holder left no working copy and no commit naming
   the item is given back by `claim` on its own, with a comment saying so.

2. Open the working copy — feature-pilot's step 0, `make wt.new` (a `problem`
   is a `fix/` branch) — and, from inside it, move the claim there:

   ```sh
   python3 scripts/backlog.py claim 12
   ```

   It answers 「#12 の着手をこの作業ツリーに移しました」, and the item stays held
   for as long as that working copy exists. A claim left on the main copy is
   given back a day later.

3. Do the work. The item's title and text are the request: follow
   `.claude/skills/feature-pilot/SKILL.md`. Quote the issue and its number in
   `intent.md`, or in the bug report for a bugfix, and put `(#12)` in the
   commit subject.

4. Close it once the change has landed on `main`, with the merge commit:

   ```sh
   python3 scripts/backlog.py done 12 --ref a1b2c3d
   ```

   If the work stops instead — blocked, failed, or the item turned out wrong —
   give it back with the reason:

   ```sh
   python3 scripts/backlog.py release 12 --reason "再現できなかった。手順が必要"
   ```

   An item that should not be done at all is rescored low or closed by the
   person; do not close one as done that was not done.

5. Record what this work in turn left over, as above.

One request takes one item. To take the next, the person asks again or runs
this in a loop.

When the person asks what is left rather than to start, show the list, report
its lines as they are, best first, and stop there:

```sh
python3 scripts/backlog.py list
```

## When something is off

- `list` shows 「着手中」 with a name and an age. `claim` gives back on its own
  an item claimed a day ago or more whose holder has no working copy here and
  no commit names it as `(#N)`; when git cannot answer, it keeps the item. A
  working copy that still exists but was abandoned is a person's call: say so,
  and `release` only when they agree. A session on another machine or clone
  that holds an item a day without pushing a commit naming it loses the item.
- 「スコアを読めません」 marks an issue labelled `backlog` by hand. `score` it
  and it joins the order.
- 「GitHub に届きません」 carries `gh`'s own first line. An item that could not
  be filed is not recorded anywhere: say so in the report, with the item's
  text, instead of dropping it.
- Two sessions can both keep one item only if GitHub answers a comment read
  without a claim it had already accepted. `list` shows the holder; if it is
  not this session, stop and `release` nothing.
