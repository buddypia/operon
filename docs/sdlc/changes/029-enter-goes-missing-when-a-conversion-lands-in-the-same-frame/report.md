# Bugfix: a second Enter in the frame that committed a conversion was dropped

- **Route**: `bugfix` — no intent and no spec by design. The reproduction below
  is the intent, and inventing a spec for it is the ceremony that gets pipelines
  abandoned.
- **Skill**: `.claude/skills/root-cause/SKILL.md`

## The cause, in one sentence

`terminal_input_events` in `src/ui/terminal.rs` computed a single boolean over
the whole frame's event list — true when any non-empty `Ime::Commit` was in it —
and dropped *every* `Key::Enter` in that frame, so a frame carrying both the
Enter that confirmed an IME conversion and the Enter the person pressed to send
the line lost the second one as well.

## The observation that proves it

A probe over the event list `[Enter, Ime::Commit("日本語"), Enter]` returned
`[Text("日本語")]`. No `Key("Enter")` at all: the send was gone.

## The test, and the message it printed

`a_second_enter_in_the_frame_that_committed_a_conversion_still_sends`, watched
failing before the fix:

```
assertion `left == right` failed: the second Enter is the person sending the line
  left: [Text("日本語")]
 right: [Text("日本語"), Key("Enter")]
```

## What changed

The flag became a count. Each non-empty commit in the frame entitles the IME to
absorb one Enter, spent in event order, so the second press survives. Counting
rather than matching on position also makes it hold whichever way round the
platform delivers the key and the commit — which is the thing a test cannot
observe from inside this repository.

Four lines in `src/ui/terminal.rs`. Nothing else.

## Guard, and whether it was watched failing

`a_second_enter_in_the_frame_that_committed_a_conversion_still_sends`. Watched
failing twice: with the count restored to a frame-wide flag, and with the
absorption removed altogether. Recorded as lesson 016, whose rule is the class
rather than this instance — a per-frame aggregate answering a per-event
question.

## Ruled out, and not done

One approach holds a chord typed during a preedit until composition ends, with a
200 ms newline fallback and a 10 s ceiling. That is a workaround for the DOM's
asynchronous `compositionend`, which arrives after the key that caused it.
`egui` delivers `Ime` and `Key` events in one ordered list within the same
frame, so the race the timers exist for is not reachable here, and two timers
guarding against it would be a mechanism with nothing to catch. Not built.
