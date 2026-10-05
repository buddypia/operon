# Lessons

Stage 6 of the pipeline in `docs/sdlc/README.md`. One entry per mistake that
reached the tree, newest last.

The part that matters is **Guard**. An entry with no guard is unfinished work:
the lesson was learned by a person and not by the repository, so the next agent
will make the same mistake. Every guard is a test in `src/tests.rs` or an eval
in `evals/`, because those are the two things that run without being remembered.

Two forms live here, and the difference is deliberate. A recent entry carries
the full account — what happened, why it was invisible, what now catches it —
because its guard is young and the reasoning is the part that persuades the next
agent not to repeat it. Entries 001 to 041 are compressed to the rule and the
guard: their guards have been green for weeks or longer and the rules they
produced now live in `CLAUDE.md`, in `.claude/rules/`, or in a test that runs
without anyone remembering it. Change 030 is where that cut was decided, what it was measured
against, and what it traded away; change 097 moved the line to 041 when the
band fired on lessons again.

**No guard was removed by the compression**, and that is checkable rather than
asserted: `scripts/pipeline-indicators.sh` with `--lessons` still reports every
entry naming a guard this repository holds.

---

## 001 — One identifier, five literals, one gate

`MANAGED_TMUX_PREFIX` was written inline at five call sites, so the product
rename moved the gate that recognises a managed session and none of the five:
the app silently refused every session it had itself created. The suite stayed
green because it only ever handed the gate a literal it had typed itself.
**The rule:** build the value through the constant, and test the round trip —
the value the app produces, put through the check that consumes it.

**Guard.** `the_session_prefix_is_written_in_exactly_one_place` and
`evals/003-identifier-ssot.md`.

---

## 002 — A document that outlived the file it described

`CLAUDE.md` described a single 21,700-line `src/main.rs` for as long as the
crate had been nineteen modules behind a 246-line entry point. No compiler
error, no lint and no test notices when prose about code goes stale.
**The rule:** the module map is checked in both directions — a module missing
from it sends the next agent to the wrong file, and an entry with no module
behind it sends it to a file that is gone.

**Guard.** `claude_md_maps_every_module_that_exists`,
`harness_documents_only_name_paths_that_exist`, and `evals/004-module-map.md`.

---

## 003 — A doc comment naming a script that was never committed

`src/i18n_tables.rs` opened by telling every reader to regenerate the tables
with a script, and neither that script nor its TSV exists in this repository —
the tables are hand-edited. Those two names are deliberately not in backticks,
here or in the test's own doc comment: the check that caught them reads every
backticked repository path and requires it to exist, so a document quoting a
dead path would fail the rule it is recording. The test written for exactly this
failure read only the root documents and `.claude/**/*.md`. **The rule:** a
module's `//!` header is read before the code under it, so it is checked like a
document.

**Guard.** `harness_documents_only_name_paths_that_exist`, extended to source
module docs.

---

## 004 — A guard blind to the rename it existed to survive

The guard written *because of* lesson 001 searched for the hardcoded prefix
strings themselves, so renaming the constant would have left it looking for a
value nothing used any more — passing, having checked nothing, on exactly the
change that caused lesson 001. A green guard for the right failure class reads
as coverage, and nobody re-derives what a passing test checked. **The rule:** a
guard reads the value it guards out of the definition rather than restating it.

**Guard.** `the_session_prefix_is_written_in_exactly_one_place`, now derived
from the constants, plus `evals/003-identifier-ssot.md`, whose check does not
rely on that test alone.

---

## 005 — A widget written for the bug, and never called

`restore_progress_footer` had been written to stop the restore modal painting
its button on top of its own sentence, with a test pinning the two rects apart
at two widths — and the modal never called it. The widget sat behind
`#[allow(dead_code)]` for the whole time the overlap was shipping, so the one
lint that would have said "nothing calls this" had been silenced at the
definition. **The rule:** a widget test proves a widget, and only the caller
proves the screen, so the guard asserts the call rather than the geometry.

**Guard.** `the_restore_modal_draws_its_footer_through_the_shared_widget`,
beside the older `the_restore_footer_never_paints_status_under_its_action`.

---

## 006 — A translation table whose order was a comment

`translation_for` looks a message id up with `binary_search_by`, and nothing
checked that the tables were sorted, that every id had a row in every language,
or that a translation kept the `{name}` holes its id declares. All three fail
silently: an unsorted row is never found, a missing row draws Japanese into an
English screen, and a renamed placeholder prints its own braces mid-sentence.
The module doc said the tests existed; they did not — lesson 003's shape.
**The rule:** every property a lookup depends on is asserted, and `catalog_keys`
is the union of every table rather than one of them, so an id that reached only
the Korean table is caught too.

**Guard.** `every_translation_table_is_sorted_by_message_id`,
`every_message_id_has_a_row_in_every_table`,
`every_translation_keeps_the_placeholder_names_of_its_message_id`, and
`an_unknown_message_id_falls_back_to_the_japanese_it_was_written_in`.

---

## 007 — Two ways to say no, and the readers only had one

A full-history restore carried 471 of 1,165 records out of a real Claude Code
conversation. The three readers in `src/history.rs` answered "is this
conversation?" with `Option`, so a record type excluded on purpose and a record
type nobody had ever seen both arrived as `None`: every `thinking` block Claude
Code wrote was dropped for want of a `text` field, Codex's `reasoning` items
fell off the end of a `match`, and Antigravity's tool results were never read.
The fixtures were JSONL the tests had typed themselves, so they could only
contain record types somebody had already thought of, and the notice said the
restore was complete over a number with no denominator. **The rule:** parsing,
classification and projection are separated; every record, payload, role, block,
flag and wrapper the three CLIs write has a row in `TRANSCRIPT_VOCABULARY`
(`src/transcript.rs`) giving its class and the reason, seeded from real local
transcripts rather than from documentation; and a kind with no row is a fourth
outcome — carried across labelled and named in the notice — not silence.
Reasoning crosses as attributed text, never as native reasoning, because both
vendors sign their reasoning and a forged one fails inside the destination long
after the restore reports success. The same conversation now restores 532 turns
and accounts for all 1,165 records.

**Guard.** `restore_counts_every_source_record_into_exactly_one_class` (the
record arithmetic must balance), `an_unclassified_record_is_restored_and_reported`,
`every_operational_wrapper_is_stripped_by_the_reader_that_declares_it`,
`reasoning_reaches_every_destination_as_attributed_text`, and
`the_transcript_vocabulary_is_well_formed`.
`scripts/check-transcript-vocabulary.sh` sweeps the local stores for kinds the
table has not met yet; it is not a band, because CI has no conversation files
and a check that can only pass is not a check.

---

## 008 — A restore reproduced the conversation the terminal had left

A full-history restore out of a managed Claude terminal carried two turns of a
conversation that holds ten. It was not lossy: it faithfully reproduced a
*different* conversation. Operon stores the launch `--session-id` as the
terminal's conversation for ever, and `/clear` starts a new one in a new file,
so the stored id went on naming the stub the terminal opened with. The identity
check asked one direction only — could this transcript have begun before the
terminal existed? — and a terminal that has moved *forward* passes it. Neither
ranking could have worked: two conversations that both begin after a terminal
cannot be ordered by when they began, and the abandoned one begins closer. 5 of
the 25 live Claude processes on the machine had already rotated. **The rule:**
stop inferring. Claude Code publishes one entry per live process naming the
conversation that process is on now; `registered_claude_conversation_in` reads
it and outranks the stored id and everything derived from timestamps, two
entries that cannot be told apart is a refusal, and an absent registry leaves
the previous behaviour exactly as it was. The first cut of the guards tested the
registry *lookup*, so deleting the consult from the decision that uses it left
all of them green — lesson 004 one module over, cover the check and miss the
caller — and they were rewritten to run through the whole decision and backed by
mutation rather than by assertion.

**Guard.** `a_cleared_claude_terminal_restores_the_conversation_it_moved_to`,
`a_resumed_claude_terminal_still_follows_its_registry`,
`a_registry_confirmation_is_not_overruled_by_the_start_time_check`,
`a_registry_entry_belonging_to_another_terminal_is_not_read_as_this_one`,
`two_claude_terminals_started_together_are_told_apart_or_refused`, and
`the_claude_store_directories_are_spelled_in_exactly_one_place`, whose literals
are built from the constants so that it survives the next rename.

---

## 009 — A test for a ceiling that never reached the ceiling

The word-level diff falls back to a linear trim past `DIFF_WORD_TOKEN_LIMIT`,
and its test built a line of exactly that many tokens with one word changed in
the middle — passing on the first run. The trim runs *before* the ceiling is
consulted, so it collapsed that line to a one-token middle and the branch under
test was never entered; a build with no ceiling at all would have passed
identically. The test named the ceiling, used the constant, and constructed an
input that looked enormous, so every signal a reader has said it was covered.
**The rule:** an input has to make the branch reachable, not merely large, and
the test pins the two paths apart so neither can be deleted quietly.

**Guard.** `a_line_past_the_word_comparison_ceiling_still_marks_its_middle`,
which fails when the ceiling branch is removed and, through its second case,
when the full comparison is removed. Verified by mutation rather than asserted.

---

## 010 — Steering that had nowhere conditional to live

Four policies — the colour system, the icon vocabulary, the transcript
vocabulary, and the identifier rule — sat in `CLAUDE.md`, which loads on every
turn, so an agent asked to fix a translation string paid 4,527 bytes for the
contrast rule and the glyph rule before it read a line of code. The band was
firing and being read as a warning about *volume* when it was a signal about
*placement*: nothing was wrong with the four policies, and the only two homes
knowledge had were "every turn" and "when a task is recognised". **The rule:**
`.claude/rules/`, loaded per-file from a `paths:` frontmatter list, is the home
for "when this file is open", and `conditional_steering_bytes` stays a separate
metric because folding it into `steering_bytes` would collapse the distinction
the mechanism exists to create. Two traps came with the cure and are the
interesting part: a rule with no `paths:` loads unconditionally, with no error
and no visible difference, while the metric goes on printing a number that is no
longer about anything — lesson 002's shape applied to a metric — and a path rule
fires when Claude *reads* a matching file, not when it writes one, so a
brand-new file is uncovered. That second one would have been unacceptable if the
rules were the enforcement; they are not, each of the four keeps its test, and
saying so out loud is what made the move safe rather than lucky.

**Guard.** `every_rule_declares_the_paths_it_applies_to` (a rule with no
`paths:`, an empty pattern list, or a pattern whose non-glob prefix does not
exist, is a rule wired to nothing), plus
`harness_documents_only_name_paths_that_exist` extended to walk `.claude/rules`,
and the `conditional_steering_bytes` and `rules` bands in
`docs/sdlc/bands.yaml`. The read-not-write gap cannot be asserted, so it is a
prompt: `evals/006-new-file-past-the-path-rule.md` asks for a brand-new drawing
file and checks whether the pointer left in `CLAUDE.md` was enough.

---

## 011 — A gate whose test passed with the gate switched off

`the_readiness_gate_refuses_a_spec_that_is_not_filled_in` copied a complete
change, gutted one flagged concern, and asserted the script returned No-Go. It
did. Then the flagged-concern check inside the script was deleted as a mutation,
and **the test still passed**: the gutted stub was built from the template's own
placeholder, whose angle brackets tripped the *placeholder* check and produced
the No-Go on their own. Every signal a reader has said it was covered — it ran
the real script rather than mocking it, used a real committed change as the
fixture, asserted both directions, and was green. **The rule, now three lessons
deep:** a guard must fail for the reason it names, and a test that only asserts
"it failed" has not established that. When a gate has several rejection paths,
the assertion belongs on the path, not on the verdict.

