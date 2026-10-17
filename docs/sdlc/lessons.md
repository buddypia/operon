# Lessons

Stage 6 of the pipeline in `docs/sdlc/README.md`. One entry per mistake that
reached the tree, newest last.

The part that matters is **Guard**. An entry with no guard is unfinished work:
every guard is a test in `src/tests.rs` or an eval in `evals/`, because those
run without being remembered.

Two forms live here. Recent entries (069–070) carry the full account; entries
001 to 068 are compressed to the rule and the guard (changes 030, 097, and 147).
**No guard was removed by the compression**: `scripts/pipeline-indicators.sh`
with `--lessons` still reports every entry naming a guard this repository holds.

---

## 001 — One identifier, five literals, one gate

`MANAGED_TMUX_PREFIX` was written inline at five call sites, so the product
rename moved the gate and none of the five. **The rule:** build the value
through the constant, and test the round trip.

**Guard.** `the_session_prefix_is_written_in_exactly_one_place` and
`evals/003-identifier-ssot.md`.

---

## 002 — A document that outlived the file it described

`CLAUDE.md` described a single `src/main.rs` after the crate split into modules.
**The rule:** check the module map in both directions against `src/main.rs`.

**Guard.** `claude_md_maps_every_module_that_exists`,
`harness_documents_only_name_paths_that_exist`, and `evals/004-module-map.md`.

---

## 003 — A doc comment naming a script that was never committed

`src/i18n_tables.rs` told readers to run a script that was never committed.
**The rule:** check source module `//!` headers for dead paths like documents.

**Guard.** `harness_documents_only_name_paths_that_exist`, extended to source
module docs.

---

## 004 — A guard blind to the rename it existed to survive

The guard for lesson 001 searched for hardcoded prefix strings rather than the
constant. **The rule:** a guard reads the value it guards out of the definition
rather than restating it.

**Guard.** `the_session_prefix_is_written_in_exactly_one_place` and
`evals/003-identifier-ssot.md`.

---

## 005 — A widget written for the bug, and never called

`restore_progress_footer` sat behind `#[allow(dead_code)]` while the restore
modal never called it. **The rule:** a widget test proves a widget; only the
caller proves the screen.

**Guard.** `the_restore_modal_draws_its_footer_through_the_shared_widget` and
`the_restore_footer_never_paints_status_under_its_action`.

---

## 006 — A translation table whose order was a comment

`translation_for` uses `binary_search_by`, and nothing checked table sort order,
completeness, or `{name}` placeholders. **The rule:** assert every property a
lookup depends on across the union of all tables.

**Guard.** `every_translation_table_is_sorted_by_message_id`,
`every_message_id_has_a_row_in_every_table`,
`every_translation_keeps_the_placeholder_names_of_its_message_id`, and
`an_unknown_message_id_falls_back_to_the_japanese_it_was_written_in`.

---

## 007 — Two ways to say no, and the readers only had one

Readers in `src/history.rs` returned `Option`, conflating excluded and unknown
record types and silently dropping `thinking`, `reasoning`, and tool results.
**The rule:** separate parsing, classification (`TRANSCRIPT_VOCABULARY` in
`src/transcript.rs`), and projection; carry unknown kinds across labelled and
counted.

**Guard.** `restore_counts_every_source_record_into_exactly_one_class`,
`an_unclassified_record_is_restored_and_reported`,
`every_operational_wrapper_is_stripped_by_the_reader_that_declares_it`,
`reasoning_reaches_every_destination_as_attributed_text`,
`the_transcript_vocabulary_is_well_formed`, and
`scripts/check-transcript-vocabulary.sh`.

---

## 008 — A restore reproduced the conversation the terminal had left

Operon kept the launch `--session-id` after `/clear` rotated Claude Code to a
new file. **The rule:** read Claude Code's per-process registry via
`registered_claude_conversation_in` rather than inferring from timestamps.

**Guard.** `a_cleared_claude_terminal_restores_the_conversation_it_moved_to`,
`a_resumed_claude_terminal_still_follows_its_registry`,
`a_registry_confirmation_is_not_overruled_by_the_start_time_check`,
`a_registry_entry_belonging_to_another_terminal_is_not_read_as_this_one`,
`two_claude_terminals_started_together_are_told_apart_or_refused`, and
`the_claude_store_directories_are_spelled_in_exactly_one_place`.

---

## 009 — A test for a ceiling that never reached the ceiling

A test for `DIFF_WORD_TOKEN_LIMIT` built a line whose common prefix/suffix trim
collapsed it to one token before the ceiling check ran. **The rule:** an input
must make the branch reachable, and the test must pin both paths apart.

**Guard.** `a_line_past_the_word_comparison_ceiling_still_marks_its_middle`.

---

## 010 — Steering that had nowhere conditional to live

Domain policies sat in `CLAUDE.md` and taxed every turn. **The rule:** move
file-scoped policies to `.claude/rules/` with `paths:` frontmatter, keep
`conditional_steering_bytes` separate, and enforce every rule with a test in
`src/tests.rs`.

