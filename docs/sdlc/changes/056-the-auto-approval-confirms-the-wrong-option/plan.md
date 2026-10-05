# Plan: read the affirmative option off the screen instead of pressing Enter

- **Spec**: `./intent.md` — the `bugfix` route in `docs/sdlc/routes.yaml` runs
  `build,test` and takes the bug report as the intent, so there is no `spec.md`.
- **Approved**: 2026-09-15
- **Status**: approved

This is the plan produced before any source file was edited. Everything it
asserts about the three CLIs was observed by launching them by hand in an empty
temporary directory on 2026-09-15, not recalled.

## What was observed, and what it decides

| CLI | option lines, in screen order | cursor | verified answer |
|---|---|---|---|
| Claude Code | `❯ No, exit` / `  Yes, I trust this folder` | on `No, exit` | `Down` then `Enter` — watched moving the `❯` to the yes line, then starting Claude with the pane alive |
| Codex | `› 1. Yes, continue` / `  2. No, quit` | on `Yes, continue` | `Enter` |
| Antigravity | `> Yes, I trust this folder` / `  No, exit` | on `Yes, I trust…` | `Enter` |

Three different cursor glyphs, two different orders, one of them numbered. No
single fixed key sequence is right for all three, which is why the answer has to
be computed from the screen.

## The decision, in one function

`auto_approve_workspace_trust_prompt` currently asks one question — "does this
look like a trust prompt?" — and answers with a key it chose when the function
was written. It will ask one question instead: **"is there a trust prompt on
screen, and which keys reach its yes?"** One pure function returns the keys or
`None`, and the polling loop does nothing but send what it returns.

Both halves are required, and the second alone would be a defect:

- The **recognition** half (`is_workspace_trust_prompt`) is what keeps this from
  answering any menu that happens to offer a "Yes". An agent asks yes/no
  questions all turn long; only the one-time workspace-trust question is
  answered without a person.
- The **option** half is what makes the answer survive a reordering.

`None` means no keystroke. Today an unrecognised layout still gets a blind
Enter; after this, a prompt Operon cannot read is left on screen for a person,
which is the safe direction for a question about trust.

A prompt caught half-drawn — body rendered, options not yet — returns `None` and
the loop keeps polling to its existing 8-second deadline rather than giving up on
the first look. That is the same mechanism that makes an unreadable prompt safe,
reused for a prompt that is merely early.

## Files that change

| File | Change |
|---|---|
| `src/tmux.rs` | `TRUST_PROMPT_YES` / `TRUST_PROMPT_NO` and `TRUST_PROMPT_CURSORS`, the option vocabularies, each entry a string one of the three CLIs was observed printing. New pure `workspace_trust_answer_keys(screen) -> Option<Vec<&'static str>>` holding the whole decision. `auto_approve_workspace_trust_prompt` sends what it returns and nothing else. |
| `src/tests.rs` | Guards over the answer, from panes captured off the three real CLIs — including Claude's with its SGR escapes intact, which is the form the existing caller passes. |
| `scripts/check-trust-prompts.sh` | New. Re-observes the three prompts on this machine and reports whether each is still answerable. Reads the vocabularies out of `src/tmux.rs` rather than restating them. |
| `docs/sdlc/lessons.md` | The entry, with its Guard column. |

## Order of work

1. Add the three captured panes as fixtures and the guards over them, and watch
   them fail — `workspace_trust_answer_keys` does not exist, so this step does
   not compile until the signature is added; the signature lands first with a
   body of `None`, which is the state in which the failure message is read.
2. Fill in the parser. Guards go green.
3. Change `auto_approve_workspace_trust_prompt` to consult it. The tree compiles
   between 2 and 3.
4. The three gates.
5. Write `scripts/check-trust-prompts.sh`, run it, and confirm it prints three
   observed prompts and no unanswerable one.
6. Mutation-verify each new guard: return a fixed `["Enter"]` and watch the
   Claude fixture go red; drop the recognition half and watch the
   not-a-trust-prompt guard go red.
7. `cargo test --locked -- --ignored` — the two Claude-destination restores.
8. `docs/sdlc/lessons.md`, `state.yaml`, then the packaged install per
   `AGENTS.md`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The captured pane carries SGR escapes inside the option text, so `no, exit` is not a contiguous substring | The parser finds no options and silently answers nothing — a session that hangs on the prompt instead of dying | The Claude fixture is stored **with** its escapes, and the parser strips them through the existing `strip_ansi_codes` rather than a second copy of that logic |
