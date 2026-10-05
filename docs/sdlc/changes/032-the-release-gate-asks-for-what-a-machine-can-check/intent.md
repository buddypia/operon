# Intent: the release gate stops the session to ask for three things a machine can check

- **Status**: approved
- **Opened**: 2026-09-06

## Problem

The last boundary of a release is a question put to a person, and the question is
not a judgement call. It asks whether the three gates passed on this tree,
whether the bundle about to be installed is the one the packaging script built,
and whether the running app was quit. All three are facts about the working
directory at that instant. A person answering them is either re-running the same
commands the session already ran, or answering from memory — and answering from
memory is exactly the failure the gate was built to prevent.

So the gate costs what a real gate costs — the session stops, a human context
switches — and buys a confirmation that is weaker than the check it stands in
for. Worse, it is asked in a session the operator has already put into a mode
that grants blanket approval, which is where a prompt starts being clicked
through rather than read. A gate that is answered reflexively has already
stopped gating.

## Who feels it, and when

Every release, at the moment the verified build is ready to install: the session
has just run the three gates and the packaging script, has the evidence in
hand, and still stops to ask the operator to vouch for it. It bites hardest in
an unattended or blanket-approval session, where the stop is pure latency and
the answer is a reflex.

## Desired outcome

The release still cannot proceed unless the three conditions hold, but holding
them is established by checking, not by asking. A release off a tree that
passes, with an untampered bundle built from that tree, and no Operon running,
proceeds without interrupting anyone. Any of the three failing — or being
impossible to establish — still stops and still asks, and says which one and
why.

The gate must get *stricter*, not looser: today a person can answer "yes" to all
three while none of them is true.

## Constraints this change inherits

- macOS only.
- `/Applications/Operon.app` is production; nothing may make it reachable by a
  session that has not established the three conditions.
- The gate lives in a hook, and a hook runs in every permission mode. That is the
  property being kept, not worked around.
- Whatever replaces the question must fail closed: an inconclusive check is a
  stop, never a pass.

## Systems likely affected

`.claude/hooks/guard-bash.sh` holds the gate. `docs/sdlc/README.md` states the
policy it enforces, including the sentence that says this boundary is the one
automation cannot cross on its own. `src/tests.rs` is where the pipeline's own
mechanisms are guarded.

## Open questions

- Do the other three `ask` decisions in the same hook — `gh release`,
  `git push`, `git tag` — get the same treatment? **Answered by the operator:
  no. This change is scoped to the bundle swap; the other three stay as they
  are.**

## Not in scope

- The two `deny` decisions. They are about records a rerun cannot rebuild and
  are not questions.
- `gh release`, `git push`, `git tag`.
- `scripts/package-macos.sh` and `scripts/replace-macos-bundle.sh`. The swap
  transaction itself is unchanged; only what is established before it is
  entered.
- The permission system. `bypassPermissions` never applied to hooks and this
  change does not make it apply.
