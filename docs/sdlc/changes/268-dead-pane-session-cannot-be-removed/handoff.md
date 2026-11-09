# Handoff: a dead session's tab does not close on its ✕

- **Route**: bugfix
- **Attempts spent**: 0 of 3 (no fix attempted; the cause is not named)
- **Written**: 2026-11-09
- **Status**: abandoned (2026-11-10)

Stopped before the ceiling: the failure does not reproduce outside the person's
running Operon, and the evidence that separates the remaining hypotheses can
only be gathered at that window.

## Why it was archived (2026-11-10)

- The question ceiling (7 of 7) was spent without the observation that would
  separate the hypotheses below, and the person asked for the branch to land.
- The instance it was seen in is gone: `/Applications/Operon.app` now carries
  `OperonSourceCommit` `4a8cbc1` (pid 94202, started 2026-11-10 02:07), and
  change 273 rebuilt the tab strip since `0d5d02b` — 6px/2px tab margins
  instead of 8px/4px, a 24px row with zero button padding around the ✕, and a
  strip `CONTROL_HEIGHT` tall instead of `CONTROL_HEIGHT_SMALL + SPACE_SM`.
  That changes the ✕'s geometry; it is not shown to remove the cause, which
  was never named.
- ⌘W closes the tab, so the person is not stuck.

If the ✕ on a finished session's tab fails again on a build at or after
`4a8cbc1`, start a new change from the ruled-out table below and
`probes.patch`; the first observation to take is whether another tab's ✕ in
the same window closes, and whether the ✕ highlights under the pointer.

## The failure, exactly

Reported by the person (their words, then their answers):

> セッションでターミナルを終了して、Pane is dead と表示されてセッション名の×ボタンを押したのにセッションが消えない

- Which ✕: the terminal tab's ✕ (hover 「タブを閉じる」), not the session list row's ✕.
- What happened: the tab itself does not close.
- Notice: none appeared.

Observed on the person's machine (read-only, 2026-11-09 ~18:30):

- `/Applications/Operon.app` is built from `0d5d02b` (main HEAD); pid 31483,
  started ~16:24, main thread idle in `NSApplication run`.
- tmux: `operon-8e932ebf dead=1 status=0`; the pane shows
  `Pane is dead (status 0, Mon Nov  9 01:34:11 2026)`.
- Store: that session is `セッション1`, `Exited`, project design-lookbook, beside
  `セッション2` (Active, gemini) and `セッション3` (Active, terminal).
- Window screenshot: tabs 「セッション2」「セッション3」「セッション1」, the last
  one active, header 「セッション1 ⊘ 終了 · main · 5 ポート稼働中」 with
  「会話 ID から再開」.

## What was tried, in order

All as `#[test]` probes in `src/tests.rs`, saved in `probes.patch` beside this
file (reverted from the tree; `probe_268_user_store*` need
`PROBE_STORE=<copy of store-v2.json>`):

1. Exited session + real dead pane on the suite socket → `request_session_removal`:
   tmux killed, record removed, notice 「セッションの記録を削除しました…」.
2. `close_workspace_tab` on an Exited session's tab, then frames of
   `ui_terminal_workspace`: tab removed and stays removed.
3. A synthetic pointer click (accesskit bounds of the ✕) through
   `ui_terminal_workspace`, two tabs and one tab: tab closes.
4. The same click through the full `eframe::App::update`, with and without the
   inspector dock: tab closes.
5. Real tmux on the suite socket, `tools.tmux = true`, poll marks the pane
   `Exited` while its tab is open, then the click: tab closes.
6. Gemini agents, three tabs, 1512×900: tab closes.
7. Dead tab alone in a split pane: closes.
8. The person's own store (copied), its tmux names recreated on the suite
   socket, window 2560×1382, tabs opened in the person's order: tab closes.
   Same with press and release in one frame, and after a 1.2 s hover
   (tooltips shown): closes.
9. Synthetic CGEvent clicks into the person's Operon: not delivered (a click on
   「ホーム」 did not switch the page either), so inconclusive. The session runs
   under cmux.app, which has no Accessibility grant (`agent-desktop status`:
   `accessibility: denied`), so no input can be sent from here.
10. Asked the person four times for the ⌘W / other-tab observation; each reply
    was 「承認」 with no observation, and the window stayed unchanged.
11. The person then answered: **⌘W on 「セッション1」 closes the tab.** So
    `close_workspace_tab` works in the running app and only the ✕ click is
    lost. `probe_268_id_stability` (the person's store, 9 s of real frames
    with polls) shows the ✕ ids stable across frames in the harness, so a
    per-frame id shift is not reproduced either.

## What was ruled out

| Hypothesis | Ruled out by |
|---|---|
| kill-session fails on a dead pane with remain-on-exit | probe 1 and a bare tmux probe: exits 0, session gone |
| removal logic for `Exited` is broken | probe 1 |
| `WorkspaceTabs::close` or `close_workspace_tab` keeps the tab | probe 2 |
| `sync_workspace_live_sessions` reopens a closed finished tab | probes 2, 5, 8 (it only opens unseen live sessions) |
| the ✕ is covered by another widget in the default layout | probes 3, 4, 8 (accesskit shows one ✕ per tab at the click point; click lands) |
| the Active→Exited transition while the tab is open breaks it | probe 5 |
| split panes break it | probe 7 |
| fast click (press+release in one frame) or a shown tooltip eats it | probe 8 variants |
| the app is hung | `sample 31483`: main thread idle in the run loop, frames flushing |
| `WorkspaceTabs` can leave a tab in two groups, or a group outside the layout, so `close` removes a hidden copy | `probe_268_workspace_invariants_random_walk`: 20 000 walks of open/close/insert_tab/move_tab/select/retain, no violation |
| the terminal still holding the keyboard when the agent exits (the person's ^C^C) eats the click | `probe_268_focused_terminal`: terminal clicked to focus while Active, pane dies, ✕ still closes the tab |
| the harness used egui's default fonts | probes 4–8 run the full `update`, which calls `apply_appearance` → `configure_macos_fonts` |

| the tab is closed and then reopened | the person: ⌘W, which calls the same `close_workspace_tab`, closes it |

## Narrowest remaining hypothesis

Something present only in the running instance — state not carried by
`store-v2.json` (egui memory, an open popup or window, a stuck drag, the
bundled CJK fonts changing layout) — sits over or steals the ✕ click. ⌘W calls
the same `close_workspace_tab` without hit testing: if ⌘W closes the tab and the
✕ does not, the cause is in hit testing; if ⌘W fails too, the tab is being
reopened and the hit-testing hypotheses are dead.

## What a person has to decide

Nothing to decide; two observations only they can make in that window:
whether ⌘W closes 「セッション1」's tab, and whether 「セッション2」's ✕ closes its tab.

## State of the tree

- Branch `fix/268-dead-pane-session-cannot-be-removed`; only this directory
  added, no source change. `probes.patch` applies to `0d5d02b`, not to the
  tab strip after change 273.
- `src/tests.rs` restored; probes live in `probes.patch` only.

## If this becomes a lesson

Not yet.