**Guard.** `every_rule_declares_the_paths_it_applies_to`,
`harness_documents_only_name_paths_that_exist`, `docs/sdlc/bands.yaml`, and
`evals/006-new-file-past-the-path-rule.md`.

---

## 011 — A gate whose test passed with the gate switched off

Deleting the flagged-concern check in `scripts/check-readiness.sh` left its test
passing because angle brackets in the fixture tripped the placeholder check
first. **The rule:** assert on the specific rejection path and report text, not
merely a non-zero exit.

**Guard.** `the_readiness_gate_refuses_a_spec_that_is_not_filled_in`,
`every_state_file_names_a_route_and_a_stage_that_exist`,
`every_risk_surface_names_paths_that_exist`, and
`every_reference_the_skill_names_exists`.

---

## 012 — An orchestration mechanism with nothing wired to enforce it

Change 008 added SDLC tables and scripts that no hook invoked. **The rule:**
verify that every gate script is wired into an active hook or explicitly
declares why not, ignoring comments.

**Guard.** `every_gate_script_is_wired_or_declares_why_not`,
`every_change_directory_carries_its_state`, `.claude/hooks/guard-stage.sh`, and
`every_hook_the_settings_file_wires_exists_and_is_executable`.

---

## 013 — Every piece passed, and nothing was ever posted

The status hook endpoint file wrote its socket path unquoted, breaking on macOS
`Application Support` paths containing a space while temp-dir tests passed.
**The rule:** test shell boundaries with paths containing spaces.

**Guard.** `an_installed_hook_command_posts_a_reading_end_to_end` and
`the_endpoint_file_sources_into_the_variable_the_script_posts_with`.

---

## 014 — A guard that named the file instead of the thing

A brand-artwork guard hardcoded `src/app.rs` and broke when drawing moved.
**The rule:** guard the target functions wherever they live rather than a
filename or tree-wide count.

**Guard.** `the_brand_is_drawn_as_artwork_and_not_from_the_glyph_vocabulary`.

---

## 015 — The same lesson, a third time: a shell script read is not a shell script run

The usage reader wrote its timestamp without a newline, causing `read -r` to
return non-zero at EOF and reset the throttle every run. **The rule:** test
generated shell scripts by running them across multiple invocations.

**Guard.** `the_installed_usage_reader_posts_a_reading_end_to_end`.

---

## 016 — A flag over the frame, answering a question about one event

A single boolean dropped every `Key::Enter` in a frame containing an
`Ime::Commit`, eating the second Enter when two arrived in one frame.
**The rule:** derive per-event decisions from the event stream in order,
counting commits rather than summarizing the frame with `.any()`.

**Guard.** `a_second_enter_in_the_frame_that_committed_a_conversion_still_sends`.

---

## 017 — A gate that went vacuous when its inputs were deleted

Removing a dead launch editor left `launch_needs_acknowledgement` reading empty
fields and ignoring dangerous switches in custom commands. **The rule:** when
deleting a producer, update its consumers and test the rendered screen.

**Guard.** `a_dangerous_switch_typed_into_a_custom_command_still_asks` and
`a_dangerous_switch_cannot_be_launched_without_the_acknowledgement`.

---

## 018 — A match that read as a miss, reconstructed from the only copy left

Under `set -uo pipefail`, `printf … | grep -q` closes the pipe on first match
and SIGPIPEs the writer when input exceeds the pipe buffer. **The rule:** under
`pipefail`, never end a pipeline with an early-closing reader (`grep -q`,
`head`); capture whole first.

**Guard.** `every_lesson_cited_by_number_is_a_lesson_that_exists` and
`every_gate_script_reads_its_pipes_to_the_end`.

---

## 019 — The review gate ran from a file the repository did not have

`scripts/check-review.sh`, `scripts/pipeline-indicators.sh`, and `.githooks/`
sat untracked in one checkout while worktrees failed without them. **The rule:**
verify what a fresh clone gets; if a `reference-transaction` hook breaks on
`main`, remove `$(git rev-parse --git-path hooks)/reference-transaction` by hand.

**Guard.** `harness_documents_only_name_paths_that_exist`,
`the_gitignore_exemption_covers_runtime_state_and_nothing_tracked`,
`every_lesson_cited_by_number_is_a_lesson_that_exists`, and
`every_gate_script_is_wired_or_declares_why_not`.

---

## 020 — A guard that reported on the machine it ran on

`scripts/check-readiness.sh` and `.claude/hooks/gate-stop.sh` used bash 4+
syntax under `#!/usr/bin/env bash`, failing on stock macOS `/bin/bash` 3.2 while
passing under Homebrew bash. **The rule:** test shell scripts under the oldest
interpreter their shebang can resolve to (`/bin/bash` and `/bin/sh`).