| A CLI renames its affirmative option again | The parser returns `None`, the prompt is left on screen, the person answers by hand — degraded, not broken | `scripts/check-trust-prompts.sh`, run by hand after updating a CLI |
| The vocabulary grows by guesswork, which is how this bug was written in the first place | An entry nobody has seen a CLI print | Every entry carries the date it was observed; the script re-observes them |
| A yes/no menu that is not about trust gets auto-answered | An agent's permission question answered by Operon without anybody seeing it | The recognition half is required, and `a_yes_no_menu_that_is_not_a_trust_prompt_is_never_answered` holds it |
| `Down` is not how a given CLI's list moves | The cursor does not move and Enter confirms the wrong option | Observed working on Claude Code, which is the only CLI that needs to move at all; a CLI whose yes is already selected is sent `Enter` alone |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked -- --ignored` — `6 passed; 0 failed`, where two of the six
  fail before this change with `normal claude resume did not render the restored
  … conversation` and a dead pane.
- `bash scripts/check-trust-prompts.sh` — `trust prompts: 3 observed, all
  answerable`.
- In the running app: open a Claude Code session in a worktree Claude Code has
  never run in, with 自動承認 on, and watch it reach a prompt rather than exit.

## Departures from the plan

**`strip_ansi_codes` does not exist.** The risks table says the parser strips
escapes "through the existing `strip_ansi_codes` rather than a second copy of
that logic". There is no such function at this base — the only escape scanner in
the crate is `append_terminal_line` in `src/ui/terminal.rs`, and it produces an
egui `LayoutJob` rather than text, so it cannot be reused from `src/tmux.rs`.
`strip_terminal_escapes` was written in `src/util.rs` instead. It handles OSC as
well as CSI, which the drawing scanner does not: Claude Code's pane carries an
OSC 8 hyperlink, and the drawing code therefore renders
`]8;id=…;https://…` as visible text in the terminal pane. That is a real cosmetic
defect in a painting surface, found here and **not fixed here** — it is a
separate change, and `screen: yes` in `docs/sdlc/risk.yaml` means it needs a
screen approval this one does not have.

**All three prompts were re-observed after all.** Step 5 was expected to report
one CLI observed and two not: an earlier probe reached Codex's startup screen and
Antigravity's login menu instead of a trust prompt. The cause was the probe, not
the CLIs — Codex only asks about a directory git knows about. With
`git init` in the temporary workspace all three prompts appeared, so
`CODEX_TRUST_PROMPT_PANE` and `ANTIGRAVITY_TRUST_PROMPT_PANE` are real captures
rather than the retyped approximations that had been written first, and the dates
in the vocabulary are measurements rather than citations of an earlier session.

**The script grew `--save` and `--print-vocabulary`.** `--save` writes the
observed panes with their escapes, which is what the failure message tells the
reader to do next — advice that is executable rather than a description.
`--print-vocabulary` exists so `the_trust_prompt_script_reads_the_same_vocabulary_the_app_uses`
can compare the script's awk parse against the consts on a machine with no agent
CLI installed. That test earned its place immediately: it found the cursor
extraction was matching from the `[` in `[char; 3]`, so the script's idea of a
cursor glyph was every character of `char;3]=[❯›>` — and it had still reported
all three prompts answerable.

**A defect introduced and caught inside this change.** Step 6's mutation of the
recognition half left every guard green, because all three negative fixtures were
screens with no option phrase on them. Adding a real Codex approval screen —
`rm -rf build`, offered as `1. Yes, continue` / `2. No, quit` — turned the
unmutated code red: chaining `TRUST_PROMPT_YES` into the recognition markers, to
avoid writing one phrase in two lists, had made `yes, continue` a reason to
believe a screen was about trust. The lists are separate again,
`every_trust_marker_names_trust` makes the separation structural, and lesson 021
carries the reasoning.

**One unrelated clippy lint fixed.** `cargo clippy --locked --all-targets` — the
stricter form, not the one `AGENTS.md` names — flagged `manual_contains` in a test
added by change 050 on this same branch. One word, changed here so the branch
passes both invocations.

**The review changed the shape of the fix.** `subprocess-safety-reviewer`
returned five Important findings and four Notable, and the first one was the
change's own worst defect. What it moved:

- **The decision now reads the visible screen.** It had been reading
  `tmux_capture`, which is `capture-pane -S -5000`. The same function is the
  *resume* path, so `claude --resume` replays a whole prior conversation into
  the pane before the first look — and a restored message that merely quotes a
  trust prompt, which is what pasting a transcript or a diff of `src/tmux.rs`
  into a message does, would put a marker, a yes line and a cursor into the
  scrollback of a folder trusted months ago. `Down` and `Enter` would then land
  in a live agent's composer. New `tmux_capture_visible_screen` — `-e` and no
  `-S` — is the form the fixtures are already in. `tmux_capture` had no other
  production caller left and is now `#[cfg(test)]`.
