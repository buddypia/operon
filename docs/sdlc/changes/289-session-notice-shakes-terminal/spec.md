# Spec: notices and warnings on the Sessions page float over the corner instead of shifting the terminal workspace

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. On `Page::Sessions` (both the terminal workspace and its inline history
   library), neither `durability_warning` nor a non-brief `notice` consumes
   vertical space above `ui_terminal_workspace`: `dock_area` (and the terminal
   pane rect inside it) keeps the exact same `top()` and `height()` across no
   message, a non-brief `notice`, a brief `notice`, a `durability_warning`, both
   together, and after dismissal.
   Test: `session_notice_289_terminal_workspace_never_shifts_on_notices_or_warnings`.
2. On `Page::Sessions`, a non-brief `notice` is drawn in the foreground
   bottom-right overlay with `ICON_NOTICE` (`palette.info`), wrapped text within
   the window bounds, and the `この通知を閉じる` (`ICON_CLOSE`) button; it is not
   cleared by `expire_brief_notice`, and clicking `この通知を閉じる` clears
   `notice` (`None`).
   Test: `session_notice_289_persistent_notice_floats_stays_past_toast_timeout_and_closes`.
3. On `Page::Sessions`, `durability_warning` is drawn in the foreground
   bottom-right overlay with `ICON_ATTENTION` (`palette.warning`) and no close
   button of its own, stacking above `notice` when both are present.
   Test: `session_notice_289_durability_warning_floats_without_close_button`.
4. Outside `Page::Sessions` (such as `Page::Home`), a non-brief `notice` and
   `durability_warning` continue to draw as top banners in `CentralPanel` and do
   not appear twice in the bottom-right overlay.
   Test: `session_notice_289_other_pages_keep_top_banner_without_duplicate_toast`.

## Behaviour

- See `./screen.md` for the before/after geometry and every state on
  `Page::Sessions`.
- Strings read by a person are existing table entries only: `この通知を閉じる` on
  the close button's hover/accessibility text, plus whatever Japanese text the
  caller placed in `self.notice` or `self.durability_warning`. No new message id
  is added.
- Empty state (`self.notice == None && self.durability_warning == None`): no
  foreground `egui::Area` is created.
- Error / persistent state (`self.notice == Some(_)` and `!brief`): drawn in the
  bottom-right overlay with `ICON_NOTICE` and a 1px `palette.info` border; stays
  on screen until `この通知を閉じる` is clicked or another notice replaces it.
- Brief completion state (`notice_is_brief`): drawn in the bottom-right overlay
  with `ICON_STATUS_DONE` (`palette.success`) and a 1px `palette.border_subtle`
  border; leaves after `NOTICE_TOAST_SECONDS` (4s) or when `この通知を閉じる` is
  clicked, clearing both `self.notice` and `self.brief_notice`.
- Unconfirmed-sync state (`self.durability_warning == Some(_)`): drawn in the
  bottom-right overlay on `Page::Sessions` with `ICON_ATTENTION` and a 1px
  `palette.warning` border, with no close button; remains until
  `retry_pending_store` or a durable write clears it.
- Too-large / long message state: text inside each floating card wraps within
  `(screen_width - SPACE_LG * 2.0).clamp(200.0, 420.0)` while reserving space
  for the leading icon and, on `notice`, the `CONTROL_HEIGHT_SMALL` close button
  so `✕` remains inside the window on long messages.

## Design

Placement: `EXTEND` in `src/app.rs` (`OperonApp::update` and
`OperonApp::ui_brief_notice_toast`). `git grep -n "ui_brief_notice_toast"` finds
the single definition and single caller in `src/app.rs`.

- Extract the `CentralPanel` body of `OperonApp::update` into
  `pub(crate) fn ui_central_panel(&mut self, ctx: &egui::Context, palette: &Palette)`
  so tests can run the real panel + overlay pass without starting background
  daemons or hooks.
- Inside `ui_central_panel`, gate the in-flow `durability_warning` and non-brief
  `notice` `banner(...)` blocks on `self.page != Page::Sessions`.