**Guard.** `the_readiness_gate_refuses_a_spec_that_is_not_filled_in`,
`every_shell_script_parses_under_the_oldest_shell_its_shebang_finds`,
`a_shebang_carrying_flags_still_names_its_interpreter`, and
`every_known_unparsed_script_is_a_script_the_sweep_still_reaches`.

---

## 021 — Pressing a key without reading the screen it lands on

`auto_approve_workspace_trust_prompt` sent a blind `Enter` (confirming `No, exit`
on Claude Code) and read scrollback. **The rule:** read the visible pane only,
separate trust recognition from option navigation, and test over real captured
panes.

**Guard.** `the_claude_trust_prompt_is_answered_by_moving_to_its_affirmative_option`,
`a_trust_prompt_already_resting_on_yes_is_only_confirmed`,
`a_menu_that_is_not_a_trust_prompt_is_never_answered`,
`every_trust_marker_names_trust`,
`a_trust_prompt_caught_before_its_options_are_drawn_is_left_alone`,
`the_escape_stripper_leaves_the_text_a_terminal_would_show`,
`auto_approval_sends_enter_only_after_a_workspace_trust_prompt`,
`the_trust_prompt_script_reads_the_same_vocabulary_the_app_uses`,
`the_trust_prompt_is_read_off_the_visible_screen_and_not_the_scrollback`,
`no_trust_marker_is_also_an_option`,
`a_screen_offering_more_options_than_a_trust_prompt_has_is_left_alone`, and
`a_trust_question_that_wrapped_is_still_a_trust_question`.

---

## 022 — The working directory was not what decided which repository git touched

`GIT_DIR` and `GIT_WORK_TREE` override `current_dir`, causing `mktemp` git tests
to commit onto the real repository. **The rule:** strip
`INHERITED_REPOSITORY_POINTERS` at the choke point in `src/exec.rs` and
`git_command`.

**Guard.** `no_git_command_in_the_crate_can_inherit_the_session_repository`.

---

## 023 — The gate answered a question next to the one it was asked

`.claude/hooks/gate-commit.sh` ran gates in `CLAUDE_PROJECT_DIR` rather than the
worktree being committed. **The rule:** resolve the target tree from `cwd` and
git pointers the same way `git commit` does, and vary each input in tests.

**Guard.** `the_commit_gate_runs_the_gates_in_the_tree_the_commit_lands_in`.

---

## 024 — The fixture was the format as it had been

Codex 0.155.0 required `ordinal` on rollout records while hand-typed test
fixtures lacked it. **The rule:** derive format conventions from real tool
output and verify what the target product accepts.

**Guard.** `codex_restore_numbers_replay_records_the_way_the_rollout_numbers_its_own`,
`codex_restore_leaves_an_unnumbered_rollout_unnumbered`, and
`a_rollout_whose_last_record_lost_its_number_fails_verification`.

---

## 025 — The band fired fifteen times into a change that was waiting for a person

`scripts/check-bands.sh` prescribed opening an `intent.md` without reporting
whether one already existed in progress or `awaiting-user`. **The rule:** report
the status of any change declaring `**Answers band**: <metric>` alongside the
breach.

**Guard.** `the_control_bands_say_whether_a_propose_breach_has_an_answer`.

---

## 026 — Three rounds closing one reported spelling at a time

`apply_codex_config` misread `[features]` headers in `~/.codex/config.toml` and
wrote duplicate tables. **The rule:** pair the strict reader with a generous
veto reader so any disagreement declines the write rather than corrupting the
file.

**Guard.** `codex_hook_install_writes_a_config_codex_can_load` and
`the_config_check_and_the_writer_read_one_file_the_same_way`.

---

## 027 — A quote character ended the program it was written inside

Mirroring TOML multiline detection into awk in
`scripts/check-codex-config-shape.sh` broke shell quoting and regex escaping.
**The rule:** verify dual-syntax mirrors by parsing under the stock shell and
running over shared fixtures.

**Guard.** `every_shell_script_parses_under_the_oldest_shell_its_shebang_finds`
and `the_config_check_and_the_writer_read_one_file_the_same_way`.

---

## 028 — The sentence that said the drift was fixed was written before it was

`apply_codex_config`'s `first_table` scan was left unconverted when
`is_table_header` was introduced. **The rule:** test every scan site on fixtures
where old and new readers diverge.

**Guard.** `the_config_check_and_the_writer_read_one_file_the_same_way`.

---

## 029 — `git add -A` staged the file that redirects git

`git add -A` swept untracked worktree config files containing `GIT_WORK_TREE`
into a commit. **The rule:** stage by explicit path and guard `.gitignore`
exemptions.

**Guard.** `the_gitignore_exemption_covers_runtime_state_and_nothing_tracked`.

---

## 030 — Which way a scan's boundary should err

A dot inside `profiles."gpt-5.6"` caused `first_table` to delete a user's
`hooks` key while still producing valid TOML (`lesson 032` later moved the
boundary test to `a_config_toml_would_refuse_is_still_not_a_config_to_delete_from`).
**The rule:** choose scan boundaries by failure cost and assert surviving values
directly, not just TOML parseability.

