# Intent: the lessons file took steering_bytes past its propose tier again

- **Status**: approved — by the evaluator, because the decision it applies is
  change 030's, taken by the person who owns it
- **Opened**: 2026-10-01

**Answers band**: steering_bytes

## Problem

`steering_bytes` read 317,433 after change 096 landed, against a `propose` tier
of 316,000. Before 096 it was 315,755: 096's lesson entry (1,505 bytes) is what
crossed the line. `docs/sdlc/lessons.md` is 167,016 of it, and entries 019 to 041
alone are about 125 KB of full-form prose whose guards have been green for weeks.

## Why this is not a new decision

Change 030 answered the same breach and recorded the next one in advance: "if
steering_bytes reaches 316000 again on lessons alone, the answer is not a third
baseline — it is that the format is the cost." The person who owns the band
approved compressing old lessons to their rule and their guard. This change
moves that line from 018 to 041. It raises no tier and deletes no guard.

## Desired outcome

- Entries 019 to 041 in the compressed form 001 to 018 use: one paragraph and
  `**The rule:**`, then the `**Guard.**` paragraph unchanged.
- Every `## NNN` heading and every `**Guard.**` paragraph byte-identical, so
  `every_lesson_cited_by_number_is_a_lesson_that_exists` and
  `bash scripts/pipeline-indicators.sh --lessons` read what they read before.
- The `lessons.md` preamble rewritten for the new line: it names 001 to 041 as
  the compressed entries, gives their guards' age as weeks rather than months,
  and says change 097 moved the line — so the file still describes the two
  forms it holds (lesson 002).
- `bash scripts/check-bands.sh` no longer reports `steering_bytes` at `propose`.

## Not doing

- Changing the two-format rule for new entries. 030's person kept recent entries
  whole on purpose; moving the line again is cheaper than re-arguing that.
- Raising the baseline or a tier.
