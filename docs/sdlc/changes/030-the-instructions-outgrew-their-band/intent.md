# Intent: what an agent must read before it reads code has passed its ceiling

- **Status**: approved — the four decisions below were taken on 2026-09-19 and
  carried out in the same change
- **Opened**: 2026-09-06
- **Re-measured**: 2026-09-19, because the numbers below were the argument and
  they had moved. See "What has happened since" before the table is read.

**Answers band**: steering_bytes

## Problem

`steering_bytes` measures everything an agent reads to know how to work here:
`CLAUDE.md`, `AGENTS.md`, `DESIGN.md`, `CONTRIBUTING.md`, `REVIEW.md`, the four
subagent briefs, the four skills, and `docs/sdlc/`. Its baseline is 91,262. It
reached 151,987 during change 029, which is past the `propose` tier of 150,000 —
the tier whose instruction is this file.

It has been over `diagnose` since before this session and was recorded as such
in five changes running, each one waiving it. Change 021's intent says what that
means: three records in a row is the point at which a record stops being
information and becomes a habit. This is the sixth.

## What has happened since

Nothing was decided, and the number kept moving. Measured 2026-09-19 with
`scripts/harness-metrics.sh`: **208,450**, against 151,987 when this file was
written and a `propose` tier of 150,000. It is now 39% past the tier and 2.3×
the baseline.

Fifteen changes recorded `steering_bytes` at `propose` in that time — 031, 032,
033, 034, 035, 036, 037, 038, 040, 050, 052, 054, 056, 057, 059 — and every one
of them waived it, each with a reason that was true on its own. This file says
the sixth recording was the point at which a record becomes a habit. It is the
twenty-first.

The habit has a cause worth naming, because it is not carelessness: the band's
prescribed response is "write an intent.md and re-enter at stage 1", that
intent.md is this file, and it has existed and been `awaiting-user` the whole
time. So the prescription was followed once and then had nowhere to go. A band
whose response is blocked is indistinguishable, change by change, from a band
being ignored — and `state.yaml`'s `bands` item in change 059 says exactly that:
the count is the finding.

Where the weight is, re-measured 2026-09-19:

| Bytes | 2026-09-06 | File |
|---|---|---|
| 79,452 | 35,770 | `docs/sdlc/lessons.md` |
| 32,814 | 32,814 | `DESIGN.md` |
| 19,308 | 18,891 | `docs/sdlc/README.md` |
| 11,378 | — | `.claude/skills/create-pr/SKILL.md` |
| 9,103 | 9,103 | `CONTRIBUTING.md` |
| 8,932 | 8,932 | `CLAUDE.md` |
| 8,861 | — | `.claude/skills/sdlc/SKILL.md` |
| 6,799 | — | `REVIEW.md` |
| ~31,800 | — | everything else, none over 6,000 |

By group: `docs/sdlc/*.md` is 98,760 of the 208,450, the root policy documents
are 48,716, the five `SKILL.md` files are 33,500, the templates 7,047, and the
four subagent briefs 8,779.

`lessons.md` is the one that grows on its own, and it is now the whole argument:
**it more than doubled — 35,770 to 79,452 — and is 38% of the metric by itself.**
It is append-only by design, one entry per mistake, newest last, and the
pipeline requires an entry for every mistake made twice. So it rises
monotonically for as long as the repository is worked on, every agent pays for
all of it, and no amount of restraint elsewhere brings the number back: even
deleting every other counted file except `CLAUDE.md` and `AGENTS.md` leaves
91,100. **This band cannot return inside itself while `lessons.md` is counted
and lessons keep being written.** That is the decision this file is waiting for,
stated as plainly as the measurement allows.

One thing became removable since 2026-09-06 without trading anything away:
`.claude/skills/create-pr/SKILL.md`, 11,378 bytes, 5.5% of the metric. `AGENTS.md`
already says it "does not run here: it needs an SSH `origin` and `gh`, and this
repository has no remote" — confirmed 2026-09-19, `git remote -v` prints
nothing. Outside `AGENTS.md` and change 051's records, nothing references it.
Deleting it is not relocation and costs no reasoning; it also leaves four
skills, which is `skills`' baseline and clear of its `warn` tier of 3. It is
listed here rather than done, because this change's route is `docs` and its
stage is `plan`.

## Who feels it, and when

Every session, before it has read a line of code. The cost is invisible at the
point it is paid, which is why it took six recordings to become a change.

## Desired outcome

`steering_bytes` is inside its band, and it got there by prose being removed or
demoted rather than by the band being moved. The lesson mechanism still works:
an agent about to repeat mistake 011 still finds out.

## Constraints this change inherits

- **Correcting a gate in order to pass it is the wrong order.** Raising the band
  is not a candidate.
- **Relocating is not reducing.** `bands.yaml` says so itself about the rules
  directory. Moving `lessons.md` into a subdirectory the glob does not match
  would make the number fall and change nothing an agent pays, and would be the
  worst outcome here: a metric that stops describing the thing it was built to
  describe.
