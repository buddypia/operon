# Plan: merging is decided by a fresh verdict, not by who is available

- **Spec**: `./spec.md`
- **Approved**: 2026-09-06
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `scripts/check-review.sh` | new. The six refusals, one reason each, and the first-view block |
| `docs/sdlc/routes.yaml` | fifth column becomes `full` / `craft` / `none`; header comment moves with it |
| `REVIEW.md` | "When a human must look" becomes what the gate requires; rewritten no larger |
| `.claude/hooks/gate-commit.sh` | runs the review gate ahead of the three gates, outside the window the hook timeout can cut off |
| `docs/sdlc/README.md` | stage 5 row and the stage 5 section |
| `.claude/skills/sdlc/SKILL.md` | the route table's `human yes` line |
| `src/tests.rs` | three guards, and the routes guard's vocabulary |
| `docs/sdlc/changes/035-*/review.yaml` | this change's own verdicts, produced by the three reviewers |

`docs/sdlc/risk.yaml` is **not** edited. Its `paused` tier is a different
checkpoint and stays exactly as it is. (One comment line in its header, which
said `high` meant a person on the diff, was reworded in round three to say what
`REVIEW.md` now says; no surface, tier, or path moved.)

## Order of work

1. `scripts/check-review.sh` first, driven by hand against change 032 — which is
   committed, so its diff is empty and every refusal is reachable cheaply. Watch
   all five refusals before any hook reads it.
2. `docs/sdlc/routes.yaml`, then the routes guard in `src/tests.rs`. These move
   together or the suite is red between them; the tree does not compile-and-pass
   in between, and that is the one step where it cannot.
3. Wire into `.claude/hooks/gate-commit.sh`. Watch a commit refused for a stale
   verdict and then succeed.
4. `REVIEW.md`, `docs/sdlc/README.md`, `.claude/skills/sdlc/SKILL.md` — measuring
   `steering_bytes` before and after, because `REVIEW.md` is inside the glob and
   the metric is already at `propose`.
5. The two new guards in `src/tests.rs`, each watched failing by inverting what it
   guards.
6. Run the three reviewers on this change's own diff and write its `review.yaml`.
   The change closes under its own mechanism or the mechanism does not work.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The digest is unstable — it changes when nothing meaningful did | every verdict is stale, every commit refused, and the gate gets switched off within a day | step 1 drives it by hand against a real change; the exclusion of `docs/sdlc/changes/` is what makes it stable, and `the_review_gate_has_no_bypass` keeps the exclusion from widening |