- **The script repeated the defect it was written to police.** It fell back to
  searching `TRUST_PROMPT_YES` when no marker matched, so a Codex approval
  screen offering `1. Yes, continue` would have been counted as an observed,
  answerable trust prompt. Markers only now.
- **Its cursor check asked a weaker question than the code.** It looked for a
  cursor glyph on *any* line, and Codex's real pane opens `> You are in …` — so
  the day Codex stops marking its selected option, the check reports green.
  `cursor_on_option_line` now requires exactly one *option* line to be marked.
  Watched discriminating: with `›` removed from `TRUST_PROMPT_CURSORS`, Codex
  alone went UNANSWERABLE while Claude and `agy` stayed green.
- **"No prompt found" was not a failure.** With no agent CLI installed at all
  the script printed `0 of 3 observed, all answerable` and exited 0 — lesson
  007's "a check that can only pass" in the script whose own header cites it.
  Installed-and-silent is now counted apart from not-installed and exits 1;
  watched firing by probing `sh`.
- **The probe ran on Operon's own tmux server under Operon's own name prefix**,
  so `parse_operon_tmux_sessions` returned it as an adoptable orphan pointing at
  a `mktemp -d` the script deletes seconds later. Private socket
  (`tmux -L operon-trustprobe`) and a name outside the prefix, plus a `trap` so
  a Ctrl-C in the 75-second sleep cannot leave a live agent CLI detached.
- **Lesson 021 had been inserted above lesson 020's Guard paragraph**, leaving
  020 without one and 021 with two. Moved back.
- Smaller: an option ceiling so the movement keys cannot be bounded by the pane
  height; `ESC ( B` handled, since half-stripping it leaves `(B` at the head of
  an option line and the cursor is then never found; the marker and option lists
  made fully disjoint, so the separation holds for every phrase rather than for
  every phrase but one; the vocabulary round-trip run under `/bin/bash` with
  `CLAUDE_PROJECT_DIR` removed, which is lesson 020 applied to a test written
  after lesson 020; three negative fixtures given real option pairs so each one
  fails for the reason it is filed under; cleanup in the end-to-end test moved
  above its assertion.

**A second defect found by running the script rather than reading it.** The
cursor fix passed a newline-separated list through `awk -v`, which is scanned for
escapes and rejects an embedded newline: `awk: newline in string`, three times
per CLI, every prompt reported UNANSWERABLE. It failed in the safe direction and
reported nothing true. Passed through the environment now.

**Review pass two found a defect in the fix that had nothing to do with the
bug.** The reviewer asked why the probe forced a 200×50 pane when
`start_tmux_agent_session` passes no size at all and takes tmux's
`default-size`. Following that gave the worst finding of the change:

`yes, i trust this folder` had just been removed from `TRUST_PROMPT_MARKERS`
— correctly, to make the two vocabularies disjoint — which left **Claude Code
recognised by exactly one 66-character phrase**. Measured rather than reasoned
about, by pointing the probe at narrower panes: at 80 columns it clears the edge
by a single word; at 60 it breaks after `one` and the prompt is not recognised
at all, which `bash scripts/check-trust-prompts.sh` reported as
`claude NO TRUST PROMPT within 25s`. Automatic approval off, no symptom, on a
prompt filling the screen.

`holds_a_trust_question` now collapses whitespace across line ends before
searching, because a marker is a sentence and a sentence wraps. The option half
deliberately does not: an option that wrapped is one the app would not find
either, and a green there would be a lie about that.
`CLAUDE_TRUST_PROMPT_PANE_WRAPPED` is the real 60-column pane, and the probe
passes no `-x`/`-y` at all now, so it inherits the same default the app does.

The rest of pass two:

- **The visible-screen guard pinned the call site's name, not the capture.**
  Adding `-S -100` to `tmux_capture_visible_screen` — the obvious response to
  "the prompt is above a short pane" — would have restored the whole scrollback
  finding with the guard green. It now slices that function out of the source
  too and asserts `capture-pane -e -p` with no `-S`.
- **`mktemp` failure was the last door left open** on "a check that can only
  pass": `installed` was already incremented and neither `observed` nor `silent`
  was, so an unwritable `TMPDIR` produced an empty report under a summary saying
  everything was fine. Fixed, and backed by an end-of-run invariant —
  `observed + silent == installed` — which catches the next such door without
  anyone naming it first.
