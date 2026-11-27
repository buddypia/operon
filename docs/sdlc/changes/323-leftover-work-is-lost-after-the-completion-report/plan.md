# Plan: a backlog kept as GitHub issues, scored by ICE, handed out one item per session

- **Spec**: `./spec.md`
- **Approved**: 2026-11-27
- **Status**: done

Interrogated before acceptance. What breaks: nothing in the application — no
`src/` file but `src/tests.rs` changes. What was assumed: GitHub returns an
accepted comment when the comments are read back (the spec states the limit),
and every session uses the same signed-in `gh`. The alternatives were a file in
the repository, labels, assignees and a local lock; the spec rejects each. What
proves it done: the eleven tests, the sweep, and one item taken through its
whole life on the real repository.

The one edit to a paused surface is the allowlist line in
`.claude/config/worktree-policy.json`: a relaxation, which the person approved
on 2026-11-27. Its effect, not only its size: main today admits reads of
GitHub and nothing that writes there; with this line, from main and with any
arguments, the script files, edits and closes issues and posts and deletes
comments. The line trusts the script's content, so the script's path is added
to the `gate-configuration` row of `docs/sdlc/risk.yaml` (a tightening): a
later edit to it is then reviewed as an edit to a gate. Nothing else under gate
configuration is edited.

## Files that change

| File | Change |
|---|---|
| scripts/backlog.py | new: `add`, `list`, `claim`, `release`, `score`, `done` over `gh api` |
| .claude/skills/backlog/SKILL.md | new: when to record, how to score, how to take and finish an item |
| `AGENTS.md` | a When-line for the skill; existing lines shortened to stay under the ceiling |
| `.claude/skills/sdlc/SKILL.md` | §10 gains one bullet sending leftovers to the backlog skill |
| `.claude/config/worktree-policy.json` | one allowlist pattern for the script on main |
| `docs/sdlc/risk.yaml` | the script's path added to `gate-configuration` |
| `src/tests.rs` | eleven `backlog_*` tests, the stand-in `gh`, the bounded runner |
| `docs/sdlc/changes/323-leftover-work-is-lost-after-the-completion-report/` | `mutations.py`, `verification.yaml`, `approvals.log`, `state.yaml` |

## Order of work

1. In `src/tests.rs`: the fixture (a directory holding a stand-in `gh` and its
   state file), the runner, and the tests for requirements 1 to 9. Watched
   failing: the script does not exist.
2. scripts/backlog.py: the `gh` call and its three failures; `add` and `list`;
   `claim` and `release`; `score` and `done`. The tests go green in that order.
3. The test for requirement 10, watched failing; then the allowlist line and
   the `docs/sdlc/risk.yaml` path.
4. The test for requirement 11, watched failing; then the skill, the `AGENTS.md`
   line with its cuts, and the `sdlc` bullet.
5. `make q.fast`, `cargo test --locked backlog_`, and the tests that pin
   `AGENTS.md` and the skills (`harness_`, `every_skill_`, `one_skill_`).
6. On the real repository: add a trial item, list it, claim it, release it,
   close it. Record what was seen under Departures if it differs from the spec.
7. `mutations.py`, `verification.yaml`, the empirical verifier, `state.yaml`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The stand-in `gh` agrees with the script and not with GitHub | tests pass, the real call fails | step 6 runs every sub-command against the real repository |
| A cut in `AGENTS.md` removes words a test pins | a `harness_` or `one_skill_` test fails | step 5 runs them before the commit |
| The allowlist pattern admits more than the script | a neighbour command runs on main | requirement 10's test names the neighbours and expects each refused |
| Two sessions keep the same item | both start the same work | requirement 6's test runs claims in parallel against one item and against several |
| A failed `claim` leaves its comment behind | an item nobody works on stays held | requirement 9's test fails the read-back and expects the claim removed |
| The skill grows a steering band | `scripts/check-bands.sh` names a new breach | the bands contract item |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked backlog_` — `test result: ok. 11 passed; 0 failed`.
- `python3 docs/sdlc/changes/323-leftover-work-is-lost-after-the-completion-report/mutations.py`
  — every mutation caught.
- `bash scripts/check-bands.sh` — no breach that main does not already have.
- The trial item's issue on GitHub: closed, with one closing comment and no
  claim comment left.

## Departures from the plan

- **A claim carries no token.** The spec gave the claim line a token to tell
  one's own claim from another's. The comment's id, which GitHub returns when
  the comment is posted, already does that, so the line holds only who claimed.
- **Who claimed is the directory name alone.** The spec said host name and
  directory. Step 6 found the repository is public; a host name is the person's
  and the first trial claim wrote it into a comment (deleted with the claim by
  `release`). The skill now says every item is readable by anyone.
- **Pages are read one by one, not with `--paginate --slurp`.** `--paginate`
  reads every page there is; the spec's ceiling of ten pages needs the page
  number in the address, so the script asks for `page=1` to `page=10` and stops
  at the first short one.
- **GitHub's issue listing trails a write by a moment.** Step 6, on the real
  repository: `list` straight after `add` answered 「backlog は空です」, and the
  same command seconds later showed the item. Two consequences. The check that
  closes the later of two items filed at the same moment rests on that listing,
  so on GitHub it is best effort: both can stay open until the next `add` of
  the title, which is answered with the earlier number. And `list` can show an
  item as free, or as open, for a moment after it was claimed or closed.
  Claims do not rest on the listing: they are judged from the issue's own
  comments, which in the trial showed each write at once (`claim`, then
  `claim 1` refused with exit 3, then `claim --json` answered `null`).
- **Step 6 as run** (2026-11-27, issue 1 of the repository): `add` printed the
  number and address; `list` showed it; `claim` took it and printed its text;
  `claim 1` exited 3 naming the holder; `score` changed 1 to 24; `release`
  freed it and `claim --json` took it again; `done` closed it as completed.
  Left on the issue: the release reason and the closing comment, no claim.
- **Three defects the verifier reproduced, fixed with its tests kept.** A
  `release` reason (or any comment) quoting a claim line held the item again:
  a claim is now the whole comment in the script's own shape. `claim` by
  number took a closed item: a closed item is refused by `claim`, `release`,
  `score` and `done`. Exactly 1000 items were reported as more than 1000: one
  page past the ceiling is read to learn whether it exists. Tests:
  `backlog_release_reason_cannot_forge_a_claim`,
  `backlog_claim_by_number_refuses_a_closed_item`,
  `backlog_list_of_exactly_one_thousand_says_nothing_of_more`; and
  `backlog_allowlist_refuses_more_neighbours`, which held.
- **Only an owner, member or collaborator can hold an item.** Not in the spec.
  The repository is public and anyone may comment on an issue, so a claim
  counts only when GitHub marks its author as one of those three.
- **Fifteen tests, not eleven**, for the two entries above.
- **The always-on files lost three phrases** to pay for the When-line and stay
  under the byte band: the title's "Claude Code notes" and the sentence
  introducing the import in `CLAUDE.md`, and the count of ignored tests in
  `AGENTS.md`.
- **The sweep runs as five groups.** The first CI run failed one obligation:
  the sweep, 52 mutations each judged by all fifteen tests, took 28 minutes
  and CI gives an obligation 900 seconds. Nothing was wrong with a guard; the
  whole sweep had passed here. Each group now names only the tests that guard
  it. Recording each group as its own obligation was refused by the second CI
  run: the gate admits one sweep command, the change's own file with nothing
  after it. So the file run bare runs the five groups in turn, four to
  seven minutes here on a loaded machine. Two attempts of three spent.