**Guard.** `codex_hook_install_writes_a_config_codex_can_load`,
`the_config_check_and_the_writer_read_one_file_the_same_way`, and
`a_config_toml_would_refuse_is_still_not_a_config_to_delete_from`.

---

## 031 — A line does not mean anything on its own

Line-by-line matching mistook array rows and multiline strings in `config.toml`
for `[features]` headers. **The rule:** use one stateful walk
(`structural_lines`) to find table boundaries while keeping vetoes generous.

**Guard.** `codex_hook_install_writes_a_config_codex_can_load` and
`the_config_check_and_the_writer_read_one_file_the_same_way`.

---

## 032 — A fix can turn a red mutation green, and nothing says so

Teaching the reader multiline values made six boundary mutations pass silently.
**The rule:** re-run the full mutation sweep after every fix and fail on
unmatched substitutions.

**Guard.** `a_config_toml_would_refuse_is_still_not_a_config_to_delete_from`.

---

## 033 — Two branches a fixture never ran in the same call

`apply_codex_config` preserved non-table `hooks` shapes and then appended
`[hooks.state."…"]` blocks that broke them whenever entries were non-empty.
**The rule:** test preservation and append branches together in one call.

**Guard.** `codex_hook_install_writes_a_config_codex_can_load`.

---

## 034 — The veto knew the root of the path, not the path

The append veto checked `hooks` but missed `hooks.state = "x"` and
`[features]` dotted keys. **The rule:** verify every proper prefix of the
written TOML path is an extendable table across all readers.

**Guard.** `codex_hook_install_writes_a_config_codex_can_load` and
`the_config_check_and_the_writer_read_one_file_the_same_way`.

---

## 035 — The general reader lost a case the narrow one handled

`table_header_shape` split on `]` inside quoted trust keys, duplicating blocks on
second launch. **The rule:** test idempotence by running the writer over its own
output.

**Guard.** `codex_hook_install_writes_a_config_codex_can_load`.

---

## 036 — The same character, closed on one edit path out of four

Refusing `]` inside quoted headers at only one step let the top-level scan delete
keys past `[projects."/Users/me/[work]"]`. **The rule:** share the predicate
across all scan and veto paths.

**Guard.** `a_config_toml_would_refuse_is_still_not_a_config_to_delete_from` and
`codex_hook_install_writes_a_config_codex_can_load`.

---

## 037 — The doc comment counted its readers, and the count was the defect

The block-removal loop in `apply_codex_config` ran before the unreadable-header
refusal, deleting existing blocks without re-appending them. **The rule:** gate
removal and replacement together and test multi-launch healing.

**Guard.** `codex_hook_install_writes_a_config_codex_can_load`.

---

## 038 — Taking the nit destroyed the approval it came with

Editing code to satisfy a nit changed the diff digest in
`scripts/check-review.sh` and voided approvals across nineteen rounds.
**The rule:** carry nits in `review.yaml` (`carried:`) and only move the digest
for Important findings.

**Guard.** `a_review_past_its_round_ceiling_lands_by_carrying_its_nits`.

---

## 039 — The table was widened, and the reader of it was on another branch

Adding a sixth column to `docs/sdlc/routes.yaml` broke commits because
`.githooks/reference-transaction` runs `refs/heads/main:scripts/check-review.sh`,
which expects five columns. **The rule:** never widen a table format that
`main`'s committed hook parses.

**Guard.** `every_route_row_is_the_width_the_gate_that_judges_commits_reads` and
`every_route_has_a_review_round_ceiling`.

---

## 040 — The fix was applied to one of the two files that had the defect

Change 057 fixed tree resolution in `gate-commit.sh` while `gate-stop.sh` kept
reading `CLAUDE_PROJECT_DIR`. **The rule:** sweep all sibling hooks for the same
defect class.

**Guard.** `the_stop_gate_reads_the_tree_the_session_is_working_in`.

---

## 041 — The counter was incremented before the decision it was counting

Transcript readers incremented `count_unclassified_kind` before checking whether
a record produced a turn, reporting dropped records as carried. **The rule:**
decide and count in one branch, and mutate the claim a notice makes (`evals/007-mutate-the-claim-not-the-counter.md`).

**Guard.** `a_kind_that_crossed_and_a_kind_that_was_dropped_are_reported_as_different_things`,
`the_notices_printed_counts_account_for_every_record_read`,
`a_codex_token_count_record_is_excluded_rather_than_reported_as_unknown`,
`an_unknown_role_is_reported_as_carried_because_its_turn_crosses`,
`the_notice_promises_no_marker_the_restored_text_does_not_carry`,
`an_unknown_flag_or_role_is_counted_against_what_the_record_did`, and
`evals/007-mutate-the-claim-not-the-counter.md`.

---

## 042 — Four failures nothing could see coming, and the shape they shared

