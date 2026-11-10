# Plan: the Dock icon counts the sessions that wait

- **Spec**: `./spec.md`
- **Approved**: 2026-11-10
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/app/dock_badge.rs` | new: `dock_badge_label`, `dock_badge_change`, `track_dock_badge` |
| `src/app.rs` | `mod dock_badge`; field `dock_badge_shown`; `update` calls `track_dock_badge` |
| `src/sys.rs` | `set_dock_badge` and `unsafe fn set_dock_badge_impl` — one new `unsafe` block (person-approved) |
| `src/tests.rs` | three `dock_281_` tests |
| `CLAUDE.md` | module map line; the `waiting_reply.rs` line shortened |
| `README.md`, `README.ja.md`, `README.ko.md`, `docs/MANUAL.md`, `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md` | a clause or sentence each |
| `docs/sdlc/changes/281-dock-icon-does-not-say-agents-wait/` | `mutations.py`, `verification.yaml` |

## Order of work

1. The person's answer about the `unsafe` block, recorded in `approvals.log`.
2. The tests; the label and the change rule; the frame's call.
3. `src/sys.rs`.
4. The documents.
5. `make q.fast`; `cargo test --locked dock_281_`, `claude_md`, `harness`,
   `readme`, `manual`; `bash scripts/check-bands.sh`.
6. `mutations.py`, `verification.yaml`, the subprocess-safety reviewer and
   the empirical verifier.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| AppKit is called off the main thread or in a test | A crash, or a warning in the log | `set_dock_badge` returns unless bundled; its only caller is the frame; the reviewer reads the `// SAFETY:` argument |
| The Dock is messaged every frame | Wasted work sixty times a second | `dock_281_badge_changes_only_when_the_count_does` |
| The badge stays after the last answer | A count with nobody waiting | The same test's return to zero |
| A wrong selector | No badge, or an exception | Read by the reviewer; seen on the real Dock after packaging |

## Proof of completion

- `make q.fast` — passes.
- `cargo test --locked dock_281_` — `3 passed; 0 failed`.
- `python3 docs/sdlc/changes/281-dock-icon-does-not-say-agents-wait/mutations.py`
  — every mutation caught.
- `bash scripts/check-bands.sh` — `unsafe_blocks` at warn, nothing at propose.

## Departures from the plan

- The new module-map line took `always_loaded_bytes` to its diagnose tier
  (7 850). The line was cut to five words and change 279's line for
  `src/app/waiting_reply.rs` was shortened, which brings the reading back to
  warn; nothing those lines said is lost that the files' own headers do not
  say.
- The safety reviewer asked that the `// SAFETY:` argument say the void
  message's return is ignored; it does now.