**Guard.** `the_readiness_gate_refuses_a_spec_that_is_not_filled_in`, which
asserts on the report text and was watched failing with the flagged-concern
check disabled and passing with the placeholder check disabled — both runs
performed, not reasoned about. Beside it
`every_state_file_names_a_route_and_a_stage_that_exist`,
`every_risk_surface_names_paths_that_exist`, and
`every_reference_the_skill_names_exists`.

---

## 012 — An orchestration mechanism with nothing wired to enforce it

Change 008 introduced an SDLC orchestrator — route table,
risk table, state file, readiness script, completion contract. Every artifact
was well-formed, seven new guards were green, each had been watched failing by
mutation, and the change reported itself done. Asked what had actually been
built, the audit found `scripts/check-readiness.sh` called by nothing but its
own test, `docs/sdlc/risk.yaml` read by no hook at all, and nothing requiring a
change to have a `state.yaml`: of the nine mechanisms claimed, two could refuse
anything. The supporting enforcement hooks had initially been deferred under
the assumption that artifact generation alone would suffice — false, because
hooks are the enforcement layer, the only part that does
not depend on the agent choosing to comply. Every guard checked that a mechanism
was *well-formed*; not one checked that it was *reachable*, and the mutation
discipline could not have caught it, because every mutation deleted part of a
mechanism and watched a guard fail. **The rule:** a check for the presence of a
string is not a check for the thing the string stands for. Its most expensive
form is this one, where the string was present in a document and the mechanism
was absent from the machine — and the first cut of the guard written to catch it
counted a hook's *mention* of a script as wiring, so the guard written to catch
lesson 011 contained lesson 011.

**Guard.** `every_gate_script_is_wired_or_declares_why_not` (watched failing
with the invocation removed, after being fixed to ignore comments — the first
version was watched *passing*, which is why the fix exists),
`every_change_directory_carries_its_state`, `.claude/hooks/guard-stage.sh`
itself, exercised directly on all three of its refusals plus the quiet case, and
`every_hook_the_settings_file_wires_exists_and_is_executable`.

---

## 013 — Every piece passed, and nothing was ever posted

Nine green tests covered the hook that has the agent CLIs report their own
state, and run against the real thing it posted nothing. The endpoint file wrote
the socket path unquoted, `.` on an unquoted assignment stops at the first
space, the variable was set to the first half of the path, the rest ran as a
command, the failure went to `/dev/null` because a status hook must never
disturb its CLI, and the script exited 0 having done nothing. On macOS the data
directory is always under a path with a space in it, so this failed for every
user, always — and every test used a temporary directory, which has no space in
its name. The endpoint test even sourced the file with a real `/bin/sh`: the
right test, run against the wrong path. **The rule:** a fixture is a claim about
production, and a temporary directory claims that paths are simple. When a value
crosses a boundary this repository does not own — a shell, another product's
settings file, a socket — at least one test has to send the value production
sends, not a convenient one.

**Guard.** `an_installed_hook_command_posts_a_reading_end_to_end` (watched
failing with the quoting removed — the mutation reproduces the shipped bug
exactly) and `the_endpoint_file_sources_into_the_variable_the_script_posts_with`,
now under a path with a space and asserting that it has one.

---

## 014 — A guard that named the file instead of the thing

`the_brand_is_drawn_as_artwork_and_not_from_the_glyph_vocabulary` opened
`src/app.rs` by name and counted call sites in it. Its subject is where Operon's
own mark comes from; the file was merely where the drawing happened to live, so
moving the drawing failed the guard for a reason that had nothing to do with the
brand. It is lesson 004 one level up — 004 restated the value it guarded, this
one restated the *location*. The rewrite written first, counting across `src/`,
passes and survives the move and is *weaker*: a third site anywhere keeps the
total at two while one of the required surfaces goes back to text, which the old
form could not be fooled by. It was caught in review. **The rule:** a guard
names what it is about, never where that thing currently is. If a guard would
have to be edited by a move that changes no behaviour, it is watching the wrong
noun.

**Guard.** `the_brand_is_drawn_as_artwork_and_not_from_the_glyph_vocabulary`
itself, now asking about two named functions and finding each wherever it is
defined. Watched failing twice: once with the first-run page reverted to text,
and once with the top bar reverted *while a third call site exists* — the case
the tree-wide count would have passed, and the reason that form was thrown away.

---

## 015 — The same lesson, a third time: a shell script read is not a shell script run

The usage reader throttles itself with a stamp file, written with `printf '%s'`
— no trailing newline — and read back with `read -r last < "$stamp" || last=0`.
`read` returns non-zero at end of file **even when it has already assigned the
value**, so `last` went back to `0` on every run, the interval check never
fired, and a status line that ticks three times a second posted three times a
second. Six unit tests passed: the script parsed under `sh -n`, printed nothing,
exited quietly with no environment, and dropped a payload with no limits in it.
None of them ran it twice. **The rule:** a generated shell script is not tested
by asserting things about its text. It is tested by running it, more than once,
with the environment and the paths production gives it. Entry 013 said the same
about one value; this says it about control flow. The fix has two halves — a
trailing newline, and judging the value rather than `read`'s exit status — and
each alone is sufficient, so neither mutation fails on its own and restoring
both is the mutation recorded.

**Guard.** `the_installed_usage_reader_posts_a_reading_end_to_end`, watched
failing with both halves of the throttle restored to the form that shipped
nothing.

---

## 016 — A flag over the frame, answering a question about one event

Stopping the Enter that confirms an IME conversion from also reaching the agent
was done with one boolean, true when the frame's event list held any non-empty
`Ime::Commit`, and every `Key::Enter` in that frame was then dropped. Pressing
Enter twice is how a sentence in Japanese ends, both presses can arrive in one
frame, and when they did the second was dropped and the line sat unsent. The
symptom is intermittent by construction — two Enters inside one 16 ms frame,
only while typing Japanese — and invisible to the tests written for it, each of
which covers one case alone, because the flag being tested has no notion of
"two". **The rule:** when a decision is about one event, derive it from the
event stream in order, not from a summary of the frame; `.any()` over
`input.events` is the shape to look at twice. Counting the commits and spending
the count is also what makes it hold whichever way round the platform delivers
the pair, which is a thing this repository cannot observe from inside a test.

**Guard.** `a_second_enter_in_the_frame_that_committed_a_conversion_still_sends`,
which puts two Enters and one commit in one event list in both orders, two
commits and three Enters in another, and the original single pair last. Watched
failing with the count restored to a flag, and again with the absorption removed
altogether.

---

## 017 — A gate that went vacuous when its inputs were deleted

Change 012 removed a launch-option editor nothing could reach, and was right to.
It left the other end of the wire: the fields stayed on the app and
`launch_needs_acknowledgement` kept deciding from them whether a person had to
tick 「理解したうえで起動する」 before an agent could run without asking
permission — and nothing could fill them. For four changes the warning never
appeared, and the one route that could still put a dangerous switch in front of
tmux, typing it into 起動コマンド, was the one route the gate never read. Every
test of the gate passed and each was correct; none asked where a mode or a flag
list comes from. This is lesson 005 with the arrow reversed, and this direction
is worse: a dead widget is a missing feature, a dead gate is a safety property
the repository still believes it has. **The rule:** when a change deletes the
only producer of a value, its consumers are part of the change. A check whose
inputs nothing can produce does not fail — it answers no, forever, and no test
notices. The gate now asks about the command that will run rather than which
picker was used.

**Guard.** `a_dangerous_switch_typed_into_a_custom_command_still_asks` drives
the gate from the flag tables, and
`a_dangerous_switch_cannot_be_launched_without_the_acknowledgement` runs the
real launch screen and asserts the warning and the checkbox are among the labels
it paints — the assertion that was missing for four changes. Watched failing
with the command clause removed, with the flag clause removed, and with the call
site removed.

---

## 018 — A match that read as a miss, reconstructed from the only copy left

**Provenance.** This entry is written from the two files that cite it, not from
the session that learned it. Both said "Entry 018 in `docs/sdlc/lessons.md`"
against the same rule, and `lessons.md` had no entry 018: the account was
written on one machine and never committed — the defect entry 019 below is
about, one file over. The rule and its cure are quoted from the code that states
them; the history is gone and is not invented here.

Under `set -uo pipefail`, `printf … | grep -q` closes the pipe as soon as it
matches. If the writer upstream is still writing it is handed SIGPIPE,
`pipefail` takes the pipeline's status from it, and **a match reads as a miss**
— in `.githooks/reference-transaction`, an already-vouched commit would be
judged again. The word "can" is load-bearing: a short input fits the pipe buffer
and the pipeline succeeds, so the failure is intermittent, grows likelier as the
input grows, and a two-line experiment on a fresh repository *disproves* it.
**The rule:** under `pipefail`, do not end a pipeline with a reader that stops
early — `grep -q`, `head`, and `sed -n '1p'` all close the pipe on a hit.
Capture whole, then search.

**Guard.** `every_lesson_cited_by_number_is_a_lesson_that_exists` is what caught
the dangling citation and what keeps this number pointing here. The rule itself
is held only by the comment at `.githooks/reference-transaction:155` — there is
no test for it, and saying so is the honest state of this entry.

---

## 019 — The review gate ran from a file the repository did not have

`scripts/check-review.sh`, the stage-5 gate `AGENTS.md` names, had never been
committed, nor had `scripts/pipeline-indicators.sh` or `.githooks/`: they sat
untracked in one checkout, and `.githooks/reference-transaction` fell back to
the main checkout's working file, so every check ran where the files were and a
fresh worktree was red with no gate. **The rule:** "it works here" is not a
property of a repository — ask what a fresh clone gets, and when the answer is
"less than this machine", the gap is the finding. Once `main` carries the gate,
a broken one refuses every ref update including its own fix, and `--no-verify`
does not bypass a `reference-transaction` hook: the way out is to delete
`$(git rev-parse --git-path hooks)/reference-transaction` by hand.

**Guard.** `harness_documents_only_name_paths_that_exist` fails on a path no
checkout has, which is what caught all three; watched failing again with
`scripts/check-review.sh` moved out of the tree.
`the_gitignore_exemption_covers_runtime_state_and_nothing_tracked` holds the new
exemption narrow — watched failing, with `src/` appended to `.gitignore`, on
`src/app.rs`. `every_lesson_cited_by_number_is_a_lesson_that_exists` catches the
fourth instance of the same defect, a lesson cited by two files and never
written; watched failing on entry 018 before it was reconstructed.
`every_gate_script_is_wired_or_declares_why_not` still refuses
`scripts/check-review.sh` as unwired, and the `not-wired:` line in its header is
where the gap above is recorded.

---

## 020 — A guard that reported on the machine it ran on

`scripts/check-readiness.sh` counted findings with a bash 4 nameref under
`#!/usr/bin/env bash`; stock macOS resolves that to 3.2, which errors, counts
`0`, and prints a blocking finding followed by `readiness: Go`. Its test ran
`bash` from `PATH`, so a Homebrew install turned it green without touching the
gate, and the sweep that followed found `.claude/hooks/gate-stop.sh` did not
even parse under 3.2. **The rule:** a guard over a shell script names the
interpreter, and names the oldest one the shebang can resolve to; a test that
lets `PATH` choose is measuring the laptop it ran on.