- **The probe did not run tmux the way the app does.** `probe_tmux` now passes
  `-u` and the same locale defaults as `tmux_command()`; without them a pane
  started from a shell with no locale hands back `❯` and `›` mangled and the
  script reports unanswerable on a tree where the app is fine.
- **`no_trust_marker_is_also_an_option` tested equality where the property is
  containment.** Narrowing the inherited `do you trust this folder` row to
  `i trust this folder` passed every guard while making any menu offering
  `Yes, I trust this folder` a trust prompt by its option alone.
- **The `"Up"` branch was reached by no test.** Mutating it to `"Down"` left the
  suite green, in the branch added to stop the original bug.
- Nits: the socket carries `$$`, since `cleanup` ends in `kill-server` and two
  concurrent runs would kill each other's probes; `trap` extended to `HUP` and
  `EXIT`; `ESC % G`, `ESC # 8` and `ESC SP F` join `ESC ( B` as three-byte
  escapes, since they leak their second byte at the head of a line, which is the
  one position where a leak costs a cursor match; the Antigravity login fixture's
  doc no longer claims to prove something it does not; `state.yaml`'s contract
  key renamed off the word "answerable", which the script no longer claims.

## Proof, as it came out

- `cargo fmt --check` — no output.
- `cargo test --locked` — `477 passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` and `--all-targets` — no output.
- `bash scripts/check-trust-prompts.sh` — `trust prompts: 3 of 3 installed CLIs
  observed, all still worded as src/tmux.rs expects`.
- `cargo test --locked -- --ignored` — **3 passed, 3 failed**, not the `6 passed`
  the plan predicted. `a_saved_codex_conversation_restores_into_a_resumable_claude_terminal`
  is the one this change was expected to fix, and it passes. The three failures
  are not this change's: two are
  `assert_eq!(imported.imported_records, 4)` returning `5` on an Antigravity
  source, which fails inside `import_full_cli_history` before any terminal is
  opened, and the third is Codex refusing to resume its own rollout file
  (`final paginated rollout record … is missing an ordinal (code -32603)`).
  Nothing outside `src/tmux.rs`, `src/util.rs`, and `src/tests.rs` is touched by
  this diff, and no file on the import path is among them.
- `cargo test --locked` again, on the staged tree — `456 passed; 22 failed`, and
  the diff had not changed. `GIT_DIR` and `GIT_WORK_TREE` are exported into the
  session's shell, every `git` a test spawns inherits them, and the 22 tests that
  build a temporary repository were running against the real one:
  `lists_main_and_linked_git_worktrees` re-initialised
  `.git/worktrees/trust-prompt-and-readiness-gate` instead of its own tempdir.
  They had been passing until this change was staged without a `review.yaml`, at
  which point the repository's own `reference-transaction` hook started refusing
  their commits — the review gate's refusal, quoted verbatim inside a unit test's
  failure. `unset GIT_DIR GIT_WORK_TREE` first and the same tree reads
  `478 passed; 0 failed; 6 ignored`. Recorded in `state.yaml`'s resume as a
  finding this change made and did not fix; the fix belongs where the temporary
  repositories are built.

## The departure that was not a choice: a child inheriting a repository

`cargo test --locked` was run on the staged tree and read `456 passed;
22 failed` with the diff unchanged from the run that had read
`478 passed; 0 failed`. The cause was not in the diff. `GIT_DIR` and
`GIT_WORK_TREE` are exported into a Claude Code worktree session, every
`Command` inherits the environment, and **`current_dir` does not decide which
repository git works on** — those two do, and they win. So the tests that build
a repository under `mktemp` and run git in it were running git against this
checkout.

It had already done damage, silently, while the suite was green: one of them
committed this change's own staged files onto the branch as `initial`, author
`Operon test <test@example.invalid>`. Recovered with `git reset --soft`. The
only reason it was found is that the *next* run went red, and it went red
because staging the change without a `review.yaml` made the repository's
`reference-transaction` hook start refusing those commits — the review gate's
refusal, quoted inside a unit test's failure message.

That also made this change unlandable: `.claude/hooks/gate-commit.sh` runs
`cargo test` before the commit, and that run would take the staged work again.
So the fix is in this change rather than after it, and it is production code and
not only test code, because the same reasoning applies to Operon: launched from a
terminal that exports either variable, every git call it makes is aimed
somewhere other than the path passed beside it.

Four layers, one list:

- `INHERITED_REPOSITORY_POINTERS` in `src/config.rs` names the two variables
  once, per `.claude/rules/identifiers.md`.
