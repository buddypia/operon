# Intent: a timeout tells the person `{timeout:?}` instead of how long it waited

- **Status**: approved
- **Opened**: 2026-09-10

## Problem

When a command run through `run_command_with_timeout` exceeds its budget, the
error a person reads is:

```
コマンドが {timeout:?} でタイムアウトしました
```

The placeholder is shown, not filled. `tr` looks up a message id and returns a
string; it does not interpolate. A message with a hole has to go through `tf!`,
which is what the call twenty lines below it does correctly:

```rust
tf!("コマンド出力が安全上限の {p0} KiB を超えました", p0 = COMMAND_OUTPUT_MAX_BYTES / 1024)
```

`CLAUDE.md` states the rule plainly — `tr` for a plain string, `tf!` for one
with holes, never `format!` — so this is not an unclear convention being
misread. It is the rule, written down, with a correct example in the same
function, and the wrong call sits above it. The id is not in
`src/i18n_tables.rs` either, so it falls back to the Japanese it was written
in, placeholder and all, in every language.

Found by accident: the suite failed under machine load, and the failure message
quoted the string.

## Who feels it, and when

Anyone whose command times out — a slow `git`, a `tmux` that will not answer, an
agent CLI that hangs. The timeout is the moment a person most needs the number,
because the question is always "was it nearly done, or is it stuck?", and the
answer is the one thing the message does not say.

## Desired outcome

A timeout says how long it waited. And the next `tr` with a hole in it cannot
reach a person, because something fails before it does.

## Constraints this change inherits

- User-facing text is Japanese; the Japanese string at the call site is also the
  message id, so changing it means a row in every table in `src/i18n_tables.rs`.
- `src/exec.rs` is on the `subprocess` surface in `docs/sdlc/risk.yaml`: medium
  risk, supervised.
- This must not be smuggled into change 035's diff. 035 is mid-review with
  reviewers reading a fixed digest, and `src/tests.rs` is partly staged for it.

## Systems likely affected

`src/exec.rs`, `src/i18n_tables.rs`, and `src/tests.rs` for the guard.

## Open questions

- What should the message say — the budget it was given, the time it actually
  waited, or both? They differ when the wait is cut short. **Answered by**:
  `spec.md`.
- The guard is the valuable half: no `tr(…)` argument may contain a `{…}` hole.
  Repo-wide there is exactly one violation today, so the guard would go green
  immediately after the fix. Should it also refuse a `tf!` whose argument has no
  hole, which is the same mistake mirrored? **Answered by**: `spec.md`.