**Guard.** `the_readiness_gate_refuses_a_spec_that_is_not_filled_in` runs
`scripts/check-readiness.sh` under `/bin/bash` *and* under a newer `bash` from
`PATH` when the machine has one, failing if the two disagree. Watched failing:
with the interpreter switched and the nameref still in place it printed the
blocking finding and asserted `left: 0, right: 1`; green after the fix.
`every_shell_script_parses_under_the_oldest_shell_its_shebang_finds` walks
`scripts/`, `.claude/hooks/` and `.githooks/` — recursively, because
`.githooks/installed/reference-transaction` is the installed copy of the hook
entries 018 and 019 are both about and a one-level read left exactly it out —
and asks each script's own interpreter to parse it. `#!/bin/sh` files are swept
too, with `/bin/sh -n`, because on macOS `/bin/sh` **is** bash 3.2 and
reproduces the identical failure: skipping them would have left the cheapest way
out of the exception list, which is to rewrite one shebang.

`a_shebang_carrying_flags_still_names_its_interpreter` holds how a `#!` line's
interpreter is read, since a script skipped by a wrong cut passes silently.
`KNOWN_UNPARSED_UNDER_BASH_3_2` holds the one exception and is policed three
ways: an unlisted broken script goes red, a listed one that starts parsing goes
red, and `every_known_unparsed_script_is_a_script_the_sweep_still_reaches` goes
red if a listed script leaves the swept set.

---

## 021 — Pressing a key without reading the screen it lands on

`auto_approve_workspace_trust_prompt` answered every CLI's trust prompt with a
blind `Enter`, which on Claude Code confirms `No, exit`. The suite checked that
the prompt was recognised and that something reached the pane, never which
option the key landed on; its typed fixtures lacked a real pane's escapes, and
the decision read scrollback, where a quoted prompt sends keys to a live agent.
**The rule:** a negative fixture must fail for the branch it is filed under;
the vocabulary that decides *whether* to act is not the one that decides *how*;
a keypress reads the screen it lands on and nothing else; and a fixture that
arranges what the product cannot is measuring something else.

**Guard.** `the_claude_trust_prompt_is_answered_by_moving_to_its_affirmative_option`
and `a_trust_prompt_already_resting_on_yes_is_only_confirmed` over real captured
panes; `a_menu_that_is_not_a_trust_prompt_is_never_answered` including the Codex
approval screen; `every_trust_marker_names_trust`, which makes the separation
structural rather than remembered; `a_trust_prompt_caught_before_its_options_are_drawn_is_left_alone`;
`the_escape_stripper_leaves_the_text_a_terminal_would_show`;
`auto_approval_sends_enter_only_after_a_workspace_trust_prompt`, rewritten to
read back the bytes that reached the pane through `od`;
`the_trust_prompt_script_reads_the_same_vocabulary_the_app_uses`, which fails when
`scripts/check-trust-prompts.sh` stops parsing `src/tmux.rs` — it already has, on
its first cut;
`the_trust_prompt_is_read_off_the_visible_screen_and_not_the_scrollback`, which
reads the capture out of the function rather than asking it to behave, because no
behaviour distinguishes the two captures without a live agent;
`no_trust_marker_is_also_an_option`, which tests containment rather than
equality because a marker that is a *substring* of an option satisfies
recognition from the option line alone;
`a_screen_offering_more_options_than_a_trust_prompt_has_is_left_alone`, which is
also the only coverage of a movement distance greater than one and of the `Up`
branch; and `a_trust_question_that_wrapped_is_still_a_trust_question`, over the
real 60-column pane. Eighteen mutations watched, each of what the guard beside
it claims.

---

## 022 — The working directory was not what decided which repository git touched

`GIT_DIR` and `GIT_WORK_TREE`, which Claude Code exports for a worktree
session, win over `current_dir`, so twenty-two tests that built a repository
under `mktemp` ran git against this checkout — one committed staged files onto
the branch as `initial` while the suite stayed green — and Operon launched from
such a terminal aimed git elsewhere too. **The rule:** a test that runs a tool
in a world it built controls the tool's whole environment; the fix belongs at
the choke point, `run_command_with_output_limit` in `src/exec.rs`, dropping
`INHERITED_REPOSITORY_POINTERS`; and a source-reading guard reads code, not a
comment that mentions it.

**Guard.** `no_git_command_in_the_crate_can_inherit_the_session_repository`,
which does three things and needs all three: it walks `src/` and demands exactly
one raw git `Command` in the crate; it runs a child through
`run_command_with_output_limit` with both variables set and asserts the child
saw neither — behaviour, not source, at the choke point — taking the names out
of `INHERITED_REPOSITORY_POINTERS` rather than retyping them, which is lesson
004's rule; and it reads `git_command` to confirm it consults that constant
instead of keeping a second copy, because a list in two places is a list where
one copy gets fixed. Three mutations watched: removing the loop from
`src/exec.rs`, giving `git_command` its own literal pair, and adding a tenth raw
command. A fourth was watched on the guard's own first draft, which sat above the
function whose body it read and matched its own copy of the opening line, then
read to the `'}'` on its next line and failed against correct code — a guard
that searches its own source for a literal it contains is counting itself.

---

## 023 — The gate answered a question next to the one it was asked

`.claude/hooks/gate-commit.sh` ran the gates in `${CLAUDE_PROJECT_DIR:-.}`,
which names the session's project, not the tree the commit lands in: from a
worktree it refused commits over another session's dirty checkout, and over a
clean one it passed them unchecked. Its first cases shared inputs across every
arm, so deleting a fallback left the suite green. **The rule:** name the subject
of a check, not a directory that usually contains it — resolve the tree the way
the commit does, honouring `GIT_DIR`/`GIT_WORK_TREE`; a guard over a gate
asserts a refusal; and a parameter every case shares is a parameter no case
tests.

**Guard.** `the_commit_gate_runs_the_gates_in_the_tree_the_commit_lands_in`,
which runs the hook eight ways. Six are the inputs it resolves a tree from: the
reported `cwd` as the only thing naming the landing tree; the pointers as the
only thing naming it; a reported `cwd` that no longer exists, so the existence
check hands the question back to the directory the hook is already in; no
resolvable tree at all, so `CLAUDE_PROJECT_DIR` is reached; a
`CLAUDE_PROJECT_DIR` that is not there either, so the hook bails out rather than
gating whatever it is standing in; and a run where nothing names the landing
tree — the negative control, which a hook that refused every commit would fail.
Measured, so would the bail-out case, which reports first; two catchers rather
than one, and it is only the uniqueness that would have been false had it been
claimed. The other two are before resolution: a commit
redirected with `-C`, and a command that is not a commit at all. Calling the
`-C` case a resolution input was this Guard line's third wrong claim.

---

## 024 — The fixture was the format as it had been

Codex 0.155.0 numbers each rollout record with an `ordinal` and refuses a
thread whose last record lacks one; Operon's replay records carried none, and
the hand-typed fixture was the format as it had been, so the suite stayed green
over a broken restore while the only real coverage was `#[ignore]`d. Change 060
repeated it sharper: a test asserting `hooks = true` forbade the repair of a
config Codex could not load. **The rule:** a hand-written fixture of another
product's format is a photograph — read the convention out of the file, and
assert what the other product accepts, never only what Operon writes.

**Guard.** `codex_restore_numbers_replay_records_the_way_the_rollout_numbers_its_own`,
`codex_restore_leaves_an_unnumbered_rollout_unnumbered`,
`a_rollout_whose_last_record_lost_its_number_fails_verification`, and the shape
script. Five mutations watched failing, plus the script's own: writing the
appended records unnumbered, numbering an unnumbered rollout from zero anyway,
taking the next number from the last record instead of the highest, dropping the
verification block, and dropping only its non-maximal arm — each reddening the
guard that claims it, and the shape script printing `ordinal` with the writer's
insert line removed. The legacy test is the one that matters most: it passes
both before and after the fix, which is what makes it a guard against the fix
being over-applied rather than a restatement of it.

---

## 025 — The band fired fifteen times into a change that was waiting for a person

`steering_bytes` crossed its `propose` tier and change 030 wrote the answer
correctly as `awaiting-user`, but nobody was asked: fifteen later changes each
waived the same breach, because `scripts/check-bands.sh` said what to do and
never whether it had been done. **The rule:** a signal that prescribes an action
must also report the state of that action — none, in progress, waiting for a person, or
already shipped — so the script prints the change declaring
`**Answers band**: <metric>` in its `intent.md`, and its status, beside the
breach.

**Guard.** `the_control_bands_say_whether_a_propose_breach_has_an_answer`, which
runs the real script against a saved reading — `--metrics` already existed — for
a metric that has a declared answer and one that has none, under both
interpreters the shebang can land on. It checks the named directory back against
disk rather than against a copy of the script's logic, so it cannot pass on a
plausible-looking line. Watched failing on twelve mutations, which are kept
beside this change as `mutations.py` and re-run rather than remembered: the
reporting never called, a mention accepted as a declaration, the explanatory
lines counted as breached metrics, the answer named without its status, a metric
with no answer reported as nothing at all, a shipped answer counted as one in
progress, a declaration with no space after its colon invisible to the script
while the guard could read it, a live answer reported as having nowhere to go,
an answer in progress filed as parked, a hyphenated near miss counted as a
declaration, and `failed` and `archived` each reported as answers in progress.
The third was a real defect this change introduced and the guard caught — two
metrics over, the header said four.

## 026 — Three rounds closing one reported spelling at a time

`apply_codex_config` edits `~/.codex/config.toml` line by line and misread a
`[features]` header four rounds running — a trailing comment, inner spaces,
`["features"]`, an array continuation row — each time writing a duplicate that
made Codex load no configuration at all. Each fix closed one spelling, and a
line editor never meets every legal spelling of a format it does not parse.
**The rule:** fix the fallback direction, not the recogniser — the strict reader
may only cause a write when a deliberately generous reader agrees nothing is
there, and the generous one wins every disagreement by vetoing.

**Guard.** `codex_hook_install_writes_a_config_codex_can_load` now carries two
spellings the code does **not** recognise — a quoted table name, and a
`[features]` whose body opens with a multi-line array — and asserts only that
the output still parses, never that the flag was set. That is the assertion
shape the lesson asks for: it passes for any future spelling handled the safe
way and fails for any handled the other way.
`the_config_check_and_the_writer_read_one_file_the_same_way` holds the script to
the same answers. `python3 docs/sdlc/changes/060-codex-cannot-load-the-configuration-operon-writes/mutations.py`
puts the "write anyway" default back in five separate places and prints 18 of
18 caught, each watched failing.

## 027 — A quote character ended the program it was written inside

Mirroring `opens_multiline_value` into the awk in
`scripts/check-codex-config-shape.sh` put `'''` inside a shell single-quoted
program, ending it and breaking the parse; beside it, `awk -v` escape-processed
a regex so it matched `[features]` zero times. A guard that reads a directory
rather than a list caught the first without being extended. **The rule:** one
rule in two syntaxes is lesson 001 again — the quoting layer between them is
where the meaning changes, so the mirror is checked by running it.

