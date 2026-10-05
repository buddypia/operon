# Review policy

The passes every change to Operon is reviewed against, whether the reviewer is
`/code-review`, `.github/workflows/claude-review.yml`, or a person. Stage 5 of the
pipeline in `docs/sdlc/README.md`.

One rule stands above the passes: **the agent that wrote a change does not
approve it.** A review by the session that produced the diff is a second opinion
from the same opinion.

The second rule makes the first one hold: **a verdict is about a diff, and merging
waits on the verdict, not on a reader.** Before change 035 this document ended at
a person finding the time, and 24 changes sat at `reviewing` with nothing saying
so.

Report findings with a `file:line` anchor and a failure scenario stated as
inputs or state → wrong outcome. A finding with no scenario is a preference.

## Important vs. Nit

**Important** — the change is wrong, unsafe, or breaks a documented guarantee:

- It produces a wrong result, panics, or deadlocks on a reachable input.
- It loses or corrupts persisted data, or breaks a durability invariant.
- It leaks a child process, or spawns one outside the budgeted wrappers.
- It bypasses a validation gate, or adds `unsafe` without the argument for why
  it is sound.
- It violates a policy in the table below.
- It claims something in a document that the repository does not do.

**Nit** — everything else: naming, ordering, a clearer expression of the same
logic, a comment that could say more. **At most five nits per review**, chosen
for value rather than order of appearance. Past five, say "further nits omitted".
Nit volume is how a review stops being read.

**Prose is a nit.** A finding whose whole fix is the wording of a comment, a
document, or a change's own paper trail is a nit — not an Important — unless the
text would mislead a reader into a wrong edit of the code. The rule above about
a document claiming what the repository does not do is about claims a person
would act on, not about every sentence that could be sharper. This is narrowed
deliberately: change 060's rounds 18 and 19 were both prose, both legitimate
under the unnarrowed rule, and both cost a full review cycle for no behaviour
change.

## A nit is never taken in the round it is raised

This is the rule the rest of the policy rests on, and it is mechanical rather
than a matter of taste.

`scripts/check-review.sh` binds every verdict to one digest, taken over every
tracked path outside `docs/sdlc/changes/`. **Editing anything to satisfy a nit
moves that digest, which voids the approvals the nit arrived with.** So taking a
nit does not cost a nit's worth of work — it costs a whole review round, which
produces another nit, which costs another round. Change 060 spent nineteen
rounds inside that loop; its last behaviour change was round 17.

Therefore:

- **Only an Important finding may move the digest during review.** Fix it,
  re-review, record the new verdicts.
- **An open nit goes into `carried:` in `review.yaml`** — verbatim, with the
  reviewer who raised it — and becomes a follow-up change. It does not block,
  and `scripts/check-review.sh` says so in its own output. The verbatim line
  is for the change that takes it up; beneath it goes a `person:` line in
  plain Japanese — what goes wrong, when, and whether the person must act —
  and that line, not the reviewer's, is what a person is shown. The gate
  refuses a carried finding without one (sdlc 085).
- `approve-with-nits` **means approve.** A session that treats it as a blocker
  has converted an approval into a loop.

`docs/sdlc/review-rounds.yaml` carries a review-round ceiling for each route in
`docs/sdlc/routes.yaml`, and `scripts/check-review.sh` prints `round N of M` on every
run. Past the ceiling it refuses, and the only exit that is not another round is
to carry the open nits and land — or, if what remains is genuinely Important and
unfixed, to write `docs/sdlc/templates/handoff.md` and hand it to a person.

A round is a return to the reviewers that a finding caused. A re-verdict made
necessary only because `main` moved under a reviewed branch — the change's own
lines untouched, nothing new but the merge of `main` and what it forces, such
as a renumbered change directory — is recorded under the round it re-confirms
and does not count against the ceiling. Asking a person to allow it would be
asking about a step another review undoes (sdlc 084).

## Waiting for a reviewer is not blocking

A review is requested, not awaited. Record in the change's `state.yaml` **who**
is being asked and **when**.

A requested reviewer is then in one of three states — running, finished and
reported, or finished and silent — and they have three different remedies.
`ListAgents` tells them apart; without it, the last two are indistinguishable
from the first.

- **Running.** Listed and working. Wait or stop, not both: either carry on with
  work that does not move the digest, or record `awaiting-user` and stop. Asking
  again here is what produces two reports on one digest.
