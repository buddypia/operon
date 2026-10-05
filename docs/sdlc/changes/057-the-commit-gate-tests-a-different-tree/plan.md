# Gate the tree the commit lands in

Status: done

## What was observed, and what it decides

`.claude/hooks/gate-commit.sh:24` was `cd "${CLAUDE_PROJECT_DIR:-.}"`. That
answers "which project does this session belong to", and the question the gate
needs answered is "which tree will this commit land in". The two differ exactly
when work lands from a worktree, which is what `AGENTS.md` asks for.

So the fix is not a path adjustment. It is asking the right question, in the way
git itself answers it: a commit resolves its repository from
`GIT_DIR`/`GIT_WORK_TREE` if they are set, and from the directory it runs in and
that directory's ancestors otherwise. This document had that order backwards
until review; the measurement is under "Proof, as it came out".

## The decision, in one function

`cd` to `git rev-parse --show-toplevel`, evaluated from the directory the tool
reports it will run the command in, with `CLAUDE_PROJECT_DIR` kept only as the
last fallback:

```sh
running_in=$(printf '%s' "$payload" | jq -r '.cwd // empty')
[ -n "$running_in" ] && [ -d "$running_in" ] || running_in=.
landing=$(cd "$running_in" 2>/dev/null && git rev-parse --show-toplevel 2>/dev/null)
cd "${landing:-${CLAUDE_PROJECT_DIR:-.}}" || exit 0
```

Three sources, in the order of how well each knows the answer. The reported
`cwd` is where the command will actually run. `git rev-parse` from there is git's
own resolution, so an exported `GIT_DIR` is honoured — which is right, because
the commit will honour it too. `CLAUDE_PROJECT_DIR` remains for the case where
there is no payload and no repository, where it is no worse than before.

The payload is now read once into `$payload` instead of being consumed by `jq`,
because two fields are needed from it.

### And a redirected commit is refused, not followed

`git -C <dir> commit`, `--git-dir`, `--work-tree`: the landing tree is then not
the one the session is in, and these cargo gates would again vouch for a
checkout nobody here is looking at. Refused with that reason. `AGENTS.md`
already says it in prose, and the uncommitted 503-line copy of this hook already
refuses it, so the tracked copy gaining the same refusal makes the two agree
rather than diverge.

## Files that change

- `.claude/hooks/gate-commit.sh` — the resolution, and the redirect refusal.
- `src/tests.rs` — the guard.

## Risks

`gate-configuration` is `high paused` in `docs/sdlc/risk.yaml`, so this was not
started until the user asked for it; `state.yaml` records that. The specific risk
of getting it wrong is a gate that runs nowhere useful and passes everything,
which is change 050's defect one file over — so the guard asserts a **block**,
not a pass. A guard that asserted "the gate said yes" would be green for a gate
that had stopped looking.

Second risk: the redirect refusal matches command text, so it fires on a command
that merely contains `-C` before the word `commit` — building a git fixture in a
temporary directory, for instance. That was measured while writing this change:
the running copy refused the fixture-building command for exactly that reason.
Accepted rather than solved, because the alternative — parsing a shell command
to find out which directory a git invocation targets — is a worse thing to be
wrong about. The guard lives in `src/tests.rs`, where `cargo test` runs it
without a PreToolUse hook in the way.

## Proof of completion

- `the_commit_gate_runs_the_gates_in_the_tree_the_commit_lands_in` passes, and
  was watched failing against the unfixed hook first.
- The three gates.
- `every_shell_script_parses_under_the_oldest_shell_its_shebang_finds` still
  passes, which is what says the new code runs on a stock macOS `bash` 3.2.

## Proof, as it came out

- The guard was written first and watched failing against the unfixed hook:
  `left: Some(0) right: Some(2)`. Exit 0 is the whole finding — the unfixed gate
  did not refuse the commit, it **passed** it, having looked at a clean tree and
  concluded there was nothing to gate. The defect is not only "runs the wrong
  tests"; in this arrangement it is "runs no tests and says fine".
- Two mutations watched after the fix. Putting `CLAUDE_PROJECT_DIR` back in
  front of `landing` turned it red. Dropping the reported `cwd`
  (`running_in=""`) turned it red too — which is the mutation a weaker version
  of this guard would have missed: the first draft ran the hook *in* the landing
  tree, so the hook's own inherited working directory would have found it
  anyway. The guard now points everything except the payload at the wrong tree.
