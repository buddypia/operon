# Intent: four failures recurred because nothing in the harness could see them coming

- **Status**: approved
- **Opened**: 2026-09-22

## Problem

Four things went wrong more than once, and in each case the repository already
knew the individual fact and still had no mechanism that would make the next
session notice. They are unrelated in subject and identical in shape: a failure
that leaves no trace a later session can read.

**1. Verifying guards by mutation cannot see a claim nobody guarded.** Change
061 shipped fifteen mutations, every one of them caught, and still went back
twice — both times because a sentence a person reads made a claim that was not
true of every place it covered. Every mutation asked whether a *count* moved,
and neither defect was in a count. The practice reads as thorough and is blind
along one axis, and nothing says which axis.

**2. A gate can report on a tree that is not the one being worked in, and say
nothing about it.** The Stop gate stopped one session three times on the same
position, naming open work from changes that do not exist in its tree. The
defect was found, fixed, and recorded — and diagnosing it still cost a person's
attention each time, because the output looks exactly like real findings. A gate
that is pointed at the wrong thing cannot currently say so.

**3. A reviewer that finishes without its report arriving looks like a reviewer
that is still thinking.** It has now happened three times. The policy covers the
reviewer that never comes back; it does not cover the one that came back
silently, and the two have different remedies. Guessing wrong produces two
verdicts on one digest that then have to be reconciled by hand.

**4. Whether a change makes the installed application stale is discovered at the
end, if at all.** It is decided by what the change touched — knowable on the
first day — and is currently established by reading a diff after everything is
committed. Twice in a row the answer was "no", by luck; the third time it was
"yes" and surfaced only when the release gate refused.

## Who feels it, and when

The session doing the next piece of work, every time.

1. Whenever a change adds or alters a sentence a person reads, and the session
   believes its mutation sweep proves the guards hold. That belief was wrong
   twice in change 061 alone, and it was wrong in the confident direction.
2. Whenever a session stops in a worktree while the hook that judges it belongs
   to another checkout. Eleven stops in change 065, three more after it.
3. Whenever a review is requested — which is every change that touches code.
4. Whenever a change touches a file compiled into the release binary. The person
   who then opens the installed app gets yesterday's behaviour and no sign of it.

## Desired outcome

- A mutation sweep that ignores every claim a person reads is reported as
  incomplete, by the repository, before a reviewer has to find the claim.
- A gate that is reporting on a tree other than the one the session is working
  in says so, in its own output, without anyone having to notice that a named
  change directory does not exist.
- A session that requests a review can tell the three reviewer states apart —
  still working, finished and reported, finished and silent — and the policy
  says what to do in each. In particular, the silent one is re-asked rather than
  replaced, because replacing it is what produces two verdicts at one digest.
- Whether a change will make `/Applications/Operon.app` stale is written down
  while the change is being planned, and the installed build's staleness is
  observable from the repository rather than inferred from a diff.

Each of these is observable: it is either in a document a session reads, or in
the output of something that runs.

## Constraints this change inherits

- User-facing text is Japanese; code, comments, and docs are English. Three of
  the four outputs here are read by agents and stay English; the Stop gate's
  refusal text is read by a person and is Japanese, like the rest of it.
- `.claude/hooks` is the `gate-configuration` surface and
  `scripts/package-macos.sh` is `bundle-swap`; both are `high paused` in
  `docs/sdlc/risk.yaml`. The repository's owner authorised this change
  explicitly, naming all four items.
- The gate that judges a commit is `refs/heads/main:scripts/check-review.sh`,
  not this tree's copy. Lesson 039: this branch may narrow what its own gate
  allows and may never widen what that gate expects to read. Nothing here adds a
  column to a file the judge parses.
- `scripts/package-macos.sh` builds the bundle and its contents are not
  hand-edited. A stamp has to be written by the script, not into the bundle.
- The branch cannot merge main, and main's copies of the hooks are what sessions
  actually run. Item 2's fix does nothing for any session until the merge — the
  same standing condition change 065 landed under.

## Systems likely affected

Not modules in `CLAUDE.md`'s map — this change is entirely in the harness:
`REVIEW.md`, `.claude/skills/sdlc/SKILL.md`, `.claude/hooks/gate-stop.sh`,
`scripts/package-macos.sh`, a new script under `scripts/`, `evals/`,
`docs/sdlc/lessons.md`, `docs/sdlc/templates/state.yaml`, and `src/tests.rs` for
the guards. No file compiled into the release binary, so this change does not
itself make the installed app stale — which is item 4's own question, answered
on the first day, which is the point.

## Open questions

None outstanding. The one that would have been — whether a session may edit two
`high paused` surfaces — was put to the repository's owner and answered yes,
naming all four items.

## Not in scope

- **The merge.** Item 2 lands a fix in this branch's hook; the hook sessions run
  is main's. The handover is `/tmp/operon-merge-handoff.md`.
- **The two nits carried out of change 061**, and the 91 untranslated message
  ids in `src/history.rs`. Different changes.
- **`scripts/check-readiness.sh` returning No-Go for every `bugfix` route.** A
  fifth recurring failure of the same shape, found while writing this. It is
  change 050's surface and is recorded in change 061's `state.yaml`; folding it
  in here would mean editing a gate from inside a change that is using it.
- **Making the mutation rule mechanically enforceable at commit time.** The
  commit gate would have to read the diff and the mutation script together. The
  rule lands as steering plus an eval, which is what
  `.claude/skills/sdlc/SKILL.md` prescribes when a guard can only be a prompt,
  and `scripts/pipeline-indicators.sh` with `--lessons` already accepts an eval path as
  a guard.
- **Re-basing `steering_bytes`.** This change will add to it. `docs/sdlc/bands.yaml`
  says a band corrected in order to be passed is the wrong way round.
