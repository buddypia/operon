# Spec: the Dock icon counts the sessions that wait

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. The badge reads the number of waiting sessions, and there is no badge when
   the number is zero. — `dock_281_badge_says_how_many_wait`
2. The number is the length of the 要対応 queue (`attention_order`, which
   reads no filter of the session list — the toolbar's 要対応 N is the same
   expression); the Dock is told only when that number
   differs from the one it was last told, and it is told to remove the badge
   when the number returns to zero. —
   `dock_281_badge_changes_only_when_the_count_does`
3. Every frame keeps the badge in step, after the queue is settled. —
   `dock_281_frame_keeps_the_badge`

## Behaviour

The Dock icon wears a red badge with the count, as Mail's does: 「2」 while two
sessions wait. It follows the queue within a frame of the queue changing —
the frame wakes at least every two seconds, six with the window in the
background (change 275).

- **Empty** (nobody waits): no badge.
- **Not the bundled app** (`cargo run`, the test binary): nothing is set; such
  a process has no Dock tile of its own worth marking, and a test must not
  start AppKit.
- **Error**: none reaches the person. A missing class or a nil Dock tile
  leaves the badge as it was.
- **Quit**: the badge goes with the process.
- **Too large**: the Dock shortens a long label itself; the count is printed
  as it is.

No string: the badge is the number.

## Design

Placement: **NEW_IN_MODULE** — the count is `track_attention`'s
(`git grep -n attention_order`); the code is a new file under `src/app/`
because `src/app.rs` stands near the `largest_module_lines` ceiling, and the
platform call joins the notification calls in `src/sys.rs`.

- `src/app/dock_badge.rs`: `dock_badge_label(waiting) -> Option<String>`;
  `impl OperonApp { dock_badge_change() -> Option<Option<String>>`, which
  compares `attention_order.len()` with `dock_badge_shown` and records it;
  `track_dock_badge()`, which passes a change to `set_dock_badge` }.
- `src/app.rs`: field `dock_badge_shown: usize` (not persisted, starts at 0);
  `update` calls `track_dock_badge` after `track_attention`;
  `mod dock_badge`.
- `src/sys.rs`: `set_dock_badge(label: Option<&str>)` returns at once unless
  `is_bundled_app()`, then calls `unsafe fn set_dock_badge_impl`:
  `[[NSApplication sharedApplication] dockTile]`, `setBadgeLabel:` with an
  `NSString` from the existing `to_nsstring`, or nil to remove it, inside an
  autorelease pool, each receiver checked for nil. One new `unsafe` block with
  its `// SAFETY:` argument: main thread (eframe runs `update` there), nil
  checks, the autoreleased string outliving the call, selector signatures
  matching the `MsgSend0`/`MsgSend1` casts already in the file.
- `CLAUDE.md`: the module map names the new file.
- `README.md`, `README.ja.md`, `README.ko.md`, `docs/MANUAL.md`,
  `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md`: a clause or sentence each.
- No persisted shape, no child process, no background thread. The frame
  gains one integer comparison; the AppKit call happens on a change.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | The badge is drawn by the Dock. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No glyph. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | Selector and class names are AppKit's, each written once. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | Nothing is stored. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | No child, but a new `unsafe` block: `// SAFETY:` states the invariants, and `.claude/agents/subprocess-safety-reviewer.md` reads it. |
| Documentation — user-facing docs change in all three languages together | Yes | The README and the manual, in English, Japanese and Korean. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | One new `unsafe` block, approved by the person before the edit (approvals.log); no network. |
| Budgets — any new scan or output path states its byte and item ceiling | No | None. |

## Flagged concerns

- **The AppKit call cannot be exercised by the suite** — a test binary is not
  the bundled app and must not start AppKit. Resolved: the decision of what
  to show and when is tested; the call itself is read by the reviewer and
  looked at on the real Dock after packaging.
- **`unsafe_blocks` reaches its warn tier** — the count goes from three to
  four, which `docs/sdlc/bands.yaml` records without asking for action.
  Resolved: accepted; the person approved the block before the edit.

## Acceptance

- `cargo test --locked dock_281_` passes (three tests).
- `make q.fast` passes; the module-map and three-language guards pass.
- `python3 docs/sdlc/changes/281-dock-icon-does-not-say-agents-wait/mutations.py`
  catches every mutation.
- In the packaged app: an agent that stops on a question puts 「1」 on the
  Dock icon; answering it removes the badge.

## Rejected alternatives

- The badge field of a notification — it changes only when a notification is
  delivered, so it could not count down.
- A crate for Objective-C calls — a dependency for three messages the file
  already knows how to send.
