# Report: 036 — two claims the app does not keep, and a third found while fixing them

- **Spec**: `./spec.md`
- **Plan**: `./plan.md`

## What was asked, and what was found

The request was to verify that the mechanisms are actually
usable, UI included. The verification was not a code read: a throwaway git
worktree at `HEAD` got a temporary test that drew each screen for real, twice
per screen, and collected the text of every `Shape::Text` egui produced. That is
what a person sees, and it disagreed with the code in two places — and then in
a third, which turned out to be the largest.

Thirteen of the fifteen implemented rows reach the screen and work. The four
findings below are what did not.

## 1. A choice a person made, on a screen that hid it

Change 031's spec drew the account row between the agent and the model. It
landed between the session name and the model — both of which live inside
`3. 詳しい設定（任意）`, a `CollapsingHeader` with `default_open(false)`. So the
row was consistent with its neighbours and invisible by default: the frame audit
of `ui_overview` painted the heading and nothing inside it, and `アカウント`
appeared on no screen.

The row is now in step 1, under the agent cards, behind a separator. And because
the general shape of the defect is "a folded section hid a decision", the folded
header now carries a summary of every choice made inside it —
`3. 詳しい設定（任意）· モデル opus · 権限モード acceptEdits · ほか 1 件` —
capped at `LAUNCH_DETAIL_SUMMARY_MAX` so the header cannot grow past its line.

## 2. Two displays disagreeing about one command

The session card printed `native resume: claude --resume abc-123` while the
button above it promised `引き継ぐ設定: 権限確認をすべてスキップする、モデル opus`.
The launch was correct — `resume_command_with_launch_options` really does carry
them — and the line was a different function, `native_resume_display`, which
takes a provider and an id and has nowhere to put them.

The card calls the same function the resume calls now. The other two call sites
of `native_resume_display` deliberately do not change: the copy button on a
discovered CLI session and the notice when one is adopted are about a
conversation Operon did not launch, so there is no launch command for them to
carry, and a guard says so in both directions.

## 3. The one that was not on the list

Found while writing the spec for the other two: `agent_mode_input`,
`agent_effort_input`, and `agent_flag_inputs` were written by no drawing code
anywhere. `agent_mode_options`, `agent_effort_options`, and `agent_flag_options`
had no caller outside `src/agents.rs`. Change 012 had removed the editor that
fed them — correctly, it was unreachable — and left the three fields, the tables,
and the check that reads them.

Two consequences. The permission mode and the switches each CLI accepts could
not be chosen from this application at all. And `launch_needs_acknowledgement`,
which decides whether 「理解したうえで起動する」 must be ticked, read only those
three fields — so the warning and the checkbox could never appear, and
`claude --dangerously-skip-permissions` typed into 起動コマンド launched with
nothing asked.

The person's answer was to restore the pickers and to make the gate read the
command. Both are done. The gate now also reads the typed line, matching whole
arguments against the `dangerous` entries of the three flag tables — not a
substring rule, which would fire on a path and miss a rename.
`LaunchAcknowledgement` carries the custom command, so consent for one line is
not held after it is edited into another. Lesson 017.

## Departures from the plan

- **`dangerous_launch_arguments` was not written.** The plan had it as a
  derived list for the guard to read. The guard drives the real function from
  the tables instead, which is the same coverage without a function whose only
  caller is a test.
- **`the_launch_screen_offers_every_mode_and_switch_the_agent_has` is two
  halves.** A folded `CollapsingHeader` does not paint its contents and egui
  gives a test no way to open one without a pointer, so the guard runs the real
  drawing for the controls and reads the call site for the reachability. The
  doc comment says which half is which; the second half is the one that was
  actually broken.
- **The reasoning-effort picker was added, and was not in the agreed screen.**
  The screen the person approved showed the mode combo and the switches. Effort
  is the third of the same dead trio, and restoring two thirds of a thing would
  be a gap no later reader could reconstruct. Stated here and in the spec rather
  than left to be discovered.

## Costs

`steering_bytes` was already at `propose` before this change began — change 030
owns its reduction and is `awaiting-user`. Lesson 017 adds 2,378 bytes to it,
which is not nothing on a metric that is over: recorded rather than waived
quietly, because the alternative was not writing down a lesson about a safety
gate that had been dead for four changes.

## What the review found that the build had not

`rust-reviewer` returned **do-not-approve with two Important** on the first
read, and both were in the safety fix itself.

**`--flag=value` walked straight through the gate.** `command_needs_acknowledgement`
matched whole whitespace-separated tokens, so it saw
`--sandbox danger-full-access` and not `--sandbox=danger-full-access`.
`is_safe_agent_command` permits `=` on purpose, and clap and commander both take
the joined form — so the hole finding 4 exists to close was still open in its
other spelling, and the doc comment claimed the match avoided both failure modes
when it had traded one for the other. Each token is split on `=` now. The guard
builds every dangerous switch both ways and adds
`claude --permission-mode=bypassPermissions`.

**A mode chosen after a switch left the switch on and unreachable.** The
checkbox for a `conflicts_with_mode` switch is disabled while a mode is set,
which handles ticking it second — not setting the mode second. Tick Codex's
`approve-for-me` with no mode, then choose `workspace-write`: the checkbox
redraws ticked and greyed, so it cannot be unticked; the launch button stays
enabled; and the only refusal is `resolve_agent_flags` after the click. The
screen offered a launch it would always reject. `set_launch_mode` is the mirror
of `toggle_launch_flag` and drops what the mode invalidates.

The guard the plan named for that risk called `resolve_agent_flags(agent, "", …)`
with an empty mode, so it exercised the `group` half and never the
`conflicts_with_mode` half at all.

A second round then found that the fixed guard still could not see an
over-clearing setter: `approve-for-me` is Codex's only `conflicts_with_mode`
switch, so a `set_launch_mode` that cleared the whole list left an empty set
`resolve_agent_flags` accepts just as happily as the right one. `search` is
ticked beside it now and asserted to survive — watched failing with the retain
replaced by `clear()`, which the previous shape passed.

## Verification

- `cargo fmt --check` — silent.
- `cargo test --locked` — 455 passed, 0 failed, 6 ignored.
- `cargo clippy --locked --all-targets -- -D warnings` — silent.
- Eight guards, thirteen mutations, each watched failing:
  the account row folded away again; the header no longer naming the model; the
  section no longer calling the controls; the effort picker dropped; a ticked
  dangerous switch no longer gated; a typed dangerous switch no longer gated;
  group exclusivity dropped; the card dropping the carried options; the copy
  button losing the display it needs; the joined `--flag=value` form escaping the gate; a mode
  leaving a switch it invalidates; a mode clearing every switch rather than the
  conflicting one; and the header no longer counting what it left out.