- **Finished and reported.** The verdict arrived. Record it in `review.yaml`.
- **Finished and silent.** Listed as idle, or no longer listed, and no report
  ever arrived. This is the state that looks exactly like still thinking, and it
  has now happened three times. **Re-ask the same reviewer by name** with
  `SendMessage` — it still holds the review in its context and answers in full.
  Do not spawn a replacement: a replacement reviews the same digest and produces
  a second verdict that then has to be reconciled by hand, which is what change
  065 did.

If the verdict has still not arrived by the time the session would otherwise
stop, escalate to the user rather than waiting again.

A reviewer must not write to the working tree. It reads the index — `git show
:path` — because the tree may be shared and because a mutation left behind by a
reviewer that then disappears is unattributable. Change 060 lost days to exactly
that: `src/tmux/hooks.rs` carried a reviewer's mutation, uncommitted and
unexplained, after the reviewer was gone.

## The passes

### 1. Correctness

Does the change do what its `spec.md` and `plan.md` say, and does it hold for the
inputs it will actually meet? Empty, absent, interrupted, denied, too large,
non-UTF-8, and concurrent. Read enough surrounding code to judge the change —
review the diff, not the file.

### 2. Durability and subprocess safety

The two surfaces where a mistake is unrecoverable rather than merely wrong. Each
has a dedicated reviewer — `.claude/agents/durability-reviewer.md` and
`.claude/agents/subprocess-safety-reviewer.md`; delegate rather than duplicate,
and see the table under **What the gate requires** for which diff pulls in which.

### 2b. Rust craft

`.claude/agents/rust-reviewer.md`, against the axes in `.claude/rules/rust.md`.
The one that is Important rather than a nit, because no test can see it: work in
a per-frame draw path. `eframe`/`egui` rebuilds the UI every frame, so a
`Command::new`, a `std::fs` call, or an uncached parse reached from `fn update`
runs at frame rate and stutters the window.

### 3. Policy conformance

Every one of these is enforced by a test in `src/tests.rs`, and the suite runs
before this gate is reached — so a violation is a red commit, not a comment. Each
row names where the policy is written; the violation shapes are there, not
restated here.

| Policy | Written in |
|---|---|
| Colour | `DESIGN.md`, the three `*_PALETTE` tables |
| Icons | `ICON_VOCABULARY` in `src/glyphs.rs` |
| Identifier SSOT | `src/config.rs` |
| Transcript vocabulary | `TRANSCRIPT_VOCABULARY` in `src/transcript.rs` |
| Language split | `CLAUDE.md` — Japanese for people, English for code |
| Documentation | `README.ja.md` and `README.ko.md` move with `README.md` |
| Local-first | `CONTRIBUTING.md` |
| Budgets | the byte and item ceiling on any new scan or output path |
| Store schema | `STORE_SCHEMA_VERSION`, and a path for versionless stores |

### 4. Harness consistency

The documents an agent reads before it reads code. **A change that adds a
mechanism and not its guard is an Important finding** — the empty-guard rule in
`docs/sdlc/lessons.md`, and the one thing in this pass no test can check, because
the missing guard is the thing that would have checked it.

**A mutation sweep that never touches a claim has not tested the claim.** When a
change adds or alters a sentence a person reads — a notice, a refusal, a label —
its `mutations.py` must mutate the claim the sentence makes, not only the
counters that feed it: weaken the sentence, or make it promise something the
code does not do, and require a guard to go red. Read the mutation script beside
the diff. A sweep that moves only counters past a new sentence is an Important
finding, because the coverage it reports is along a different axis from the one
the change is about. Change 061 shipped fifteen mutations, every one caught, and
still went back twice; both defects were in what its new sentence claimed, and
no mutation asked. `evals/007-mutate-the-claim-not-the-counter.md` is the same
rule as a prompt.

The rest of the pass is already guards: paths named in documents, `CLAUDE.md`'s
module map, the six copies of the three gates, test erosion, `paths:` on every
rule, routes naming stages that exist, and `docs/sdlc/bands.yaml` against
`scripts/harness-metrics.sh`. Read their output, do not re-derive it.

## Excluded from review

Not read, and findings in them are not reported:

- `target/` — cargo's build cache.
- `dist/` — packaging output, owned by `scripts/replace-macos-bundle.sh`.
- `Cargo.lock` — reviewed only for whether a dependency was added, never line by
  line.
- `assets/` — binary fonts and icons; provenance is in
  `assets/agent-icons/SOURCES.md`.
- `src/i18n_tables.rs` — translation data. Reviewed for sort order and for
  whether a new message id has all three languages, not for content.
- Anything the suite already enforces deterministically, restated as a comment.
  If a test catches it, the test is the review.

## What the gate requires