**Guard.** `every_shell_script_parses_under_the_oldest_shell_its_shebang_finds`
already covered it — it reads the directory rather than a list, so a script
added later is covered the moment it exists. The `awk -v` escaping needs no
guard of its own: `the_config_check_and_the_writer_read_one_file_the_same_way`
runs the real script over nine files, and a header pattern that matches nothing
fails there on the first row. Watched failing — `mutations.py` puts the single
backslash back and the run goes red.

## 028 — The sentence that said the drift was fixed was written before it was

Change 060 moved "where does a TOML table start" onto `is_table_header` and
wrote that the two readers "cannot drift again". It was false when written:
`apply_codex_config`'s `first_table` was never converted, so the checker refused
a file and said Operon removes the line while the writer removed nothing.
**The rule:** a claim about coverage — *every* reader, *cannot* drift — has a
cardinality in it and needs something counting; assert on the writer's
behaviour, not on the comment.

**Guard.** `the_config_check_and_the_writer_read_one_file_the_same_way` carries
`a top-level array before the scalar`, where the two scans disagree and no other
fixture does, and it asserts on the writer's behaviour rather than on the
comment. `python3 docs/sdlc/changes/060-codex-cannot-load-the-configuration-operon-writes/mutations.py`
puts the old test back at each call site separately — `the top-level scan ends
at a continuation row` and `the [features] body scan ends at a continuation row`
are two mutations, not one — so converting one scan and not its neighbour goes
red. Watched failing; 20 of 20 caught.

## 029 — `git add -A` staged the file that redirects git

`.claude/config.json` and `.codex/config.toml`, untracked in a worktree and
holding `GIT_WORK_TREE`/`GIT_DIR` lines, were swept into a commit by
`git add -A`; committed, they would aim git at one worktree from every checkout,
routing around the refusal of `git -C` in `gate-commit.sh`. A diff review reads
inside files and cannot see a file that should not be in the set; review caught
it by reading the staged list. **The rule:** stage by explicit path.

**Guard.** None, and saying so plainly rather than inventing one: a `.gitignore`
entry would be this session's guess at what those files are for, and they were
not written by this session. What changed is the practice — stage by explicit
path, which `docs/sdlc/changes/060-codex-cannot-load-the-configuration-operon-writes/state.yaml`
now says in its `resume` line so the next session reads it before it stages
anything. `the_gitignore_exemption_covers_runtime_state_and_nothing_tracked`
already holds the related invariant for files the repository does ignore.

## 030 — Which way a scan's boundary should err

`apply_codex_config`'s two header-bounded scans shared one reader, but their
mistakes cost opposite amounts: `first_table` reaching too far deletes a
person's `hooks` key, `end` reaching too short lets a duplicate be inserted. A
dot inside a quoted table name, `profiles."gpt-5.6"`, made the reader delete a
key and **the output still parsed**, so every parse-based guard was blind.
**The rule:** the direction the mistake costs picks the reader, and a guard that
checks the result is well-formed cannot see a well-formed result that is wrong.

**Guard.** `codex_hook_install_writes_a_config_codex_can_load` asserts the
survivor directly — `parsed["profiles"]["gpt-5.6"]["hooks"]` and
`parsed["hooks"]["state"]["/x/y.json:stop:1:0"]["hooks"]` — rather than only
that the document parses. `the_config_check_and_the_writer_read_one_file_the_same_way`
carries `an array row before a top-level scalar` and `an array row inside
features, before the key`, the two fixtures on which the generous and strict
readers give different answers, so swapping either scan onto the other reader
goes red. `python3 docs/sdlc/changes/060-codex-cannot-load-the-configuration-operon-writes/mutations.py`
does exactly that swap in both directions and in both implementations — four
mutations, each watched failing; 25 of 25 caught.

Since lesson 031 taught the reader multi-line values, those two fixtures no
longer tell the readers apart; the distinction survives only over input TOML
would refuse, and `a_config_toml_would_refuse_is_still_not_a_config_to_delete_from`
now holds it (lesson 032).

## 031 — A line does not mean anything on its own

Lines of `config.toml` were read for what they look like, when what they mean
depends on the lines before — an array row read as a header, a note inside a
multi-line string read as `[features]` or `hooks = true` — and each pattern fix
left the class open; the first state machine was flat while the values nest.
**The rule:** one walk, `structural_lines`, decides where tables begin and end,
never two functions that must agree; the vetoes keep reading everything,
because a veto that mistakes text for code only declines a write, while one
taught to skip text writes a duplicate.

**Guard.** `codex_hook_install_writes_a_config_codex_can_load` carries the rows
above as fixtures — including the three nested ones — and asserts the person's
text by VALUE, not by the document parsing: `parsed["note"]`,
`parsed["model"][1]` and `parsed["features"]["hooks"]` must be exactly what they
were. `the_config_check_and_the_writer_read_one_file_the_same_way` runs the same
shapes through `scripts/check-codex-config-shape.sh`, which carries the same
single walk in awk, and its table now includes the `[[name]]` rows whose absence
let the two drift apart inside one round. Sixteen mutations in
`docs/sdlc/changes/060-codex-cannot-load-the-configuration-operon-writes/mutations.py`
take the state machine away from one scan at a time, flatten the nesting, put
the old pattern back, and put the machine INTO a veto — which is the mistake in
the other direction — each watched failing.

## 032 — A fix can turn a red mutation green, and nothing says so

Round 7's correct fix stopped bracket rows inside arrays reading as headers, and
silently turned six mutations — lesson 030's evidence — green while the suite
gained a test. The gates run the tests; nothing asks whether a test can still
fail except the mutation set, and only when it is re-run. **The rule:**
re-running the whole set is part of a fix, not part of writing the guard, and a
substitution that matches nothing is a `[SCRIPT BUG]`, never a green guard.

**Guard.** `a_config_toml_would_refuse_is_still_not_a_config_to_delete_from` is
the test that run demanded, and it is registered in `mutations.py`'s `TESTS` so
the four boundary mutations have something to be caught by. `mutations.py`
itself reports a substitution that matches nothing as its own failure rather
than as a green guard, which is what turned "the code moved" into a visible
`[SCRIPT BUG]` line instead of a silent pass; 44 of 44 caught after the repair,
from 25 of 36 in the run that found it and 28 of 36 in the round after.

## 033 — Two branches a fixture never ran in the same call

`apply_codex_config` preserves top-level `hooks` shapes that TOML will not let a
dotted key extend, then appends `[hooks.state."…"]` blocks, so the preserved
shapes were exactly the ones the append broke. Nineteen of twenty-eight call
sites passed `&[]` for entries, so the two branches never ran in one call, and
no mutation targets the gap between two branches. **The rule:** when a function
has a branch that keeps input and a branch that adds input, at least one
fixture has to supply both at once.

**Guard.** `codex_hook_install_writes_a_config_codex_can_load` now runs all
three preserved shapes with entries non-empty, plus the boundary the veto must
NOT take (`hooks.enabled = true` is a real table, so the blocks extend it and
are written). Two mutations in `mutations.py` hold it: one appends
unconditionally, one widens the veto's key test to a prefix so a dotted key
loses its blocks.

## 034 — The veto knew the root of the path, not the path

The veto guarding the `[hooks.state."K"]` append asked only whether `hooks`
itself was bound, missing `hooks.state = "x"` and a `state` key under
`[hooks]`; later rounds found a duplicate veto, `might_assign`, inside
`[features]`, and Operon's own leaf header compared as raw text. A semantic
fuzz oracle, not a parse, found the class. **The rule:** write the guard against
the thing being written, not the thing being read — every proper prefix of the
written path must be a table TOML will create — and fix every place that asks
the same *question*, because a duplicate reader survives a correction.

**Guard.** `codex_hook_install_writes_a_config_codex_can_load` carries the
three root binding spellings plus the array-of-tables one — which cannot be
caught by parsing the output, so it asserts the shape — the two that must still
be appended, the four dotted spellings inside `[features]`, the `hooks_enabled`
neighbour that must still be enabled, the four reformattings of Operon's own
block (each with a stale `trusted_hash`, so the assertion is that it was
*replaced* and not merely survived), the five shapes that bind the leaf without
being ours, the four that stand clear of it and must still be written, and the
three keys the writer cannot round-trip — of which `\b` is the one that used to
fail silently, TOML reading it as a backspace so that Codex stored a key Operon
never looks for. `the_config_check_and_the_writer_read_one_file_the_same_way`
pins the script against the same three `[features]` rows.
The same test pins all three reachable `UNKNOWN:` paths — no config, an
unrecognised option, and codex absent from a deliberately bare `PATH` — plus
the premise they rest on, that bash answers 2 with an empty stdout for a script
it cannot parse. The fourth, the timeout, needs a live Codex and is checked by
hand with `OPERON_CODEX_PATIENCE=0.05`.

## 035 — The general reader lost a case the narrow one handled

Replacing a raw-text header comparison with a key-path one lost a case the
narrow reader handled: `table_header_shape` ignores quotes, so a trust key
holding `]` stopped being a header and every launch after the first appended a
duplicate block Codex refused. The quote-unaware search was a nit already
deferred, and no fixture ran the writer twice. **The rule:** when a narrow
reader is replaced by a general one, ask what the narrow one used to catch — a
deferred nit is deferred against the code as it was — and apply the function to
its own output.

**Guard.** `codex_hook_install_writes_a_config_codex_can_load` now runs the
writer over its own output for five keys — `]`, a lone `[`, `[[a]]`, a path
with a space and an ordinary one — and asserts the second run parses, keeps the
flag, declares the table at most once, and is byte-identical to the first.
Watched failing: with the fix reverted, the failure output shows the duplicated
`[hooks.state."…[work]…"]` block verbatim. The same test asserts that every
header present in the input is still present in the output, which is what
catches a reader that DELETES too much — watched failing by restoring the
inner `.trim()` in `unquote_segment`, which drops a table TOML says is not
ours. Both are in `mutations.py`.

## 036 — The same character, closed on one edit path out of four

Lesson 035's refusal of an unreadable header went in at `can_hold_hook_state`,
the last of four steps, so the top-level scan still ran past
`[projects."/Users/me/[work]"]` and deleted the person's `hooks = false` — valid
TOML in and out. **The rule:** a guard placed at one caller is a guard against
one caller; share the predicate, not the answer — the boundary stops, the veto
refuses — and a refusal aimed at a misreading must tell a `]` present but
mis-split from a line that is simply not a header.

**Guard.** `a_config_toml_would_refuse_is_still_not_a_config_to_delete_from`
carries three bracket-in-the-name headers — a profile, a project path and an
array of tables — each with a scalar under it that the writer never writes, so
a deletion cannot be masked by the flag the writer adds. Each row is also run
with the top-level scalar above it, asserting the repair still happens: that is
the assertion the abandoned fix fails, so the trade cannot be made again
without the suite saying so. Watched failing: with
the refusal disabled, the first row reports the person's key gone. Its
neighbour, the `[a` row, is what holds the narrowing, and was watched failing
by widening the refusal to every unreadable `[` line.
`codex_hook_install_writes_a_config_codex_can_load` holds the `[[features]]`
discrimination. Three mutations in `mutations.py` cover the boundary, the veto and
the `]` test; `76 of 76` caught.

---

## 037 — The doc comment counted its readers, and the count was the defect

