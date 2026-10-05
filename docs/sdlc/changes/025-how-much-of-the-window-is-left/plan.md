# Plan: read the limits Claude Code already reports

- **Spec**: `./spec.md`
- **Approved**: 2026-09-06
- **Status**: in progress

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | The event name, the two file names, the interval, the staleness window, the warning share. |
| `src/tmux/hooks.rs` | The reader script; the slot policy; the parse. Installed and removed alongside the Claude hooks. |
| `src/util.rs` | `time_until`. |
| `src/app.rs` | `rate_limits`; the event routed before the state machine; the chip. |
| `src/app/screens.rs` | One call at the right of the toolbar. |
| `src/i18n_tables.rs` | EN and KO rows. |
| `src/tests.rs` | The seven tests named in the spec's Acceptance. |
| `README.md`, `README.ja.md`, `README.ko.md` | One bullet each. |

## Order of work

1. Constants, then the parse, with its tests: both spellings, both types of
   `resets_at`, absence, clamping.
2. The script and the slot policy, with the settings tests.
3. The end-to-end test through the installed command, under a path with a
   space. **Before** the drawing, because it is the test that can find a shell
   bug and shell bugs are silent.
4. The routing, the chip, `time_until`, the i18n rows.
5. The three READMEs.
6. Gates, bands, commit.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The script prints something | A JSON blob where the status line was, in everybody's terminal | `the_usage_reader_is_valid_shell_that_prints_nothing` asserts empty stdout on a real run, twice. |
| Somebody's status line is replaced | Their own tooling silently stops | The slot policy, with both halves tested, and the install marker so an emptied slot stays empty. |
| The throttle does nothing | Thousands of posts per turn | The end-to-end test posts three times and asserts the middle one is dropped. |
| The reading is attributed to a pane | A stale session's numbers overwrite a live one's | It is not per pane at all: one reading for the account, newest wins. |
| A wall clock in the wrong timezone | A reset time that is hours out | There is no wall clock; the duration needs no offset. |

## Proof of completion

- `cargo fmt --check`, `cargo test --locked` seven higher with `6 ignored`,
  `cargo clippy --locked -- -D warnings`, `bash scripts/check-bands.sh`.
- Each guard watched failing by mutation.
- One turn of an authenticated Claude Code fills the chip in.

## Departures from the plan

- **The throttle was written wrong and shipped nothing.** The stamp was written
  with `printf '%s'` — no trailing newline — and read with
  `read -r last < "$stamp" || last=0`. `read` returns non-zero at end of file
  *even when it assigned the value*, so `last` was reset to `0` every time and
  the throttle never fired: every status-line tick posted. Every unit test
  passed, because none of them ran the script twice. The end-to-end test found
  it on the first run.

  Both halves are fixed and each alone is sufficient, which is why neither
  mutation alone fails; restoring both reproduces the defect exactly, and that
  is the mutation recorded. The comment in the script says so rather than
  claiming one of them is the fix.

  This is entry 013 again, a third time: a fixture is a claim about production,
  and a shell script tested by reading it is not tested.
- **The reset time is a duration, not a wall clock.** The agreed screen showed
  「14:20 にリセット」. There is no date library here, adding one is a paused
  surface, and `localtime_r` is `unsafe`. It reads 「あと 2 時間でリセット」,
  which is the same fact and answers the question — whether to start something
  long — more directly.
