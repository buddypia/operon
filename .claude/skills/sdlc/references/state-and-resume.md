# The state of a change, and how to hand it over

Read when writing or updating `state.yaml`, or arriving on a change cold.
`docs/sdlc/templates/state.yaml` is the SSOT for fields and closed value sets.

## Read it first

Open `state.yaml` before `intent.md`, `spec.md`, or `plan.md`:

- `status` — which of the nine positions the change is in.
- `resume` — one sentence saying what to do next.
- `contract` — what is still `pending`, and the command that settles it.

## Non-progress statuses

`planning`, `ready`, `in-progress`, `reviewing`, and `done` are the happy path.
Stopping is a state, not the absence of one:

| Status | Enter when | Leave when |
|---|---|---|
| `awaiting-user` | question ceiling reached, or touching a `paused` surface in `docs/sdlc/risk.yaml` | the person answers |
| `blocked` | a question nothing in the repository can answer | answered, or recorded as an assumption |
| `failed` | route's attempt ceiling in `docs/sdlc/routes.yaml` spent | a person restarts it from `handoff.md` |
| `archived` | abandoned or superseded | never — start a new change |

## The question ceiling

Maximum seven questions (`questions: "2 7"` is spent, then ceiling). At seven,
stop asking, adopt recommended defaults, record the assumption in `resume` and
`spec.md`, and set `awaiting-user`.

## Writing `resume`

One sentence stating what to **do** next (not a multi-clause history report).
Update it whenever position changes.

## The contract & Backfilling

Each item is `<kind> <verdict> <what decides it>` (`spec.md` **Acceptance** is
authoritative):

- `machine` items name a command; `.claude/hooks/gate-stop.sh` refuses a finish
  while one is `pending`.
- `human` items name an observation (person on diff, guard watched failing,
  screen verified).
- `waived` requires its reason on the same line.
- Backfilled `state.yaml` files record how the change actually ended, without
  tidying history.