A doc comment said two readers needed lesson 036's predicate; a third, the loop
that drops Operon's `[hooks.state."…"]` blocks before re-appending them, was
ungated, so an unreadable project header made every launch delete trust blocks
the refused append never restored. Every fixture started without our blocks, so
none could see the regression. **The rule:** the two halves of a replacement
have to refuse together; a prose list of callers is an assertion with no check
behind it; and a comment stating a consequence in another function's behaviour
is measured or not written.

**Guard.** `codex_hook_install_writes_a_config_codex_can_load` ends with a
four-launch sequence that starts from a config Operon has already written its
blocks into, has an unreadable project header appended to it the way Codex
appends one, and then asserts a constant two blocks, the person's table intact,
and a parse on each of launches 1-3. Three launches and not one, because "it
does not heal" is the finding. Watched failing: with the gate removed the first
launch reports `left: 0, right: 2`. A `remove_only` row follows it, because
review measured that the caveat a reader reaches for first —
`|| entries.is_empty()`, letting the removal path delete since nothing appends
after it — passed the entire suite while the sequence was enable-only. **A
decision no fixture can see is a habit, not a decision.** Watched failing with
that exact caveat: `left: 0, right: 2`. Three mutations in `mutations.py` cover
it — removing the gate, asking the predicate per line instead of per document,
and the removal-path caveat.

## 038 — Taking the nit destroyed the approval it came with

`scripts/check-review.sh` binds verdicts to a digest of the diff, so taking an
`approve-with-nits` nit moved the digest, forced a new round, and the round
raised another nit; change 060 spent nineteen rounds so, while the script's own
`nit は blocking しません` was read and ignored, and a wait on vanished reviewers
had no liveness check. **The rule:** a statement inside a script is not a
mechanism — a nit is carried, not taken, in the round it is raised, and past a
review-round ceiling `check-review.sh` refuses with one exit: carry the open
nits and land.

**Guard.** `a_review_past_its_round_ceiling_lands_by_carrying_its_nits` builds a
fixture repository with its own `routes.yaml`, stages a code change so a
reviewer is required, reads the real digest back out of the gate's own refusal
rather than computing it, and then asserts both directions: `rounds: 5` against
a ceiling of 2 with no `carried:` exits 1 and names the count, and the same
record with `carried:` exits 0 and prints `round 5 of 2`. Watched failing under
four mutations, each run and each red — the ceiling never refusing (`if false`),
the ceiling refusing even when the nits are carried (the exit removed), the
count never printed, and the route's ceiling ignored for a permanently generous
one. The fixture also needed `INHERITED_REPOSITORY_POINTERS` dropped from the
`bash` child: with `GIT_DIR` and `GIT_WORK_TREE` inherited, every `git` call
inside the gate read the real checkout and the guard reported on change 061
instead — lesson 057's defect, met again one layer down.

## 039 — The table was widened, and the reader of it was on another branch

Change 064 added a sixth column to `docs/sdlc/routes.yaml`, but
`.githooks/reference-transaction` judges commits with
`refs/heads/main:scripts/check-review.sh`, whose parser wants five values, so
the next code commit found no route at all. The branch's parser had been taught
the column in the same diff, and 064's own commit was carried by its parent's
table. **The rule:** a branch may narrow what its own gate allows, and may never
widen what the gate expects to read; a compatibility claim names the reader it
is compatible with.

**Guard.** `every_route_row_is_the_width_the_gate_that_judges_commits_reads`
reads the width out of `scripts/check-review.sh`'s own count rather than
restating it — lesson 004 — anchored on the line that reads `routes.yaml`,
because the script counts positional values in four places and stopping at the
first one after the anchor is what makes it the right count. It asserts first
that `.githooks/reference-transaction` still names
`refs/heads/main:scripts/check-review.sh`: that is the premise, and a guard that
goes on enforcing a rule after its cause has moved is worse than no guard.
`every_route_has_a_review_round_ceiling` holds the two files to the same route
names in both directions — a route with no ceiling falls back to the default in
silence, and a ceiling for a route nobody takes bounds nothing while reading
exactly like a bound.

## 040 — The fix was applied to one of the two files that had the defect

Change 057 fixed `.claude/hooks/gate-commit.sh` resolving its tree from
`CLAUDE_PROJECT_DIR`, but `.claude/hooks/gate-stop.sh` beside it did the same,
reporting other changes' open items to a worktree session; its git half
honoured `GIT_DIR`/`GIT_WORK_TREE` while its glob did not, so the exit status
looked right. **The rule:** when a defect is fixed, sweep the repository for the
same defect rather than the same file; and a paused surface pauses the fix, not
the record — a deferral with its cause, fix and reason written down is a
decision.

**Guard.** `the_stop_gate_reads_the_tree_the_session_is_working_in`, which runs
the hook against two repositories — one holding an untracked `.rs` file and a
change with an open `machine pending` item, one clean — over all five inputs the
hook can resolve a tree from: the reported `cwd` alone; the repository pointers
alone; a reported `cwd` that no longer exists, so the existence check hands the
question back; nothing resolvable, so `CLAUDE_PROJECT_DIR` is reached; and
nothing naming the tree at all, which is the negative control. The pointers case
asserts the text of the refusal and not its exit status, for the reason above.
The run is under `/bin/bash` — the 3.2 — so the parse fix is measured
behaviourally rather than only by `-n`, and `KNOWN_UNPARSED_UNDER_BASH_3_2` is
now empty, which its own two readers turn red about if an entry is left behind
after its script is fixed.

## 041 — The counter was incremented before the decision it was counting

Every transcript reader called `count_unclassified_kind` before `record_text`
decided whether the record could be carried, so kinds with no text — 2385
`codex/token_usage_record`s in one thread — were dropped and reported as
carried; the fixtures all carried text, and the arithmetic guard watched the
struct while the person read the sentence. The fix then rewrote the defect at
the two sites it could not absorb. **The rule:** a count taken before the branch
it is about is a count of something else — decide and count in one statement —
and a report is only as true as its narrowest claim, so mutate the claim, not
only the counter. The claim axis is guarded by the prompt
`evals/007-mutate-the-claim-not-the-counter.md`.

**Guard.** Six tests, and the two load-bearing ones name no copy.
`a_kind_that_crossed_and_a_kind_that_was_dropped_are_reported_as_different_things`
renders the notice for a carried kind alone and for a dropped kind alone, swaps
the kind names out of both, and requires the results to still differ — so one
sentence covering both arms fails however it is worded.
`the_notices_printed_counts_account_for_every_record_read` reads a ten-record
fixture that gives all six counters distinct values, then bumps each counter in
turn and requires the notice to change: a counter that does not reach the
sentence cannot change it.
`a_codex_token_count_record_is_excluded_rather_than_reported_as_unknown` holds a
real `token_usage_record` copied out of `~/.codex/sessions` rather than typed,
and `an_unknown_role_is_reported_as_carried_because_its_turn_crosses` covers the
third arm — an unknown role does not stop the message, so it is carried, and
filing it under dropped would report a person's own screen back to them as
missing. That last test exists because the mutation for it was not caught.
`the_notice_promises_no_marker_the_restored_text_does_not_carry` is the fifth,
added in round 2 for the finding above: it reads the marker out of
`unclassified_label` rather than typing it, and requires that a marker the
notice names appears in the restored turns — over one fixture where the label is
attached and one where it is not.
`an_unknown_flag_or_role_is_counted_against_what_the_record_did` is the sixth,
added in round 3, over the reviewer's own two reproductions and their crossing
counterparts: it asserts that `carried > 0` and "a turn was produced" are the
same statement, and that the top-level arithmetic closes in both cases — which
is where the defect had been hiding.

## 042 — Four failures nothing could see coming, and the shape they shared

Four things went wrong more than once. They have nothing to do with one another
in subject, and they are the same failure: **a failure that leaves no trace a
later session can read.** The repository held the individual fact every time —
in a lesson, in a diff, in a policy — and held no mechanism that would put the
fact in front of the next session at the moment it mattered.

**Mutation testing cannot see a claim nobody guarded.** Change 061 shipped
fifteen mutations, every one caught, and still went back twice; both findings
were sentences a person reads making a claim that was not true of every site
they covered. Every mutation asked whether a *count* moved, and neither defect
was in a count. The sweep read as thorough and was blind along exactly one axis,
and nothing named the axis. **A verification technique reports coverage along
the axis it varies, and silence along every other.**

**A gate pointed at another tree does not know it.** The Stop gate stopped one
session three times on the same position, naming open contract items from change
directories that do not exist in the tree being worked in — because a worktree
session runs the main checkout's copy of the hook, which change 065 fixed for
*which tree the findings come from* and not for *saying so*. Diagnosing it cost
attention every time, because a refusal about another tree looks exactly like a
real finding. **A report that cannot say where it is reading from makes its
reader re-derive that each time.**

**A reviewer that finished without reporting looks like one still thinking.**
Three occurrences. The policy covered the reviewer that never comes back; the
one that came back silently has a different remedy, and guessing wrong is how
change 065 ended up with two verdicts at one digest to reconcile by hand. The
remedy is cheap once it is written down: re-ask the same agent by name — it
still holds the review — rather than spawning a replacement.

**Whether a change makes the installed application stale was discovered last.**
It is decided by what the change touches, which is knowable on its first day,
and was established by reading a diff after everything was committed. Twice the
answer was "no" by luck; the third time it was "yes" and surfaced only when the
release gate refused, with the app running and the session unable to finish.

The four share a shape and no code, which is why they are one change and one
entry: four directories would have carried the same authorisation, the same
surfaces and the same lesson four times over. **When several failures share a
shape, the shape is the thing to write down; the instances are examples.**

**Guard.** One per failure, each watched failing —
`docs/sdlc/changes/067-*/mutations.py`, fourteen mutations, all caught.

`the_review_policy_names_every_policy_the_repo_enforces` gains a row for the
claim-mutation rule and one for the three reviewer states, so dropping either
sentence from `REVIEW.md` is a red suite rather than a policy nobody reads.
`the_pipeline_documents_ask_their_questions_before_review_does` holds the same
rule in `.claude/skills/sdlc/SKILL.md`'s step 10 and the `release-binary` row in
`docs/sdlc/templates/state.yaml`: both are places a session is already reading
while doing the work, which is the difference between a rule that costs a
mutation and the same rule costing a review round. A test cannot decide what
counts as a claim, so the axis itself is guarded by a prompt —
`evals/007-mutate-the-claim-not-the-counter.md`, which seeds a change whose
sentence makes a claim and whose `mutations.py` moves counters only, and passes
only when breaking the truth of the sentence turns the suite red. Its check was
verified by hand in both directions, because an eval nobody has watched fail is
the same guess as a guard nobody has watched fail.

`the_stop_gate_says_when_it_is_reading_another_checkout` runs the hook with its
own file in one repository and the session's tree in another, over three
arrangements: named absolutely, named relatively — `dirname "$0"` resolves
against the process's working directory, which for a worktree session is the
main checkout, so resolving it after the hook's own `cd` would answer with the
tree under suspicion — and the control, where hook and tree agree and the line
must not appear at all. It asserts the two repository roots rather than the
sentence, and asserts their **order**, because a line naming both the other way
round is true in every particular and sends the reader to fix the wrong
checkout. That one is this entry's own claim mutation.

