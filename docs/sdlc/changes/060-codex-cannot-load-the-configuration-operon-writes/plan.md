# Plan: Operon writes a `hooks` key Codex's own schema rejects

- **Spec**: none — `bugfix` route, and `docs/sdlc/routes.yaml` says the bug
  report is the intent. The report is the measurement below.
- **Approved**: 2026-09-20
- **Status**: in progress

## The cause, named

`apply_codex_config` in `src/tmux/hooks.rs:700` inserts a top-level
`hooks = true` into `~/.codex/config.toml`. Codex's top-level `hooks` key is a
**table** (`HooksToml`), not a scalar, so the line is the wrong type for the
schema and Codex refuses to load its configuration at all.

Measured against `codex app-server` 0.155.1, three readings from one probe so
the answer is a comparison rather than a single observation
(`~/.operon-hook-backups/probe-codex-config-line1.py`, which copies `~/.codex`
into a temporary `CODEX_HOME` and never edits the developer's own file):

| Config | `thread/start` |
|---|---|
| as-is | `REFUSED … config.toml:421:2: cannot extend value of type boolean with a dotted key` |
| top-level `hooks =` removed | `started thread 01a0ba8d-…` |
| `[hooks.state.…]` blocks removed instead | `REFUSED … config.toml:1:9: invalid type: boolean `true`, expected struct HooksToml` |

The third row is the one that names the cause. Dropping the blocks the same
function appends does **not** fix it, and the error changes from a TOML syntax
complaint to Codex's own deserializer saying what the key is supposed to be. So
this is not two features colliding and not a version drift: the line has never
been valid, and it fails two different ways depending on what else is in the
file.

The same function makes both halves of the collision — it inserts the scalar at
line 691-702 and appends `[hooks.state."…"]` blocks at line 706-711 — so one
function on its own produces a file that cannot be parsed.

**Blast radius is not the restore.** Every Codex session on the machine dies at
configuration load, whether or not it was launched through Operon, because
Operon edits the CLI's own global config. This was found while investigating
why a restored Codex thread opens empty (change 059's remaining cause); that
investigation cannot proceed until this is fixed, because `thread/start` never
returns.

## Why nothing caught it

`apply_codex_config` has **no test** — `grep -n apply_codex_config src/tests.rs`
returns nothing — and the crate has **no TOML parser**, so nothing in the suite
has ever parsed what that function writes. The function's own doc comment says
it is line-based "rather than parsed" for good reasons about preserving a
hand-edited file; that argument is about the *input*, and it was allowed to
excuse never checking the *output*.

## Files that change

| File | Change |
|---|---|
| `src/tmux/hooks.rs` | stop inserting the top-level scalar; write `hooks = true` under `[features]`, which is the key Codex actually has, and only when it is not already there |
| `Cargo.toml` | `toml` as a **dev-dependency** — authorised, see below |
| `src/tests.rs` | the failing test, then the guards |

`Cargo.toml` is `high paused` on the `dependencies` surface in
`docs/sdlc/risk.yaml`. The repository's owner was asked before it was touched and
chose a `toml` dev-dependency over a hand-written structural check. It is a
dev-dependency, so it is not in the shipped binary and the local-first promise
that row exists to protect is untouched.

## Order of work

1. `#[test]` that runs `apply_codex_config` over a fixture and parses the result
   with `toml`. Watch it fail, and read the failure to confirm it says what was
   predicted — a wrong-type/extend error, not something else.
2. Add the dev-dependency (step 1 cannot compile without it; this is the one
   place the tree does not compile between steps).
3. Fix `apply_codex_config`.
4. A second guard that is not a TOML parse: `codex app-server` must reach
   `thread/start` against a config this function produced. `hooks = true` is
   *valid TOML* and still wrong for Codex, so a parse alone cannot see this
   class of defect. `#[ignore]`d, because it needs a live `codex`.
5. Mutations, each watched failing.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| `[features] hooks = true` is not what enables the feature either | hooks registered but never run | step 4's live check reaches `thread/start`, but not that a hook fires; stated as a limit rather than claimed |
| Another writer already put `hooks` under `[features]` | duplicate key, invalid TOML again | the parse guard, with a fixture that already contains an existing block |
| A seventh `#[ignore]` | `CLAUDE.md` says six is the number that matters | counted and explained here, not absorbed silently — `tests_ignored` is banded `max 6 7 8 10`, so a seventh is a `warn` |
| The line-based editor corrupts something else it does not parse | a config that loads but lost a key | the parse guard compares the whole document, not just the key under test |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 7 ignored`, and
  the seventh explained above.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- The new test fails before step 3 and passes after, for the stated reason.
- `cargo test --locked -- --ignored` — the live check reaches `thread/start`.
- Mutations: reinstating the top-level insert goes red.

## Departures from the plan

**The live check is a script, not a seventh `#[ignore]`d test.** Step 4 planned
an ignored test and budgeted the `warn` on `tests_ignored`.
`scripts/check-codex-rollout-shape.sh` already solves this exact problem for
this exact product, and its header says why an automated caller would be wrong:
it reads what is on THIS
machine, CI has none of it, and a check that can only pass is not a check
(lesson 007). `scripts/check-codex-config-shape.sh` follows it — not-wired, run
by hand after updating Codex or touching `src/tmux/hooks.rs`. `tests_ignored`
stays at 6.

It was watched failing against a real artifact rather than a synthetic one: the
backup taken before repairing this machine's config. Both halves fired — the
top-level scalar, and `codex cannot load it: … cannot extend value of type
boolean with a dotted key`.

**The fix also repairs, which the plan did not say.** Not writing the bad line
again leaves every machine that already has it broken forever, because nothing
else will ever remove a line Operon wrote. `apply_codex_config` now drops a
top-level `hooks` scalar on the way through, including on the remove-only path —
that path is how a person turns Operon's hooks off, and it would be the one
moment to leave Codex unable to start.

**A branch was deleted for being untestable.** `key_of` began with an explicit
`starts_with('#')` case for comments. Its mutation came back green, and the
reason is that the branch cannot matter: `# hooks = true` yields the key
`# hooks`, which never equals `hooks`. It read as carefulness and could not be
made to fail, so it is gone rather than left standing. The mutation slot now
holds `key.trim()`, which is the load-bearing part.

**The lesson extends 024 rather than opening 025+.** Same file, same product,
same tell — an assertion naming what Operon writes instead of what Codex
accepts. The new part is recorded there: a stale fixture lets a defect through,
while a fixture written from our own intention forbids the repair, and this one
did. `docs/sdlc/bands.yaml` having just recorded that the entry format is the
cost the band keeps reporting, a sixth kilobyte to say 024 again would be the
failure that file describes.

**Proof of completion, as measured.** `cargo fmt --check` silent;
`cargo test --locked` `485 passed; 0 failed; 6 ignored`; clippy silent;
`bash scripts/check-bands.sh` exit 0 with one `warn` (`always_loaded_bytes`
11648, not this change's); `bash scripts/check-codex-config-shape.sh` exit 0 and
exit 1 on the pre-repair backup; `mutations.py` 5 of 5 caught.

**The recognition rule was inverted, which the plan did not contemplate at all.**
The plan treated "recognise a `[features]` header" as a thing to get right, and
three review rounds each handed over one more spelling it got wrong: a trailing
comment, space inside the brackets, then a quoted name and a body opening with a
multi-line array. Each round closed the spelling it was handed, and each fix
reproduced the failure it was fixing. The fourth round refused to supply a fifth
spelling and named the shape instead.

The shape is that recognition failing is not the problem — falling towards the
keyboard when it fails is. The two outcomes are not comparable:

| when recognition fails | cost |
|---|---|
| write nothing | the flag is not set, so hooks register and never fire. Reversible, and `check-codex-config-shape.sh` says `features.hooks: not set` |
| write anyway | duplicate table or duplicate key, and Codex loads no configuration at all — every session on the machine, from the first frame after launch |

So `set_features_hooks` now asks a deliberately generous pair of readers
(`might_open_table`, `might_bind`) before it writes, and stays out of the file
whenever they disagree with the strict one. This is the only part of the change
that keeps working against a spelling nobody has thought of yet.

**Three things fell out of implementing that, none of them asked for.** The
generous readers were written with `contains` and were wrong in the direction
the veto exists to make safe: `[features.context_management]` is a different
table, and `hooks_enabled` is a different key, and both would have stopped an
ordinary config ever getting its flag. Both now compare a trimmed, unquoted
identity — generous about spelling, strict about which thing it is. And the
narrow strict scan the generous one was added beside turned out to be fully
subsumed by it; its mutation came back green, so the branch went with the
mutation. That is the second branch deleted for being untestable in this change.

**A `'''` inside a single-quoted awk program ends the program.** Mirroring
`opens_multiline_value` into the shape script's awk was three lines of work and
one of them closed the shell string, so the script stopped parsing entirely.
What caught it was `every_shell_script_parses_under_the_oldest_shell_its_shebang_
finds` — a guard from another change, firing on a file it had never been written
for. `\047` now. Its neighbour: `awk -v` runs escape processing over the value,
so the shared header pattern needs doubled backslashes, measured by watching the
single-backslash version match `[features]` zero times and read straight past
the first table into the body below it.

**The fix for the last departure introduced two more of the same kind.** Moving
the shared "where does a table start" rule onto `is_table_header` converted one
of the two Rust scans that ask it, and the comment announcing the move claimed
both. `first_table` kept the old test for a round, and the two implementations
disagreed again — the checker refusing a file and printing that Operon removes
the line, the writer removing nothing. That is lesson 028, and the mutation set
now mutates each call site separately so converting one and not its neighbour
cannot come back green.

The second: `table_header_name` took anything between brackets as a name, so an
array row without a trailing comma (`  [1, 2]`) was a table called `1, 2`. This
is the one thing in the change that had to get STRICTER, and the reason is worth
keeping straight. The "write nothing when recognition fails" rule protects
against a reader that does not know; it cannot protect against a reader that is
confidently wrong, because such a reader never consults the veto. A header name
now has to look like a TOML key path.

**Two files were staged that had nothing to do with this change**, and the
mechanism was `git add -A` rather than a decision: `.claude/config.json` and
`.codex/config.toml`, both setting `GIT_WORK_TREE` and `GIT_DIR` to absolute
paths inside this worktree. Committing them would aim git at this tree from
every checkout — the thing `gate-commit.sh` refuses for `git -C`, reached by an
environment variable it cannot see. Unstaged, left untracked, and not deleted:
they were not written by this change. Lesson 029.

**One thing is recorded and not fixed.**
`caps_child_output_during_capture_instead_of_after_buffering` failed once in ten
full-suite runs, on its wall-clock assertion rather than on either behavioural
one — `stdout_truncated` and the 4 KiB length both held, so the code was right
and the machine was busy. This change adds 11 subprocess spawns costing 0.50s in
total, which makes a contribution to load plausible and not demonstrated against
an assertion carrying two seconds of headroom. It is lesson 020's class and it
belongs to its own change; absorbing it into this one would mean editing
`src/exec.rs`'s test to make this change's numbers look clean.

**The two scans needed opposite readers, which the previous departure got
exactly backwards by unifying them.** Round 4's fix put `first_table` and `end`
on one header test and called that the end of the drift. Round 5 showed the
unification was itself the defect: the two scans' mistakes cost opposite
amounts, so one reader cannot serve both. `first_table` reaching too far deletes
a key inside somebody's table; `end` reaching too far only declines to write.
They now take the generous and the strict reader respectively, and
`docs/sdlc/lessons.md` entry 030 carries the table.

The input that showed it: a dot inside a quoted table name. `profiles."gpt-5.6"`
is one table, and splitting the name on every dot makes the line stop being a
header — after which the top-level region swallows the file. Operon writes that
shape itself (`[hooks.state."…/hooks.json:stop:1:0"]`, 221 on the machine this
was found on), so the reader could not recognise its own output.

**And the guard this change was built around could not see it.** Every fixture
so far asked whether what Operon writes still parses. This output parses
perfectly — it is simply missing a line the person wrote. A well-formedness
guard cannot see a well-formed wrong answer, so the fixtures now assert the
survivor by name rather than only the parse.

**The `2>/dev/null` nit is closed as accepted, not fixed**, with the reviewer
agreeing the trade is stated correctly: it costs the reason a binary failed to
start and never a verdict, because that path falls to TIMEOUT rather than
REFUSE.

**Every reader here was reading one line at a time, and that was the assumption
under all three of round 7's Importants.** Six rounds of review went into making
the PATTERNS right — a trailing comment, space inside brackets, a quoted name, a
key path — and no pattern can answer a question about context. `  [1]` is a
table name TOML would accept and a row of `matrix = [` at the same time;
`[features]` is a header and three words of somebody's note; `hooks = true` is
the line an older Operon wrote and a line of that same note. What decides is the
lines before it.

`structural_lines` is the state machine, run once per call, and the same machine
now exists in the script's awk. The plan did not have it: the plan had a list of
spellings, which is what a list of spellings always turns into.

**The vetoes deliberately do not consult it**, and that is a decision rather than
an oversight. Being wrong about context is not symmetric: a veto that reads prose
as code declines to write and costs a flag, while a veto taught to skip prose
misses a real key and writes a duplicate — which costs the machine its whole
Codex configuration. So the boundaries got the state machine and the vetoes kept
reading every line. The consequence is pinned as a fixture: a file whose only
`[features]` is inside a multi-line string gets no flag.

**Six mutations went from red to green in one run because of that fix, and
nothing but re-running them would have said so.** Every fixture that told the
strict reader from the generous one used a bracket-shaped row inside an array,
and those rows are no longer headers to either reader. Two were genuinely
subsumed and went; four had moved out of the space a parse-based fixture can
reach — `is_key_path` accepts every name TOML accepts, so the two readers now
differ only over input TOML would refuse, and such a file does not parse before
or after. `a_config_toml_would_refuse_is_still_not_a_config_to_delete_from`
reads lines instead, and is registered in `mutations.py`'s `TESTS`. Lesson 030's
Guard paragraph had been false for that round and now says so; lessons 031 and
032 are the two general forms.

**The state machine had two flat states and the two values NEST**, which is the
previous departure's fix being half a fix, for the fourth time in this change.
A `"""` opened inside an array could not be represented, so the brackets in
somebody's prose were counted as structure, and review reproduced three losses
from that one gap: the flag written into the middle of a string, a line deleted
out of one, and a person's own `features.hooks` deleted because a `]` in their
text closed the array early and hid the header below it. All three produce valid
TOML.

What fixed it is worth separating from what it fixed. Not a third state:
`starts_open_value` and `continues_value` were TWO functions that had to agree
about one walk, and the gap sat exactly where they met. They are now one
`advance`/`scan` that carries the array depth through the string inside it. The
same round found `opens_multiline_value` still testing a pattern of its own
(`starts_with('[') && !contains(']')`), which called `hooks = [[],` a value that
ends on its line; it is the same walk now. Two expressions of one rule is the
same defect as two implementations of one rule, which this change has recorded
as lesson 001's shape three times and hit twice more here.

**`[[name]]` drifted the two implementations apart inside a single round.** The
Rust learned array-of-tables headers and the awk did not, so the script printed
`REFUSE … Operon now removes it` about a `hooks` inside somebody's
`[[profiles]]` that the writer correctly leaves alone. Both reviewers found it
independently. The cause is not the pattern: the cross-check table had no `[[`
row at all, so one implementation could learn a construct alone. It has them
now.

**The removal side gets a veto, which is this round's one design change rather
than a repair.** `set_features_hooks` had two vetoes and the top-level removal
had none, and two of review's three losses landed on the side that deletes.
`closes_every_value` is the rule stated once: a file this cannot read to the end
is a file it does not edit — not a removal, and not an append either, since an
appended `[features]` would land inside whatever is still open.

**Three Nits were taken rather than carried**, and one of them is a branch
removed for the third time in this change on the same grounds: `might_assign`'s
`starts_with('#')` and `starts_with('[')` tests cannot change an answer, because
the identity test already refuses `# hooks` and `[features] # hooks`. A guard
whose mutation cannot fail is not a guard, and the comment now says so where the
branch was.

**A table can be defined without a header, and the veto only knew headers.** The
plan's whole safety argument rests on `set_features_hooks` declining when it
does not recognise the file, and the thing it was declining over was a bracket
header. TOML defines the same root table two other ways — a dotted key
(`features.hooks = true`) and an inline table (`features = { … }`) — and against
either of them the veto said "no `features` here", appended one, and produced
`Cannot declare features twice`: Codex loads no configuration at all. The
spelling that costs the most is the first, because it is the one this change's
own explanation teaches somebody to type.

This is the veto's third costume for one mistake — round 1 closed a commented
header, round 7 added `[[features]]` by name — and the pattern is the point:
each fix closed a spelling somebody had written down rather than a shape TOML
defines. `might_bind` compares the FIRST SEGMENT of a key path,
quote-aware, so all six reported spellings and every dotted descendant answer
from one test. The cost is chosen rather than inherited and is written at the
call site: the veto reads every line, so a `features.hooks` under `[other]` is a
different table that `[features]` could legally join, and it loses its flag. A
mutation stopping the veto at the first table pins that, so narrowing it later
is a decision and not an edit.

**The checker was worse than the writer here, and that is the half worth
remembering.** It answered `features.hooks: not set`, exit 0, against a file
whose first line sets the flag — telling a person nothing is amiss is the worst
of the three things that line can say. The awk tracks a root region before the
first structural header and reads the dotted spelling in it. The inline spelling
stays unreported, and the script says so in a comment instead of pretending:
under-claiming only says the hooks may not run, nothing is deleted on the
strength of it, and the writer's veto reads the line either way — so the two
never disagree about an EDIT, only about a sentence.

**One path in this function had never been given anything.** The block-dropping
loop at the head of `apply_codex_config` still tested `trimmed.starts_with('[')`
after every other scan had stopped, so against valid TOML holding a
`[hooks.state."K"]` inside somebody's `note = """ … """` it took the prose for
its own block and deleted to the next bracket — losing the person's words and
the closing fence, and producing `invalid multiline basic string`. Both halves
of the failure this change exists to stop, in the path nobody had looked at.
Reaching it is contrived; the structural fact is that two paths here delete a
line and only one had six rounds of care.

The fix generalises rather than patching that loop: `structural_lines` returns
whether the document closed as well as which lines are structural — one walk,
one answer, which is this round's own lesson pointed at the code that learned it
— and the boolean becomes a single early return covering every edit path at
once. The previous round had it at three of four, while the comment above it
claimed the whole function; review found the fourth.

**A fifth branch went for being untestable, and again a fix put it there.**
Taking the round-8 nit about the removal comparing `key == "hooks"` raw meant
unquoting, and the unquoting carried a `.trim()` over a key `assignment_of` had
already trimmed. The mutation removing `assignment_of`'s trim then came back
green — the silent kind, found only by re-running the whole set. The redundancy
came out of the call site rather than out of `assignment_of`, because what is
left is a real difference and not duplication: `" hooks "` is a key TOML names
` hooks `, a veto may be generous about it, and a branch that DELETES may not.

**The same rule had a second expression, for the fourth time.** "Where does this
quoted thing end" was answered twice: an escape-aware walk for `"` and `'`, and
a plain `find(fence)` for `"""` and `'''`. `\"""` is the ONLY way TOML lets
somebody write a literal `"""` inside a `"""` string, so the gap is not a corner
— it is in a note explaining Operon's own config format. Three losses came out
of it, all from valid TOML: the flag written into the note under a `[features]`
that was the person's prose, a `hooks = true` in the note deleted as though
Operon had written it, and a pasted `[hooks.state."K"]` starting the
block-dropping loop, which ate the closing fence and the real `[features]` and
produced `Unterminated string`.

That last one is the block-dropping loop found a second time in two rounds, and
the answer is the opposite of the first: gating it on `outside` was right, and
`outside` was what was wrong. Two reviewers reaching one loop by two routes is
the clearest evidence in this change that the loop was the least-examined code
in it.

The fix is a MOVE rather than a copy, which is the part worth keeping:
`quoted_length` is gone, and `closing_offset(text, delimiter, escapes)` answers
for all four quoted things TOML has — the delimiter is a `&str` because the only
difference between the one-line kinds and the multi-line ones is its length, and
`escapes` is a parameter because the two literal kinds have none at all. Three
call sites ask it, and the awk makes the same move in the same commit. Copying
the escape rule into the fence search would have worked today and left the fifth
occurrence to be found by the next reviewer.

One thing it deliberately does not do: carry `escaped` across the newline. TOML's
line-ending backslash swallows the newline and the whitespace after it, and what
follows is not itself escaped, so each continuation line starts clean — written
at the call site rather than left to be rediscovered.

**Two nits were carried with reasons rather than as sentences.** Change 062 now
holds TWO readers of this file that do not know about structure —
`table_header_shape`'s quote-unaware closing-bracket search, and the script's
`trust_blocks` grep, which counts a line of prose inside a multi-line string.
Same file, same class, one change. Change 063 holds the one user-visible
consequence this change has never shown a person: when Operon is asked for the
flag and cannot read the file, it writes nothing and says nothing, and the
visibility rests on a not-wired script. That is a new message id needing a row in
every table in `src/i18n_tables.rs` and a warning plumbed onto a frame —
`feature`/`modify` route work, which `AGENTS.md` says runs the pipeline rather
than riding a `bugfix` at round 10.

**The two halves of this function had never run in the same call.** Everything
above is about reading a file correctly. This one is not: both branches were
right and the defect was between them. `apply_codex_config` preserves a
top-level `hooks` it must not touch and, separately, appends
`[hooks.state."…"]` trust blocks — and a dotted key cannot extend a non-table,
so the shapes deliberately preserved were exactly the shapes the append broke.
Every protection produced the failure it was added to prevent, and the
hand-written inline table is round 1's finding 2 returning through the other
half of the function.

The cause is one argument. Nineteen of the twenty-eight `apply_codex_config`
call sites in `src/tests.rs` pass `&[]` for entries — including both fixtures
that build these shapes — while the enabling path in `install_codex_hooks`
always passes entries. The only configuration in which the protection matters
was the one no fixture built.

The mutation set could not have caught this, which is the part worth carrying:
a mutation is written against a branch somebody has thought of, so a gap
*between* two branches has nothing to be caught by. That is lesson 033, and its
guard is the three preserved shapes run with entries non-empty plus the
boundary the veto must not take (`hooks.enabled = true` is a real table, so the
blocks extend it and are written).

**Proof of completion, re-measured at round 9.** `cargo fmt --check` silent;
`cargo test --locked` `487 passed; 0 failed; 6 ignored`; clippy silent;
`bash scripts/check-bands.sh` exit 0 with two `warn`s — `always_loaded_bytes`
11648 pre-existing, `steering_bytes` 221083 against a 221000 tier and this
change's own, recorded rather than trimmed away;
`bash scripts/pipeline-indicators.sh --lessons` 32 of 32;
`check-codex-config-shape.sh` on all four exit paths, 0 / 1 / 1 / 2;
`mutations.py` 50 of 50 caught, after a run that reported 45 of 50 with four
`[SCRIPT BUG]` lines and one `[NOT CAUGHT]`.

**And the veto knew the root of that path, not the path.** The fix above asked
whether anything binding `hooks` survived. `[hooks.state."<key>"]` creates
*two* tables on its way to the key, so a document that binds `hooks.state`
leaves `hooks` a perfectly good table and still cannot take the append. Three
spellings reach it — `hooks.state = "x"`, `"hooks".state = "x"`, and a `state`
key inside a `[hooks]` table, which the scan could not even see because it
stopped at the first table header — and all three print
`Cannot declare ('hooks', 'state', 'K') twice`.

This is the departure above wearing lesson 026's opposite costume. There the
recogniser kept missing another spelling of one name; here the name was
recognised perfectly and the question was scoped to one segment of a path. Both
correct the same way: **write the guard against the thing being written.** The
write is `[hooks.state."<key>"]`, so the subject is the path, and
`can_hold_hook_state` asks every proper prefix of it. Asked that way,
`hooks.state.inner = true` and `[hooks.state]` fall out as allowed without a
fixture apiece, and `[[hooks]]` falls out as refused — a spelling nobody
reported, which parses cleanly and quietly files Operon's trust state inside
the last element of somebody's array. Its fixture asserts a shape, because a
parse guard cannot see it.

Round 11's rust nit closes in the same edit and by deletion rather than by
rewrite. It asked for `removing.len() == toplevel_hooks.len()` to become
`toplevel_hooks.iter().all(removable)` — the fact asked instead of restated.
The subprocess pass said the fact itself was wrong, so `extendable`,
`toplevel_hooks` and `removable` all go and the removal filter collapses back
into one chain. The two readings meet: a restatement has no place to put the
question it stands in for, which is exactly why the wrong question was easy to
leave in place. Lesson 034.

**Proof of completion, re-measured at round 12.** `cargo fmt --check` silent;
`cargo test --locked` `487 passed; 0 failed; 6 ignored`; clippy silent;
`bash scripts/check-bands.sh` exit 0 with two `warn`s — `always_loaded_bytes`
11648 pre-existing, `steering_bytes` 225717 against a 221000 tier and this
change's own, recorded rather than trimmed away;
`bash scripts/pipeline-indicators.sh --lessons` 34 of 34;
`check-codex-config-shape.sh` on all four exit paths, 0 / 1 / 1 / 2;
`mutations.py` 63 of 63 caught, after a run that reported 58 of 63 with five
`[SCRIPT BUG]` lines and no `[NOT CAUGHT]` — the four new mutations landed red
first time and the five stale find-strings were the ones this round moved.

**And one veto was never asked the question the other two had learned.**
`set_features_hooks` holds two of them, and they are one question: does this
line bind `hooks`? Round 11 rewrote the root one to compare the first segment
of the key path. The `[features]`-body one kept its own `might_assign`,
comparing the whole path, and review round 12 reached it with two lines of
ordinary TOML:

```toml
[features]
hooks.state.inner = true
```

That binds `hooks` as a table. The veto does not see it, `hooks = true` is
inserted beside it, and Codex answers `dotted key hooks attempted to extend
non-table type (boolean)`. All four dotted spellings do it; `hooks_enabled`
correctly does not, and the two already-vetoed spellings still are.
`scripts/check-codex-config-shape.sh` reported `features.hooks: not set` and
exited `0` about the same file — the two implementations agreed while the
writer wrote a config Codex cannot load, which is the limit this change already
had written down arriving for real.

The fix is a deletion. The correct reader was already written fifty lines up,
so `might_assign` goes and `might_define_table` is renamed `might_bind`: the
old name described one of its two call sites, and a corrected duplicate is
still a duplicate. One reader in the file still compares the whole path on
purpose, and its comment now says why instead of pointing at a deleted
function — the top-level removal DELETES a line, and a dotted
`hooks.enabled = true` is somebody's real table.

Found by fuzzing with a semantic oracle rather than a parse — "after the
append, is `hooks.state[K].trusted_hash` at the root?" — which is lesson 009's
class for the third time in this change: 2,500 configs, 89 that do not parse,
60 more that parse with the block landing nowhere, all 149 one cause. Added to
lesson 034 rather than opened as 035, because it is the same defect one call
site over.

**Proof of completion, re-measured at round 13.** `cargo fmt --check` silent;
`cargo test --locked` `487 passed; 0 failed; 6 ignored`; clippy silent;
`bash scripts/check-bands.sh` exit 0 with two `warn`s — `always_loaded_bytes`
11648 pre-existing, `steering_bytes` 227975 this change's own;
`bash scripts/pipeline-indicators.sh --lessons` 34 of 34;
`check-codex-config-shape.sh` on all four exit paths, 0 / 1 / 1 / 2;
`mutations.py` 64 of 64 caught on the FIRST run, with no `[SCRIPT BUG]` and no
`[NOT CAUGHT]` — the first round of this change where that happened.

**And nobody had asked about the leaf.** `[hooks.state."<key>"]` declares a
three-segment path. Round 11 asked about the first segment, round 12 about
every proper prefix, round 13 about the second call site that asked the same
question — and the LEAF, the thing the write actually declares, was covered by
the drop loop alone, which compared the header as raw text. So Operon's own
block written back in a different shape was recognised by nobody:
`[ hooks.state."K" ]`, `[hooks . state . "K"]`, `[hooks.state.'K']` were not
dropped and not vetoed, and the append declared the table a second time. Those
are `set_features_hooks`'s first three findings exactly, arriving at the only
header reader in the file that had never had a review round. **The part with no
review history is where the defect was** — six readers had six rounds each and
the seventh had none.

Fixed in two halves, as reported. The drop loop matches by key path; the veto
takes the entries and asks about the whole path per key. A double-bracket
header is still not dropped, because `[[hooks.state."K"]]` is an array and not
a block this ever wrote — deleting it would be taking somebody's data — so the
veto refuses the append instead.

One thing the report did not contain, and it came from fixturing the side that
must still be WRITTEN rather than the side that must be refused: **the prefix
rule is not symmetric.** A dotted key that reaches past our leaf creates it
implicitly and `[hooks.state."K"]` afterwards is `Cannot declare … twice`; a
table header that reaches past it does not, because defining a super-table
after its sub-table is legal. The first draft of this round had one rule for
both and refused `[hooks.state."K".sub]`, which would have cost the trust
blocks for nothing and said nothing about it. An over-firing veto is the half
that is silent when it is wrong, which is why both sides are fixtured now.

Three nits also taken. The trust key is a filesystem path, and it was written
into the header unescaped — a `"` or a newline makes the file unparseable, and
`\b` is the quiet one, read by TOML as a backspace so that Codex stores a key
Operon never looks for. It is REFUSED rather than escaped: escaping would make
the append the only encoder in a file with five decoders that strip quotes
without unescaping, which is the writer/reader seam change 062 already holds. A
sentence recorded beside the "vetoes read every line" trade claimed its cost
was unobservable in valid TOML; it is observable, and the measured version
replaces it. And every deliberate `exit 2` in `check-codex-config-shape.sh`
prints an `UNKNOWN:` line now, because bash returns 2 as well when it cannot
parse the script and a caller reading only the number cannot tell a healthy
refusal from a broken file.

**Proof of completion, re-measured at round 14.** `cargo fmt --check` silent;
`cargo test --locked` `487 passed; 0 failed; 6 ignored`; clippy silent;
`bash scripts/check-bands.sh` exit 0 with two `warn`s — `always_loaded_bytes`
11648 pre-existing, `steering_bytes` 229602 this change's own;
`bash scripts/pipeline-indicators.sh --lessons` 34 of 34;
`check-codex-config-shape.sh` on all four exit paths, 0 / 1 / 1 / 2;
`mutations.py` 70 of 70 caught, after a first run of 68 of 70 with two
`[SCRIPT BUG]` lines and no `[NOT CAUGHT]`.

**And the fix for the leaf broke something the old code got right.** Round 14
replaced the drop loop's raw-text header comparison with one by key path, which
was correct and closed the class it was aimed at. It also lost a case: a key
containing `]`. `table_header_shape` finds its closing bracket without
respecting quotes, so `[hooks.state."/Users/me/Accounts/[work]/hooks.json:stop:1:0"]`
splits inside the key and is a header to nobody. The raw comparison never
cared — it matched the whole line as text. The key-path reader gets `None`, the
line falls through to `assignment_of`, which returns `None` for anything
opening with `[`, and the block Operon itself wrote became invisible to every
reader in the function.

The account folder is one a person picks in a file dialog and `]` is legal in a
macOS folder name, so this is reachable rather than theoretical: the first
launch writes a correct block, every launch after it appends a duplicate, Codex
loads no configuration under that root, and the backup taken before each write
replaces the good copy with the broken one.

The quote-unaware bracket search was **already a known nit**, carried to change
062 on the argument that it was invisible to a generous reader. That argument
was true of the code it was written against. Generalising the caller changed
what the nit costs, and nobody re-read the deferral — which is the part worth
carrying forward, and lesson 035 says so: *a nit is deferred against the code
as it was.*

Closed by refusing, in the shape the reviewer proposed and the shape the branch
one line up already argues for: `can_hold_hook_state` returns false when a
structural line opens a table it cannot READ as one. That comment was already
written — "a header misread means the rest of this walk attributes keys to a
table it invented" — and applied only to an unterminated quote.

**The guard that was missing is one idea: apply the writer to its own output.**
All forty `apply_codex_config` call sites in the suite hand it a config
somebody typed, and this defect needs two calls in sequence — which is exactly
what Operon does, since it installs its hooks on every launch. Lesson 033 was
two branches never run in one call; this is one call never run twice.

Six smaller things went with it, and three of them are the same class the
change keeps producing — a name that stopped describing the thing, or one rule
with two expressions. `paths_collide` named a function the tree does not have.
`unquote_segment` trimmed inside the quotes, so `" hooks "` compared equal to
`"hooks"` while the removal filter fifty lines down argued in its own comment
that trimming twice is what must not happen — and the reader with the wrong
answer was the one that DELETES. Two `might_assign`/`might_define_table`
mentions still read as the present design. The cross-check table was missing
the one quoted spelling where the writer and the script actually disagree. The
new `bash` call site ignored the `script` and `root` bound at the top of its
own function. And the Guard column for `UNKNOWN:` named the wrong test, which
no automated reading could see: `mutations.py` runs the whole suite, and
`--lessons` only asks that the identifier exists as a `fn`.

**Proof of completion, re-measured at round 15.** `cargo fmt --check` silent;
`cargo test --locked` `487 passed; 0 failed; 6 ignored`; clippy silent;
`bash scripts/check-bands.sh` exit 0 with two `warn`s — `always_loaded_bytes`
11648 pre-existing, `steering_bytes` 233450 this change's own;
`bash scripts/pipeline-indicators.sh --lessons` 35 of 35;
`check-codex-config-shape.sh` 0 on the real 60,423-byte config and 0 on its
backup; `mutations.py` **74 of 74 caught on a clean first run** — no
`[SCRIPT BUG]`, no `[NOT CAUGHT]` — with the four new ones each watched failing
on its own beforehand.

The test count does not move, because every addition went into fixtures inside
three existing tests. Worth saying out loud: from the number alone, a round
that adds an Important's guard and six more looks like a round that added
nothing.

**And the same character was still open on the path that deletes.** Round 15
refused an unreadable table header at `can_hold_hook_state`, which is the last
of four things `apply_codex_config` does. The scan deciding where the top-level
region ends runs earlier and still walked past
`[projects."/Users/me/[work]"]`, so the `hooks = false` under it was removed as
a top-level scalar an older Operon had written. Valid TOML in, valid TOML out,
a line the person typed simply gone — and the dotted twin of that config
(`[profiles."gpt-5.6"]`) has been pinned in this suite for nine rounds.

**Both reviewers found it independently and proposed different fixes, and what
shipped is neither.** One proposed making the document not-understood and
taking the existing early return — one line, all four paths. The other proposed
stopping the top-level scan at the unreadable header — the deleting path only.
Measuring the first against the second is what decided it: refusing to edit the
document also refuses to **repair** it, so a machine whose config carries both
an unreadable first header and the scalar an older Operon wrote would be left
unable to start Codex, which is the defect this change exists for.

So the question is asked once, in `opens_unreadable_table`, and the two readers
answer it for their own reasons: `first_table` stops, because running past
deletes somebody's key; `can_hold_hook_state` refuses, because it cannot know
whether its block would be a second declaration. That is lesson 030's rule,
which every other pair of readers in this file already follows.

**The narrowing came from the suite, not from either proposal.** Treating every
unreadable `[` line that way also catches `[a`, which closes nothing and is a
header to nobody — round 5 decided the `hooks = true` under it is genuinely
top-level and must still be repaired. The discriminator is whether the line
*closes*: a `]` present but mis-split is a header we may have misread, a `]`
absent is not a header at all.

**And the first attempt turned a mutation from red to green.** With an
unreadable header stopping the whole write, the mutation that breaks `[[…]]`
recognition produced a refusal rather than a duplicate, and the fixture
asserted only that the output parses. `[NOT CAUGHT]` on a set that had been
clean — lesson 032 again, reported by the mutation run and not by either
reviewer. Repairing it took two attempts, the first of which asserted a
substring the writer itself adds: a well-formed wrong answer inside the test.

**Proof of completion, re-measured at round 16.** `cargo fmt --check` silent;
`cargo test --locked` `487 passed; 0 failed; 6 ignored`; clippy silent;
`bash scripts/check-bands.sh` exit 0 with two `warn`s (`steering_bytes`
236844); `--lessons` 36 of 36; `mutations.py` **76 of 76**;
`check-codex-config-shape.sh` `config.toml` 0, `config.toml.operon.bak` 0, and
the pre-repair backup 1 — which is what it is supposed to be, and is now named
rather than left as "backup".

**And there was a third reader, which the doc comment said there were not.**
Round 16 wrote, in `opens_unreadable_table`'s own doc comment, *"Two readers
need to know that"*, and listed them. Review found the third in the same
round: the **drop loop**, which runs before both and removes Operon's own
`[hooks.state."…"]` blocks so the append that follows can write them again.
The append is the thing `can_hold_hook_state` refuses. So on a machine where
the hooks already worked, the launch after Codex appends
`[projects."/Users/me/src/repo[1]"]` measured trust blocks 2, 0, 0, 0 across
four launches. **It deletes what it cannot re-add, and it does not heal** —
the header is still there next launch, so every launch is launch 1 again.

That re-prices the refusal. Declining to append was cheap while the
alternative was a config Codex cannot load; for a machine that HAD the blocks
the same rule destroys working trust state, and destroys it with the call that
decided not to write. Nothing in the change had noticed the two populations
differ, because every fixture started from a config without our blocks.

**The fix is the same predicate at document grain**, not per line: a readable
header of ours can precede the unreadable one, and asked per line the loop
deletes there and is refused anyway. Both grains are mutations now. It is
deliberately not a second early return — round 16 measured what that costs.
The removal path stands still too, which is a decision: a block left behind is
removed on the next launch once change 062 makes the header readable, a block
deleted beside a veto is somebody's trust state gone.

**The fixture the reviewer asked for did not exist.** Four launches now, from a
config Operon has already written its blocks into, asserting a constant two
blocks, the person's table intact, and a parse. Three launches and not one,
because "it does not heal" is the finding. Watched failing at `left: 0,
right: 2`.

**Proof of completion, re-measured at round 17** (digest `f6cace06375f32bc`).
`cargo fmt --check` silent; `cargo test --locked` `487 passed; 0 failed;
6 ignored`; `cargo clippy --locked -- -D warnings` exit 0 and silent past the
compile lines; `bash scripts/check-bands.sh` exit 0 with the same two `warn`s
(`steering_bytes` 240673, up from 236844 for lesson 037, same tier);
`--lessons` **37 of 37**; `mutations.py` **78 of 78**;
`check-codex-config-shape.sh` `config.toml` 0 (60423 bytes, 221 blocks, flag
set), `config.toml.operon.bak` 0 (60266 bytes, 221 blocks), and the pre-repair
backup 1, unchanged and supposed to be.

**And then the fix's own comment made the same class of claim, three times.**
Both reviewers approved round 17 with nits, and every nit landed on one
paragraph — the justification for gating the drop loop, which is stated in
three places and stated slightly wrong in each. Two were imprecise: the rule
was written with no exception for the removal path, and the one sentence that
did mention removal borrowed the enabling path's reason for it ("a block
deleted beside a veto", where there is no veto). The third was **false**: "a
block left behind is removed on the next launch once the document is
readable". `install_event_groups` only pushes to `written` inside
`if !remove_only`, so with `remove_only` the keys come back empty and the
caller stores `codex_trust_keys: []` — the same call that decided to keep the
blocks forgets their names. They stay until the hooks are turned back ON.

This session had repeated that expiry claim into `review.yaml` after taking it
from one reviewer unmeasured, and the second reviewer found it by reading the
callers. **An expiry claim is a claim about another function's behaviour**, and
it was believed here because it was plausible and came from a reviewer. The
honest argument replaces it, and it is about the data rather than a future
repair: what remains is inert — the `hooks.json` registration is gone so Codex
never matches these entries, a stale `trusted_hash` can only fail closed, and
nobody's own configuration is touched.

**And the decision had no guard.** A reviewer measured that the caveat a reader
reaches for first — `dropping = (!holds_unreadable_table || entries.is_empty())`,
letting the removal path delete since nothing appends after it — passes the
entire suite, because round 17's four-launch sequence is all
`enable_hooks = true`. A `remove_only` row now follows it, plus a row asserting
the top-level scalar is still repaired on the way out, because turning Operon
off would be the one moment to leave a machine unable to start Codex. Watched
failing with that exact caveat: `left: 0, right: 2`.

**The counter-example worth keeping**, from the reviewer whose own one-line
proposal it refutes, on the early-return design:

    in:  hooks = true                    out: [projects."/Users/me/]w/repo"]
         [projects."/Users/me/]w/repo"]       hooks = false
         hooks = false
                                              [features]
                                              hooks = true

Scalar repaired, person's key kept, flag set — three at once, on an input the
one-liner would have passed through untouched, leaving Codex unable to start.

**Proof of completion, re-measured at round 18** (digest `a141302e47fc7d74`).
`cargo fmt --check` silent; `cargo test --locked` `487 passed; 0 failed;
6 ignored`; `cargo clippy --locked -- -D warnings` exit 0 and silent past the
compile lines; `bash scripts/check-bands.sh` exit 0 with the same two `warn`s
(`steering_bytes` 242312, same tier); `--lessons` **37 of 37**; `mutations.py`
**79 of 79** with no `[SCRIPT BUG]` and no `[NOT CAUGHT]`;
`check-codex-config-shape.sh` `config.toml` 0 (60423 bytes, 221 blocks),
`config.toml.operon.bak` 0, pre-repair backup 1 and supposed to be.

**And the correction needed a guard of its own.** Round 18 replaced a false
claim with a true one and left it unheld: two comments assert that a removal
hands back no keys, `install_event_groups` has no test and no mutation and is
module-private, and `src/tests.rs` was already calling
`install_codex_hooks(…, true)` for its side effect and dropping the
`Vec<String>`. Binding it and asserting it is empty is one line, at the
caller's height — where the claim is made rather than where it is implemented.
Taken rather than carried, because the previous version of that same claim was
false and survived three rounds: shipping "a corrected claim with no guard is
the same claim waiting for its next correction" as an open nit, out of the
round that wrote lesson 037, would have been shipping the lesson's own
counter-example.

**The verification is the part worth keeping, and it nearly did not happen.** A
reviewer warned twice, before the mutation run finished, that "watched failing"
might not be evidence the NEW guard fired: the new assertion sits three lines
above `assert_eq!(read_json(&codex_hooks_in(…)), original)`, so a mutation that
also wrote to `hooks.json` would fail both and the ordering would show only the
new message. Their prescription was one move — remove the new assertion,
re-run the same mutation, see whether it still fails. Run here: it **passes**,
`1 passed`. Nothing else catches that mutation. The mutation happened to be the
isolated shape already, but that was luck rather than design, and the check is
what turns it into a measurement. **A mutation caught by a guard other than the
one it was written for reports green and proves nothing** — lesson 032's shape,
which this change has already committed once.

**Two non-findings recorded rather than fixed**, both being ways a true
statement can quietly become false: the guard asserts on
`install_codex_hooks`'s return while the comments claim about the
`codex_trust_keys` the caller then stores (no branch between them today, but a
naive litter fix — `if codex_keys.is_empty() { keep the stored keys }` —
inserted between those two lines would pass the guard and falsify both
comments); and the fixture reaches the meaningful empty-return path only
because it installs first, so tidying its setup would make the assertion pass
vacuously.

**Proof of completion, re-measured at round 19** (digest `3f5d8372d0bf5448`).
`cargo fmt --check` silent; `cargo test --locked` `487 passed; 0 failed;
6 ignored`; `cargo clippy --locked -- -D warnings` exit 0 and silent past the
compile lines; `bash scripts/check-bands.sh` exit 0 with the same two `warn`s
(`steering_bytes` 242312, same tier); `--lessons` **37 of 37**; `mutations.py`
**80 of 80** with no `[SCRIPT BUG]` and no `[NOT CAUGHT]`; and the isolation
check above. The mutation run was announced to both reviewers before it
started, and one of them reviewed round 19's code out of
`git show :src/tmux/hooks.rs` rather than the tree while it ran — the protocol
working in three directions after four rounds of losing a measurement to its
absence.