`scripts/check-review.sh` runs from `.githooks/reference-transaction` — git's
own hook, handed a ref update after git has built the commits and before the
branch moves, which `--no-verify` does not skip — and refuses the update unless
every commit it judges records, in `review.yaml` beside its `state.yaml`, a
verdict from every reviewer it needs, carrying the digest of the diff that
verdict was about. The diff is the commit's own, against its first parent: for
a plain commit that is the index against HEAD, for an amend it is against
HEAD's parent, and `-a`, a pathspec, `-C` from another directory, or any
spelling at all arrive here as nothing but a commit. The record is read from
the commit too, so a verdict lands in the same commit as the code it judged, or
not at all. A commit that lands only the paper trail has nothing to review, and
passes.

**What is judged** is every commit a branch is about to reach for the first
time — any branch, from any worktree, checked out or not, and a detached HEAD —
that no other direct branch already reaches (a branch made symbolic to a tag
vouches for nothing, and is judged at its target when it is written): a plain
commit, an amend, a merge commit
as the diff against its first parent, each commit a rebase or a cherry-pick
replays (a rebase over a moved base re-reviews what it moves), a root commit, a
branch created at a commit nothing holds, a fast-forward to a stash, a tag, a
fetched commit, or a detached one. A ref that is not a branch vouches for
nothing; nine rounds of review found that every narrower rule was a way in.
**What is not judged** is a commit this branch's own reflog remembers — it was
on this branch before, so a reset back and forward, `ORIG_HEAD`, a checkout of
an earlier tip are not refused and the gate is not a ratchet. Past the reflog
(`gc.reflogExpireUnreachable`, thirty days) or on a branch that never had one,
a return re-judges the commit on its own record, which refuses only one that would not
pass today: returning to a commit that passed passes again, and moving the
branch back before the gate and forward across pre-gate history a month later
is refused, and made by hand with the hook off. The tables that decide what a
diff needs are read from both sides of it, so a commit that lowers `risk.yaml`
or `routes.yaml` is still judged by the table it lowered.

`.claude/hooks/gate-commit.sh` installs a trampoline into the repository's own
hooks directory — shared by every linked worktree, kept through every checkout
— before every commit made here. What it installs is `main`'s committed
trampoline, and only when there is none, this checkout's working copy: the
install runs *before* the cargo gates, so a trampoline edited to decide nothing
used to be written into the shared directory, go red on its pins, have its
commit refused, and stay — governing every worktree until the next commit from
a good tree. It is written beside and renamed over, so no other worktree runs a
half-written one, and the bytes are `fsync`ed before the rename and the
directory entry after it, so a power loss does not leave a zero-byte hook —
which bash exits 0 on and git reads as approval.

The trampoline runs `main`'s committed hook, and only when there is none, the
tracked hook from the main checkout's working tree, else this checkout's. That
order is what stops a branch carrying an older hook from downgrading the gate:
the main checkout's working tree is whatever branch it sits at, so preferring
it meant `git checkout old-branch` there put that branch's hook in charge of
every worktree. With no hook at any of the three it lets a checkout move to a
branch name and no branch move to a commit. The script the hook runs is `main`'s committed copy,
or the main checkout's working one only before a committed one exists — never
the judged branch's own, so a branch that stubs `scripts/check-review.sh` does
not judge itself, and never a working copy over a committed one, so an edit
interrupted mid-way does not judge either; the committed copy judges the change
that lands its successor. Every reader asks that a file's last line be its
marker, since `-s` is one byte deep. It refuses a `core.hooksPath` that would point git
elsewhere, and refuses the two plain spellings of a one-command override —
`core.hooksPath` and `GIT_CONFIG_*` — on any git command that could update a
ref. A fresh clone installs the trampoline once by hand. And
`.claude/hooks/gate-stop.sh` reads the branch itself before a session ends:
every commit since the last one it saw pass goes through the same script, so a
commit that reached the branch around the hook — an override spelled a way the
text filter does not read, a hook removed by hand — is named at the stop that
would have ended the session, and at every later stop until the branch is
clean; the stops it lets through are the retry Claude Code makes inside the
same turn after a refusal, which a hook cannot refuse without becoming a loop,
and one under `OPERON_SKIP_STOP_GATE=1`, which asks the session to say why. The
mark is one ref per branch, `refs/operon/audited/<branch>` with the branch's
slashes flattened, advances as each commit passes so an audit the hook's budget
cuts short resumes rather than restarts, forward to the tip the walk started
from, and back only to where an amend or a rebase parted the branch from it; a clone has none and
audits from the commit that added the hook; a branch whose history has no such
commit is not audited at stop, because the git hook run from `main`'s tree is
what guards it; and when the script cannot be had whole — every reader asks
for its last line, since a prefix cut at a statement boundary parses and passes
everything — the stop is refused, not the audit skipped.

