# Plan: a backlog kept as GitHub issues, scored by ICE, handed out one item per session

- **Spec**: `./spec.md`
- **Approved**: 2026-11-27
- **Status**: approved

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
