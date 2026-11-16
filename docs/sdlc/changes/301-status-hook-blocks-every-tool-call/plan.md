# Plan: the status hook blocks every tool call

- **Spec**: n/a (`bugfix` route — the bug report is in `./state.yaml`)
- **Approved**: 2026-11-16
- **Status**: approved

## Root cause

`managed_handler` in `src/tmux/hooks.rs` builds every handler as
`{type, command, timeout}`, and `install_claude_hooks` strips Operon's groups
from `~/.claude/settings.json` and re-appends them through it on every launch.
A handler is therefore always synchronous: Claude Code waits for
`claude-hook.sh` (a shell start plus a curl capped at 1.5 s) before every tool
call and after it, and the `"async": true` the person added by hand is dropped
on the next launch.

## The fix

1. `hook_runs_async(provider, event)` answers `true` for exactly Claude's
   `PreToolUse`, `PostToolUse`, `PostToolUseFailure`; `managed_handler` takes
   that answer and writes `"async": true` only when it is `true`. Nothing
   else is written for the other events, so Codex handlers (and their trust
   hashes) and Claude's synchronous events stay byte-for-byte as they are, and
   the person's hand-made settings now equal what Operon writes, so a relaunch
   no longer rewrites the file.
2. The ordering guard `state.yaml` decides, in `apply_hook_reading`:
   - `normalise_hook_event` reads, for Claude only, `tool_use_id`, a 64-bit
     hash of `tool_name` and `tool_input` (`tool_call`), and whether the event
     is a `PostToolUse`/`PostToolUseFailure` (`tool_finished`).
   - `HookStatus` keeps the last `HOOK_SEEN_TOOL_IDS_MAX` (64) tool ids seen
     and, while `AwaitingInput`, the `tool_call` that put it there.
   - A Working tool reading is late — the status stays, only `received_at`
     moves, no notice — when the row is `Idle` and its id was seen, or the row
     is `AwaitingInput` and either (a) it is the `PreToolUse` of the very call
     being waited on, or (b) it is a different call whose id was seen. The
     `PostToolUse` of the call being waited on always passes.
   - Codex and Antigravity readings carry no id, so nothing changes for them.

## Files that change

| File | Change |
|---|---|
| `src/tmux/hooks.rs` | `hook_runs_async`, `managed_handler(command, run_async)`, `HookReading`/`HookStatus` fields, the late-event check |
| `src/config.rs` | `HOOK_SEEN_TOOL_IDS_MAX` |
| `src/tests.rs` | `claude_tool_hooks_run_async_and_every_other_handler_waits`, `a_late_pre_tool_use_does_not_hide_a_permission_prompt`, `a_late_tool_event_does_not_reopen_a_finished_turn`, `a_late_tool_event_does_not_hide_a_question`; three `HookStatus` literals gain the new fields |
| `docs/sdlc/changes/301-…/` | state, plan, approvals, mutations, verification |

## Order of work

1. The four tests, watched failing before the fix.
2. `hook_runs_async` and `managed_handler`.
3. The reading fields and the late-event check.
4. `mutations.py`: async on a fourth event, async dropped from one of the
   three, async put on Codex, the permission-race check removed, the seen-id
   check removed, the waited-on call's `PostToolUse` refused — each turns a
   test red.
5. Land, then the ship skill: `/Applications/Operon.app` replaced.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| An async event whose stdout mattered | a refused tool, a missing brief | only the three tool events are async; `PermissionRequest` and `SessionStart` stay synchronous; the install test |
| Codex trust lost | every Codex hook "needs review" | Codex handlers unchanged; the install test and the existing hash tests |
| The guard swallows a real change | a working row shown as idle or waiting | it acts only on ids already seen (or the waited-on call's own `PreToolUse`); a new id passes — the finished-turn test's continuation step |
| A late `PreToolUse` of an earlier, different call | waiting row shown as working | accepted residual, in `state.yaml` |

## Proof of completion

The four named tests pass; `mutations.py` catches every mutation; CI green on
the branch head; after shipping, `/Applications/Operon.app` is the new build.