- `cargo fmt --check` silent, `cargo test --locked` `480 passed; 0 failed;
  6 ignored`, `cargo clippy --locked --all-targets -- -D warnings` silent.
- `/bin/bash -n` on the hook: parses under 3.2.

After review, three more measurements.

- git's precedence, measured rather than recalled. With the session's
  `GIT_DIR`/`GIT_WORK_TREE` set, `git rev-parse --show-toplevel` answers the
  worktree from a temporary directory and from `/tmp`; with both dropped it
  answers `fatal: not a git repository` from either. The pointers override the
  search. The script that measured it also demonstrated the hazard it is about:
  written without dropping them, its `git init` re-initialised this worktree's
  own git directory and its `git -C <fixture> commit` was aimed at this branch,
  where only the review gate stopped it. Entry 022's rule, broken by the script
  written to check entry 022's rule.
- Ten mutations watched after the fix, each caught by the case it belongs to
  rather than merely caught — the mutation script compares which arrangement
  string the failure was reported against, and refuses a substitution that
  matched nothing. The redirection refusal made never to match; the pointers
  dropped before resolving; `CLAUDE_PROJECT_DIR` back in front of the resolved
  tree; the reported `cwd` no longer read; both fallbacks deleted; only the
  `CLAUDE_PROJECT_DIR` fallback deleted; the existence check deleted; the
  unreachable-directory bail-out replaced with `|| true`; and the trigger both
  widened to match every command and narrowed to match none.
- Three cases were watched **failing to catch their own mutation** before they
  were fixed, which is the part of this change worth carrying forward. The
  pointers case: with neither tree carrying a `Cargo.toml`, `exit 2` naming
  `cargo fmt --check` was true of both trees, so it decided nothing until the
  wrong tree got a buildable crate and its refusal became `cargo test --locked`
  — a different sentence. The `CLAUDE_PROJECT_DIR` case: the hook's own working
  directory was a repository, so git answered from there and the fallback the
  case was named after was never reached. And the existence check, which had no
  case at all.
- Every one of those was found by the reviewer, not by the guard. Three review
  rounds, three findings, all real. The closing gate — `.claude/hooks/gate-commit.sh`,
  which is what `docs/sdlc/routes.yaml:46` puts the attempt ceiling on — has not
  failed once on this change.

## What review changed

`subprocess-safety-reviewer` returned `request-changes 2 3` on the staged diff
at digest `ea8436748af5c908`, after re-affirming the four questions an earlier
pass had answered. Both Important findings were outside those four questions,
and both were right.

**The redirection refusal had no guard.** Deleting the block left the suite at
480 passed, so `git -C <main checkout> commit` would have been followed and the
commit landed in a repository these cargo gates never looked at — the defect
this change exists to fix, invisible to the suite. Now a case.

**The stated precedence was inverted, and the fixture tested the route
production does not take.** `GIT_DIR`/`GIT_WORK_TREE` override the directory
search rather than yielding to it, and `.claude/scripts/worktree-init.mjs:278`
writes both into every worktree it creates — so in the arrangement this change
is about, the pointers decide and the reported `cwd` decides nothing. Measured:
with them set, `git rev-parse --show-toplevel` run from `/tmp` answers the
worktree. The fixture removed exactly those two variables, so the only route it
covered was the one a plain `git worktree add` or a fresh clone takes.

The reviewer's proposed fix — drop the pointers before resolving — was **not
taken**, and the reason is now a case. The commit obeys the pointers, so the
gate has to: unset them and a session whose pointers name tree A while its
working directory is elsewhere would have the gates run outside A and the commit
land inside it, which is this change's own defect in a new spelling. What was
wrong was the rationale and the coverage, not the behaviour. Both are fixed: the
hook and `lessons.md` state git's order correctly and say why honouring is
correct, and the guard now runs the hook four ways instead of one.

Round two found one more, and it is the best of the three. **The fourth case did
not exercise the fallback it was named after.** With no reported `cwd` the hook's
own working directory was still a repository, so `git rev-parse` answered from
there, `${landing:-${CLAUDE_PROJECT_DIR:-.}}` never reached its second term, and
deleting both fallbacks left the suite at 480 passed. The case's own label, the
**Guard** line in `lessons.md` and `state.yaml`'s contract all said that route
was covered. The cause is one line: the hook's working directory was a constant
in the harness rather than a field of the fixture, so no case could vary it and
every case shared it. Now `started_in` and `project_dir` are fields like the
rest, one case starts the hook outside any repository so the fallback is the
only thing left to answer, and the old run stays as an explicit negative control
— a hook that refused every commit would satisfy the other four cases and fail
that one.