- Extend `OperonApp::ui_brief_notice_toast` (keeping its name or calling a
  shared helper from `update`) into the window's bottom-right foreground overlay:
  - Compute `show_warning = self.page == Page::Sessions && self.durability_warning.is_some()`,
    `brief = notice_is_brief(&self.notice, &self.brief_notice)`, and
    `show_notice = self.notice.is_some() && (brief || self.page == Page::Sessions)`.
  - Return immediately when `!show_warning && !show_notice`.
  - Anchor one `egui::Area` (`"brief-notice-toast"`, `Order::Foreground`,
    `Align2::RIGHT_BOTTOM`, `vec2(-SPACE_LG, -SPACE_LG)`).
  - Cap card width at `card_max_w = (ctx.screen_rect().width() - SPACE_LG * 2.0).clamp(200.0, 420.0)`.
  - When `show_warning`, draw the raised card (`palette.raised`, 1px
    `palette.warning` stroke, `RADIUS_CARD`, symmetric `12, 8` margin, popup
    shadow) with `ICON_ATTENTION` (`14.0`, `palette.warning`) and wrapped
    warning text (`13.5`, `palette.text`), followed by `SPACE_SM` when
    `show_notice` is also true.
  - When `show_notice`, draw the raised card with `(ICON_STATUS_DONE, palette.success, palette.border_subtle)`
    when `brief` and `(ICON_NOTICE, palette.info, palette.info)` otherwise,
    wrapped text (`13.5`, `palette.text`) bounded so `small_icon_button(ui, palette, ICON_CLOSE, tr("この通知を閉じる"))`
    always fits inside `card_max_w`, and clear `self.notice` and
    `self.brief_notice` on click.
- No persisted-shape change, no external command, no background thread change,
  and zero allocation on frames with nothing to show (`notice` and
  `durability_warning` are borrowed as `&str` during painting rather than cloned
  per frame).

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | yes | Uses existing `Palette` roles only (`raised`, `border_subtle`, `info`, `success`, `warning`, `text`), all verified at >= 4.5:1 on `raised` by `every_theme_stays_readable` (`DESIGN.md` and `.claude/rules/palette-and-glyphs.md` read). |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | yes | Reuses `ICON_NOTICE`, `ICON_ATTENTION`, `ICON_STATUS_DONE`, and `ICON_CLOSE` from `src/glyphs.rs` (`src/glyphs.rs` read). |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | Reuses `NOTICE_TOAST_SECONDS`, `SPACE_LG`, `SPACE_SM`, `RADIUS_CARD`, and `CONTROL_HEIGHT_SMALL`; no new shared identifier or constant (`src/config.rs` read). |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | yes | `durability_warning` keeps its persistent, non-dismissible display until a durable retry clears it, and no store schema, write order, or sidecar behaviour changes (`.claude/skills/durability-invariants/SKILL.md` read). |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | Pure UI layout change; no child process or command construction (`src/exec.rs` / `.claude/rules/rust.md` checked). |
| Documentation — user-facing docs change in all three languages together | no | `CONTRIBUTING.md` checked; `README*.md` do not document banner vs toast coordinates on the Sessions page, and no new `tr` string is added. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | no | `CONTRIBUTING.md` checked; local immediate-mode UI drawing only, no network or `unsafe`. |
| Budgets — any new scan or output path states its byte and item ceiling | no | `src/config.rs` checked; no scan, file read, or subprocess output path added. |

## Flagged concerns

None.

## Acceptance

- `cargo test --locked session_notice_289_` passes all four requirement tests.
- `make q.fast` passes (`cargo fmt --check` and `cargo clippy --locked -- -D warnings`).
- `bash scripts/check-bands.sh` reaches no new tier (`largest_module_lines` stays below `10000`).
- `python3 docs/sdlc/changes/289-session-notice-shakes-terminal/mutations.py` catches every mutation.
- In `/Applications/Operon.app` after packaging from `main`, opening or closing a terminal tab or triggering a notice on the Sessions page shows the notice in the bottom-right corner while the terminal workspace stays motionless.

## Rejected alternatives

- Convert every `self.notice = Some(...)` call site to `notice_briefly` and keep the top banner for errors — failures and `durability_warning` would still push the terminal down and resize `tmux` whenever they arrived, and new call sites would regress silently.
- Reserve a permanent blank 48px strip above `ui_terminal_workspace` — wastes vertical terminal rows on every frame when no notice is present, directly against change 088's goal of giving the screen to the terminal.
- Float the overlay at `CENTER_TOP` or `RIGHT_TOP` on `Page::Sessions` — covers the workspace tab strip or the session header's `停止` verb; `RIGHT_BOTTOM` is already the agreed toast anchor from change 088.