`the_stop_gate_reports_an_installed_build_that_is_behind` asks behaviourally,
with a fixture check that says something no other part of the refusal would:
`every_gate_script_is_wired_or_declares_why_not` sees only that the hook *names*
the script, and a hook that names it in an `[ -f ]` test and never runs it
satisfies that. **A guard that a mechanism is referenced is not a guard that it
runs.** `the_installed_build_check_reads_its_stamp_key_out_of_the_packager`
renames the stamp key in a copy of `scripts/package-macos.sh` and requires
`scripts/check-installed-build.sh` to follow it, and pins the part that decides
whether the check is noise: an unstamped bundle is reported as unknown, never as
stale, because every bundle installed before this change is unstamped.

Three things this change declined to do, each recorded so the next session does
not re-decide them. The Stop gate does **not** refuse on a mismatched checkout:
that would stop every worktree session on this machine until main takes the
merge, which is worse than the defect — the whole cost was not knowing. The
mutation rule is **not** enforced at commit time: the gate would have to read
the diff and the mutation script together and decide what counts as a claim, and
a wrong answer there refuses commits on a heuristic, on the one surface that can
switch off every other gate. And `scripts/check-readiness.sh` returning No-Go for
every `bugfix` route — a fifth failure of the same shape, found while writing
this — is left alone, because it is the gate this change's own route runs
through.

One near-miss worth the line. A comment added to `.claude/hooks/gate-stop.sh`
named `.githooks` in prose, and `the_review_gate_is_still_not_installed_by_
anything_committed` reads that directory together with a hooks destination as
evidence that something now *installs* the git hook — so a sentence about a
mechanism went red as if it were a line that runs it. The comment was reworded.
`shell_code` in `src/tests.rs` exists for precisely this and is not reached from
that guard; the next occurrence should move the guard rather than the prose.

## 043 — The hardening was lossy along the axis nothing compares

Change 069 made `docs/sdlc/templates/mutations.py` restore a mutated file
atomically: write a `mkstemp` sibling, `os.replace` it over the target. That is
the correct shape — rename is the only all-or-nothing write a filesystem
promises — and it replaced a `path.write_text` that could leave half a file
behind. It also introduced a defect the whole repository was blind to.
`mkstemp` creates at `0600`, and a rename swaps the inode, so every file the
sweep touched came back with the temporary's permissions.
`scripts/check-installed-build.sh` went from `755` to `600`, twice, in the
middle of its own review.

Nothing could see it. The bytes were identical, so `git diff` was empty and
`restore_all`'s read-back — which compares contents — reported the restore as
complete. The sweep printed `7 of 7 caught`. The three gates were green. The
only trace anywhere was one line in `git diff --cached --summary`, and it took a
reviewer reading the diff *header* to report it, as an unexplained mode change
rather than as what it was.

**A write that replaces a file replaces everything about it that is not the
contents.** Permissions, owner, times, extended attributes: the old call wrote
through the inode and carried them for free, the new one does not carry any of
them, and the substitution looks like an improvement in every respect the tests
examine. The general form is worth more than the instance: a verification
compares the axis it was written to compare and is silent on the rest, so
*changing how* something is done needs its own question — what did the old
mechanism preserve incidentally? — because no existing check will ask it.

What it was one sweep away from: a sweep's subject is whatever the change
touched, and changes here touch `.claude/hooks/*.sh` and `scripts/*.sh`. A hook
restored without its executable bit is a gate that silently stops running, in a
tree whose `git status` is clean.

**Guard.** `the_mutation_sweep_template_restores_the_mode_it_found` in
`src/tests.rs` runs the template over a `755` fixture and requires `755` and no
stray temporary afterwards. Watched failing through
`docs/sdlc/changes/069-a-stamp-from-outside-the-repository-is-trusted-as-a-revision/mutations.py`'s
`restore-drops-the-file-mode` entry, which deletes the two lines that carry the
mode across — 8 of 8 caught. The rule is also written into the template's own
rule 6, where the next person copying it will read it.

---

## 044 — A predicate that stopped reading, and a gate that read the wrong answer

**Numbering.** Written on main as entry 018 while entries 018–043 were being written on another branch, and renumbered when the two met. It is the original account of the rule entry 018 above reconstructs.

**What happened.** Change 035's git hook decides whether a commit is being
*made* or being *returned to* by looking for it in every reflog:
`git reflog show --all | grep -q "$new"`. Under `set -o pipefail`, `grep -q`
exits at its first match, git gets SIGPIPE while still writing the rest, and the
pipeline's status is git's 141 — a match read as a miss. A commit made with the
hook off could never be reset back to. The same shape had been found and fixed
once already that round, in `scripts/check-review.sh`'s table union, and a third
copy sat in the same script's content trigger, where a large diff that added
`Command::new` early could read as one that did not.

**Why it was invisible.** `grep -q` is the idiomatic predicate, and it is
correct in a script without `pipefail`, and correct under `pipefail` whenever the
producer happens to finish before grep stops reading — which for a short reflog
is most of the time. The probe that checked the filter ran without `pipefail`
and said yes ten times out of ten; the same line in the hook said yes about half
the time.

**Cure.** `grep … >/dev/null`, which reads to the end and returns grep's status,
or the producer captured into a variable and searched afterwards. Every shell
gate in the repository was swept, and the sweep found the predicate in three
gates and two guards.

**The rule.** In a script that sets `pipefail`, the consumer at the end of a
pipe must read everything the producer writes, or the pipeline's status belongs
to the producer's interruption rather than to the question asked.

**Guard.** `every_gate_script_reads_its_pipes_to_the_end`: in any script under
`scripts/`, `.claude/hooks/`, or `.githooks/` that sets `pipefail`, no `grep`
consuming a pipe may carry `-q`. Watched failing with the hook's reflog line put
back to `grep -q`, and with the review script's trigger put back. The fixture
scenario that found it — a commit made with the hook off, stepped back from and
moved forward to through `ORIG_HEAD` — is the behavioural half, in
`the_review_gate_refuses_what_it_says_it_refuses`.

---

## 045 — A gate written in a shell the machine it ships to does not have

**Numbering.** Written on main as entry 019 and renumbered at the same merge. Entry 040 is the same shell failing in the other file that had it.

**What happened.** In `.claude/hooks/gate-stop.sh` and
`.githooks/reference-transaction`, round 16 of change 035. One round rewrote three `whole()` helpers at once and
tidied the surrounding lines. Two of the edits did not survive a parser.
`.claude/hooks/gate-stop.sh` was left with

```sh
case "$status" in done|archived) continue ;; esac
```

inside an `open_contract=$( … )` command substitution — valid in bash 4 and 5,
rejected by bash 3.2, which is what `/bin/bash` is on macOS and what
`#!/usr/bin/env bash` finds on any checkout without Homebrew's. Everything
below it, the whole branch audit, was unreachable there. And
`.githooks/reference-transaction` was left with an unterminated `"` on a line
a patch script had eaten the closing quote from — that one no bash parses.

**Why it was invisible.** Every test that touches these scripts either reads
them as text — pinning a line, counting an exit — or runs them through
`std::process::Command::new("bash")`, which resolves to whatever is first on
`PATH`. In this session that is Homebrew's 5.3. So the suite ran the scripts
under a shell that parses them and asserted on scripts under a shell that
does not, and 477 tests passed with one gate that could never refuse and one
that could never run at all. The fifteen preceding rounds of adversarial
review had asked what the scripts decide, never whether they parse.

**Cure.** `bash -n` under `/bin/bash` specifically, over every script in the
repository whose shebang names bash — not the `bash` on `PATH`, because the
question is about the oldest shell a person here can end up with, not the
newest.

**The rule.** A shell gate is not verified by a test that runs it under the
author's shell. The version it must parse under is the stock one.

**Guard.** `every_gate_script_parses_under_the_stock_shell`: `/bin/bash -n`
over every bash script (seventeen) under `scripts/`, `.claude/hooks/`, `.githooks/`,
and `.githooks/installed/`, with a floor on the count so a broken directory
list reads as a failure rather than as nothing to check. Watched failing with
the one-line `case … esac` put back.

---

## 046 — Codex CLI rollout replay projection drift and session ID parsing

**Numbering.** Written on main as entry 020 and renumbered at the same merge.

**What happened.** When restoring conversations into Codex CLI, Operon previously emitted legacy `event_msg` records with `user_message`/`agent_message`. In modern Codex CLI (v0.154.0+), the internal projection engine ignores those payload types and requires `task_started`, `item_completed` (`UserMessage`/`AgentMessage`), and `task_complete` with non-null `rollout_ordinal` to project into `thread_history_1.sqlite`. Furthermore, falling back to parsing the session UUID from rollout filenames using `rsplit_once('-')` truncated the UUID to its final 12 hex characters rather than the full 36-character UUID.

**Why it was invisible.** Legacy unit tests only asserted that the strings appeared in the output file, and did not run Codex CLI's actual SQLite projection engine or test rollout filenames with hyphens. The ignored e2e tests were skipped during standard `cargo test`.

**Cure.** Operon generates modern Codex replay events with strictly sequential ordinals and monotonically increasing timestamps, defaults `next_ordinal` to `Some(0)` to satisfy SQLite's `NOT NULL` constraint, uses `crate::cli::codex_session_id` as the SSOT for UUID filename extraction, and resolves Gemini native session paths.

**The rule.** A cross-CLI restore is only complete when the target CLI's internal projection engine consumes and replays the records. Replay schemas must be verified against the CLI's projection database constraints.

**Guard.** `codex_rollout_metadata_recovers_full_uuid_from_filename`, `codex_replay_records_default_to_ordinals_and_monotonic_time`, `codex_replay_records_preserve_and_continue_paginated_ordinals`, and live e2e restore tests (`a_saved_antigravity_conversation_restores_into_a_resumable_codex_terminal`).

---

## 047 — A merged worktree nobody was told to remove

**What happened.** On 2026-09-23 the repository held six linked worktrees and
six stray branches. Three worktrees were fully merged and held nothing but
generated pointer files. A person asked why the agents never tidied up.

**Why it was invisible.** Three separate facts, and each was reasonable on its
own. `AGENTS.md`'s landing sentence stopped at `git merge --no-ff`. The one
removal the guards allow, `ops.mjs cleanup-worktree`, measured against
`origin/main` and fetched `origin` in a repository that has no remote, so it
failed every time. The Stop guard treated "clean and nothing ahead" as finished,
and that is exactly what a merged worktree looks like. The session-owner guard
then turned each leftover into a permanent one: once its owner had gone, no
session was allowed to remove it.

**Cure.** The landing sentence now ends by removing the worktree. `cleanup-worktree`
measures against the local base when there is no remote, and its removal is no
longer cut off by a 30-second timeout on a tree that holds a `target/`. The Stop
guard names a worktree that is owned, clean, and whose tip is a merge parent on
base.

**The rule.** A procedure that creates something ends by removing it, and the
removal command has to succeed in this repository under its local configuration.

**Guard.** `cleanup_removes_a_merged_worktree_in_a_repository_with_no_remote`,
`the_stop_guard_names_a_merged_worktree_the_session_still_owns`,
`the_landing_procedure_ends_by_removing_the_worktree` — six mutations over
`ops.mjs` and the Stop guard, each caught, and the landing test watched failing
on the unedited `AGENTS.md`.

---

## 048 — A test fixture that commits to whichever repository the session names

