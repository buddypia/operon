# Spec: read the limits Claude Code already reports

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. A managed script is installed as Claude Code's `statusLine` command, beside
   the hooks change 015 already installs, in the same settings file and written
   the same way.
2. The script prints **nothing**. Whatever a status-line command writes becomes
   the line inside the terminal, and this is a way of reading what the CLI
   already says rather than a place to put a second status line.
3. A `statusLine` slot holding somebody else's command is never taken. An
   **empty** slot that Operon has filled before is also not taken: the person
   removed it, and putting it back on the next launch would be an argument.
4. Turning the hook setting off removes the reader and leaves every other
   setting in the file untouched.
5. The script posts at most once every `STATUSLINE_MIN_POST_INTERVAL_MS` per
   session. A status line runs several times a second while a response streams
   and the number it carries moves about once a turn.
6. The throttle spawns no process. Its clock is the payload's own
   `total_duration_ms`, read with parameter expansion.
7. A payload without `rate_limits` is dropped before anything else is done. Only
   a subscriber session emits it, and only after its first response, so its
   absence is the ordinary case.
8. Both `used_percentage` and `utilization` are read, and `resets_at` is
   accepted as a number or a string. A percentage outside 0–100 is clamped.
9. A usage reading is not an agent state: it never reaches the state machine and
   is not gated on a launch token, because a pane that outlived this process
   still knows the true numbers.
10. The toolbar draws one chip. Nothing is drawn until something has been said.
    A window at or past `STATUSLINE_WARN_PERCENT` draws that number, and only
    that number, in the warning ink. A reading older than
    `STATUSLINE_STALE_AFTER_SECONDS` is drawn faintly and says its age.

## Behaviour

The screen was agreed before this was written:

```
┌─ トップバー ─────────────────── 1280px ─┐
│ ● Operon  ホーム プロジェクト セッション           │
│                        5h 62% · 7d 18%   🔍 ＋ ⚙ │
└────────────────────────────────────────┘
```

Hovering gives 「5 時間枠 62%（あと 2 時間でリセット）」 and the seven-day line
under it.

The states that are not the happy one:

- **Nothing said yet.** No chip. This is every machine until the first turn of
  a subscriber session.
- **One window only.** The one that was reported is drawn; the other is absent
  rather than shown as zero.
- **No reset time.** The percentage alone, with no bracket after it.
- **Past 80%.** That number turns amber. The other stays muted, so a busy window
  does not make a quiet one look urgent.
- **Older than thirty minutes.** Both numbers go faint and the hover gains
  「{p0}の読みです」.
- **Somebody else owns the status line.** Nothing is installed, nothing is
  drawn, and their setting is untouched.

## Design

`src/tmux/hooks.rs` gains the reader beside the machinery it already has:

```rust
pub(crate) struct UsageWindow { used_percent: f32, resets_at: Option<u64> }
pub(crate) struct RateLimits { five_hour: Option<UsageWindow>, seven_day: Option<UsageWindow> }
pub(crate) fn parse_rate_limits(payload: &serde_json::Value) -> Option<RateLimits>
pub(crate) fn statusline_script() -> String
pub(crate) fn statusline_script_path(data_file: &Path) -> PathBuf
pub(crate) enum StatusLineSlot { Free, Ours, Theirs }
pub(crate) fn statusline_slot(settings, script, installed_before) -> StatusLineSlot
pub(crate) fn apply_statusline(settings, script, installed_before, enable) -> bool
```

`apply_statusline` returns whether anything changed, so a settings file that
would be rewritten identically is left alone — the rule change 015 established.

The reset time is drawn as a duration, not a wall clock. This crate has no date
library, adding one is a paused surface, and reaching `localtime_r` means
`unsafe`. "In two hours" is also the answer to the question being asked, which
is whether to start something long.

`src/app.rs` holds `rate_limits: Option<(RateLimits, Instant)>` — one reading
for the account, not one per session — and routes the event before the state
machine sees it.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | `text_muted`, `text_faint`, and `warning`, all existing roles used for what they are named for. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | The chip is two numbers and a separator. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | The event name, the script file name, the marker file name, the interval, the staleness window, and the warning share are all `src/config.rs` constants. The script names the event through the same constant the listener routes on. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes | Nothing persisted. A percentage is true for minutes; one restored from a previous run would be drawn as current. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | Operon spawns nothing new. The script is run by Claude Code, exits 0 on every path, prints nothing, and its post is capped at 1.5 seconds. |
| Documentation — user-facing docs change in all three languages together | Yes | One bullet in each README. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | The number is handed to a local script by a local CLI and travels over a `0600` Unix socket. Operon makes no request of its own; the OAuth endpoint is rejected below for exactly that reason. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | `STATUSLINE_MIN_POST_INTERVAL_MS` bounds the rate; the existing `HOOK_BODY_MAX_BYTES` bounds the body, since this arrives on the socket that already has one. |

## Flagged concerns

- **Editing another product's settings file again.** Resolved by requirement 3:
  a slot that is not ours is never taken, and one we filled and the person
  emptied stays empty. Both halves have a test.
- **A shell throttle is easy to get silently wrong.** It was got wrong: see the
  Acceptance. The end-to-end test is what found it and is what keeps it.

## Acceptance

- `cargo test --locked` passes, including:
  - `reads_both_spellings_of_a_usage_window`
  - `a_payload_without_limits_is_not_a_reading`
  - `clamps_a_percentage_to_its_range`
  - `never_takes_a_status_line_slot_that_belongs_to_someone_else`
  - `removing_the_reader_leaves_every_other_setting_intact`
  - `the_usage_reader_is_valid_shell_that_prints_nothing`
  - `the_installed_usage_reader_posts_a_reading_end_to_end`
- In the running app, with an authenticated Claude Code: one turn fills the
  chip in; `~/.claude/settings.json` gains a `statusLine` entry naming the
  managed script and nothing else changes.

## Rejected alternatives

- **The OAuth usage endpoint.** A cloud call, and one that 429s under polling —
  which is the reason for the status-line path in the first place.
- **The hidden-PTY `/usage` scrape.** It types into somebody's session.
- **Reuse the existing hook script with a new event.** It prints `{}` before
  anything else, and `{}` would become the status line.
- **A wall-clock reset time.** Needs the local offset, which needs a date
  library or `unsafe`. A duration answers the question better anyway.
- **One reading per session.** The limit is the account's. Two sessions of one
  account reporting different numbers means one of them is stale, and the
  newest is simply right.