Counter-only mutations missed false prose claims, `gate-stop.sh` did not report
when reading another checkout, silent reviewers were respawned instead of
re-messaged, and stale `/Applications/Operon.app` builds were discovered only at
stop. **The rule:** surface silent state mismatches early in the documents and
gates that read them.

**Guard.** `the_review_policy_names_every_policy_the_repo_enforces`,
`the_pipeline_documents_ask_their_questions_before_review_does`,
`the_stop_gate_says_when_it_is_reading_another_checkout`,
`the_stop_gate_reports_an_installed_build_that_is_behind`,
`the_installed_build_check_reads_its_stamp_key_out_of_the_packager`, and
`evals/007-mutate-the-claim-not-the-counter.md`.

---

## 043 — The hardening was lossy along the axis nothing compares

`docs/sdlc/templates/mutations.py` restored files via `mkstemp` + `os.replace`,
resetting file permissions from `755` to `600`. **The rule:** preserve file mode
when atomically replacing files in mutation sweeps.

**Guard.** `the_mutation_sweep_template_restores_the_mode_it_found`.

---

## 044 — A predicate that stopped reading, and a gate that read the wrong answer

Under `set -o pipefail`, `git reflog show --all | grep -q` exited early and
caused SIGPIPE (141) to read a match as a miss. **The rule:** in `pipefail`
scripts, never pipe into `grep -q`; redirect `>/dev/null` or capture first.

**Guard.** `every_gate_script_reads_its_pipes_to_the_end` and
`the_review_gate_refuses_what_it_says_it_refuses`.

---

## 045 — A gate written in a shell the machine it ships to does not have

A one-line `case … esac` inside `$()` in `.claude/hooks/gate-stop.sh` failed to
parse under stock macOS `/bin/bash` 3.2. **The rule:** verify all bash gate
scripts with `/bin/bash -n`.

**Guard.** `every_gate_script_parses_under_the_stock_shell`.

---

## 046 — Codex CLI rollout replay projection drift and session ID parsing

Modern Codex CLI requires `task_started`, `item_completed`, and `task_complete`
with non-null `rollout_ordinal`, and `rsplit_once('-')` truncated 36-char UUIDs.
**The rule:** match the target CLI's SQLite projection constraints and extract
full UUIDs via `codex_session_id`.

**Guard.** `codex_rollout_metadata_recovers_full_uuid_from_filename`,
`codex_replay_records_default_to_ordinals_and_monotonic_time`,
`codex_replay_records_preserve_and_continue_paginated_ordinals`, and
`a_saved_antigravity_conversation_restores_into_a_resumable_codex_terminal`.

---

## 047 — A merged worktree nobody was told to remove

Merged worktrees lingered because `AGENTS.md` omitted cleanup and
`cleanup-worktree` failed when no remote existed or `target/` was slow to
delete. **The rule:** end the landing procedure with `cleanup-worktree` and
support local-base measurement.

**Guard.** `cleanup_removes_a_merged_worktree_in_a_repository_with_no_remote`,
`the_stop_guard_names_a_merged_worktree_the_session_still_owns`, and
`the_landing_procedure_ends_by_removing_the_worktree`.

---

## 048 — A test fixture that commits to whichever repository the session names

`ReviewGateFixture::command` spawned `git` via a variable without stripping
`GIT_DIR`/`GIT_WORK_TREE`, committing fixture data into the real repository.
**The rule:** strip `INHERITED_REPOSITORY_POINTERS` on every git spawn in test
fixtures and hooks.

**Guard.** `the_review_gate_fixture_cannot_reach_the_session_repository`.

---

## 049 — The worktree tools asked git about a directory, and git answered about the session

Node worktree scripts inherited `GIT_DIR` and `GIT_WORK_TREE`, misidentifying
worktree branches and failing once the pointed-to worktree was removed.
**The rule:** scrub `INHERITED_REPOSITORY_POINTERS` in `.cli/lib/utils.mjs` and
worktree scripts.

**Guard.** `cleanup_removes_the_worktree_it_was_given_when_the_session_names_another`
and `a_git_dir_that_no_longer_exists_does_not_stop_the_worktree_tools`.

---

## 050 — The Stop gate handed every session every open change

`.claude/hooks/gate-stop.sh` reported pending `machine` items from parked
changes unrelated to the current branch, and porcelain parsing missed quoted or
renamed paths (sdlc 083). **The rule:** scope stop-gate checks to changes
touched since `git merge-base refs/heads/main HEAD` using `-z` path lists.

**Guard.** `the_stop_gate_holds_a_session_only_to_the_changes_it_is_working_on`.

---

## 051 — A step that undoes itself was handed to a person to type

Sessions asked the user to run `git branch -d`, `git merge --abort`, or approve
rebase re-verdicts when `main` moved. **The rule:** ask a person only about
unrecoverable decisions, and fix guards that refuse harmless recovery steps.

**Guard.** `the_trunk_allowlist_and_the_ownership_check_are_switched_on_here`
and `a_session_asks_a_person_only_about_what_cannot_be_undone`.