**What happened.** On 2026-09-25 a session entered its worktree with Claude
Code's EnterWorktree, which exports `GIT_DIR` and `GIT_WORK_TREE`, and ran
`cargo test --locked`. The review-gate fixture's `git init`, `git config` and
`git commit` inherited both and ran against the real repository: the shared
`.git/config` gained `core.worktree` pointing at the worktree and the
fixture's identity, three times, and the third run committed the session's
staged change onto its branch as "before the gate" by `fixture@operon`. The
first `core.worktree` also sent another session's merge into this worktree.
`.claude/hooks/gate-commit.sh` already recorded the same failure twice, as
commits named `initial`.

**Why it was invisible.** Change 058 built the defence —
`INHERITED_REPOSITORY_POINTERS`, `forget_inherited_repository`, and a guard
that counts raw git spawns under `src/`. `ReviewGateFixture::command` builds
its `Command` from a variable, `Command::new(program)`, so the count never saw
it, and the suite is green under a clean environment, which is the only one CI
and most sessions have.

**Cure.** `ReviewGateFixture::command` drops the pointers for every process it
starts. `.claude/hooks/gate-commit.sh` unsets the repository variables before
it runs the gates, as a second layer for the next spawn that forgets.

**The rule.** A test that shells out to git proves it cannot reach the
session's repository on the built command, not by the absence of a raw
`Command::new("git")` in the source.

**Guard.** `the_review_gate_fixture_cannot_reach_the_session_repository` —
watched failing with the call removed. End to end: the whole suite run with
`GIT_DIR` and `GIT_WORK_TREE` exported at a scratch repository left it
byte-identical (591 passed), and with the call removed the same run wrote the
fixture's identity into its config.

---

## 049 — The worktree tools asked git about a directory, and git answered about the session

**What happened.** In the same session as 048, with `GIT_DIR` and
`GIT_WORK_TREE` still exported, `ops.mjs cleanup-worktree` removed a merged
worktree and read its branch in that worktree's directory as `main` — the
session's — so the branch it should have deleted stayed, and the command
reported `ok`. Then the worktree those variables named was gone, and every git
the tooling started failed: the trunk allowlist could not read the branch and
denied every command, fail-closed, and `make wt.new` said `main` did not exist.
The session could not make the worktree that would have let it continue, nor
unset the variables, and had to be restarted.

**Why it was invisible.** git reads the pointers before the `cwd` it is given,
and the Node entry points (`execOnce` and `resolveProjectRoot` in `ops.mjs`,
`defaultGitFn` and `defaultNodeFn` in `worktree-new.mjs`, `safeExec` in
`.cli/lib/utils.mjs` behind every hook) passed the whole environment on. 048
fixed the Rust side only. And the tests that drive these scripts run them
through `node_in`, which drops the pointers, and `run_command_with_timeout`,
which drops them again — so no test ran a script the way a session does. The
first draft of this change's own test passed on the broken scripts for exactly
that reason.

**Cure.** `withoutInheritedRepository` in `.cli/lib/utils.mjs`, over
`INHERITED_REPOSITORY_POINTERS` — the same two names as the Rust constant —
applied at each of the spawn sites above.

**The rule.** A test for an environment-borne failure must show the variable
reached the process that misbehaves; a harness that scrubs it proves nothing
until the test has been watched failing.

**Guard.** `cleanup_removes_the_worktree_it_was_given_when_the_session_names_another`
and `a_git_dir_that_no_longer_exists_does_not_stop_the_worktree_tools`, which
hand node the pointers through `--env-file` so its children inherit them. Both
watched failing on the unedited scripts for the named cause; four mutations —
the scrub removed from `execOnce`, `defaultGitFn`, and `safeExec`, and
`GIT_WORK_TREE` dropped from the list — each turned one red. Not guarded:
`resolveProjectRoot` (the tests set `CLAUDE_PROJECT_DIR`, which it prefers),
`defaultNodeFn`, and the other scripts under `.claude/scripts/` that spawn git
(`wt-run.mjs`, `worktree-init.mjs`, …), which still inherit.

---

## 050 — The Stop gate handed every session every open change

**What happened.** Each Stop, in every session and every worktree, listed the
pending `machine` contract items of every open change in the tree — 035, 038,
039, 040, 041, 049, 077 — whatever the session was working on. Most are parked
behind 035 or have no diff yet, so no session could settle them; each answered
the refusal by hand, and because the stamp that stops a position being refused
twice lives in the tree's own `target/`, a new worktree or the main checkout
after a landing heard it again. The person asked for it to stop for good.

**Why it was invisible.** The section was written when a tree held one open
change, so "every open change" and "this session's change" were the same set.
Thirty open changes later they were not, and every test of the section used a
fixture with one change, which is the case where the two readings agree.

**Cure.** `.claude/hooks/gate-stop.sh` reports a change's contract only when
the tree is changing it: uncommitted files under its directory, or a commit on
this branch since `git merge-base main HEAD` touching it.

**The rule.** A gate that reports obligations reports the ones the session can
discharge. A list it cannot act on is not a stricter gate, it is a gate that is
answered by rote.

**Guard.** `the_stop_gate_holds_a_session_only_to_the_changes_it_is_working_on`
— a fixture with a change parked on `main` and a branch committing another.
Watched failing on the unedited hook (it named the parked change on the
branch). Three mutations — the uncommitted half, the branch half, and the
filter itself removed — each turned it red.

**Follow-up (sdlc 083).** 081's scope read `git status --porcelain` text, and
so lost a change renamed with `git mv` (read under the directory it left), a
change whose only edit was a path git quotes, and — in a repository with no
`main` — every committed change, failing open where the gate had failed
closed. It now asks for paths with `-z`, and with no fork point reads every
open change. Five more runs in the same test, each watched failing on 081's
hook except the untracked one, which pins a path 081 had and did not test.
Eight mutations each caught — among them `-z` removed, which survived until
the test pinned `core.quotePath=true`: this machine's `~/.gitconfig` turns
quoting off, so a guard for quoting passes here whatever the hook does.

---

## 051 — A step that undoes itself was handed to a person to type

**What happened.** In one session the person was asked to type
`git branch -d feature/prompt-audit` (a branch already merged), and then
`git merge --abort` (a landing merge the review gate had stopped half-way),
because the trunk allowlist admitted neither; and a re-verdict needed only
because another session's change moved `main` under a reviewed branch was put
to them as a question past the review-round ceiling. None of the three could
do harm if the session had simply done it. The person: the criterion for
asking is whether a reversal would be fatal, and a person is not a place to
send commands.

**Why it was invisible.** The allowlist was written from the landing path
forward — worktree, merge, gates — and not from what the landing leaves
behind when it stops. The ceiling counted every return to a reviewer, and
nothing distinguished a return a finding caused from one the base caused.
Nothing written down said when a person is the one to ask, so the session
asked whenever a guard said no.

**Cure.** `git merge --abort` and `git branch -d <branch>…` on the trunk
allowlist (never `-D`, never a flag after `-d`); in `REVIEW.md`, a re-verdict
caused only by `main` moving is recorded under the round it re-confirms; in
`AGENTS.md`, ask a person only about what cannot be undone, and fix the guard
rather than hand over a command.

**The rule.** When a guard refuses a step that undoes itself, the defect is
in the guard.

**Guard.** `the_trunk_allowlist_and_the_ownership_check_are_switched_on_here`
gains the allowed and the still-denied spellings, watched failing on the
unedited policy; `a_session_asks_a_person_only_about_what_cannot_be_undone`
holds the two sentences. Six mutations each red — each pattern removed, the
branch pattern widened to `-D` and to flags, and each sentence weakened.

---

## 052 — A finding left unfixed was reported in words only the session understood

**What happened.** Closing change 083, the session told the person "carry
した nit が7件" and pointed at reviewer notes written in English, by file and
line. The person could not tell whether any of them mattered, and said so:
that is language only the session understands.

**Why it was invisible.** A carried finding had one text, the reviewer's,
kept verbatim so a later change can act on it. `scripts/check-review.sh`
printed that text under "Nits (blocking しない)", and nothing asked what the
finding meant to someone who had not read the diff — so a report was written
from the only words there were, in the review's own vocabulary.

**Cure.** Each carried finding has a `person:` line in plain Japanese beneath
the reviewer's: what goes wrong, when, and whether the person must act. The
review gate refuses one that is missing, has no Japanese, or uses the review's
words (nit, digest, carried, round, Important), and a change that ships shows
the person's lines. `AGENTS.md` and the sdlc skill's Report say a report is
written from them.

**The rule.** A record written for the next session is not a report to a
person. When both are needed, both are written, and the gate asks for the one
a person reads.

**Guard.** `a_finding_left_unfixed_reaches_a_person_in_their_words` — a
carried finding with no line, an English line, and a jargon line each refused;
a plain line ships and is shown instead of the reviewer's. Watched failing on
the unedited gate on all five counts. `a_report_to_a_person_is_written_in_their_words`
holds the two sentences.

---

## 053 — The trunk refused the reads it had promised to admit

**What happened.** Change 058 said the trunk admits read-only inspection, and
the allowlist that carries it out named `cat`, `grep`, `head` and `tail`. Every
other read — `sed -n`, `rg`, `jq`, `sort`, a `for` header, `git for-each-ref` —
was refused: 188 refusals in thirteen days, each a retry or a detour.

**Why it was invisible.** The test held what must be refused and the handful
of gates that must pass; nothing held the promise itself, so the list could
fall short of it with the suite green. A refusal reads as the guard working.

**Cure.** Change 095 names the reads, and fences each one's writing or
program-running spelling (`sort -o`, `date -s`, `git cat-file
--textconv`), abbreviated long options included, and every fence now looks
past a newline kept inside quotes — the older git, find and cargo fences
stopped at one. sed is admitted only as a
line-range print, because its script can write (`w`) wherever it appears; awk
stays out, because it is an interpreter, and rg, because three of its routes
run a program and `grep` already reads.

**The rule.** A guard's log of refusals is evidence about the guard. When the
same harmless command is refused again and again, read the log before the
next workaround.

**Guard.** `the_trunk_allowlist_and_the_ownership_check_are_switched_on_here`
holds each read allowed and each writing spelling refused. Watched failing on
the unedited policy; nine mutations each red.

---

## 054 — The Stop gate said no gate had run without looking for one

**What happened.** `gate-stop.sh` refused a Stop with "uncommitted Rust … with
no gate run behind them" whenever a `.rs` file was uncommitted. It never read
whether a gate had run, so in one session it said so four times, each after
the three gates had just passed on that tree.

**Why it was invisible.** The refusal is stamped once per position, so it
never looped; each stop cost one restatement and looked like diligence.

**Cure.** Change 096 removed that half and put the gates on the paths out
instead: `scripts/package-macos.sh` now runs the three over the tree before it
builds the application, which is the one path the Stop refusal had stood on —
a packaged tree can hold work no commit gated. A commit touching Rust already
ran them. The contract, approvals, branch audit and installed-build report
stay, and the position still counts the code, so further work re-arms a
pending contract item.

**The rule.** A refusal must be about something it read. When a check cannot
observe what its message asserts, make it read that or delete it — and delete
it when another gate already stands on the path.

**Guard.** `the_stop_gate_reports_an_installed_build_that_is_behind` asserts a
tree holding only an uncommitted `.rs` file stops clean, and that an edit after
a refusal refuses again; `the_packager_runs_the_three_gates_before_it_builds`
holds the packager's order. Each watched failing on the unedited script.

---

## 055 — The suite's tmux sessions lived on the person's tmux server

