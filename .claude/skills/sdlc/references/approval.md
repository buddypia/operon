# Who approves an artifact

Read whenever an intent, spec, plan, screen, or `human` contract item is about
to be called approved (sdlc 082).

## The rule

A person decides when a wrong answer would **change what the product is or where
it is going**, be **unrecoverable or costly to undo**, **remove or weaken a
protection**, or change what a user sees enough to read as **a different
product**. Everything else is decided by an evaluator that did not write the
artifact. Judge from the change's context, not from file names. When unsure,
escalate — `escalate` is a correct verdict, not a failure. An edit to this file,
to `scripts/check-approvals.sh`, or to the categories is always a person's.

Screens split by structure. A new screen or modal, a re-arranged layout, or a
feature removed or moved goes to a person. Wording, spacing, and colour or state
inside existing palette roles goes to the evaluator.

Each `paused` surface in `docs/sdlc/risk.yaml` splits by what the edit does, judged
from `plan.md` before editing; a diff that does more than its plan is Important:

| Surface | Person | Evaluator |
|---|---|---|
| `store` | persisted shape, schema version, migration, write order, cancellation | an edit touching none of these |
| `unsafe-and-path` | new or changed `unsafe`, `PATH` resolution | any other edit |
| `bundle-swap` | always | — |
| `gate-configuration` | relaxation: a check removed or narrowed, a bypass added, a tier lowered, what the main-branch judge reads changed | tightening: a refusal, check, guard, or surface path added |
| `dependencies` | a new crate or feature | a bump inside an existing requirement |

Per artifact: an **intent** goes to a person when it opens a direction or the
agent originated it, and to the evaluator when it restates the person's request
quoted inside it. A **spec** is judged against its intent, a **plan** against its
spec, a small **screen** against `screen.md` and then a screenshot, a `human`
**contract** item against the file that backs it. Plan mode is for a plan a
person decides.

## The evaluator

A fresh subagent, never the author. It reads the artifact, its referent, this
file, and the risk rows the plan names, and returns `approve`, `revise` with a
finding against the referent, or `escalate`. Two `revise` rounds, then
`escalate`: open-ended review of a moving target does not converge (035).

## The ledger

`approvals.log` in the change directory, one line per verdict, excluded from the
review digest:

    <utc> <by> <category> <artifact> <digest> <verdict> <reason...>

`by` is `auto`, `person`, or `escape`; `category` is `intent spec plan screen
contract` or a paused surface. The digest is the checker's `digest()`: 16 hex of
the artifact without its `Status` line, and a plan only up to its departures.

**A `person` line is written on the person's behalf**, so it carries what they
said. A `person` line quotes the person's own words in 「…」 (or "…") — the
answer they gave, or the request that asks for exactly what the artifact does —
and says why those words cover this artifact. A request counts only for what it
asks: one that does not name a relaxation, a removal, or a direction does not
approve it, and the person is asked instead. When an approval is read from a
request rather than given as an answer, the session's final report says so in
the person's words — which request, read as approving what — so they can take
it back. `scripts/check-approvals.sh` refuses a `person` line without a quote
from change 300 on (sdlc 300); earlier ledgers stay as written.

**An escape** is a defect a person found in something approved `auto`. Once its
lesson and guard exist, write it in the fixing change: `escape <category>
<NNN-slug/artifact> <that digest> lesson-NNN <what>`. The category then goes to
a person, the evaluator's verdict shown beside it, until `clean_run` approvals in
a row where the two agreed. Paused surfaces start there. Sample with
`bash scripts/check-approvals.sh --list`.