- `run_command_with_output_limit` in `src/exec.rs` drops them. Every external
  tool the crate runs arrives there — `run_command_with_timeout` delegates to it
  — so one edit covers git, tmux, gh, bash and the agent CLIs.
- `git_command` in `src/git.rs` drops them too, for the commands that never
  reach `src/exec.rs`: a `Command` handed straight to `.status()` or `.output()`,
  which is how the tests build theirs. The nine raw `Command::new("git")` sites
  in `src/git.rs`, `src/app.rs` and `src/tests.rs` now go through it.
- `CodexAppServer::start` in `src/history.rs` drops them itself. It is the one
  production child that cannot use `src/exec.rs`: long-lived and spoken to over
  stdio, where that function pipes and reads to completion. Found by asking the
  question that was put to the reviewer — does every child actually arrive at
  the choke point — and answering it rather than waiting: one `.spawn()` in
  production sits outside `src/exec.rs`. Codex reads git state for its own
  context, so an inherited `GIT_DIR` would have pointed it at a repository other
  than the one being restored into.

Measured at each layer, because the first two attempts each fixed some of it:
22 failures, then 18 after the test sites were routed through the builder, then
12 after `src/app.rs` and `src/git.rs`, then 0 once `src/exec.rs` covered
`command_output_in_dir` — a second way git was spawned that the builder could
not see. `479 passed; 0 failed; 6 ignored`, with both variables still exported.

`no_git_command_in_the_crate_can_inherit_the_session_repository` is the guard.
It walks `src/` and demands exactly one raw git command in the crate; demands
that no production file starting a child lacks the drop; runs a child through
`run_command_with_output_limit` with both variables set and asserts the child
saw neither, taking the names from the constant rather than retyping them; and
reads `git_command` to confirm it consults the constant instead of keeping a
second copy.

The sixth mutation is the review's one nit, fixed rather than recorded. The
`//` filter is only complete while production code has no block comments: a
`/* … in INHERITED_REPOSITORY_POINTERS … */` would satisfy the spawn check
without dropping anything — the mention hole again, through the other comment
syntax, and in the unsafe direction. Closed by an invariant and not by a
stripper: production files may not contain `/*` at all. A stripper was the
obvious fix and would have been wrong, because the only three `/*` in this crate
are inside string literals in the syntax-highlighting fixtures
(`"/* CSS */\n"`), and removing everything between `/*` and `*/` would have
eaten the code between two of them. Watched: a block comment added above the
`CodexAppServer::start` loop printed `["history.rs"]`.

Five more mutations watched, and the fifth is the one worth recording because
the guard was wrong. Removing the `env_remove` loop from `CodexAppServer::start`
left it **green**: the check was `code.contains("INHERITED_REPOSITORY_POINTERS")`
and the comment above the deleted loop still said the word. It passed on a
mention — the same hole review found in change 054's wiring test, reproduced by
the session that had just read about it. Fixed by dropping `//` lines before
anything is counted and by matching `in INHERITED_REPOSITORY_POINTERS`, the code
shape rather than the name; the mutation then printed
`["history.rs"]`. The other four: removing the loop from `src/exec.rs`
(`GIT_DIR を継承しました`), giving `git_command` its own literal pair
(`INHERITED_REPOSITORY_POINTERS を読んでいません`), adding a tenth raw command
(`[("git.rs", 2)]`), and the guard's own first draft, which sat above the builder
it read and matched its own copy of the opening line — it failed against correct
code, and moving it below fixed it.

## What the probe script does not check, and why

`scripts/check-trust-prompts.sh` captures two different ways, and the difference
is deliberate rather than an oversight. Line 345, the check, is
`capture-pane -p` with no `-e`: the question it answers is whether the CLI still
words its prompt the way `src/tmux.rs` expects, and tmux's plain capture is the
rendered text with the escapes already resolved. Line 362, `--save`, is
`capture-pane -e -p`, because a fixture has to carry the escapes — a pane with
its colour removed would not exercise `strip_terminal_escapes` at all, and the
SGR run inside Claude Code's `❯ No, exit` is the case that made the stripper
necessary.

What this gives up: the script measures the wording in a form the app never
sees. `strip_terminal_escapes` deletes escape bytes, while tmux's plain capture
resolves them — for a CLI that drew an option with cursor-positioning escapes
rather than colour, the two would disagree and the script would report a wording
match for a line the app reads differently. None of the three CLIs draws that
way, and the fixtures in `src/tests.rs` are real `-e` captures, so the app's path
is covered by the guards rather than by the probe. Written down because the
third reviewer read line 345 and reported the script as capturing without
escapes throughout, which is the reading the file invites.
