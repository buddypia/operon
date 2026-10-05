# Report: 037 — the gate stops depending on the table being complete

- **Spec**: `./spec.md`
- **Plan**: `./plan.md`

## What this closes

Everything the reviews of 034 and 036 left as non-blocking, plus the reason
they kept finding the same kind of thing.

## The gate no longer needs the table to be complete

Change 036 made the acknowledgement gate read the command rather than the
picker. Three arguments were then found missing from the tables that gate reads
— `agy`'s single-hyphen spelling, `codex --yolo`, and
`codex --dangerously-bypass-hook-trust` — and every one of them was found by a
reviewer running the CLI and reading its own help. Nothing in this repository
can do that: a test that ran `--help` would need a live agent CLI, and a seventh
`#[ignore]` is a documented regression here that `gate-commit.sh` refuses.

So a hand-kept table cannot be the whole of a safety gate. There is a net under
it now: a dashed argument whose own name carries `dangerously`, or that is
`yolo`, asks for the acknowledgement even when no table has heard of it. Two
rules, not a second table, because a second table would be behind for the same
reason the first was.

The tables keep the last word for what they name, in both directions.
`--allow-dangerously-skip-permissions` carries the word and is deliberately
`dangerous: false` — it only lets a person choose the escape later — and the net
must not overrule that, or the application would disagree with its own table
about one flag, which is the shape change 036 existed to remove.
`any_table_names` reads that out of the tables rather than listing exceptions,
so a flag that stops being dangerous, or starts, moves the net with it.

`--allowed-tools` is judged by its value rather than its name, because the same
flag hands over a shell or a file reader depending on what it is given. It is in
no flag table for exactly that reason.

## A value that spells a flag is not a flag

Dropping the leading dashes is what caught `agy -dangerously-skip-permissions`.
It also made `aider --preset=yolo` match `--yolo` and demand a confirmation for
a launch that needed none. Safe in direction, and exactly how somebody learns to
tick the box without reading it — the same argument that kept
`/opt/--dangerously-skip-permissions-notes/run.sh` ungated. Tokens now carry
whether they were written with a dash, and the flag and alias arms require one.
Modes are the deliberate exception: `--sandbox danger-full-access` puts its
value in a token with no dash of its own, so mode values are still matched as
values — and they go through the same normaliser now, which the documentation
claimed and the code did not.

## A worktree inside the project folder is the worktree's

`document_root_for` tried the project first, and a project contains a worktree
made under it — which is what `.worktrees/` means. So the worktree's own files
were attributed to the project: no branch on the tab, the Diff view offered when
it reads the wrong tree, and a file the agent had just written reporting itself
unchanged, because the project's `git diff` says nothing about a path it
ignores. Deepest match wins now, the rule `src/git/ports.rs` already uses to
decide which worktree a listening socket belongs to.

## One question about whether a document may have a diff

Three places acted on it and one asked nothing, so a worktree document made the
project's diff cache be discarded and `git diff` run one frame before the view
refused to draw. `document_may_have_a_diff` is the one question; the view
switcher, the 差分を見る button and the refresh all call it.

Writing that guard turned up three fixtures in a state production cannot reach:
`active_document` set to a document that was never in `open_documents`. Harmless
while only the drawing asked the question — drawing never runs for a document
that is not open — and load-bearing the moment the refresh asked it too. They
open the document now, through a `show_document` helper that says why.

## Departures from the plan

- **`OpenDocument::named` needed a structural assertion as well as a value
  one.** The message id and today's English row are the same text, so comparing
  `named()` against `tf!(…)` cannot tell it from a `format!` producing the same
  bytes — the first version of that guard passed with the `format!` restored.
  It reads the function body too, and its comment says what that does not cover.
- **The first shape of the table override returned on the first matching
  flag**, which made `codex --sandbox danger-full-access` safe: Antigravity's
  `sandbox` flag matched first and is `dangerous: false`, so the mode arm was
  never reached. Caught by 036's own guard within a minute. The override is now
  scoped to the token it names rather than to the whole command.

## Costs

`steering_bytes` is at `propose` and this change adds none of it — the growth in
this reading is another session's uncommitted work in `REVIEW.md`,
`docs/sdlc/README.md`, `docs/sdlc/lessons.md` and `.claude/skills/sdlc/SKILL.md`.
Change 030 owns the reduction and is `awaiting-user`.

