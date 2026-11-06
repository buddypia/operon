# Spec: readable state at the beginning of session tabs

- **Intent**: `./intent.md`
- **Status**: draft

## Requirements

1. R1: idle, running and done have recognizable different shapes: pause-circle, circular arrows, check-circle. Use the shared status glyph constants so tabs and rows agree. `session_tab_state_icons_are_recognizable` pins the marks.
2. R2: each session tab shows its state word before the title without hovering, using the current `SessionStatusKind` and palette tone. It updates when activity or lifecycle changes. `session_tabs_show_live_state_before_title` covers idle, working, waiting, exited, failed, cancelled, lost and unknown states and tab geometry.
3. R3: the existing icon hover explanation, truncated title, click, drag, close and document tabs keep their behavior. `workspace_session_tab_shows_its_status_glyph` and `workspace_pointer_` tests pin the existing interactions.

## Behaviour

The leading mark plus Japanese word is visible: pause-circle 指示待ち, circular arrows 実行中, check-circle 終了. A question needing a response stays 要対応 with the attention icon. Failure, cancellation and missing terminals retain their distinct cross, stop and minus marks rather than borrowing the success check. Hover explains the precise state as today.

No sessions means no tabs. A missing session retains the existing blank fallback. Starting and unknown keep their existing labels and marks; no unobserved result is called successful. Long titles truncate after the state, and many tabs scroll horizontally. Concurrent changes read the latest in-memory activity on each frame. No new I/O, error notice or output scan is introduced.

## Design

Placement: EXTEND. `ui_workspace` in `src/app/screens.rs` already draws `kind.icon()` before the title, but only hover names the state. Add the existing translated `kind.label()` beside that icon, outside the truncated title. Keep labels separate so existing glyph hover and title interactions survive.

Change `ICON_STATUS_IDLE` to Phosphor PAUSE_CIRCLE and `ICON_STATUS_RUNNING` to ARROWS_CLOCKWISE in `src/glyphs.rs`; update the icon explanation in DESIGN.md. Reuse the current status classification and group words, rather than inventing a second classification. No persisted shape, subprocess, background task, font asset or dependency changes.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — DESIGN.md | Yes | Existing palette.status(kind.tone()); no new colours. |
| Icons — src/glyphs.rs | Yes | Bundled Phosphor constants remain in ICON_VOCABULARY; document updated shapes. |
| Identifier SSOT — src/config.rs | No | No new identifier or external contract. |
| Durability | No | No store, cancellation path, migration or byte limit changes. |
| Subprocess safety | No | No child or launch changes. |
| Documentation — CONTRIBUTING.md | Yes | DESIGN.md internal design vocabulary only; no README changes. |
| Local-first — CONTRIBUTING.md | Yes | No cloud calls, telemetry, accounts or unsafe. |
| Budgets | No | Reads current memory only. |

## Flagged concerns

None.

## Acceptance

Named R1–R3 tests pass; each new guard is watched failing. Mutation sweep catches removed state text, changed idle/running marks and stale state text. make q.fast passes. CI passes before main merge. Package and verify installed app after landing, subject to the rule requiring permission before quitting an external user process.

## Rejected alternatives

Raster-generated icons: unnecessary asset and theme work for small status marks already covered by the bundled icon font. A hover-only word leaves the problem in the screenshot. Calling every finished session successful would hide failures. Renaming sessions does not reveal live state.
