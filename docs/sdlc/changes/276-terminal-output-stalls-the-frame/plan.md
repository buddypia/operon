# Plan: lay terminal output out where it is captured, not on the frame

- **Spec**: none — route `refactor`; the request and the measurement are below
- **Approved**: 2026-11-10
- **Status**: done

## Why

The person's request, verbatim: 「AIを主に利用するIDEです。利便性やパフォーマンスなど多様な観点で改善を行ってください。」
Of the candidates offered they chose 「ターミナル出力の再レイアウトを軽量化」.

Measured on this machine, release build, a 5 000-line coloured capture
(`TERMINAL_SCROLLBACK_LINES`): `terminal_line_layouts` takes 2.7 ms and the
copy kept beside it 0.36 ms. Both run on the UI thread inside
`set_session_output`, once per capture whose text differs — every two seconds
for each terminal on screen while an agent prints, and up to thirty times a
second for the second after a keystroke. The capture itself already runs on a
background thread; only the parsing was left on the frame.

Measured and left alone, so nobody measures them again: copying every session
and project for the session lists is 35 µs at 400 sessions against a 1.35 ms
frame, and rebuilding a 20 000-line diff's rows is 60 µs.

## What stays the same

Every row, colour, caret, search hit and link the terminal shows; when a
capture is taken; the result types in `src/models.rs`; the signatures of
`set_session_output` and `apply_session_poll`, which the suite calls directly.

## Files that change

| File | Change |
|---|---|
| `src/app/terminal_output.rs` | new. `LaidOutTerminal { theme, fingerprint, layouts: Option<Vec<LayoutJob>> }`; `lay_out_capture(output, theme, known_fingerprint)`, which a capture thread calls and which skips the layout when the text's fingerprint is the one the app already holds; `capture_laid_out(tmux_name, theme, known)`; and `set_session_output` and `request_session_output`, moved here from `src/app.rs` |
| `src/app.rs` | `mod terminal_output`; fields `terminal_prelaid: HashMap<Uuid, LaidOutTerminal>` and `terminal_output_fingerprints: HashMap<Uuid, u64>`; `BackgroundResult::SessionOutput` gains `laid_out: Option<LaidOutTerminal>` and `SessionPoll` gains `laid_out: Vec<(Uuid, LaidOutTerminal)>`, stashed in `terminal_prelaid` before the existing handlers run; the session poll's on-screen captures call `capture_laid_out`; a theme switch moves `terminal_layouts` into `terminal_fallback_layouts` and clears the fingerprints instead of clearing the layouts; closing a session drops its two new entries |
| `src/tests.rs` | `laid_out` added to the four places that build those results; the `terminal_276_` tests |
| `CLAUDE.md` | the module map names `src/app/terminal_output.rs` |

`set_session_output` takes the stashed layout for its session when the theme
it was laid out under is still the app's, records its fingerprint, and
otherwise lays out on the spot exactly as today — so a caller that hands it
bare text (the suite, a theme switched mid-flight) gets today's behaviour. It
no longer copies the layout into `terminal_fallback_layouts`: the fallback is
only ever read when `terminal_layouts` has no entry, which happens only after
a theme switch, and that is now the moment the entry is moved there. When it
stores a fresh layout it removes the session's fallback entry, so a session
that has seen a theme switch does not carry a second layout for ever.

The fingerprint (`std::collections::hash_map::DefaultHasher` over the text)
is only a hint to the capture thread to skip work. It is never what decides
that the app's rows are current: a result that arrives with no layout is
still compared with the held text itself, as today, and is laid out on the
spot when the text differs or `terminal_layouts` has no entry. So neither a
collision nor a second result that changed the text in between (the session
poll's capture and a pane's own can be in flight together) can leave stale
rows. When a layout is not taken from the stash, the session's fingerprint
is forgotten, so the next capture is laid out off the frame again.

