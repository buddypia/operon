# Intent: the Stop gate exits 2 with a syntax error on a stock macOS

- **Status**: approved
- **Opened**: 2026-09-17

## Problem

`.claude/hooks/gate-stop.sh` does not parse under `/bin/bash`:

```
.claude/hooks/gate-stop.sh: line 63: syntax error near unexpected token `;;'
.claude/hooks/gate-stop.sh: line 63: `    case "$status" in done|archived) continue ;; esac'
```

Exit 2 under `/bin/bash` 3.2.57; exit 0 under `/opt/homebrew/bin/bash` 5.3.20.
The shebang is `#!/usr/bin/env bash`, so which of those runs depends on what is
first on `PATH` — and a stock macOS has only the 3.2. There, this hook dies at
its first line of work, before it reads a single `state.yaml`, and a Stop hook
that exits 2 blocks the stop with a shell syntax error.

Everything that hook is for is therefore not happening on those machines: the
uncommitted-Rust-with-no-gate-run check, and the open-contract report that says
which changes still have a `pending` machine item.

The cause is not the one-line `case`, which is what it looks like. Measured:

| form | `/bin/bash -n` |
|---|---|
| one-line `case` inside `$( )` | syntax error |
| the same `case` over three lines, still inside `$( )` | syntax error |
| the same one-line `case` outside a `$( )` | parses |
| pattern written `(done\|archived)`, inside `$( )` | parses, under 3.2 and 5.3 |

bash 3.2 scans a command substitution by counting parentheses, so the unbalanced
`)` in the pattern `done|archived)` ends the substitution early and the `;;`
that follows has nothing to belong to. The `$( )` here opens at line 59.

## Who feels it, and when

Anyone running this repository's agent harness on a macOS without a newer bash
installed — which is the default state of the machine. The failure is loud (the
stop is blocked and the error is printed) but it is blocked *for the wrong
reason*, and the checks it exists to run never happen. It is also self-hiding on
the machine most likely to notice: this one has bash 5.3.20, installed on 16 Sep
at 02:36, so the hook works here and has worked here since.

## Desired outcome

1. `.claude/hooks/gate-stop.sh` parses under `/bin/bash`, so the Stop gate runs
   its checks instead of dying.
2. `KNOWN_UNPARSED_UNDER_BASH_3_2` in `src/tests.rs` loses its only entry, which
   `every_shell_script_parses_under_the_oldest_shell_its_shebang_finds` already
   requires: that test fails if a listed script starts parsing, so the fix and
   the removal of the excuse cannot be separated.

## Systems likely affected

- `.claude/hooks/gate-stop.sh` — one pattern, from `done|archived)` to
  `(done|archived)`.
- `src/tests.rs` — the exception list and its reason paragraph.

## Why this is not already done

`docs/sdlc/risk.yaml` classifies `.claude/hooks` as the `gate-configuration`
surface at tier `high`, autonomy `paused`. The pipeline's own rule for `paused`
is to stop before editing and ask, and `REVIEW.md` lists gate configuration among
the places a person must look. Change 050 found this while applying lesson 020's
rule to every script rather than to the one that taught it; it recorded the
finding, the measurement, and the fix, and did not apply it.

So this change is open and waiting on one decision: whether a session may make
that one-character edit to a Stop hook. Nothing else about it is uncertain — the
fix is verified under both interpreters and the guard that proves it is already
in the tree.

## Not in scope

- The rest of `gate-stop.sh`. Only the pattern that does not parse.
- The other `paused`-surface finding change 050 and 054 turned up: nothing
  committed installs `.githooks/installed/reference-transaction` where git looks
  for it, so the review gate is a file and not a gate in a fresh clone. Same
  surface, different decision, recorded in `scripts/check-review.sh`'s header and
  in lesson 019.