---

## 052 — A finding left unfixed was reported in words only the session understood

Carried review notes reached the user as English reviewer jargon (`nit`, `carry`,
`digest`). **The rule:** require a plain Japanese `person:` line on every
carried finding in `review.yaml` and report from it.

**Guard.** `a_finding_left_unfixed_reaches_a_person_in_their_words` and
`a_report_to_a_person_is_written_in_their_words`.

---

## 053 — The trunk refused the reads it had promised to admit

The trunk allowlist omitted read-only commands (`sed -n`, `jq`, `sort`,
`git for-each-ref`), causing 188 refusals. **The rule:** admit read-only
inspection commands on trunk while fencing their writing flags.

**Guard.** `the_trunk_allowlist_and_the_ownership_check_are_switched_on_here`.

---

## 054 — The Stop gate said no gate had run without looking for one

`gate-stop.sh` complained about uncommitted `.rs` files even right after the
three gates passed. **The rule:** enforce the three gates in
`scripts/package-macos.sh` and `gate-commit.sh` rather than guessing at stop.

**Guard.** `the_stop_gate_reports_an_installed_build_that_is_behind` and
`the_packager_runs_the_three_gates_before_it_builds`.

---

## 055 — The suite's tmux sessions lived on the person's tmux server

Tests ran on the user's default tmux server and leaked `operon-test-*` sessions.
**The rule:** route all crate and test tmux spawns through `tmux_command` with
an isolated `-L operon_suite` socket under `#[cfg(test)]`.

**Guard.** `a_session_a_test_starts_is_not_on_the_persons_tmux_server` and
`every_tmux_the_crate_runs_goes_through_tmux_command`.

---

## 056 — A test pause in the bundle swap outlived its test by nine days

`scripts/replace-macos-bundle.sh` waited indefinitely on test pause hooks when a
test panicked mid-run. **The rule:** bound test pauses with
`OPERON_TEST_PAUSE_SECONDS` so abandoned pauses roll back and exit cleanly.

**Guard.** `a_bundle_test_pause_nobody_releases_ends_and_rolls_back`.

---

## 057 — Every eval that edits a file failed, and the steering was blamed

Detached eval worktrees were blocked by the worktree-policy guard. **The rule:**
put eval worktrees on a named `eval/` branch allowed by the policy.

**Guard.** `the_eval_worktree_is_on_a_branch_the_worktree_policy_lets_the_agent_edit`.

---

## 058 — A request had two entry points, and the second wrote before the worktree existed

Two skills claimed to be the entry point and wrote artifacts before creating a
worktree. **The rule:** keep a single entry point (`feature-pilot`) that creates
the worktree first and stores facts only in the canonical SDLC files.

**Guard.** `one_skill_is_the_entry_point_and_the_root_documents_name_it`,
`a_skill_that_opens_a_change_makes_the_worktree_first`, and
`every_skill_loads_under_codex_and_antigravity`.

---

## 059 — An `origin` arrived, and the worktree tools started measuring against it

When local `main` was ahead of `origin/main`, `cleanup-worktree` refused landed
branches and `make wt.new` branched from stale `origin/main`. **The rule:**
measure landed state and worktree base against whichever of `main` and
`origin/main` is ahead.

**Guard.** `cleanup_removes_a_worktree_landed_on_main_while_origin_trails` and
`a_new_worktree_starts_from_main_when_origin_trails_it`.

---

## 060 — A catalog was built for a constraint nobody had measured

A flat skill catalog was added on an unverified assumption that Antigravity and
Codex ignore `.agents/skills/<id>/SKILL.md`. **The rule:** measure CLI behavior
directly and guard the `.agents -> .claude` symlink and `SKILL.md` frontmatter.

**Guard.** `every_skill_loads_under_codex_and_antigravity`.

---

## 061 — An undo that undid somebody else's work, and said it had restored

Failed worktree landings ran `git merge --abort` unconditionally and claimed the
checkout was unchanged without checking `MERGE_HEAD` or tree status. **The rule:**
refuse to land over an existing `MERGE_HEAD` and verify `HEAD`, `MERGE_HEAD`,
and status before claiming restoration.

**Guard.** `refuses_to_land_over_a_merge_already_in_progress`,
`a_landing_that_cannot_be_undone_says_so`, and
`a_landing_that_left_the_tree_changed_is_not_called_unchanged`.

---

## 062 — Missing tmux socket connection error was treated as Unknown instead of Gone

macOS tmux socket errors (`error connecting to ...`, `connection refused`)
returned `TmuxState::Unknown`, blocking session deletion and spamming resize
banners. **The rule:** classify socket connection refusals and missing sockets
as `TmuxState::Gone`.

**Guard.** `recognizes_gone_tmux_server_errors`,
`terminal_resize_failure_from_missing_tmux_socket_transitions_session_to_lost_without_error_banner`,
`deleting_session_with_missing_tmux_socket_removes_session_record`, and
`closing_completed_terminal_with_missing_tmux_socket_removes_session_record`.