The stash is emptied by the handler that filled it, after it has run: the
`SessionOutput` arm after its `Ok` or `Err`, the `SessionPoll` arm after
`apply_session_poll`, which skips a failed capture and a session that is
gone. The fingerprints are pruned with the same `retain` that prunes
`terminal_fallback_layouts` when a project is removed.

## Order of work

1. Tests, failing: `terminal_276_a_capture_laid_out_off_the_frame_is_shown_as_is`
   (a sentinel layout arrives with the capture and is what `terminal_layouts`
   holds), `terminal_276_a_layout_from_another_theme_is_laid_out_again`,
   `terminal_276_unchanged_text_is_not_laid_out_again` (`lay_out_capture` with
   the known fingerprint returns no layout, and with another returns the same
   rows `terminal_line_layouts` gives),
   `terminal_276_bare_text_is_still_laid_out` (no stash),
   `terminal_276_a_theme_switch_keeps_rows_until_the_next_capture` (after
   `apply_appearance` with a new theme the fallback holds the rows and no
   fingerprint is left),
   `terminal_276_output_is_not_copied_beside_itself` (no fallback entry after
   an ordinary capture, nor after a theme switch followed by a capture),
   `terminal_276_no_layout_for_text_the_app_does_not_hold_is_laid_out` (a
   result saying "unchanged" arrives after another changed the text),
   `terminal_276_a_stash_does_not_outlive_its_result` (a failed capture and a
   removed session, through both result variants).
2. `src/app/terminal_output.rs` with the type and the two functions; move the
   two methods; `mod` line; `CLAUDE.md`.
3. Fields, the two result variants, the stash, the theme switch, the close.
4. `make q.fast`; `cargo test --locked terminal_276_`, `terminal_`,
   `session_255_`, `repaint_loop_`, `claude_md`, `harness`.
5. `mutations.py`, `verification.yaml`, `empirical-verifier`, `rust-reviewer`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A layout is shown for text it was not made from | Wrong rows in a pane | The layout travels in the same result as its text and is consumed in the same handler; `terminal_276_a_stash_does_not_outlive_its_result` |
| Old-theme colours survive a theme switch | A pane keeps the previous palette after its next capture | `terminal_276_a_layout_from_another_theme_is_laid_out_again`; fingerprints cleared on the switch, so the next capture is laid out |
| "Unchanged" is reported for text the app does not hold | An empty or stale pane | `set_session_output` still compares the text itself and lays out when there is no layout to keep; `terminal_276_bare_text_is_still_laid_out` |
| A pane is blank between a theme switch and the next capture | Flash of nothing | `terminal_276_a_theme_switch_keeps_rows_until_the_next_capture` |
| Moving two methods changes them | Any terminal test | The move is verbatim apart from the lines named above; the existing `terminal_` and `session_255_` tests |

## Proof of completion

- `make q.fast` — passes.
- `cargo test --locked terminal_276_` — `8 passed; 0 failed`.
- `python3 docs/sdlc/changes/276-terminal-output-stalls-the-frame/mutations.py`
  — every mutation caught.
- `bash scripts/check-bands.sh` — `largest_module_lines` no higher than before.
- In the running app: an agent printing into a full scrollback, typing into it
  stays smooth, and switching theme recolours the pane on its next capture.

## Departures from the plan

- When a capture's text is the text already held, with rows, its fingerprint
  is recorded too. The verifier found that rows the frame had laid out never
  got one, so every later capture of the same text was laid out again on the
  capture thread: wasted work, never wrong rows.
- `LaidOutTerminal` is not `Clone` (the reviewer): nothing copies a layout.
- The session poll lays its on-screen captures out inside the poll task, so
  with N panes on screen the poll's other results arrive about N × 3 ms later
  than before. Accepted: the poll runs every two seconds.
- `terminal_276_random_sequences_match_the_oracle`, written by the verifier,
  stays: 300 seeded sequences of results, theme switches and failures, each
  step compared with `terminal_line_layouts` of the held text.