The reviewer also **withdrew** its proposed `unset`, after measuring it: with
the pointers dropped before resolving, the pointers case reads
`left: Some(0) right: Some(2)`, the gate having looked at the wrong tree and
passed a commit bound for the right one. Recorded because the disagreement was
settled by a mutation rather than by either side restating itself.

Round three found the same failure a third time, and it is the one with real
teeth. **A sixth input the decision reads had no case varying it**: whether the
reported `cwd` exists, which the hook tests with `[ -d "$running_in" ]`.
Deleting the check left the suite green, and the two behaviours are not
equivalent. With a stale reported `cwd` — a deleted worktree, a session record
that outlived its tree — the check sends the question back to the directory the
hook is already in, which is the tree the commit lands in. Without it, `cd`
fails, `landing` comes back empty, and the hook gates `CLAUDE_PROJECT_DIR`: the
main checkout, which is not where the commit is going. It fails in the dangerous
direction. Now a sixth case, reporting a path that is not there.

That also corrected the three documents for the second time. `lessons.md`'s own
corollary had said "every input the decision reads is a field of the fixture"
and closed with "every such input is a named field now" — wrong inside the round
it was written in, because the existence of a path is a property of a value and
no field can hold it. The corollary is now **enumerate what the decision reads,
and vary every one of them**, with "make it a field" demoted to the usual way
rather than the rule.

Round three's nits, all three taken. The fixture asserts its own discriminating
premise — `cargo fmt --check` passing and `cargo test --locked` failing in the
wrong tree — which no case observes, so on a machine with no cargo on `PATH`
both trees would answer with the same sentence and the pointers case would go
back to deciding nothing. The `$TMPDIR` note covers `.git` as well as
`Cargo.toml`, and the two matter differently: a manifest above the fixture would
be loud, a repository above it would quietly turn the `CLAUDE_PROJECT_DIR` case
into a second copy of the own-working-directory case. And the payload write no
longer `unwrap`s between `spawn` and `wait_with_output`, where a panic would
drop a child unwaited and skip the fixture's removal.

Round four approved with four nits, and all four were taken. Two of them were
the gap between `lessons.md` entry 023's rule and the guard beside it — the
unreachable-`CLAUDE_PROJECT_DIR` branch and the trigger's non-matching outcome
were still constant across every case — and shipping a rule a guard does not
satisfy is the failure this change had already made three times. Eight cases
now, ten mutations, all caught by the case that owns them. The fixture's premise
assertions run cargo on the same `PATH` the hook gives it. The bands reading is
recorded in `state.yaml`'s own `bands` item rather than left for the next change
to find.

One correction to the reviewer, worth stating because it cuts the other way:
the case it designed for the bail-out branch — hook started outside any
repository, fallback pointed at a path that is gone, `exit 0` due — passes
against the mutation as well as against the fix. With the bail-out removed the
hook stays outside any repository, finds nothing changed and exits 0 either way.
What makes that branch observable is starting the hook in a tree that *does*
have something to gate. Watched green, redesigned, watched red.

Three nits from round one. The `--` on both `cd`s is taken. The `-C` regex matching anywhere in
the command text is accepted, with the reason already in **Risks** below and the
reviewer's agreement that it errs toward refusal. The unpinned
`git rev-parse --show-toplevel` step is accepted and the reason is written into
the guard beside the cases: inside a fixture whose first cargo gate cannot pass,
a subdirectory and the repository root are indistinguishable, because the one
place the hook uses a path relative to where it landed is the erosion count and
that sits behind the three cargo gates. Covering it means a buildable crate
inside a unit test. A case that would have passed either way was written, run,
and deleted rather than kept.

## What this does not fix

The hook that runs in this session is the main checkout's 503-line uncommitted
copy, resolved through `${CLAUDE_PROJECT_DIR}` by `.claude/settings.json`. This
change fixes the tracked 94-line copy, which is what a fresh clone gets and what
every session whose project directory is its own tree runs. It does not unblock
this session's commit. That needs one of: a session whose project directory is
the worktree, or the paused decision in change 054 about the uncommitted
installer. Both are the user's.