---

## 063 — Two sessions took the same change number

Concurrent sessions picked the same next number under `docs/sdlc/changes/`.
**The rule:** change numbers must be unique; the second lander renumbers before
merging.

**Guard.** `change_directory_numbers_are_unique`.

---

## 064 — Hook tests judged by elapsed time failed on a loaded machine

Hook tests asserted wall-clock speed and tight timeouts that failed under
parallel load. **The rule:** assert the marker event rather than elapsed time,
and size node hook test timeouts for shared-machine contention.

**Guard.** `the_stop_gate_reports_an_installed_build_that_is_behind` and
`the_trunk_allowlist_and_the_ownership_check_are_switched_on_here`.

---

## 065 — One guard advised the command another guard refuses

`.cli/hooks/commit-guard.mjs` advised `git -C .worktrees/<task> commit`, which
`.claude/hooks/gate-commit.sh` refuses. **The rule:** advice printed by a guard
must be accepted by every other guard on that path.

**Guard.** `the_commit_guard_never_advises_a_commit_the_commit_gate_refuses`.

---

## 066 — A hook read stdin when it was only imported

Importing `.cli/hooks/worktree-policy-guard.mjs` ran its top-level `main()` and
hung waiting on stdin. **The rule:** guard every hook module's entrypoint with
`isDirectInvocation(import.meta.url)`.

**Guard.** `importing_a_hook_never_reads_its_stdin`.

---

## 067 — The trunk guard judged another repository as this trunk

Commands `cd`ing into an external repository were judged as trunk commands of
Operon. **The rule:** exempt command segments whose resolved working directory
lies inside another repository with no path reaching back into Operon's trunk.

**Guard.** `work_inside_another_repository_is_not_judged_as_this_trunk`.

---

## 068 — state.yaml stage is checked against the closed pipeline stage set, not human review terminology

Setting `stage: "review"` in `state.yaml` failed CI because the closed stage set
is `plan design build test deploy maintain`. **The rule:** `stage` in
`state.yaml` must match the route's declared stages in `docs/sdlc/routes.yaml`.

**Guard.** `every_state_file_names_a_route_and_a_stage_that_exist`.

---

## 069 — A raw egui button ignored the app's control metrics

**What happened.** The "パスを手入力" toggle, the sidebar's session-removal prompt, the find bar arrows, and diff note actions were drawn with egui's `small_button` / `Button::small()`. The person saw a button with its label pressed against the frame beside buttons with padding. Thirty-two text sizes sat between the levels of `DESIGN.md`'s type scale.

**Why it was invisible.** `apply_interface_metrics` sets the padding and the 28px height on the context, so every button *looked* governed; egui's small variant opts out of both, and no rule or test said which constructor a button may use or which sizes text may take. Colour and icons had a rule and a test; geometry and type had prose only.

**Cure.** Change 137 moved every site onto the helpers in `src/ui/widgets.rs` and the scale, and added the helper table to `DESIGN.md` *Controls* and the rule to `.claude/rules/palette-and-glyphs.md`.

**The rule.** A button is drawn by its helper in `src/ui/widgets.rs`; a size is a level of `DESIGN.md`'s `typography` block.

**Guard.** `every_button_keeps_a_documented_height` and `every_text_size_is_a_level_on_the_type_scale`, watched failing on main's tree (13 and 32 sites) and on a reintroduced `small_button` and `.size(13.0)`.

## 070 — The app promised a move the screen did not make

**What happened.** Change 144 put grips on the side panel's tabs that could only move the panel within its column; the person had to point out that a grip which cannot put a panel where a grip promises is meaningless. Change 145 made every panel dock on any edge, and announced 「…を下部に移動しました」 for a move onto an edge a small window gives up for room — the panel, or another one it pushed off, vanished with its own grip. 145's review saw it and carried it to the person as "no action needed".

**Why it was invisible.** 144's intent stated the cue (an icon, a cursor) as the outcome, so nothing asked what the grip could actually do. In 145 whether a panel is drawn was decided after the move, by a function the move never consulted, and every dock test ran in a window large enough for all four edges. `REVIEW.md`'s Important list had no line for a message the screen does not bear out, so the finding was graded a nit.

**Cure.** Change 146: drawing and the move check read one `DockLayout::rects`; every move passes `dock_commit`, which refuses one that would take a panel off the screen and says why; the drop zones and menus show the refusal before it is tried. `REVIEW.md` grades the class Important, and the intent template says a cue is not an outcome.

**The rule.** An action that tells the person it happened is checked against what is painted afterwards, at the sizes the person uses, not against the state it changed.

**Guard.** `test_dock_move_keeps_every_shown_panel_on_screen` (every panel × edge × three window sizes, judged by painted grips) and `test_dock_refused_move_says_why`, watched failing on 145's `dock_panel_to` and on the gate replaced by `true`; `test_dock_drag_by_pointer_events`, watched failing with the Escape branch deleted and with the drag never armed.