## What the reviews found, and what they found in the fix

Two reviewers, two rounds each, and every Important was in the new code rather
than the old.

**Round 1, both independently: `claude --allowed-tools Edit Bash` walked past
the gate.** Space-separated is the spelling Claude Code's own help gives first,
and the scan looked only at the token immediately after the flag. The sharper
half of the same finding was mine to answer for: three of the four values that
guard pinned — `Bash,Read`, `Read,Bash`, `Bash(git:*)` — **cannot reach this
gate at all**, because `is_safe_agent_command` refuses `,` `(` `)` `*`. The test
looked like coverage of "the value decides" and covered one token. It asserts
`is_safe_agent_command` on every reachable case it pins now, so a value that
cannot arrive can no longer sit there looking like evidence.

**Round 1, subprocess-safety-reviewer, by running codex: `-sdanger-full-access`.**
clap lets a short option carry its value with no space, and the binary confirms
it by reporting only the *next* argument as unexpected. `LaunchToken` carries
whether a token had exactly one dash, and the mode arm tries `text[1..]` for
those.

**Its nit 5 was a live defect.** `carried_launch_is_dangerous` read only the
agent's flag table, so a session launched with `--sandbox danger-full-access` —
a mode, which the resume does carry, and which the launch screen makes a person
sign for — came back with the restart chip drawn as ordinary.

**Round 2, both independently: the guard I wrote for that could not fail.** It
compared `carried_launch_is_dangerous(agent, command)` with the expression that
*is* that function's body, and its comment called that "the property rather than
the list" — so the comment named the dead half as the durable one. Lesson 004,
written into a change about documents claiming what code does not do. It now
crosses from one side of the screen to the other: for every mode and switch a
picker offers, the chip's answer about the built command must equal the launch
screen's answer about the choice that built it, over at least twelve choices.

**And two comments of mine were making claims the code had stopped keeping.**
`resume_verb` said "no allocation beyond that hint", which was true until the
danger question started reading the command; and the note on
`carried_launch_is_dangerous` named two symptoms, of which the custom-session one
cannot happen — `carried_launch_tokens` returns nothing for a line that does not
start with the agent's binary, and a custom session is never drawn a resume verb
at all. Both corrected. The allocation is also gone: the gate splits into a
`&str` outer and a token-taking inner, so the resume path no longer joins a
`String` only to have it split again.

## A mistake made twice, in how this change was staged

This tree is shared with two other sessions and `src/tests.rs` is 20,000 lines
of it. Staging change 033 swept in two of another session's tests. Staging this
one nearly swept in 2,216 lines of theirs: the filter identified hunks by the
function name in the `@@` header, and git truncates that name, so
`the_release_gate_consults_the_preconditions_script` did not match
`the_release_gate_consults_the_precondi` and the hunk was kept.

**The rule: select hunks by the line range in the `@@` header, never by the
function name after it.** The name is git's own summary, it is cut to a width,
and a filter that depends on it fails silently in the direction of taking too
much. Recorded here rather than in `docs/sdlc/lessons.md` only because that file
is itself being edited by another session right now; the entry belongs there and
follows once the tree is not contested on it.

A `git reset` with no arguments also unstaged a third session's 22-file
staging — recoverable, and their working tree was untouched, but the lesson is
the same shape: in a shared index, every command names its own paths.

## Verification

- `cargo fmt --check` — silent.
- `cargo test --locked` — see below. Under a load average of 250-290 this tree
  produces wall-clock failures that pass in isolation: the suite went from 17 s
  to 434 s while forty sessions compiled, and `ranking_the_palette_over_a_full_
  store_stays_inside_a_frame` and one editor-diff test each failed once and
  passed alone and on four repeats. The number to record is the one from a run
  the machine was not saturated during.
- `cargo clippy --locked --all-targets -- -D warnings` — silent.
- Eight guards, eighteen mutations, each watched failing: an escape no table names
  not asked about; the net overruling a table that called a flag safe; a bare
  value matching a flag again; pre-approving a reader asking too; the shallowest
  root winning again; the refresh no longer asking whether a diff is allowed;
  the notice going back to a `format!`; and, before they were strengthened, the
  two guards above that passed their own mutation and were rewritten. From the
  reviews: the joined value list escaping the gate, `codex -sdanger-full-access`
  escaping it, `BashOutput` treated as a shell, and the restart chip reverting to
  the flag table.