**What happened.** Every tmux test ran on the default server, or the one `$TMUX`
names — the server a person's Operon sessions live on. Five `operon-test-*`
sessions had leaked there, two from that day's packaging run, and the server
itself had been started by a test on 2026-09-17: with none running, the test
made the default one, and 186 real sessions then lived on it. A test killed
mid-run left its session for the recovery screen to offer.

**Why it was invisible.** A test that cleans up leaves nothing, and a leak reads
as one more session. Lesson 048 is the same shape with git: a fixture reaching
the environment of the session that runs it.

**Cure.** Change 098 gives `tmux_command` a `-L operon_suite` socket in the test
build, which wins over `$TMUX`, and routes the three calls that bypassed it — a
production session stop, a screen capture, and seven test sites — through it.

**The rule.** A test drives an external server only through the one builder
that isolates it, and proves the isolation on a session it started, not only by
counting spellings in the source.

**Guard.** `a_session_a_test_starts_is_not_on_the_persons_tmux_server` starts a
session through the production path and asks a bare `tmux` whether it exists;
`every_tmux_the_crate_runs_goes_through_tmux_command` counts spawns with
whitespace removed, so a call rustfmt broke across lines is still seen. Both
watched failing on the unedited tree.

---

## 056 — A test pause in the bundle swap outlived its test by nine days

**What happened.** `scripts/replace-macos-bundle.sh` stops at a test's request
after a swap or inside its rollback, and waited with no end for the test to
signal or release it. A test that died first — a failed assert between spawn
and signal is enough, since dropping a `Child` does not kill it — left the
script looping. Two had run nine days; the one in the rollback ignores TERM by
design and took a KILL.

**Why it was invisible.** A passing suite never reaches the abandoned branch,
and the leftover is a `bash` among many. Lesson 055 is the same shape: a test's
fixture outliving the test.

**Cure.** Change 099 gives every pause a deadline (`OPERON_TEST_PAUSE_SECONDS`,
60 by default). An unsignalled swap pause exits non-zero, so the normal
rollback restores the previous generation; an unreleased rollback pause lets
the rollback finish.

**The rule.** A test hook that waits for its test ends on its own, in a state
the production path would accept.

**Guard.** `a_bundle_test_pause_nobody_releases_ends_and_rolls_back` sets both
pauses, never signals, and requires the script to exit with the previous
generation restored. Watched failing on the unedited script.

---

## 057 — Every eval that edits a file failed, and the steering was blamed

**What happened.** The first measured eval run (change 103) scored eval 004
0/2. Both transcripts said the agent knew what to do and was refused: the eval
worktree was detached, the worktree-policy guard cannot name a detached
branch, and it fails closed — so every edit, and every command off the trunk
allowlist, was denied. Any eval that edits a file could only fail.

**Why it was invisible.** An eval run is a pass or a fail, and a refusal by the
harness reads as a fail by the agent. The guard was tightened for the trunk,
correctly, long after the runner was written, and nothing ran the suite in
between. Lesson 055 is the same shape from the other side: an environment the
test shares with something it did not write.

**Cure.** Change 104 puts each eval worktree on a throwaway `eval/` branch,
deleted afterwards. Change 103's always-fails warning and per-run transcripts
are what found it.

**The rule.** An eval environment runs the repository's own hooks; when an
eval fails every run, read the transcript for a refusal before reading it for
a mistake.

**Guard.** `the_eval_worktree_is_on_a_branch_the_worktree_policy_lets_the_agent_edit`
reads the policy's protected branches and requires the branch the agent sees
to be named and not among them. Watched failing on `--detach`, on a protected
name, on branches left behind, and on a policy widened to every branch.

---

## 058 — A request had two entry points, and the second wrote before the worktree existed

**What happened.** A set of development skills was brought in to take a
request from "add this" to an installed build. Its first skill called itself
the single entry point for a development request; so did `sdlc`. `CLAUDE.md`
named one and `AGENTS.md` the other. The new skill kept its own record of a
change's position beside `state.yaml`, its own approval rules beside
`approval.md`, and wrote its spec and context files before step 7 created the
worktree. Followed literally, all five of those writes were refused on `main`
by the worktree-policy guard. It also put the steering and reference bands at
`propose`.

**Why it was invisible.** Each skill read well alone. Nothing compared one
skill's claim with another's, or the order of steps in a skill with the guard
that refuses them; and a second record of state fails silently, because the
hooks read only the first.

**Cure.** Change 112 cuts the set down to one thin entry, `feature-pilot`, that
owns only the order. It opens the worktree first, runs stages 1–2 through
`sdlc`, and records every fact in the place the pipeline already keeps it.

**The rule.** One skill claims the entry; every skill that writes a change
directory opens the worktree first; a fact has one home.

**Guard.** `one_skill_is_the_entry_point_and_the_root_documents_name_it`,
`a_skill_that_opens_a_change_makes_the_worktree_first`, and
`every_skill_is_named_in_a_flat_catalog`. Watched failing under six mutations:
a second skill claiming the entry, `AGENTS.md` calling another skill the entry,
`CLAUDE.md` no longer naming it, another skill creating a change directory with
no `make wt.new` before it, the worktree step removed from the entry skill, and
a skill dropped from the catalog.

## 059 — An `origin` arrived, and the worktree tools started measuring against it

**What happened.** Landing here is a local `git merge --no-ff`. On 2026-10-03
an `origin` was added and `main` pushed by hand; from then on `origin/main`
trailed `main` by every landing not yet pushed. `cleanup-worktree` refused
change 112's worktree after it landed — "6 commit(s) not provably on origin",
all six on `main` — and `git branch -d` refused the branch for the same reason,
because `make wt.new` had set its upstream to `origin/main`. Worse, `make
wt.new` called a `main` that was ahead "up to date" and created the next
worktree from `origin/main`, without the landings.

**Why it was invisible.** Change 071 had fixed the same tools for a repository
with no remote, and its test built none. Both tools chose `origin/<base>`
whenever a remote existed, and a fixture without one never reaches that branch.
`AGENTS.md` still said there was no remote.

**Cure.** Change 113: `cleanup-worktree` also accepts a tip that is on local
`main` from a branch other than `main` (a worktree on `main` itself would be
deleted with it, its unpushed commits along), and `make wt.new` bases a worktree on whichever of `main` and
`origin/main` contains the other.

**The rule.** Where work lands is where the tools measure. A fixture for a
landing tool includes the remote state the repository can actually be in.

**Guard.** `cleanup_removes_a_worktree_landed_on_main_while_origin_trails` and
`a_new_worktree_starts_from_main_when_origin_trails_it`, both red on the
unfixed scripts. Watched failing under two over-permissive mutations: every
tip counted as landed, a worktree on `main` itself counted as landed, and local
`main` taken as the base even when it trails.

## 060 — A catalog was built for a constraint nobody had measured

**What happened.** Change 112 added a flat skill catalog,
`development-skills.md`, and a test requiring every skill to be
named in one, because Antigravity CLI was said to scan only flat
`.agents/skills/*.md` files. Asked on 2026-10-04 which skills it had,
`agy --print` named the directory skills and the `SKILL.md` each was loaded
from, and said the catalog was not listed; `codex exec` agreed. The catalog was
read by no CLI, and what the CLIs do need was checked by nothing.

**Why it was invisible.** The premise arrived written down as "unverified", and
a written caveat read as a measured constraint. A test then made the file
mandatory, which looked like the premise being enforced.

**Cure.** Change 114 deletes the catalog and replaces the test with what the
CLIs load: `.agents` linked to `.claude`, a `SKILL.md` in every skill
directory, its `name:` equal to the directory, and a description of 1 to 1024
characters.

**The rule.** Before building for a CLI's behaviour, ask the CLI. A premise
marked unverified is a measurement to take, not a constraint to satisfy.

**Guard.** `every_skill_loads_under_codex_and_antigravity`. Watched failing
under four mutations: a `name:` unlike its directory, an empty description, a
1025-character description, and a skill directory with no `SKILL.md`.

---

## 061 — An undo that undid somebody else's work, and said it had restored

**What happened.** Change 110's worktree landing ran `git merge --abort` after
any failed merge. When the main checkout was already mid-merge with a
resolution equal to `HEAD`, the status was clean, the refusals passed, git
refused the second merge, and the abort discarded the person's resolution. The
notice then said the operation had been undone — which nothing had checked.
Review of change 115 found a second shape of the same claim: a merge killed
after writing the tree but before writing `MERGE_HEAD` left `HEAD` where it was,
and "unchanged" was reported over a dirty checkout.

**Why it was invisible.** The only test was the happy path plus two refusals.
An undo is code that runs when something already went wrong, and a sentence
saying it worked reads as true until somebody reproduces the failure.

**Cure.** Change 115 refuses a checkout that already has `MERGE_HEAD`, sends
every failed merge (exit, timeout, spawn error) through one path, and claims
"unchanged" or "restored" only when `HEAD`, `MERGE_HEAD`, and the status all
verify.

**The rule.** An undo only reverses what it started, and its message states the
state it verified, not the step it ran. Each failure the undo handles gets a
fixture that produces it.

**Guard.** `refuses_to_land_over_a_merge_already_in_progress`,
`a_landing_that_cannot_be_undone_says_so`, and
`a_landing_that_left_the_tree_changed_is_not_called_unchanged`. Watched failing
under change 115's mutations 1, 2, and 10.

---

## 062 — Missing tmux socket connection error was treated as Unknown instead of Gone

**What happened.** When a tmux server process terminated, exited, or was never started on macOS, running any tmux client command (`list-panes`, `resize-window`, `kill-session`) failed with `error connecting to /private/tmp/tmux-<UID>/default (No such file or directory)` or `(Connection refused)`. `tmux_error_state` checked only for `"can't find session"`, `"can't find window"`, `"no server running"`, and `"no such session"`. Because it did not match `"error connecting to"`, `"failed to connect to"`, or `"connection refused"`, it returned `TmuxState::Unknown`. An active session was never reconciled as `Lost`, remaining stuck as `Active`. Every render and window resize triggered `tmux_resize`, which repeatedly failed with the user-visible banner `ターミナルのサイズを変更できませんでした: error connecting to /private/tmp/tmux-<UID>/default (No such file or directory)`. Deleting the session failed because `stop_session` and `close_completed_terminal` treated `Unknown` as an incomplete stop, and `SessionStopped` did not transition `session.status` before calling `remove_session_record`, which refused removal on an `Active` session.

**Why it was invisible.** Tests had verified `tmux_error_state` with `"no server running on /tmp/tmux-1/default"` (the Linux string), but on macOS when the socket does not exist tmux prints `error connecting to ... (No such file or directory)`.

**Cure.** Change 123 matches `"error connecting to"`, `"failed to connect to"`, and `"connection refused"` in `tmux_error_state` to classify them as `TmuxState::Gone`. `TerminalResized` suppresses the error banner when the tmux server is gone and marks the session as `Lost`. `SessionStopped` and `TerminalClosed` allow session record removal when tmux is confirmed `Gone`.

**The rule.** A socket connection refusal or missing socket file is the definition of a gone daemon server; error classifiers must recognize the platform's socket connect error message, not just the server's own banner.

**Guard.** `recognizes_gone_tmux_server_errors`, `terminal_resize_failure_from_missing_tmux_socket_transitions_session_to_lost_without_error_banner`, `deleting_session_with_missing_tmux_socket_removes_session_record`, and `closing_completed_terminal_with_missing_tmux_socket_removes_session_record`. Watched failing on the unedited tree.

