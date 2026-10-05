# A review has no way to end

Route `bugfix`. Risk: no surface in `docs/sdlc/risk.yaml` matches
`scripts/check-review.sh`, `REVIEW.md`, `docs/sdlc/routes.yaml`,
`docs/sdlc/lessons.md` or `src/tests.rs`, so this change needs no reviewer and
cannot itself enter the loop it is closing. Autonomy `full`.

## The cause, named

`scripts/check-review.sh:563` voids every recorded verdict when the digest
moves, and the digest at `:221` covers every tracked path outside
`docs/sdlc/changes/`. So:

1. A reviewer records `approve-with-nits` at digest D.
2. The session takes the nit — any edit to a tracked file outside the change
   directory — and the digest becomes D'.
3. The gate refuses: `判定が古い diff のものです`.
4. The session requests another review at D'. The reviewer, reading the diff
   afresh, finds another nit. Return to 1.

Nothing bounds the repetitions, and nothing says a nit must not be taken. So
`approve-with-nits` is a non-terminating state: **each nit, when taken,
destroys the approval that accompanied it and buys another round.**

Measured on change 060: nineteen rounds. The last behaviour change was round
17. Rounds 18 and 19 changed comment prose, one test assertion, and one
mutation — no behaviour. Cost per round: a ~28-minute mutation run plus two
reviews.

Two aggravating facts, both measured rather than inferred:

- **`REVIEW.md` grades comment prose as Important.** "It claims something in a
  document that the repository does not do" makes a wrong sentence in a code
  comment a blocking finding, and §4 makes a mechanism without a guard one too.
  Rounds 18 and 19 were both legitimate under that policy. The policy is
  therefore part of the generator: it makes prose review-grade, and prose edits
  move the digest.
- **`scripts/check-review.sh` already says nits do not block** — at `:548` and
  at `:617`. The gate was right and was not obeyed. A statement inside a script
  is not a mechanism.

And the loop's other half, which is why it ran for days rather than hours: a
session that blocks on a reviewer's reply has no way to notice the reviewer no
longer exists. Change 060 waited on `review-060` and `review-060-rust`; neither
was in `ListAgents`, and one had left a mutation in the working tree
(`Ok(if remove_only { previous_keys.to_vec() } else { keys })` at
`src/tmux/hooks.rs:1789`) that was never restored.

## What changes

**1. `REVIEW.md` — a nit is never taken in the round it is raised.**
With the reason stated, because the reason is the part that transfers: taking
it moves the digest and voids the approval it came with. Open nits go to a
`carried:` list in `review.yaml` and become a follow-up change. Only an
Important finding may move the digest during review.

**2. `REVIEW.md` — prose is a nit.**
A finding whose whole fix is comment text, document text, or a change's own
paper trail is a nit, not an Important, unless the text would mislead a reader
into a wrong edit of the code. This narrows the rule that produced rounds 18
and 19 without giving up the guarantee behind it.

**3. `docs/sdlc/routes.yaml` — a review-round ceiling, per route.**
A sixth positional value beside the existing attempt ceiling, so a reader
comparing "3 attempts / 2 review rounds" sees both on one line. `bugfix` and
`refactor` get 2; `feature` and `security` get 3; `docs` gets 1.

**4. `scripts/check-review.sh` — the counter is printed and the ceiling is
enforced.**
`review.yaml` gains `rounds: N`. The first-view block prints `round N of M`
on every run, which is the part that reaches a session mid-loop: this script
is run by hand every round. Past the ceiling it refuses, and the refusal has
exactly one way out that is not another round — move the open nits into
`carried:` naming the follow-up change directory, and land. The block is
therefore a forcing function toward shipping, not another obstacle.

**5. `docs/sdlc/templates/review.yaml` — the shape, written down.**
`review.yaml` has no template; its shape lives only inside the parser. That is
how `rounds:` and `carried:` would be forgotten.

**6. A reviewer request is never blocking.**
`state.yaml` records who is being waited on and since when. If the verdict has
not arrived by the time the session would stop, the session escalates to the
user rather than waiting again. A reviewer that is not in `ListAgents` is not
coming back.

**7. `docs/sdlc/lessons.md` entry 038, with a Guard that is watched failing.**

## What proves it done

- A guard in `src/tests.rs` that drives `scripts/check-review.sh` over a
  fixture whose `rounds` exceeds its route's ceiling with no `carried:`, and
  sees exit 1; and over the same fixture with `carried:`, and sees exit 0.
  Watched failing by reverting the ceiling check.
- `every_route_the_pipeline_offers_names_stages_that_exist` updated to the
  six-value tuple and validating the ceiling as an integer ≥ 1.
- The three gates, `check-bands.sh`, and `pipeline-indicators.sh --lessons`.

## Deliberately not in this change

`scripts/check-review.sh` names three guards that do not exist anywhere in
`src/`: `the_review_gate_has_no_bypass` (:45),
`the_review_gate_names_surfaces_that_risk_yaml_still_has` (:229), and
`the_review_gate_requires_the_reviewers_review_md_names` (:230). So the
three-way agreement between `risk.yaml`, this script's `reviewer_for_surface`,
and `REVIEW.md`'s table is unguarded, and the script's own no-bypass claim is
unheld. That is the same defect class as lesson 037 and it is a change of its
own — writing three guards is not the work of the change that noticed them.
