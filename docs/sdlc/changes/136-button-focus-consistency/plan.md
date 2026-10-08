# Plan: one keyboard focus mark for every button

- **Spec**: `./spec.md`
- **Status**: approved

## Files that change

- `src/ui/widgets.rs` — add `paint_button_focus(ui, response, palette)` beside
  `focus_stroke`; call it at the end of `primary_button`, `quiet_button`,
  `nav_tab`, `tab_item_with_count`, `icon_button`, and `small_icon_button`.
  Each helper already returns a `Response`, so the call goes after `ui.add`
  and before the return.
- `src/app/screens.rs` — call the same helper after `ui.add` for the inline
  buttons there: the brand button and the two "Operon から削除" danger buttons
  (built with `.stroke(egui::Stroke::NONE)`), and the session close (×) button
  and its "…" menu (built with `.frame(false)`). `ui_sessions` and
  `ui_session_grid` (both in this file) gain `let palette = self.store.theme.palette();`
  to supply the palette to their helper callers. The roughly 40
  `icon_button` / `small_icon_button` call sites in this file each pass
  `palette` as the new second argument; this is the bulk of the diff here.
- `src/app.rs` — call the helper on the `.response` of the running-port badge
  menu (built with `.frame(false)`), and pass `palette` at the two
  `small_icon_button` callers in this file (around lines 5957 and 6061).
- `src/ui/session_tree.rs` — pass `palette` at the three `icon_button` callers
  (around lines 266, 280, and 337). See Departures.
- `src/tests.rs` — add `focus_ring_on_a_button_is_the_strong_outline_at_control_radius`
  beside `keyboard_focus_on_a_custom_row_is_a_strong_outline_or_nothing`. It
  iterates `AppTheme::all()`, draws each button helper once focused and once
  unfocused, and reads the ring back from the emitted shapes: one ring at
  `RADIUS_CONTROL` when focused, none when not. It does not paint a ring itself.
- `docs/sdlc/changes/136-button-focus-consistency/state.yaml` — status and
  screen updates as work moves.

No change to `src/theme.rs`, `src/glyphs.rs`, `src/i18n_tables.rs`, or
`DESIGN.md`. Callers of `icon_button`, `small_icon_button`, and
`resume_copy_button` change only to pass `palette` (see Departures).

## Order of work

1. Read the six helper bodies in `src/ui/widgets.rs` and confirm the response
   variable each returns.
2. Add the `palette: &Palette` parameter to `icon_button`, `small_icon_button`,
   and `resume_copy_button`, and update every caller, without painting any
   ring yet. The test then compiles and runs. Run it and confirm the failure is
   the focused case finding no ring: that is the "watched failing" step. Keep
   that output.
3. Add `paint_button_focus` and its calls: the six helpers in
   `src/ui/widgets.rs` (`primary_button`, `quiet_button`, `nav_tab`,
   `tab_item_with_count`, `icon_button`, `small_icon_button`), then the inline
   sites in `src/app/screens.rs` (brand, the two "Operon から削除", session
   close, session menu) and `src/app.rs` (port badge). `resume_copy_button`
   gets its ring through `small_icon_button` and needs no call of its own.
4. Run the test; it must pass.
5. Run `cargo fmt --check` and `cargo clippy --locked -- -D warnings`.
6. Run `cargo test --locked`, `bash scripts/check-bands.sh`, and
   `bash scripts/check-transcript-vocabulary.sh`.
7. Commit in the worktree; `.claude/hooks/gate-commit.sh` runs fmt and clippy.
8. Package with `bash scripts/package-macos.sh`, then replace the installed
   bundle through `scripts/replace-macos-bundle.sh`. This step is held for the
   person's confirmation, per the AGENTS.md destructive-action rule.
9. Manual check in the running app, against the spec Acceptance list.

## Risks

- **Ring drawn over a neighbour.** `Inside` keeps the ring within the control
  rectangle, so the packed toolbar and tab rows do not overlap. Checked by eye
  in step 9.
- **Ring on a hover-only state.** `has_focus()` is true only for keyboard
  focus, so pointer use is unaffected. The unit test pins the decision; the
  manual check confirms it.
- **Hidden test drop.** The gate refuses a commit that loses a test; the new
  test is additive, so it does not trigger that refusal.
- **Inline buttons untested.** The new test covers the six helpers only. The
  five inline sites in `screens.rs` and `app.rs` have no automated check, so a
  later edit could drop their ring without a failing test. Carried in
  `review.yaml` (rust-reviewer/a); the manual check in step 9 covers them.
- **`primary_button` contrast.** The ring is `border_strong` on the accent
  fill at the edge. The contrast guard `every_theme_stays_readable` covers
  text, not edges. The manual check in step 9 looks at this directly, and any
  gap is reported rather than changed silently.

## Proof of completion

- The new test fails before `paint_button_focus` is called and passes after it,
  with both outputs shown.
- The commands in the spec Acceptance list print their healthy output. The two
  script checks (`check-bands.sh`, `check-transcript-vocabulary.sh`) are the
  exception: `state.yaml` records them as waived, with the reason on each line.
- Spec requirement 5, that custom rows keep their existing focus ring, is
  covered by the unchanged test `keyboard_focus_on_a_custom_row_is_a_strong_outline_or_nothing`.
- The `focus-visible-on-every-button` item in `state.yaml` is marked passed by
  the person after they look at the running app; it is not marked by the
  session.

## Departures

- `icon_button` and `small_icon_button` take a new `palette: &Palette` second
  parameter, so the focus ring reads its colour from a role and not a literal.
  Their call sites in `src/app/screens.rs`, `src/app.rs`, and
  `src/ui/session_tree.rs` pass it. `ui_sessions` and `ui_session_grid` in
  `src/app/screens.rs` gained `let palette = self.store.theme.palette();` to
  supply it.
- Frameless inline buttons, found by review after the first draft, now take the
  ring as well: the session close (×) and "…" menu in `src/app/screens.rs`, and
  the running-port badge in `src/app.rs`. The menus call the helper on the
  `.response` of `menu_custom_button`. Spec, Flagged concerns, and screen.md
  record the same set.
- `verb_button` in `src/ui/widgets.rs`: the danger form calls
  `paint_button_focus` over its own danger outline; the plain form does not,
  because egui's native outline is the same `border_strong` ring. Not in the
  first plan's Files list. Spec and screen record it.
- `src/tests.rs`: the focus test installs `theme.visuals()` on its context,
  as `app.rs` does. Without it the test measured egui's default outline, not
  the palette's, and the plain verb failed in the light theme.
- `resume_copy_button` takes the same `palette: &Palette` second parameter; its
  one caller passes it. Not listed in the plan because the plan did not inspect
  its body.
- The focus test paints no ring itself. A first draft painted one after the
  helper had run, which produced two rings and failed. The test now reads only
  what the helpers paint.