- Every guard named in a lesson's **Guard** column is a test that must keep
  existing. The prose can go; the guard cannot.

## Systems likely affected

`docs/sdlc/lessons.md`, `DESIGN.md`, `docs/sdlc/README.md`, and possibly the
`.md` set `scripts/harness-metrics.sh` counts.

## Decisions taken, 2026-09-19

All four were put to the person who owns them, with the re-measurement above in
front of them. Recorded as answers, not as a summary of the discussion:

1. **Compress old lessons.** Chosen. The entries whose guards have been green
   longest are reduced to their rule plus their `**Guard.**` paragraph; the
   guards themselves — every test and eval named — are untouched, which is the
   constraint this change carried from the start.
2. **Keep `.claude/skills/create-pr/SKILL.md`.** Not deleted, against the
   recommendation here, so the 11,378 bytes stay and `AGENTS.md` keeps saying the
   skill does not run in a repository with no remote.
3. **Leave `DESIGN.md` where it is.** No move to the conditional tier; this file
   already argued that relocating is not reducing.
4. **Raise the baseline to a realistic value.** Chosen, and it is the one answer
   this file had ruled out — "correcting a gate in order to pass it is the wrong
   order" is written above and still means what it says. What makes it defensible
   is the order of operations, and the order is not optional: **the compression
   happens first, the new baseline is read off the tree afterwards.** A baseline
   set before the cut would be the band being moved to meet the number; a
   baseline read after it is the pipeline's own cost — four skills, four subagent
   briefs, six routes and their templates — measured rather than assumed. If the
   reading after the cut is still far above the old tiers, that is the finding
   and it gets written down here, not smoothed over by picking a threshold above
   it.

The measurement that makes the shape of the cut clear, and that this file did not
have before: the weight is not in the old entries. Entries 019–024, all written
in the last days, are 38,697 bytes — 49% of the file across six entries — while
001–018 are 40,228 across eighteen. The new prose format costs 6.4 KB an entry
where the old table cost 1.7 KB. So compressing every old entry to one line
cannot reach the tier on its own, which is exactly why answer 4 was needed
alongside answer 1, and why the recent six are left whole: they are the most
expensive and the least likely to have finished their work.

## Open questions

Answered above on 2026-09-19. Kept in place rather than deleted, because the
reasoning is what makes the answers legible:

- **Do old lessons expire?** An entry whose guard has been green for months and
  whose rule is now in `CLAUDE.md` or a test may have done its work. Compressing
  it to one line plus its guard name would take `lessons.md` from 79 KB to
  perhaps 20 KB — the estimate scaled with the file, since the format did not
  change. What is lost is the reasoning, which is the part that persuades the
  next agent not to repeat it. **This is the only question whose answer can
  bring the metric back inside its band**, which was not true when this file was
  written: at 35 KB the other files still mattered, and at 79 KB they do not.
- **Is `DESIGN.md` steering?** At 32.8 KB it is the second largest, and it is a
  reference an agent consults rather than a rule it must hold. If it belongs in
  the conditional tier as a `.claude/rules/` file scoped to `src/ui/` and
  `src/theme.rs`, the number falls by a third — but that is relocation unless
  the reason is that it genuinely does not need reading for most work.
- **Is the baseline of 91,262 still the right target**, or was it taken before
  the pipeline had four skills and four subagents that legitimately cost bytes?

## What was already ruled out

Measured on 2026-09-06, so that the decision below starts from evidence rather
than from the same investigation being run twice:

- **Deleting an unguarded lesson.** There are none. All sixteen entries carry a
  filled Guard column naming a test in `src/tests.rs`, which was the whole point
  of that column. Re-counted 2026-09-19: twenty-four entries, and the format has
  since moved from a table with a Guard column to prose that names the guard in
  a closing paragraph — so the original count of sixteen filled columns cannot
  be re-run as written, and this line does not claim it was. What is measured is
  the count and the growth: eight more entries, 35,770 bytes to 79,452, which is
  5.5 KB per entry against 2.2 KB under the old format. The prose format is
  where the doubling came from, and that is a fact about the format rather than
  about any entry being wasteful — which makes it a question for a person, not a
  deletion.
- **Trimming the small files.** Nothing in the counted set is over 9,000 bytes
  except `lessons.md` (35,770) and `docs/sdlc/README.md` (18,891). The other
  fourteen files together are about a third of the total and none of them is
  where the growth is.
- **Reclassifying `lessons.md` out of the counted glob.** It would work, and it
  is the one option this change refuses on its own: an agent would pay exactly
  the same bytes and the number would stop describing what it was built to
  describe. It is listed here because it is the obvious idea and somebody will
  have it again.

## Not in scope

- Deleting a guard, or a lesson's Guard column.
- Raising any tier in `docs/sdlc/bands.yaml`.