---

## 071 — Unhandled SGR subparameters leaked into foreground colour codes and dropped half-block backgrounds

`apply_terminal_sgr` only parsed foreground SGR codes and stepped one token at a
time over unknown codes, so `48;5;97` (256-colour background) dropped the lower
half of `▀`/`▄` banner cells and read `97` as top-level `SGR 97` (bright white
foreground). **The rule:** parse background (`40..=47`, `48`, `49`, `100..=107`)
and inverse (`7`, `27`) SGR attributes into `TextFormat::background`, and
advance past multi-token extended colour sequences (`38`, `48`, `58` and `:`
subparameters).

**Guard.** `renders_terminal_background_colors_and_half_block_banners` and
`bundled_terminal_fonts_keep_cjk_at_exactly_two_ascii_cells`.


---

## 072 — A redraw removed for speed was the only thing reading the results

`spawn_background` sent each finished job's result down a channel and never
woke egui, so the result waited for whatever repaint was already scheduled —
100ms while any job ran. A focused terminal pane used to repaint every frame,
which drained the channel at frame rate and hid it; change 153 removed that
repaint, and every keystroke's echo then crossed two 100ms waits. The pane also
read tmux once per keystroke, straight after `send-keys`, before the CLI had
redrawn, so the last character typed could wait for the two-second poll.
**The rule:** a background result wakes the frame that reads it; a per-frame
repaint is never what delivers work, and removing one is checked for what it
was carrying.

**Guard.** `a_finished_background_job_wakes_the_frame_that_reads_it` (watched
failing on main: no wake within 5s) and
`typing_in_the_pane_keeps_it_reading_tmux_until_the_echo_lands` (watched failing
on main, and with the keystroke timestamp removed from `queue_terminal_input`).

---

## 073 — A review was voided by work it never touched

Change 158 was reviewed, its CI green, and its landing refused: two other
sessions landed on main meanwhile, 158 took main in as the pipeline asks, and
the review digest — a hash of the diff byte for byte — moved because the hunk
line numbers and the whole-file blob ids on each `index` line describe the rest
of the file, which the other changes had edited. No commit could bring the
record up to date, since one holding only the record is refused. Change 135 had
paid the same cost as an extra review round. On the way to the fix, the next
change number, 161, turned out to be held by another session's branch; four
numbers on main already name two changes each.
**The rule:** what binds a verdict describes the change and nothing else
around it; and anything parallel sessions pick from a shared sequence is
checked against where in-flight work lives — branches, not main.

**Guard.** `a_review_survives_main_moving_the_lines_above_it` (watched failing
on main's gate: the digest moved when main added lines above the edit; its
invalid-byte arm watched failing with `LC_ALL=C` removed),
`a_review_recorded_before_change_162_still_counts`, the exact-line pins in
`the_review_gate_has_no_bypass`, and
`a_new_change_cannot_take_a_number_already_in_use` (watched failing before
`scripts/check-change-number.sh` existed; on this repository it refuses
`fix/161-…` and names 163).

## 074 — A tmux server outlived the worktree it was started in

A tmux server keeps the working directory of whichever command started it, and
on tmux 3.8 a server whose directory is gone starts every pane there, whatever
`-c` says. `tmux_command` set no directory, so a run from a worktree left a
server behind in it, and cleanup deleted the worktree. Change 160 met it as the
suite's own server failing every live restore test, and recorded it as a stale
server to be stopped rather than a fault to be fixed; change 164 met it again
on the suite's server, and on the person's own: every agent Operon launched
began in a missing directory. **The rule:** a symptom waived twice is a cause
not yet named — and a process that outlives the command that started it must
not inherit that command's directory.

**Guard.** `a_tmux_server_never_holds_a_directory_that_can_be_deleted` and
`an_agent_starts_in_its_directory_even_where_tmux_cannot_put_it` (both watched
failing on main; a suite server started from a directory then deleted failed
`codex_app_server_restores_a_short_transcript_into_a_resumable_terminal`
before the fix and passed after).

---

## 075 — The translation check compared the tables with each other, never with the code

`every_message_id_has_a_row_in_every_table` asserted that the English and Korean
tables hold the same ids, and `tr` falls back to the Japanese id for an id no
table knows. So a message added with a row in neither table passed every check
and showed in Japanese on an English or Korean screen: 598 such messages had
built up when change 165 looked, plus 23 launch-option labels that reach `tr`
through a table field. **The rule:** the catalogue is checked against the
messages the code passes to `tr`/`tf!` — literal ids read from the source, and
ids held in a table read from that table — not only against itself; a `tr` call
whose argument is neither is refused until its table is checked too.

**Guard.** `every_message_the_code_shows_has_a_row_in_every_table` (watched
failing before the rows were added: 611 file/message pairs, then 23 launch
options; and with `kind.label()` dropped from its list of covered arguments).
