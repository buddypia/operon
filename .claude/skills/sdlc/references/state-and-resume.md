# The state of a change, and how to hand it over

Read when writing or updating a `state.yaml`, or when arriving on a change you
did not start. `docs/sdlc/templates/state.yaml` is the SSOT for the field set and
the closed value sets; this says how to use them.

## Read it first, before the prose

A change directory holds four documents and one data file. The data file is the
one to open first. `intent.md`, `spec.md`, and `plan.md` say what the change is
*for*; `state.yaml` says where it *is*. Reading the prose to work out the
position is what this file exists to stop.

The three fields that answer "what now":

- `status` — which of the nine positions the change is in.
- `resume` — one sentence saying what to do next.
- `contract` — what is still `pending`, and the command that would settle it.

## The four statuses that are not progress

`planning`, `ready`, `in-progress`, `reviewing`, and `done` are the happy path and
need no explanation. The other four are the reason a closed set is worth having.

| Status | Enter when | Leave when |
|---|---|---|
| `awaiting-user` | the question ceiling is reached, or the change touches a `paused` surface in `docs/sdlc/risk.yaml` | the person answers |
| `blocked` | a question exists that nothing in the repository can answer | the question is answered, or is reframed as an assumption and recorded |
| `failed` | the route's attempt ceiling in `docs/sdlc/routes.yaml` is spent | a person restarts it, usually from the handoff |
| `archived` | abandoned or superseded | never — start a new change instead |

A session that stops without recording one of these is indistinguishable, to the
next reader, from a session that finished. That is the whole argument for the set:
**stopping is a state, not the absence of one.**

## The question ceiling

Seven questions in a session, because the failure the ceiling prevents is real: a session that asks an eighth, ninth, and tenth question has
stopped making progress and started interviewing.

At seven: stop asking. Adopt the recommended default for anything still open,
write the assumption into `resume` and into the spec, and set `awaiting-user`.
The person can correct an assumption in one line; they cannot recover a session
that spent its context asking.

`questions: "2 7"` is spent, then ceiling. Count a question as asked when it was
put to the person and work waited on the answer — not a rhetorical one in a
report.

## Writing `resume`

One sentence. What to **do**, not what happened — `plan.md` already has what
happened. If it needs a semicolon it is doing two jobs.

Good:

> Guards are in and verified; what remains is entry 011 in `docs/sdlc/lessons.md`
> and a person on the diff, because gate-configuration is a paused surface.

Not good, because it is a status report and leaves the next action to inference:

> Most of the work is done. Some tests were added. There were some issues with
> the metrics.

Rewrite it whenever the position changes. A stale `resume` is worse than none: an
absent one is obviously absent, and a wrong one is read and believed.

## The contract

What `done` means for *this* change, beyond the three gates that apply to every
change. Each item is `<kind> <verdict> <what decides it>`.

- `machine` items name a command. `.claude/hooks/gate-stop.sh` reads them and
  refuses a finish while one is `pending`, so a machine item is a promise the
  repository can hold you to.
- `human` items name an observation — a person on the diff, a guard watched
  failing, a screen that looked right. Nothing can check these, which is exactly
  why they are written down rather than remembered.
- `waived` is a real verdict and needs its reason in the same line. Change 007
  waived its `bands` item because `steering_bytes` sat at diagnose deliberately;
  a `waived` with no reason is a `pending` that gave up.

Write the contract at stage 3, from the spec's **Acceptance** section. If the two
disagree, the spec is right and the contract is wrong.

## Backfilling

A change that predates its `state.yaml` gets one written as the change actually
ended, not as it would look if it had gone smoothly. `007`'s records a `waived`
band and a defect that outlived the change. A backfill that tidies the history is
a backfill that teaches nothing.
