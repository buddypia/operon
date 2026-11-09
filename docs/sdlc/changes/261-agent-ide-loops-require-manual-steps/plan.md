# Plan: Hunk-level diff actions, agent context handoff, `@` mentions, external IDE line jump, parallel arena launch, and `⌘K` file/symbol jump

- **Spec**: `./spec.md`
- **Approved**: 2026-11-09
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/git.rs` | Add `extract_single_hunk_patch`, `git_stage_hunk`, and `git_discard_hunk`. |
| `src/files.rs` | Add `ContextMentionCandidate`, `active_at_mention_query`, `rank_context_mentions`, `apply_context_mention`, `CodeSymbol`, and `extract_file_symbols`. |
| `src/util.rs` | Add `ExternalIde` (`VsCode`, `Cursor`, `Zed`, `Windsurf`, `Antigravity`, `Void`, `Finder`), `external_ide_command_args`, `open_in_external_ide`, `format_code_context_for_agent`, `extract_recent_terminal_error`, `parallel_arena_agents`, and `arena_worktree_branch_name`. |
| `src/ui/diff.rs` | Add `DiffHunkAction` and `diff_pane_with_actions` drawing `ステージ` and `破棄` buttons on hunk header rows. |
| `src/app.rs` | Add `PaletteKind::File`, `parallel_arena_selection: [bool; 3]` on `OperonApp`, and keep `src/app.rs` line count under its band by placing UI helpers in `src/app/screens.rs`. |
| `src/app/screens.rs` | Wire hunk stage/discard in diff views, `エージェントに送る` in editor header, `直近のエラーをエージェントに送る` in terminal menu, `@` mention picker and parallel arena launch in `ui_launch_form`, `ExternalIde::ALL` menus, and `PaletteKind::File` search/activation in `palette_entries` / `activate_palette_entry`. |
| `src/i18n_tables.rs` | Add sorted English and Korean translations for all new Japanese UI strings. |
| `README.md`, `README.ja.md`, `README.ko.md` | Document hunk actions, editor/terminal agent handoff, `@` mentions, expanded external IDEs with line jump, parallel arena launch, and `⌘K` file/symbol jump in all three languages. |
| `src/tests.rs` | Add `ide_261_` tests covering all six requirements. |
| `docs/sdlc/changes/261-agent-ide-loops-require-manual-steps/` | SDLC state, intent, spec, screen, plan, approvals, verification, and mutation sweep. |

## Order of work

1. Evaluate `intent.md`, `spec.md`, `screen.md`, and `plan.md` with an independent evaluator subagent, record approvals in `approvals.log`, verify `bash scripts/check-readiness.sh` and `bash scripts/check-approvals.sh`, and commit the stage 1–2 artifacts before touching `src/`.
2. Write the six `ide_261_` requirement tests in `src/tests.rs` and watch them fail before implementing the feature logic.
3. Implement the pure and subprocess functions in `src/git.rs`, `src/files.rs`, `src/util.rs`, and `src/ui/diff.rs`.
4. Wire the UI controls and actions in `src/app.rs`, `src/app/screens.rs`, `src/i18n_tables.rs`, and `README*.md`.
5. Run `make q.fast`, `cargo test --locked ide_261_`, translation and harness tests, and `bash scripts/check-bands.sh`.
6. Create `mutations.py` and `verification.yaml`, verify all mutations are caught, close `state.yaml`, commit, push, wait for CI, merge on `main`, package, and clean up the worktree.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Single-hunk patch omits file headers or includes adjacent hunks | `git apply` fails or stages the wrong hunk | `ide_261_extract_and_apply_single_hunk_stage_and_discard` |
| Unsorted message IDs in `src/i18n_tables.rs` break binary search lookup | `every_translation_table_is_sorted_by_message_id` fails | `cargo test --locked every_translation_` |
| `src/app.rs` grows toward the 10,000-line propose band | `check-bands.sh` reports `largest_module_lines` propose breach | Put implementation methods in `src/app/screens.rs`, `src/util.rs`, `src/files.rs`, and `src/git.rs`; check with `bash scripts/check-bands.sh` |
| Hunk buttons in `diff_pane` shift line numbers or break existing diff comment clicks | Existing `diff_` tests or `ide_261_diff_pane_hunk_buttons_emit_stage_and_discard` fail | Buttons are drawn only on `DiffLineKind::Hunk` rows at `Align2::RIGHT_CENTER`, leaving `DiffLineKind::Added`/`Removed`/`Context` rows untouched |

## Proof of completion

- `make q.fast` passes with zero warnings.
- `cargo test --locked ide_261_` passes all 6 requirement test suites.
- `python3 docs/sdlc/changes/261-agent-ide-loops-require-manual-steps/mutations.py` catches every mutation.
- `bash scripts/check-review.sh` and `bash scripts/check-approvals.sh docs/sdlc/changes/261-agent-ide-loops-require-manual-steps` pass.

## Departures from the plan