The required set is the route's `review` column in `docs/sdlc/routes.yaml`
(`full`, `craft`, or `none`), **widened** by the surfaces the diff touches in
`docs/sdlc/risk.yaml`. A route may ask for more than the files touched; never for
less. The column said `yes` or `no` before change 035 — whether a person had to
sign off — and a commit judged while that table is still what history carries
reads them as what they meant: `yes` is `full`, `no` is `none`, widened as ever.
Which tree the script reads is the mode: no argument is the index when
anything outside `docs/sdlc/changes` is staged and otherwise the working tree, `--index` what a commit will
land, `--commit` a commit's own tree, which is what the hook reads; the three can
disagree while a table is edited and not yet committed, and the hook's answer is
the one that counts.

| Surface | Reviewer it adds |
|---|---|
| `store`, `bundle-swap` — persistence, migration, cancellation, the packaging and swap scripts, against `.claude/skills/durability-invariants/SKILL.md` | `durability-reviewer` |
| `unsafe-and-path`, `subprocess`, `gate-configuration` — `unsafe`, the `PATH` setup, `Command::new`, child lifecycle, and the hooks and settings that decide what every other gate permits | `subprocess-safety-reviewer` |
| `dependencies` — `Cargo.toml`, `Cargo.lock`: local-first is a promise a crate can break on its own | `rust-reviewer` |

Seven `Command::new` sites live outside those paths, so an added line holding
`Command::new`, `.spawn(`, `.output(`, or `unsafe` adds `subprocess-safety-reviewer`
whatever file it is in. Every `high` surface in `docs/sdlc/risk.yaml` has a row
here, so no route can land one on the deterministic checks alone.

It refuses one reason at a time, cheapest first: `review.yaml` unstaged or
absent; a reviewer not defined in `.claude/agents/`; a verdict that is
`do-not-approve` or carries an Important, from any reviewer that ran; a required
reviewer missing; a stale digest. A gate whose first line is a list is a gate
whose first line stops being read; `--all` prints them all.

**What it cannot do.** It establishes that a verdict exists, is fresh, is clean,
and names a real reviewer — not that the session ran that reviewer rather than
writing the line. That is the exposure of a person typing LGTM, and a smaller one
than a review that never happened and left no trace. Freshness is what does the
work: a verdict cannot outlive the code it judged. A commit with no change
directory is the one-line fix and passes silently, unless it is on a `high`
surface, a reviewed one, or adds a child or `unsafe` — those are refused. Code
landed the silent way is ungated by design. A commit made in a clone before its
first commit through Claude Code has installed the trampoline, or in one where a
person removed it, pointed `core.hooksPath` elsewhere by hand, or spelled an
override past the text filter — `HOME=`, an `include.path`, a quoted-apart
`hooksPath` — is skipped by git in silence, which is the exposure any git hook
has; the stop gate's read of the branch is what catches it, after the fact and
before the session ends. History from before the gate can be moved to, not
rewritten: a rebase that replays a commit from before `review.yaml` existed is
refused, and `git rebase --abort` recovers. Nothing here judges whether a design
is *good*, and no check does.

The four scripts that judge a commit run with `GIT_NO_REPLACE_OBJECTS=1`
exported. `git replace` writes a ref in `refs/replace/*` that makes every later
`git` call read a different object — a commit handed another tree, so the diff
the gate digests stops being the diff the repository keeps — and those refs are
fetchable rather than local-only.

Every bash script under `scripts/`, `.claude/hooks/` and `.githooks/` is parsed
by `/bin/bash -n` in the suite. They carry `#!/usr/bin/env bash`, so on a
checkout without Homebrew's bash they run under macOS's stock 3.2, and a gate
3.2 cannot parse is a gate that never refuses — which is what round 16 shipped
for the length of one round. Entry 019 in `docs/sdlc/lessons.md`.

`docs/sdlc/risk.yaml`'s `paused` tier is a different checkpoint, and an earlier
one: decided **before** editing, by the split in
`.claude/skills/sdlc/references/approval.md`. A diff on a paused surface that
does more than its approved plan said — a relaxation where the plan claimed a
tightening — is Important.

## Feeding findings back

A finding that recurs is a steering failure, not a review success. Twice is the
threshold, and `CLAUDE.md` owns where the correction goes.
