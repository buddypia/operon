# The terminal pane stops answering clicks

Carried over from `hotfix/terminal-pane-loses-clicks`, where it was numbered 053
and never landed: that worktree's diff was this change's three lines plus the
whole uncommitted tree of changes 042–050, and all three reviewers refused it for
the carried work. That work has since reached `main` by other changes; the
collision it introduced is still there. So this is the three lines and the guard,
on top of `main`, and nothing else.

## The report

> アプリ内で Claude Code が最初は TUI で入力して指示もできた。時間が経ってから
> もう一度 TUI にプロンプトを入力しようとしたら入力できず、画面上の AI は動いて
> いるけど、画面をマウスでタップしても反応がない。

Two answers narrowed it: the rest of the window kept working, so it was the pane
and not the frame loop; and it happened at `RUNNING`, so it was not the
`SessionStatus::Active` gate on `queue_terminal_input`.

## Root cause

`egui::Ui::new_child` salts **every** anonymous child `Ui` with the same
constant — `stable_id = self.id.with("child")` — so two siblings created by
`allocate_ui_with_layout` under one `horizontal_top` share a `Ui::id`. Change 046
put the prompt timeline beside the terminal in exactly that shape, and both panes
built their scroll area without naming it. The two hashed to one `Id`:

```
term-scroll id=3615 inner=[[17,91]-[595,583]]
tl-scroll   id=3615 inner=[[621,124]-[883,583]]
```

One id is one persisted `ScrollArea::State`. A scroll area registers a
drag-to-scroll widget — `id.with("area")`, `Sense::drag()` — only once
`content_is_too_large`, and with the id shared it was registered twice per frame:
the second registration landed *after*, and therefore on top of,
`ui.interact(terminal_rect, terminal_id, Sense::click())`.

egui resolves that as no click at all. `hit_test_on_close`:

> The top things senses only drags, so we ignore the click-widget, because it
> would be confusing if clicking a drag-widget would actually click something
> else below it.
> → `click: None`

So `terminal_response.clicked()` was false forever, `request_focus()` never ran,
and the pane could not take the keyboard. The cursor was not drawn either, which
is the "no reaction" the report describes.

The trigger is "the terminal has printed more than one screenful", because that
is when `content_is_too_large` turns on. Measured at 900×600: fine at 20 lines,
dead at 60. That is why it worked at first and then stopped — nothing changed
except the agent filling the pane.

## The fix

Name both scroll areas, and stop deriving the pane's own id from a `Ui` it
shares with its neighbour.

- `src/ui/terminal.rs` — `.id_salt("terminal-rows")`
- `src/app/screens.rs` — `.id_salt("prompt-timeline")`
- `src/app/screens.rs` — `terminal_id` from `egui::Id::new(…)` rather than
  `ui.make_persistent_id(…)`

## What was considered and rejected

- **`drag_to_scroll(false)` on the terminal's scroll area.** Tried; still fails.
  The thief is the *second* registration of the shared id, so silencing one pane
  leaves the other to steal the click.
- **`Sense::click_and_drag()` on the pane's interaction.** Would have won the hit
  test, but leaves the two panes sharing one scroll offset and one
  `stick_to_bottom`. It treats the symptom and keeps the collision.

## The guard

`a_full_terminal_pane_still_answers_a_click` in `src/tests.rs` drives the real
`ui_terminal_panel` through `egui::Context::run`, with the timeline on and 400
lines of output, and asserts a click leaves the keyboard somewhere. Watched
failing with both `id_salt` lines removed, and passing with them.

## What this does not fix

A session whose status is not `Active` still swallows every keystroke silently —
`queue_terminal_input` (`src/app.rs`) returns without a word, and
`session_status_after_observation` (`src/models.rs`) makes `Lost`/`Failed`/
`Exited` absorbing, so one mis-observed poll is unrecoverable from inside the
app. That is a real second defect and it is not this change.
