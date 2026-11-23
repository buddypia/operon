# Prove it in the running application

Read when a change paints, alters what a feature shows, or when asked whether
a feature still works. A test proves a function; this proves the built binary
with a real tmux behind it (change 314).

```sh
python3 scripts/verify-app.py run --list
python3 scripts/verify-app.py run <feature>
```

Exit 0 pass, 1 a step did not hold, 2 it could not start; the last line is the
evidence folder. A failed step is a finding about the application or about the
steps: read the picture before deciding which, and never weaken a step to pass.
`python3 scripts/verify-app.py --help` has the verbs and the rest.

## Launch

`run` builds `target/debug/operon` and starts it with `HOME` and `TMUX_TMPDIR`
in a scratch folder and `TMUX` unset, so its data, the hook files it installs,
and its tmux server never touch the person's. The window shows on the person's
screen for the run. By hand: `launch --seed project`, then `tree`, `shot <name>`.

## Doctor

`python3 scripts/verify-app.py doctor` — read-only; run it first when anything
looks off. Each failure names its fix.

## Drive

Controls are pressed through the accessibility tree: the pointer is not moved
and nothing is typed, so presses only, no typing, no key chords. The window is
brought to the front when a run starts and before a press is re-sent, because the application answers a
press only while it draws; a run fails with "the press changed nothing" if
another window is kept over it. A control with no accessible name cannot be
pressed (the toolbar's gear is one).

A feature's steps live in its `docs/features/<id>/spec.md` under
`## Seen in the running app`, in the words the screen shows. The suite fails a
step whose words the source no longer draws, so a change that alters what a
feature shows updates its steps in the same branch, and a feature without
steps gains them when it is next changed.

## Evidence

`target/verify/<utc>-<feature>/`: a picture per `shot`, the window's tree and
verdict per step, the log. Not committed. Name the folder in the change's
contract, and show the person the pictures when a screen changed.

## Cleanup

`run` stops what it started on pass, fail, and signal;
`python3 scripts/verify-app.py stop` does the same after a `launch`. It ends
only the recorded process while it is still the recorded binary, never a
process by name, and removes the scratch folder, never the evidence.