| A session writes its own verdict lines instead of running the reviewers | the gate passes on an unreviewed diff | not caught, and not claimable — stated in `REVIEW.md` and in the spec's first flagged concern |
| The routes column changes and one of the four documents that name routes does not | the routes guard already checks two of them; the other two drift silently | `every_route_the_pipeline_offers_names_stages_that_exist` reads `SKILL.md` and `README.md` and is extended to the new vocabulary |
| `REVIEW.md` grows and pushes `steering_bytes` further past `propose` | change 030's breach gets worse under a change that was supposed to be neutral | measured before and after in step 4; the acceptance names the number |
| The gate refuses a one-line fix that has no change directory | every small edit needs a paper trail, and the pipeline gets abandoned | the gate stays silent with no in-flight change, the same rule `guard-stage.sh` follows; watched with a change directory absent |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. 436 passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/harness-metrics.sh` — `steering_bytes` ≤ 152 647; `routes` 6,
  `hooks` 6, `gated_commands` 6 all unmoved.
- Five watched refusals from `scripts/check-review.sh`, each naming one reason.
- One watched commit refused by `gate-commit.sh` for a stale verdict, and the same
  commit succeeding after the verdict is refreshed.
- Both new guards watched failing before they are trusted.
- This change's own `review.yaml`, written from the three reviewers' actual
  reports rather than assumed.

## Departures from the plan

**Selecting the change by status does not work in this repository, and finding out
why was the most useful thing in the change.** The first cut copied
`guard-stage.sh`: scan for the first change whose status is in flight. It selected
change **008** and would have made every commit answer for it — because **24 of
the 34 changes sit at `reviewing`**. That is not stale data, it is the intent's
thesis as a measurement: `reviewing` is where a change stops when nobody gets to
it, and nothing anywhere said so. The gate now selects by what the diff touches —
staged change directories first, then the working tree — and refuses to guess when
more than one is present rather than picking the lowest-numbered.

**A seventh refusal, not in the spec: two change directories in one commit.**
It fell out of the selector and is kept, because in a shared tree it is reachable
today: 033, 034, and 035 were all dirty at once while two sessions worked. It
exits 2, which blocks. Guessing would have been worse than refusing.

**Selecting from the staged set was not in the design and had to be.** Judging the
working tree makes two concurrent sessions block each other — each one's dirty
change directory makes the other's commit ambiguous. A commit stages what it is
committing, so staged is the honest signal, with the working tree as the fallback
when nothing is staged.

**The undefined-reviewer check was written to cover only required reviewers, which
is the wrong half.** As first written, an invented name sitting in `review.yaml`
was simply ignored — it would have looked like coverage and been none. It now
validates every recorded line against `.claude/agents/`. Found by watching refusal
F and getting the wrong refusal back.

**A bash bug that only shows in Japanese.** `$digest。` — a bare variable followed
immediately by a full-width character — was parsed as a variable named
`digest。`, so the staleness refusal aborted with "unbound variable" instead of
refusing. Every message in this repository's hooks is Japanese, so this is a class,
not an incident: `grep -P '\$[A-Za-z_][A-Za-z0-9_]*[^\x00-\x7F]'` finds it, and the
fix is to brace the expansion.

**`REVIEW.md` had to shrink to make room for itself.** The mechanism cost +1 998
bytes on `steering_bytes`, which is already at `propose` and owned by change 030.
Three cuts brought the document from 8 797 back to 6 668 — under its 6 799
starting size — by deleting what the suite already enforces: the policy table's
violation prose (each row now names where the policy is written), the durability
and subprocess bullets that the new gate table states exactly, and the
finding-feedback paragraph that restates `CLAUDE.md`. Net across the whole change:
**+247 bytes**, not the zero the spec asked for. Recorded rather than rounded.

**The digest moved from the working tree to the index, and the reason that
settled it was not the one raised.** The other session argued the working-tree
digest answers a different question than the gate asks — true, and a commit
records the index, so a verdict about the working tree is about something that is
not what lands. What made it not a preference was a defect neither of us had
named: **`git diff HEAD` cannot see an untracked file at all**, so a brand-new
file could be added to a reviewed change and move no digest. `scripts/check-review.sh`
itself was outside its own digest until it was staged. The gate now digests
`git diff --cached`, falling back to the working tree when nothing is staged.

That opened a hole in the same edit: **`git commit -a` stages after a PreToolUse
hook runs**, so at gate time the index is not what the commit will record.
`gate-commit.sh` is the only place that sees the command, so it greps for
`-a`/`--all` and passes `--include-unstaged`, which widens the digest back. The
regex was handed to `subprocess-safety-reviewer` to attack rather than trusted.

**Moving the digest to the index made the index a shared resource, and a third
session found that within the hour.** `operon-dc` could not commit change 034
because 035's files were sitting staged: staging theirs on top would have made the
digest cover both diffs, so their reviewer's verdict would have been recorded
against a diff it never read. They asked rather than unstaging, which is the gate
working — but the cost is real and was not in the spec. **An index-scoped digest
serializes commits between sessions sharing a tree.** The working-tree digest did
not have that cost; it had the untracked-file hole instead, which is worse. The
trade is recorded rather than hidden, and the gap it leaves is that the gate does
not *say* a turn is needed — it only refuses. Naming that in `REVIEW.md`, or
detecting more than one session's files staged at once, is the follow-up.

**The digest was irreproducible and is not any more.** Capturing the diff into a
shell variable drops its trailing newline, so the number printed in the refusal
could not be recomputed by the person reading the refusal — most of the value of
printing it. The diff is now piped straight into `shasum`, and the exclusion
pathspec is hoisted into one `EXCLUDE` variable. That hoisting made
`the_review_gate_has_no_bypass` *stronger* rather than weaker — the literal now
exists in exactly one place, which is lesson 004's prescription — and the guard
was re-watched failing against the single assignment to confirm it.

**`scripts/harness-metrics.sh` was edited again**, for the same reason as in change
032: `invariant_tests` counts by name prefix and `the_review_gate_` had to be added
or the three new guards would exist uncounted. It reads 32, up from 26.

**The change was refused by its own gate, and the refusal was right.** The three
reviewers were run on the diff as first staged and all three returned
do-not-approve — twelve Important findings between them, seven distinct once the
overlaps were removed. Recorded in full because the mechanism this change adds is
what caught them, and because the `review.yaml` that had been sitting in this
directory carried placeholder nits rather than any reviewer's report: the gate
could not know that, but its staleness check refused the file anyway.

What they found, and what moved:

- *The guard's `exit 0` ceiling was never reached* — lesson 009, in the guard
  written for this change. The script held one line under a ceiling of two, so a
  bypass line fitted under it unseen. The guard now asserts each of exactly two
  `exit 0` lines by what it says.
- *The exclusion check used `contains`*, so a second pathspec on the same line —
  `src` — would have made every verdict permanently fresh. Exactly one
  `:(exclude)`, on exactly one line, naming the change directories.
- *A comment named a guard that did not exist* and claimed the direction the real
  one does not check. The comment names the real guards, and a new one,
  `the_review_gate_requires_the_reviewers_review_md_names`, round-trips
  `REVIEW.md`'s table against the script's `case`, so the mapping written twice
  cannot drift.
- *`review.yaml` was read from the working tree while the digest came from the
  index.* A verdict written and never staged let the gate say SHIP while HEAD
  received the old record — the verdict outliving the code, one file over.
  Everything the gate reads — the record, the route table, the risk table, the
  state, the reviewer definitions — now comes from what the commit will land:
  the index under `--index`, the tracked working tree under `--include-unstaged`.
- *The review gate sat inside the hook's timeout window, behind a minute of
  cargo.* A cold-cache commit could time the hook out and land unreviewed with no
  trace of the gate not having run. It runs first now, and a guard pins the order.
- *A commit touching no change directory passed silently, whatever it touched.*
  By design for the one-line fix, and a hole on `src/store.rs`. A directory-less
  diff on a `high` surface, on a reviewed surface, or one adding `Command::new`,
  `.spawn(`, `.output(`, or `unsafe` is refused. The low-risk directory-less
  commit still passes, and `REVIEW.md` says so.
- *`git add -A && git commit` staged after the gate had judged the index*, so an
  untracked file added that way was in no digest at all. The hook refuses staging
  in the same command, refuses `-i`, `-o`, `-p`, `--`, and a pathspec after
  `commit`, and reads only the tokens after `commit` with quoted text masked —
  which also stopped it running the whole gate on a grep pattern that merely says
  "git commit", something it did to this session twice.
- *A non-required reviewer's do-not-approve was ignored*, and printed in the SHIP
  line as "3 Important". Every recorded verdict is judged; only "missing" is
  scoped to the required set.
- *A paper-trail-only commit needed a verdict on the empty diff.* The gate's own
  intent commit would have required an approval of nothing. A commit with no code
  diff passes as paper trail only — unless `review.yaml` itself is changing in
  it, which is refused: a verdict may not land apart from its code.
- *Seven `Command::new` sites live outside the `subprocess` surface's paths*, so
  `REVIEW.md` claimed a coverage the paths did not give. The added-line trigger
  closes it, and the policy says so in the same words.
- Smaller: the empty-array expansion bash 3.2 rejects under `set -u` is a scalar;
  counts are validated as digits; a comment after a closing quote is tolerated;
  the finding budget is stated in characters, which is what `cut -c` counts under
  the hook's locale.

Cost: `REVIEW.md` went from 6 683 to 7 486 bytes against a `steering_bytes`
already at `propose` and owned by change 030. Recorded, not rounded: the growth
is the two exposures the round found, written where the gate points.

Nine mutations were run against the four guards. Eight went red as expected. The
ninth — a `case` arm rewritten as `echo "${X}"` — went red in the round-trip
guard rather than passing unchecked as the reviewer had shown for the old `}`
split. That is the safe direction, and it is left that way.

**The second round refused it again, and was right again.** The rust reviewer,
on the repaired diff, found four more: the guard matched the text `exit 0` and
so missed `exit 0;`, `exit  0`, `exit 00`, and a bare `exit` after a true test;
it counted `:(exclude)` and so missed git's `:!` and `:^` spellings and a second
pathspec on the diff line; the hook's option strip ran before the separator cut,
so `git commit -am fix && echo commit` read as no options at all; and the
commit shape every commit in this repository takes — `-m "$(cat <<'EOF' … EOF)"`
— was refused as a pathspec, and refused again for a body that mentioned `git
add`. The guard now reads each line as tokens and treats any `exit` with an
absent or zero status as a success exit, refuses the short pathspec magic, and
requires every digest-feeding `git diff` to carry `. "$EXCLUDE"` and nothing
else after `--`. The hook drops heredoc bodies before it reads anything, cuts
at the separator before it strips, and calls a token a pathspec only when it is
a path that exists. Nine more mutations, all red — including the five `exit`
spellings the reviewer had listed.

**The third round found the gate's own reading of the command was still the weak
half, and gave it a test.** Rust: the guard's comment stripping cut at any `#`,
so `${#X}` hid an `exit` behind it; the allowed success exits were identified by
substring, so `[ -z "$target" ] || exit 0` passed; the digest line itself was not
pinned, so `--relative`, a second `shasum`, or a reassigned `EXCLUDE` narrowed
what was hashed under a green guard; a quoted pathspec became `__quoted__` and
was never a path; and `head -1` dropped whatever followed a heredoc's closing
line, so `)" -a` was not `-a`. Durability: an indented or unterminated heredoc
swallowed the commit and the hook exited 0 in silence, which is the failure the
change exists to remove; `--amend` was judged against HEAD while the amended
commit records against HEAD's parent; `git -C dir add … && git -C dir commit`
walked past a refusal written for `git add`; and a `docs` route touching
`.claude/hooks` or `Cargo.toml` — two `high` surfaces with no reviewer row —
shipped a stale record as `0 Important`.

What moved: a shell comment starts only at a `#` that begins a word; the two
success exits are pinned to their exact text and the paper-trail one to its
enclosing `if`; the digest's every line is pinned and `--relative`, a second
`shasum`, and a second `EXCLUDE=` are refused; `__quoted__` in pathspec position
is refused; lines are joined while a `$(` is open; a reader that fails, or a
heredoc it cannot delimit, blocks instead of exiting 0; `--amend` passes
`--against HEAD^` and `scripts/check-review.sh` digests against that base after
verifying it is a commit; any git command ahead of the commit in the same line
is refused as a class; `gate-configuration` and `dependencies` gained reviewer
rows in `REVIEW.md`, so every `high` surface now has one; a record that is
required or is changing in the commit — including being deleted — must match
the digest, and one that is neither is left alone and not displayed.

And the test the durability reviewer asked for:
`the_commit_gate_reads_the_command_it_will_gate` drives the real hook under
`OPERON_GATE_PARSE_ONLY`, which prints how the command was read and then blocks
— never allows — across twenty-six command shapes, every one of which a
reviewer had found by hand in some round. It failed on its first run: the
`before_commit` cut used `sed` with `\b`, which BSD sed does not have, so every
commit read as "a git command ahead of the commit". Four hook mutations
watched red under it. `REVIEW.md` reads 8 953 bytes now, from 6 683 when this
change began: each round's growth is an exposure named where the gate points.

**The fourth round said the same thing about the other script, and it was right
there too.** The rust reviewer showed three one-line edits that removed the
review gate while every text pin stayed green — `run_diff() { :; }` after the
pinned definition, `trap 'exit 0' EXIT`, and `index) diff_mode=worktree` — and
two pathspec shapes the existence test let through, `src/*.rs` and `tests.rs`
after a `cd`. Text can be pinned but not exhausted. So `scripts/check-review.sh`
has the test the hook got a round earlier: `the_review_gate_refuses_what_it_says_
it_refuses` builds a repository shaped like this one under a temporary
directory and drives every refusal the script claims and both passes it allows,
through `--index` and `--against`. All three one-liners, and the deletion of a
record, watched red under it. The hook now refuses every unconsumed token in
pathspec position except the three pieces a masked heredoc body leaves and a
redirection; refuses a second commit in the same command; reads the
"git ahead of the commit" class on the unmasked text, so `bash -c "git add -A"`
no longer hides in its quotes; and counts an unterminated heredoc as a reason
to block rather than a reason to guess. `scripts/check-review.sh` and
`scripts/check-release-preconditions.sh` joined the `gate-configuration`
surface, and a change to any hook or gate script now runs the suite at the
commit, because the suite is what reads them.

The subprocess reviewer's first completed verdict — its earlier two runs never
finished — added four: a backslash continuation was not joined, so `-a` on the
next line was not read; `-mtest src/store.rs` skipped the pathspec because the
letter loop read `m` and consumed the next token; the change-directory selector
compared the index to HEAD while the digest compared it to `--against`, so an
amend that staged code alone found no change and exited 0; and a second line
for the same reviewer was invisible to `field`, which reads the first. All four
are rows or scenarios in the two behavioural tests now, beside `--trailer`,
`--message`, `-m"attached"`, `:/path`, a brace glob, `--pathspec-from-file`,
non-ASCII paths under `core.quotePath`, and a route that is not a plain word.

**The fifth round turned on the tests themselves.** The rust reviewer mutated
the script outside the repository and found two branches the fixture never
drove: `--include-unstaged`, which the hook passes for every `-a`, and the
`craft` and `none` reviewer sets — so `all) diff_mode=cached` and `craft) ;;`
both kept the suite green while shipping unreviewed code. The durability
reviewer found that git accepts any unique prefix of a long option, so `--ame`
was an amend the hook read as nothing and `--pat` a patch commit it let
through; and that `git -C other commit`, `--git-dir`, and `cd other &&` were
all judged against this session's index while landing elsewhere — the exact
`git -C` shape the global instructions recommend over `cd`. Now the fixture
drives both modes, all three reviewer sets, the malformed-record shapes, an
untracked record, a deleted record, and the amend pass as well as its refusals;
the hook resolves a long option by unique prefix and refuses an ambiguous one;
a commit aimed at any directory but this session's own is refused, so a
worktree is committed from the session whose project directory it is; colour is
off on the digest diffs; and `routes.yaml` and `risk.yaml` joined the
`gate-configuration` surface. Six more mutations watched red, three per script.

**The sixth round ended the contest by moving the gate.** Subprocess found four
more spellings the reader missed — `--"all"`, `--al\l`, `--${x:-all}`,
`(cd /tmp && …)`, `pushd`, `GIT_INDEX_FILE=`, `bash -c "git commit …"`,
`"git" commit` — and durability found that the Bash tool's working directory
persists between calls, so a `cd` in one call and a `git commit` in the next
lands in a worktree the hook never looked at, and that the table test had this
machine's checkout path written into it and would fail in CI. Reading a shell
command to learn what `git commit` will record is a contest nobody wins; git
already knows. The review gate is now a `prepare-commit-msg` hook, run by
git inside the commit: after `-a` has staged, on the temporary index of a
pathspec commit, against HEAD's parent when git says the commit is an amend,
in whichever repository the command reached, and not skipped by `--no-verify`.
`.claude/hooks/gate-commit.sh` keeps the cargo gates and one new line — it
points `core.hooksPath` at `.githooks` before every commit and refuses a clone
that points it elsewhere — and lost its reader, its parse-only mode, and the
fifty-four-row table that tested them. The fixture now sets `core.hooksPath`
and drives real commits: `-a` with an unstaged edit, `--no-verify`, a pathspec,
`git -C <repo> commit` from another directory, and an amend refused and then
landed. `--include-unstaged` is gone from `scripts/check-review.sh` with the
reader that needed it.

The rust reviewer's sixth-round findings that outlived the reader: the fixture
could not tell the `full` set from the surfaces widening it, so `full) ;;`
stayed green — the no-record refusal now asserts the whole required line, and a
`feature` route on a low surface asks for all three; and the guard's narrowing
filter looked for the literal `git diff` after the digest lines had become
`git -c … diff`, so a second arm narrowed to `src/` passed both tests — the
filter reads either spelling, and the fixture carries a `Cargo.toml` outside
`src/` so a digest that stops at one directory has something to miss. A risk
table missing from the index is exit 2, not zero surfaces. Ten more mutations
watched red across the two scripts and the git hook.

**The seventh round moved the gate one hook further.** `prepare-commit-msg` is
told a commit is an amend only for some spellings: `--amend --no-edit` arrives
as `commit HEAD`, `--amend -m` — the shape every commit in this repository
takes — arrives as `message`, and the rust reviewer landed an amend under the
wrong base to prove it. The gate is now `.githooks/reference-transaction`,
handed the commit object git has already built, in the `prepared` state of the
ref update: it judges the commit's own diff against its first parent, which is
the right diff for a plain commit and for an amend alike, and needs no telling.
Only a commit built on the current tip is judged — `new^ == old` or
`new^ == old^` — so a reset, a rebase, or a checkout is not refused for carrying
a record about a different diff. `scripts/check-review.sh` gained `--commit`,
which reads every input out of the commit's own tree. The fixture drives
`--amend -m` refused and then landed, and a `reset --hard` that must pass. The
same reviewer's second finding — the git hook's success exits counted as text —
is answered by reading them as tokens, the way the script's are.

The durability reviewer, on the same round, found the hole a relative
`core.hooksPath` leaves: the setting is shared by every linked worktree and
resolves inside whichever worktree the commit lands in, so a worktree checked
out at a branch from before this gate has no `.githooks` and git skips it in
silence. `.claude/hooks/gate-commit.sh` now sets the absolute path of the main
checkout's `.githooks`, resolved through the common git directory, so the same
hook runs for all of them and refuses a worktree that has no script to run. The
fixture carries its own copy of the hook and points at it relatively, as a
checkout does. `REVIEW.md` names what is left: a clone before its first commit
through Claude Code, and a hooks path a person moved by hand.

The subprocess reviewer's seventh round, on the same old design, added the
one-command override: `git -c core.hooksPath=/tmp/none commit` and a
`GIT_CONFIG_*` variable point git at no hook for that commit and git says
nothing. No commit here needs either, so `.claude/hooks/gate-commit.sh`
refuses a commit command that mentions them, refuses `--git-dir` and
`--work-tree`, and allows `-C` only when it resolves to this session's project
directory — because this directory's cargo gates must not vouch for another
checkout's code, whatever git's own hook then does there.

**The eighth round found the two things a commit-object gate can still get
wrong.** Rust: `new^ == old` is also true of moving the branch to an existing
child — `reset --hard ORIG_HEAD` after stepping back, a one-commit fast-forward,
a detached checkout — so a commit that could not pass the gate could never be
returned to, and the fixture's reset pair was green only because both commits
passed. The hook now judges only a commit that exists nowhere else yet: one
reachable from another ref or present in any reflog is being moved to, not
made. The fixture makes a commit with the hook off, proves it cannot pass on
its own, steps back, and moves forward to it through `ORIG_HEAD`. Durability:
the hook is a tracked file, so checking the main checkout out at a branch from
before it removed it and every worktree with it — git skips a missing hook in
silence; and the tables are read from the commit's own tree, so a commit that
lowered `risk.yaml` was judged by the lowered table. The Claude Code hook now
copies the tracked hook into the repository's own hooks directory, which no
checkout removes and every worktree shares, and refuses a `core.hooksPath`
rather than setting one; and `scripts/check-review.sh` classifies by the union
of both sides' risk tables and the stricter of both sides' route sets, with a
fixture scenario that lowers the table and is still refused.

**The ninth round found the same mistake in three places, and named it.** The
eighth round's reflog filter was written `git reflog show … | grep -q`, in a
hook that runs under `pipefail`: `grep -q` stops reading at its first match,
git gets SIGPIPE while still writing, and the pipeline's status is git's — so a
commit being returned to read as one being made, and the fixture's forward
move was refused. The same shape had been fixed once already in the review
script's table union that round, and was still in its content trigger, where a
large diff adding `Command::new` early could read as one that did not. Every
shell gate now spells the predicate `grep … >/dev/null`, which reads to the end,
and `every_gate_script_reads_its_pipes_to_the_end` refuses the other spelling in
any script that sets `pipefail`; entry 018 in `docs/sdlc/lessons.md` records
it. The subprocess reviewer's findings on the same round: a null `old` is
`pack-refs` rewriting every ref as well as a root commit, so a root is a null
`old` *and* no parent and any other null `old` is skipped; the one-command
override refusal in `.claude/hooks/gate-commit.sh` now comes before the line
that decides whether the command is a commit, so a merge, a cherry-pick, a
rebase, or a pull carrying `-c core.hooksPath=` or a `GIT_CONFIG_*` variable is
refused too; a second `-C`, which git resolves relative to the first, is refused
outright, as are `GIT_DIR` and `GIT_WORK_TREE`. The fixture gained the
scenarios the eighth round had only argued: `pack-refs` with the reflogs
expired, a root commit on an unborn branch, a merge commit, and a commit from a
checkout at a branch before the gate existed, refused by the copy in the
repository's hooks directory. `REVIEW.md` now says a rebase replays each
commit through the gate, and reads 9 597 bytes; `steering_bytes` reads 160 671,
change 030's breach still, moved here by that and by the lessons entry.
`invariant_tests` reads 32 with the new guard counted. Six more mutations
watched red: the hook's reflog line and the script's trigger put back to
`grep -q`, the override refusal moved below the commit filter, the chained
`-C` refusal removed, and the root condition without its parent check — under
the text pin and, at four minutes on a loaded machine, under the fixture's
`pack-refs`.

**The tenth round replaced the rule for what is judged, and added a second
reader.** Round 8's reachability exemption — a commit under any other ref or in
any reflog is being returned to — was an entrance: the subprocess reviewer
laundered a refused commit through `git stash` and a fast-forward, through a
branch created at a dangling sha, and through a detached checkout that wrote
HEAD's reflog; the durability reviewer showed the reflog half expiring under
`git gc`. The hook now judges every update to a branch or a detached HEAD,
whichever is checked out, and within it every commit the update makes
reachable that no branch reaches yet — `git rev-list new ^old --not
--branches` — exempting only what the branch's own reflog remembers. A ref
that is not a branch vouches for nothing, and a root commit's amend, which the
old shape rule skipped, falls out of the same line. An exemption for history
from before the gate — ancestors of the commit that first added the hook — was
written and found vacuous by its own fixture scenario: such a commit is fresh
only when no branch reaches it, and then `git log --all` cannot find the
anchoring commit either; the durability reviewer's reflog-expiry finding is
answered in `REVIEW.md` as the narrow ratchet it is, a month-old rewind across
pre-gate history made by hand, and the fixture's last scenario pins that
refusal rather than a promise the hook cannot keep. The same reviewer showed the one-command
override refusal is a text filter and can be spelled around — `HOME=`, an
`include.path`, `core.hooks""Path` — so `REVIEW.md` says it catches the plain
spellings, and `.claude/hooks/gate-stop.sh` now reads the branch itself before
a session ends: every commit since the last it saw pass goes through
`scripts/check-review.sh` with `--commit`, and an unjudged one is refused at every stop
until the branch is clean, with the mark in `refs/operon/review-audited`. The
durability reviewer's second finding — the installed copy of the hook is
last-writer-wins across worktrees, so a branch carrying a different hook
downgrades the gate for every other worktree — is answered by installing a
trampoline, `.githooks/installed/reference-transaction`, that runs the tracked
hook from the main checkout and decides nothing itself; and the third — the
fixture resolved the hooks directory through a developer's global
`core.hooksPath` and wrote outside its temp tree — by running every fixture
process with no global or system git config. The rust reviewer found the
lowered-table scenario green under a `full` route with the union removed; it
runs under `docs` now, and a sibling lowers `routes.yaml` itself. The fixture
gained the laundering routes, a root commit that lands and whose amend is
judged, a linked worktree at a branch from before the gate judged by main's
hook and script, and a checkout with no hook anywhere refused. Two guards were
text pins only and are driven now: `the_commit_gate_refuses_the_overrides_it_names`
feeds gate-commit.sh the commands it names, and
`the_stop_gate_refuses_a_branch_carrying_an_unjudged_commit` drives the audit.
The route's attempt ceiling, two, was passed at round two and `handoff.md`
records the search; the repository's owner asked for the work to continue
without a person at the gate, and it did.
Numbers after the tenth round: `REVIEW.md` 11 208 bytes; `steering_bytes`
162 519, change 030's still; `invariant_tests` 33; the suite in what lands
460 passed, 0 failed, 6 ignored (the working tree ran 461, carrying another
session's test for `src/tmux/hooks.rs`, left out of this commit with its
code). Six more mutations watched red: the hook narrowed to the checked-out
ref, under the text pin and under the fixture's laundering scenarios; the
hook itself copied in place of the trampoline; the stop gate's audit loop
emptied; the read-only exemption removed, refusing `git log -S hooksPath`;
the reflog exemption removed, refusing the return through `ORIG_HEAD`; and the
trampoline's last resort turned into `exit 0`. The seventh, the pre-gate
exemption removed, stayed green, which is how it was found vacuous.

**The eleventh round hardened what the tenth added.** Rust: the override and
`-C` reads took heredoc bodies as command text, so the change's own commit
message — which names `core.hooksPath` because that is what it does — was
refused; bodies are stripped before either read, and the commit filter reads
the same text, so a file written through `cat <<'EOF'` that spells a commit no
longer runs the cargo gates. The stop gate's mark moved backward after a
detached checkout of an old commit and then refused the gate's own commit at
every stop; it only advances now. The pipe guard did not read a `grep -q` on
the line after a trailing `|`; it joins continuations first, and reads
`.githooks/installed/`. Durability: `cp` onto the shared installed hook
truncates it in place, and a commit from another worktree inside that window
runs an empty file that bash exits 0 on — the trampoline is written beside and
renamed over; the hook's copy of `main`'s script into a temp file was
unchecked, so a full `TMPDIR` produced a truncated script that ran — the copy
is compared byte for byte with `git cat-file -s` and refused otherwise, and an
empty hook blob is not exec'd; `stop_hook_active` exits before the audit, so
"refused at every stop" was one stop short — the header, `REVIEW.md`, and the
test now say that the in-turn retry is the one stop let through, since a hook
that refuses it is a loop; and the audit was skipped in silence in a checkout
without `scripts/check-review.sh` — it resolves the script from `main`'s tree
the way the hook does, and `REVIEW.md` says a branch with no gate commit in its
history is guarded by the hook, not the stop. The read-only exemption in
`gate-commit.sh` reads past git's leading options, so `git -C dir log` and
`git config --get` are not refused. Three more mutations watched red: heredoc
bodies read again, the mark moving backward, and a `grep -q` after a trailing
pipe.
Numbers after the eleventh round: `REVIEW.md` 11 966 bytes; `steering_bytes`
163 277; `invariant_tests` 34 with `the_commit_gate_` counted; 460 tests in
what lands, 461 in the working tree. Six mutations watched red this round,
eighty-one across eleven.

**The twelfth round closed the fail-open paths the eleventh had left.** The
rust reviewer approved; the other two did not. Durability: a zero-byte script
or hook exits 0 and every `-f` on the way to it read as a gate — the judge, the
trampoline, and gate-commit.sh read `-s` now, and the fixture commits past an
empty script and past an empty tracked hook and is refused by `main`'s copy
both times; a failed `mktemp` in the stop gate exited 0 and switched the whole
Stop hook off — the audit's failures are their own refusal now, printed, never
a silent skip; and the audit was all-or-nothing under the Stop budget, so a
clone's first stop could never finish and never mark — the mark advances as
each commit passes. Subprocess: `strip_heredocs` took a `<<EOF` inside quotes
for an operator and, finding no terminator, swallowed every later line — a
commit two lines down was never read, in silence; a `<<` after a quote is not
an operator now, and an unterminated body means the raw text is read. The two
readers of what a git invocation is disagreed — `/usr/bin/git` was a commit to
one and not a git command to the other, so its override went unrefused — one
spelling, `git_word`, feeds every read. A stop on a side branch pinned the mark
at the fork and every later stop on `main` re-read everything since — the mark
is one ref per branch, `refs/operon/audited/<branch>`. And the stop gate's copy
of `main`'s script had no whole-or-nothing check — it has the hook's. Nits
taken with them: `grep "-q"` and `-m1` read by the pipe guard, exit codes
asserted in the worktree scenario, a quoted `-C` path read whole, `set -f`
around the tuple expansion, `update-ref -m`, the stale "267" in the erosion
message, and `OPERON_SKIP_STOP_GATE` named beside the in-turn retry.
Numbers after the twelfth round: `REVIEW.md` 12 200 bytes; `steering_bytes`
163 511; `invariant_tests` 34; 460 tests in what lands, 461 in the working
tree. Four mutations watched red this round — the judge reading `-f`, under the
fixture's empty-script scenario; a quoted `<<EOF` taken for an operator, under
a row whose bare `EOF` line comes later; `/usr/bin/git` not a git command; an
unterminated body still dropped — eighty-five across twelve.

**The thirteenth round answered what the twelfth's reviewers found before
their budget ran out.** Two of the three stopped mid-review on the model's
spend limit; their transcripts carried two findings each had reproduced, and
both are answered. Durability: a zero-byte `.githooks/installed/reference-transaction`
passed gate-commit.sh's `-f` and was installed as git's hook, which bash exits
0 on — the read is `-s`, and a behavioural guard,
`the_commit_gate_installs_the_trampoline_and_refuses_an_empty_one`, drives the
install itself: a whole trampoline lands in `.git/hooks` byte for byte and
executable, an empty one is refused before it can. Subprocess: a tag is not
judged, and a branch made symbolic to the tag resolved to the tag's commit
under `--not --branches`, so `git update-ref refs/heads/main <sha>` found
nothing fresh — a refused commit laundered through `git tag` and
`git symbolic-ref`. What vouches is now the set of direct branches, read from
`for-each-ref` with symbolic ones left out, and a symbolic write under
`refs/heads/` is judged at what it resolves to; the fixture drives both halves:
the symbolic write refused, and, made with the hook off, not vouching. The
same reviewer confirmed under `/bin/bash` 3.2 that a plain unreviewed commit is
refused and a reviewed one lands, and that `git update-ref`, `git branch`, and
a self-`fetch` into `refs/heads/` are judged.
Numbers after the thirteenth round: `REVIEW.md` 12 309 bytes; `steering_bytes`
163 620; `invariant_tests` 35 with the install guard counted; the working tree
ran 470 (it now also carries another session's change 037 tests); three
mutations watched red this round — the installed trampoline read with `-f`,
symbolic branches vouching and symbolic writes skipped — eighty-eight across
thirteen. Round twelve's reviewers stopped on the model's spend limit before a
verdict, and the same limit holds round thirteen's until it resets; the tree
is staged and green, waiting on that.

**The fourteenth round came from the first commit the gate judged in this
repository.** Change 037, from another session in the same checkout, was
refused before 035 had landed: its diff touched neither side's `routes.yaml`,
both sides still spelled the column `yes`/`no`, and the reader passed both
over as vocabulary it could not read and said the route was absent. The old
words are read now as the sets they meant — `yes`, a person must look, is
`full`; `no` is `none`, widened by the surfaces as ever — with three fixture
scenarios under a table committed in the old form, and `REVIEW.md` says which
tree each mode reads and that the hook's is the answer that counts. The
refused commit itself, judged again by hand, passes on its own record.
Numbers after the fourteenth round, on top of change 037: `REVIEW.md` 12 833
bytes; `steering_bytes` 164 144; `invariant_tests` 35; 469 tests in what
lands, 470 in the working tree.

**The fifteenth round closed what the fourteenth's reviewers found.** Rust:
`REVIEW.md` said the bare mode reads the working tree, and the script reads the
index whenever anything is staged — the sentence says so now; the tokenizer
behind both success-exit guards split on whitespace and `;` only, so
`]&&exit 0` was no exit to it — `&`, `|`, and braces split too; and the
numbers in `state.yaml` and `handoff.md` were a round behind. Durability: the
stop gate advanced its mark to a re-read `HEAD` after the walk, so a commit
that reached the branch during the audit was marked audited unread — the tip
is fixed before the walk and the mark advances to it; a checkout with no
script and a `main` without one skipped the audit in silence where the header
promised a refusal — it refuses; and `-s` is one byte deep, so a script cut
short at a statement boundary parsed, exited 0, and passed everything — every
reader of the script, the hook, and the trampoline now asks for the file's
last line, a marker a prefix cannot have, and the fixture commits past a
sixty-line prefix of each and is refused by `main`'s whole copy. Nits taken:
the pipe guard reads `LC_ALL=C grep` and `/usr/bin/grep` as grep; the mark's
ref name flattens the branch's slashes; a refused commit's output in the stop
message is capped at eight lines.
Numbers after the fifteenth round: `REVIEW.md` 13 032 bytes; `steering_bytes`
164 343; `invariant_tests` 35; 469 tests in what lands, 470 in the working
tree. Four mutations watched red this round — an `exit` glued to `&&`, the
stop gate's tip re-read after the walk, a script nowhere skipped again, and
`-s` in place of the last-line read under the fixture's cut-short scenarios —
ninety-two across fifteen.

**The sixteenth round closed the fifteenth's five.** Subprocess: the hook
resolved `scripts/check-review.sh` from the tree it was judging before the main
checkout's, so a side branch stubbed the script — marker line and all — and
judged itself; durability: the same read took the main checkout's working copy
over the committed one, so an edit interrupted mid-way, exiting 0 early with
its last line untouched, judged and marked what it never read. The script is
`main`'s committed copy now, in the hook and in the stop gate, the main
checkout's working copy only before a committed one exists, never the judged
branch's own — the committed copy judges the change that lands its successor
— with a fixture scenario that stubs it from a linked worktree and is refused.
A per-branch mark that could only move forward was stranded by an amend, the
walk growing at every stop; it steps back to where the branch and the mark
part, no earlier than the commit that added the hook, and resumes — the
stop-gate test amends a marked tip and sees the mark follow. Rust and
durability both: the stop gate's two round-15 mechanisms were pinned by the
text they print, not driven; the test now commits a cut-short script to
`main` and sees the stop refused with its reason, and runs a stop under a
TMPDIR nothing can write to and sees that refusal too. Nits taken: `whole()`
reads the last line with `tail`, not the marker's presence anywhere; `set -f`
around every tuple expansion in the script; `.claude/agents` joins the
gate-configuration surface, since a reviewer definition rewritten to approve
everything needed no reviewer; the stop message names the mark's ref and the
way back; the bare mode's sentence in `REVIEW.md` names the exclusion; the
tokenizer's unreachable paren strip and the doc comments straightened.

**And the sixteenth round's own edits broke two of its scripts.** Rewriting
three `whole()` helpers at once left `.claude/hooks/gate-stop.sh` with a
one-line `case … ;; esac` inside a command substitution, which macOS's stock
`/bin/bash` 3.2 refuses to parse — the entire branch audit under it was
unreachable on the machine this ships to, and a stop gate that cannot parse
never refuses — and left `.githooks/reference-transaction` with an
unterminated `"`, which no bash parses. Neither was caught by the suite,
because nothing in it had ever run a gate script through a parser. Now
`every_gate_script_parses_under_the_stock_shell` runs `/bin/bash -n` over all
seventeen, because 3.2 is the oldest shell a `#!/usr/bin/env bash` here can find.
The same edits also tripled `set -f` at four sites — an additive patch applied
twice — and the collapse back to one is what surfaced the last gap: dropping a
`set -f` was caught by nothing. A fixture scenario for it is impossible, since
writing `docs/sdlc/routes.yaml` puts the diff on the `gate-configuration`
surface, which asks for all three reviewers whatever the route says; the
scenario passes with the guard removed, so it was withdrawn rather than kept
green. `every_table_split_in_the_review_gate_runs_with_globbing_off` reads the
script's shape instead, and the mechanism was probed separately: `$#` goes
5 → 8 with globbing left on.

Round 14's two late Important findings are closed with it. `git replace`
puts a ref in `refs/replace/*` that hands a commit a different tree to every
later `git` call, and the refs are fetchable rather than local-only — so the
four scripts that judge a commit now `export GIT_NO_REPLACE_OBJECTS=1`, with
`every_judging_script_reads_the_real_objects` holding it. The trampoline's own
`whole()` had no pin and an `-s` left in it survived; it is pinned now.

Six mutations watched red this round — the branch's own script preferred over
`main`'s committed one, a mark that cannot step back to the fork, an
unwritable TMPDIR skipping the audit, `-s` in place of the last-line read in
the stop gate and again in the trampoline, and a dropped `set -f` — plus the
two new guards watched failing under a restored `case … esac` and a removed
`GIT_NO_REPLACE_OBJECTS`. A hundred across sixteen rounds.

The three gates on the whole tree: `cargo fmt --check` silent, `cargo test
--locked` 473 passed 0 failed 6 ignored, `cargo clippy --locked -- -D
warnings` clean past the compile lines. `bash scripts/check-bands.sh` reports
`steering_bytes` at 164948 against a 150000 propose tier — the tier was
already crossed at `HEAD` (155025) before any of this change's prose, which
adds 9923 of it. It is change 030's breach to settle and this change widens
it; nothing in the gates blocks on it.

**The seventeenth round's rust reviewer refused it over the guard round 16 had
just written.** `every_judging_script_reads_the_real_objects` searched the raw
file for `export GIT_NO_REPLACE_OBJECTS=1`, so a `#` in front of that line — or
the sentence above it that explains why it is there — kept it green. That is
entry 011 exactly, in the round that added the guard, and `plan.md` above claims
the guard holds the mechanism when it did not. It reads `shell_code` now and
matches the whole line, and pins the position: before the first `git` call, or
it is an export that arrives too late to change what any of them read. Both
mutations watched red — the export commented out, and the export moved below the
first `git`.

Its nits are taken. The parse guard's floor was `>= 8` against seventeen actual
scripts, and `REVIEW.md`, `lessons.md` and this file all said "nine" — the floor
is `>= 16` and the three documents say seventeen. `stop_env` was twenty
duplicated lines of `stop_with`; one runner takes the payload and an optional
`TMPDIR`. `shell_code_owned` had `shell_code`'s doc comment stuck to the front
of its own and one caller that only needed a `let` binding; the helper is gone.
`every_judging_script_` and `every_table_split_` join the `invariant_tests`
prefixes in `scripts/harness-metrics.sh`, which now reads 38 — the same
omission this change recorded once already in round 1. And two assertion
messages had their source indentation baked into what a person reads, from a
missing line-continuation `\`.

**The seventeenth round's durability reviewer refused it over three, all in the
install path.** The trampoline was installed from the *committing checkout's
working tree*, and the install runs before the cargo gates — so a trampoline
edited to decide nothing, marker line intact, was written into
`$(git rev-parse --git-path hooks)`, went red on its own text pins, had its
commit refused, and stayed. That directory is shared by every linked worktree
and nothing removes it, so every later ref update in the repository passed in
silence until the next commit from a good tree; the same state arrived from a
timeout during `cargo test`, or from no edit at all if the checked-out branch
carried a different trampoline. The existing scenario asserted the installed
copy matched the working tree byte for byte, which pinned the behaviour rather
than catching it. `main`'s committed trampoline is what is installed now, the
working copy only when no commit carries one, size-checked and last-line
checked — round 16's order for `scripts/check-review.sh`, and a trampoline is
not more trustworthy for judging nothing itself. The scenario now writes a
whole-but-neutered trampoline and watches it never reach the shared directory.

Second: the install had no `fsync` anywhere, and `REVIEW.md` called it a
guarantee. `cp` → `chmod` → `mv -f` lets APFS journal the directory entry with
the extents unwritten, so a power loss leaves a zero-byte hook — which bash
exits 0 on and git reads as approval, for every worktree, until the next commit
through Claude Code. The bytes are `fsync`ed before the rename and the
directory entry after it, and a `trap` removes the temporary file on the paths
where the hook is killed. `REVIEW.md`'s "no other worktree ever runs a
half-written one" held for concurrency and not for crash recovery; it says both
now.

Third: `REVIEW.md` claimed a branch carrying an older hook could not downgrade
the gate, and the trampoline `exec`d the main checkout's *working tree* first.
The main checkout is whatever branch it sits at, so `git checkout old-branch`
there put that branch's hook in charge of every worktree — the exact shape
rounds 15 and 16 closed for `scripts/check-review.sh`, still open on
`.githooks/reference-transaction`. `main`'s committed hook is first now, and
the order is pinned, not just its parts: swapping the two blocks was green
until this round.

Four mutations watched red — the working copy installed again, the `fsync`
dropped, the `trap` dropped, and the trampoline's two resolution blocks
swapped. A hundred and four across seventeen rounds.

**The seventeenth round's subprocess reviewer found the freshness rule turned
off by an environment variable, and the round-16 answer only half-closed.**
`GIT_EXTERNAL_DIFF=/usr/bin/true`, or `diff.external` in a config every linked
worktree shares, makes git print the program's output instead of its own and
exit 0 either way. Reproduced here before fixing: the digest became
`e3b0c44298fc1c14`, the sha256 of empty input, **for every diff there has ever
been**. One verdict recorded at that digest would be fresh forever — and
freshness is the one mechanism `REVIEW.md` says does the work. The same empty
input also silenced the trigger that widens the reviewer set on an added
`Command::new` or `unsafe`, while `--name-only` kept `changed` correct, so it
did not even fall through to the paper-trail path. Every `git diff` in the
script — the three `run_diff` arms and the three `review.yaml` freshness
checks — carries `--no-ext-diff --no-textconv` now, and the scenario sets the
variable and watches the digest not move. A pin alone would not have caught
this: the pinned line was correct, and wrong.

Round 16's `GIT_NO_REPLACE_OBJECTS=1` stops `refs/replace/*` and nothing else.
`$GIT_DIR/info/grafts` rewrites a commit's parents by a different mechanism and
that variable does not touch it. Verified in a throwaway repository: a twin
commit carrying the same tree, named as the parent, makes `sha^..sha` zero
bytes, so the gate finds nothing to judge and exits 0; `GIT_GRAFT_FILE=/dev/null`
restores the real parent and a 103-byte diff, where `-c core.graftFile=/dev/null`
does not. All four judging scripts export it, and the guard that holds the
replace export holds this one. **Its position check does not hold** — see
round 18 below; the sentence as first written here was false for one of the
four scripts and weak for two more.
`REVIEW.md` said this class was closed when half of it was open.

Its third finding is the durability reviewer's third, reached independently
from the other side: the trampoline resolved the tracked hook from a working
tree before `main`'s committed copy, so an `exit 0` inserted after
`set -uo pipefail` — marker line intact, so `gate-commit.sh`'s `-x` and
last-line check both pass — made the hook judge nothing. Closed by the same
reordering.

Two more mutations watched red: `--no-ext-diff` dropped from the digest path,
and the graft export dropped. A hundred and six across seventeen rounds.

**One thing to record about the round itself.** The subprocess reviewer wrote
`.git/info/grafts` and `advice.graftFileDeprecated` into this repository while
proving the graft finding, reported it unprompted, and cleaned both up. Checked
here rather than taken on trust: the grafts file is absent, `advice.graftFileDeprecated`
and `core.graftFile` are unset, `refs/replace/*` is empty, `git rev-parse HEAD^`
is `99bd0279`, and the working tree and index are untouched. A reviewer that
proves a finding on the live repository is worth more than one that reasons
about it, and is worth the check afterwards.

**Round 17's three verdicts, in full: `do-not-approve 1 5`, `do-not-approve 3 5`,
`do-not-approve 3 5`.** The subprocess reviewer re-ran both of its Importants
against the fixes and reports them closed — the digest holds under
`GIT_EXTERNAL_DIFF` in both `--index` and bare modes, and `GIT_GRAFT_FILE=/dev/null`
is in all four scripts ahead of the first `git` call. It also checked the three
`git diff` calls that still lack `--no-ext-diff` and confirmed the omission is
correct: `--quiet` at the mode selection and the two `--name-only` calls invoke
neither an external driver nor textconv.

Two of its answers are worth keeping as findings that were *not* findings. It
could not build a sequence where the mark ends up ahead of an unjudged commit:
the reverse topo walk plus the `[ -z "$unjudged" ]` guard means the mark only
advances past commits whose in-range ancestors have all passed. And it found no
construct that parses under `/bin/bash` 3.2 but behaves differently there —
`${var^^}`, `printf -v`, `mapfile`, `declare -A` and `[[ =~ ]]` appear in none
of the four, and it ran the script under 3.2.57 and bash 5 and got identical
output and exit codes. The one real 3.2 divergence of that kind that is present
— `"${arr[@]}"` aborting on an empty array under `set -u` — is guarded at both
sites. That is the lesson-019 class checked one level deeper than the parse
guard reaches, and it came back clean.

**Two open edges in the mark step-back, carried into round 18 rather than fixed
under the reviewers reading it.** When `git merge-base "$since" "$head"` finds
no common ancestor — an orphan branch reusing a name that still carries a mark
— `fork` is empty, `since` is emptied, and the walk block is skipped without
`audit_failure` being set: the stop passes with no audit and no notice, which is
what the file's header promises never happens. That is fail-open, and the same
class as two mutations already in the ledger. The second: when `fork` is an
ancestor of `gate_commit`, `since` becomes `$gate_commit`, which need not be an
ancestor of `head` — so both `is-ancestor` guards stay false forever, the mark
never advances again, and every stop re-walks pre-gate history until the budget
cuts it. That one fails closed. Both were sent to the round-18 durability
reviewer to judge in place, deliberately unfixed, so the digest it is reading
stays valid — which is the freshness rule applied to the reviewers rather than
to the commit.

**Round 18's rust reviewer found the position half of that guard vacuous.**
`every_judging_script_reads_the_real_objects` locates the first `git` call with
`line.split_whitespace().any(|word| word == "git")`, which sees a bare `git`
token and not `$(git …)`. Every git call in
`.githooks/installed/reference-transaction` is a command substitution, so
`first_git` is `None`, `is_none_or` returns true without evaluating anything,
and both exports could be moved to the last line of the file with the guard
still green. Measured with the same predicate over all four:

| script | first_git as the guard sees it | where git first runs |
|---|---|---|
| `.githooks/reference-transaction` | 16 | 7 |
| `.githooks/installed/reference-transaction` | none | 3 |
| `scripts/check-review.sh` | 32 | 4 |
| `.claude/hooks/gate-stop.sh` | 10 | 10 |

Vacuous on one, and on two more the guard permits the exports to sit after
calls that have already run — including the trampoline's
`git show refs/heads/main:.githooks/reference-transaction`, which is the call
that decides which hook judges everything. A `refs/replace/*` entry giving
`main` a different tree with a hook that judges nothing would be read by that
call with replacement still on. This is the third time in this change a guard
has passed by reading text that describes a mechanism rather than the mechanism
— entry 011's shape, now in its position check rather than its presence check.

Its nits are the same kind of thing one level out. The resolution-order pin
compares two positions, so inserting a `whole "$here/…" && exec bash "$here/…"`
line *above* the committed block shifts both by one and keeps `committed <
working` true — the judged branch's own working tree back in front, with every
pin green; there are exactly two `exec` lines and asserting that, in order, is
what closes it. `gate_with_env` duplicates `gate_in` for one `.env` and pins
`--index`, so the `--commit` arm the git hook actually runs cannot be driven
under an environment variable — the same fold `stop_env` got last round. Two
closures driving `gate-commit.sh` are near-identical. `lines[index - 1]` and
`lines[index + 1]` are unguarded at a file's first and last line. And the
neutered-trampoline scenario discards its exit code and prints a message that
would also fire when nothing was installed at all, pointing a reader at the
wrong failure.

None of it is fixed yet, on purpose: two reviewers are still reading this
digest, and moving it under them would cost their round to save mine.

**Round 18's durability reviewer confirmed all three of its previous Importants
closed — by running them, not by reading them — and re-judged its five nits as
still nits.** It recomputed the digest independently, checked that the empty
blob is caught (size 0 passes the `-s` comparison but the last-line check
blocks), confirmed `dd conv=fsync` is real on this machine's `dd`, and verified
the resolution-order pin compares positions rather than mere presence. Its
re-judgement of the five is the useful part: the silent `mktemp` failure is
diagnostics only, because git names the hook in its own `fatal:` and `git stash`
aborts before touching the working tree; the `tr '/' '-'` mark collision costs
re-auditing and never skips a commit, because the audit is per-sha and a shared
prefix is already audited; the `[ "" -ne N ]` reading as a size match is backed
by `! whole "$gate"` on the next arm of the same `||` chain, and the two places
with the same shape use `&&` and fail closed.

Two new nits, both worth taking. `chmod +x` sits *after* the `dd conv=fsync`
and the rename, so the mode bit is inode metadata covered by neither sync,
while the directory `fsync` covers only the entry — a replay that lands the
rename without the mode leaves the hook `0644`, and git skips a non-executable
hook with a `hint:` nobody in this repository reads. Reproduced in a throwaway
repository: the commit landed without passing the gate. Ordering the `chmod`
before the sync is one line, and puts the mode inside the write → sync →
rename → parent-sync order this repository already keeps elsewhere.

The second is a claim that outruns its primitive. `REVIEW.md` says the bytes
are `fsync`ed "so a power loss does not leave a zero-byte hook", but Apple
documents `fsync(2)` as not flushing the drive cache and directs power-loss
durability to `F_FULLFSYNC` — which is what `dd conv=fsync` calls and what
`src/store.rs`'s `sync_all()` gets for free on Apple targets. The shell path is
one primitive weaker than the Rust path while naming the same guarantee. The
fix is either to reach `F_FULLFSYNC` through the `python3` this hook already
shells out to, or to narrow the sentence; the reviewer has been asked which,
since a `python3` that is absent currently makes the directory `fsync` a silent
no-op through `|| true`.

**Round 18's subprocess reviewer confirmed the three previous Importants closed
and found two more, both reproduced end to end and both verified again here.**

`--no-ext-diff --no-textconv` stops an external driver and a textconv filter. It
does not stop the **`-diff` attribute**, which makes git print
`Binary files a/X and b/X differ` and no content at all — so the trigger in
`scripts/check-review.sh` that pulls in `subprocess-safety-reviewer` on an
added `Command::new`, `.spawn(`, `.output(` or `unsafe` sees no `+` lines and
never fires. Measured here: one matching added line without the attribute, zero
with `*.rs -diff` in `.git/info/attributes`, one again with `--text`. The three
delivery paths all leave nothing in any commit — `.git/info/attributes` is
untracked and shared by every linked worktree, a working-tree `.gitattributes`
need never be added, and `core.attributesFile` can be set at any scope. Because
`--name-only` is unaffected, `changed` stays correct and the `index <old>..<new>`
line keeps the digest moving, so neither the freshness check nor the surface
classification notices. `REVIEW.md`'s promise that an added `Command::new`
adds the subprocess reviewer "whatever file it is in" is void: on the
`foreign-store`, `drawing` and `text` surfaces, which carry no reviewer of their
own, a `craft` route would land a child process on the rust reviewer's approval
alone, and with no change directory it lands with no record at all. This is the
same "the pin is right and still wrong" shape as `GIT_EXTERNAL_DIFF`, and it
needs a scenario that watches the required reviewer *set* rather than the digest.

Second, and fail-open: `.githooks/reference-transaction:137-142` builds
`$vouched` from every ref under `refs/heads`, and **one ref whose object does
not resolve makes `git rev-list` exit 128 with empty output**. The status is
never read, so `[ -n "$fresh" ] || continue` reads the emptiness as "no commits
to judge" and allows the ref update. Verified in a throwaway repository:
`rc=128`, empty output, `continue`. A single dangling branch ref anywhere in the
repository turns the whole gate off, silently, for every commit — and the hook
was written on the premise that a gate which cannot run must say so.

Its other measurements are worth keeping: `git show <rev>:<path>` does not apply
textconv on git 2.55, so `landing()` and `landed_before()` are safe;
`GIT_LITERAL_PATHSPECS` widens the digest and fails closed; sparse-checkout does
not affect a tree-to-tree diff; and git itself refuses a non-commit object under
`refs/heads/`.

**Round 21 refused it twice more, and both Importants were mine to own.**

The rust reviewer found that round 20's heredoc fix closed one spelling of the
class it claimed. Reading the single character in front of `<<` rejects
`$((1<<X))` because that character is `1` — but shell arithmetic is normally
written with spaces, and then the character in front is a space, which is
exactly where a redirection can sit. Measured at rc 0: `$(( 1 << X ))` with a
bare `X` later, `$(( n << shift ))` with a bare `shift`, and `"a <<EOF b"` with
a bare `EOF`, each swallowing the `git commit` between them so the hook exited
before its own commit filter — three cargo gates, trampoline install, test
erosion count and every override refusal unrun. The comment claimed the class;
the case covered one spelling of it. That is the fourth time in this change a
guard has described more than it checked.

The stripper now blanks quoted spans and arithmetic expansions to spaces of the
same length before looking for `<<`, so offsets still line up and the operator
is only read where a shell would read it. Two corrections were needed on the
way: a command substitution inside double quotes re-enters shell context, so
`"$(cat <<'EOF' …)"` really is a heredoc and `$(` has to pop the quote state;
and a terminator carries its own quotes, so `<<'EOF'` must be copied through
rather than blanked as a quoted span. Eight shapes measured, all correct.

The durability reviewer found the other half of a fix from round 20. The
post-rename sync failure correctly keeps the commit — and wrote its warning to
stderr, on a path that exits 0. This hook surfaces stderr only when it exits 2,
which `.claude/hooks/guard-write.sh` states in the repository's own words, so
the only line a person saw was the one saying the gate passed. Measured end to
end with a `python3` shim that renames and then fails the directory sync. The
invariant in `durability-invariants` is keep-and-retry *and* show a persistent
warning; the second half was going to a channel nobody reads, while this file's
own comments claimed it was "said out loud rather than taken quietly". The
caveat now rides every `note`, and the nothing-staged path stopped being a bare
`exit 0` that would have swallowed it. Its nit about `except OSError` was taken
too — anything else escaping that block exited 1 and was read as an install
failure, refusing a commit after the trampoline had landed.

Four mutations watched red: arithmetic no longer blanked, quoted spans no
longer blanked, the caveat dropped from one notice, and the nothing-staged path
back to a bare `exit 0`. A hundred and sixteen across twenty-one rounds.

Two of the reviewers' observations are recorded rather than acted on. The
herestring case buys no coverage — it dies on the `redirection` clause before
reaching the `after != "<"` clause it was written for, so that clause is
unreached today; it stays because the shape is worth stating even when another
guard gets there first. And `gate-commit.sh` now runs the cargo gates on every
`.claude/`, `.githooks/` and `scripts/check-*.sh` edit, which means the
load-dependent 5ms assertion stands between a person and the very edits that
would fix the gate — 038's finding, but this change is what put it in that
position, and that belongs here.

**Round 21's third reviewer arrived late and found the same class again, twice
over.** It measured the working-tree fix as well as the staged one — which is
more than it was asked to do — and reported that `blanked()` closed the quoted
and arithmetic shapes and left two: a `<<EOF` inside a comment, because comments
were not blanked, and `"a \" <<EOF"`, because there was no backslash handling so
the escaped quote read as closing the quote. Both measured at rc 0.

Its headline reproduction is the one that matters. This repository's own commit
form supplies a heredoc terminator:

    git commit -m "$(cat <<'EOF'
    subject
    EOF
    )"

so a single preceding line holding `<<EOF` in quotes swallowed everything down
to that `EOF`, the commit included, and the hook exited 0. On a fresh clone the
first commit is the one that installs the trampoline, so that one decoy line
meant the review gate was never installed at all.

Its third nit is the same defect pointing the other way, and I had not looked
for it: `cat 2<<EOF` and `cat >f<<EOF` are real redirections that the
one-character test rejected, so their bodies survived into the command text and
a legitimate command was refused for a reason untrue of it — breaking the
property round 20 was protecting, that a `git commit` in a body is not a commit.

The rule is no longer about one character. Comments, backslash escapes, quoted
spans, arithmetic and command substitutions are resolved in `blanked()`, and the
operator test reads the whole word in front: nothing means a plain `cat <<EOF`,
all digits means a file descriptor, a word holding `<` or `>` means a chained
redirection, and anything else is not a redirection. `<<<` is excluded on both
sides, because the pattern can land on its second `<` and make the character in
front look like a chain — which is how the herestring case went red while I was
fixing the others, and is why it stays in the table now that it discriminates.

Ten shapes measured, all correct, plus the headline decoy in both directions.
Four mutations watched red: comments unblanked, backslashes ignored, the
lookback narrowed to one character, and the herestring exclusion dropped. A
hundred and twenty across twenty-one rounds.

**Round 22's durability reviewer was asked whether a text pin was enough and
said no, then proved it.** Deleting one line — the `unsynced=` assignment in the
exit-3 branch — left every guard green while the notice went back to saying only
that the gate passed. The pin asserted that a `note` contained the literal
`${unsynced`, never that the variable was ever set. Reproduced here before it
was accepted.

That is the fifth guard in this change that described more than it checked, and
the first one written *to close a previous Important from the same reviewer*.
The pattern is now specific enough to name: a pin that reads the text of a
mechanism is checking the sentence, and the sentence survives the mechanism
being deleted. Only something that runs the path can tell.

So the scenario it asked for exists. A `python3` shim ahead of the real one on
PATH renames the trampoline and then fails the directory sync, driving the
exit-3 branch end to end: exit 0, the landed file byte-identical to `main`'s
committed trampoline, and the caveat present in what a person actually reads.
The reviewer's own mutation now fails it.

Its nit 5 is the same mistake in the shell, and I would not have found it:
`exit 3` was taken as proof the rename had happened. A `python3` that exits 3
without renaming had the hook announce a gate that was not there, with an empty
hooks directory — every other reader in this change observes its artefact by
size and last line, and this one alone trusted a status. The landed file is read
back now, and a second scenario drives that.

Its nit 2 corrected a comment that was factually wrong: `fsync` and
`F_FULLFSYNC` do flush inode metadata, mode included, so the reason given for
ordering `chmod` before the sync was untrue. The order is still right — what it
buys is that the mode change is inside the state the sync forces rather than
applied after it — and the comment says that now. Nits 1 and 4 taken: the trap
no longer names a `.sync` file nothing creates (its paired pin moved in the same
edit, which is lesson 004's shape), and the refusal names `python3`, which the
reviewer measured resolving to a mise shim on this machine rather than
`/usr/bin/python3`.

Its nit 3 corrected me in the other direction, which is worth recording because
it means a finding I had carried for two rounds was wrong: `mktemp` failing in
`.githooks/reference-transaction` does *not* refuse silently. Measured with a
TMPDIR at 0500, `mktemp` prints its own diagnostic and git names the hook that
stopped the update, the branch does not move and the index is left as it was.
What is missing is only the gate's own wording.

Two mutations watched red, including the reviewer's. A hundred and twenty-two
across twenty-two rounds.

**Round 22's rust reviewer found two more guards that could not fail, and both
were about the fixes made this round.**

The `$(` clause in `blanked()` — the one that lets a command substitution
re-enter shell context inside double quotes — had no test at all. Deleting it
left every one of the thirty-six rows green. The pass-through row meant to cover
it carried a *prose* body, `Refuse a core.hooksPath override; GIT_CONFIG too`,
which holds no `git`, so the override scan found nothing either way and the row
passed regardless of the clause. Its body is a real `git -c` command now. What
that clause protects is this repository's own commit form: `-m "$(cat <<'EOF' …
EOF)"`, whose message text would otherwise be read as command text and refuse
ordinary commits for reasons untrue of them.

And the `notes` pin could not fail for one of the three notes it claimed to
cover. That call is guarded by `[ -n "${unsynced:-}" ] &&`, so the literal
`${unsynced` was present on the line even with the caveat dropped from the
message itself. It reads only what follows `note "` now. Two reviewers found
two different holes in the same pin in the same round, from opposite directions
— the durability reviewer that the variable need never be assigned, the rust
reviewer that one line's guard supplied the literal.

Its nit 1 is taken: `block` is matched anywhere in the exit-3 branch rather than
only at the start of a line, because two rewrite spellings escaped. Its nit 3 is
taken: `is_none_or` after an `is_some()` assertion says the opposite of what the
code means, and is `is_some_and` now.

Its nit 2 is a correction to this record rather than to the code, and is kept
visible for that reason: the two arithmetic rows and the decoy row do not prove
three independent things. They die on the same clause as rows that were already
there. The table above should not be read as one proof per row — measured, and
only the comment, backslash, herestring and file-descriptor rows discriminate
alone.

Three mutations watched red, including the two this reviewer had proved green.
A hundred and twenty-five across twenty-two rounds.

Seven Importants across rounds 21 and 22, and every one of them was a guard of
this change that did not check what it said. That is no longer a run of bad
luck; it is the shape of the work. A pin reads a sentence about a mechanism, and
the sentence survives the mechanism being deleted — so the pins that matter have
been replaced by scenarios that run the path, and the ones that remain are the
cheap half of a pair.
