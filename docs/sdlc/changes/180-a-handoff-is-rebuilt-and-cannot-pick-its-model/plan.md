# Plan: Handoff destinations take a model and effort, and an unchanged conversation's bundle is reused until it expires

- **Spec**: `./spec.md`
- **Approved**: 2026-10-20
- **Status**: approved

This is the plan produced in plan mode and accepted before any file was edited.
If the implementation departs from it, update this file — an abandoned plan is
worse than no plan, because the next reader trusts it.

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | TTL, scan ceiling, manifest byte ceiling; the three archive identifiers |
| `src/agents/settings.rs` | `HandoffLaunchSettings`, `RecentAgentSettings::handoff`, sanitiser, `handoff_launch_summary`, `handoff_menu_label` |
| `src/cli.rs` | `ImportedNativeSession::reused_bundle_created_at`; snapshot file name through its constant |
| `src/history.rs` | launch options into the import; archive lock; sweep; reuse lookup; manifest v6; constants for the literals |
| `src/app.rs` | sweep key/result and once-at-startup; options read and validated in `handoff_cli_session`; reuse notice |
| `src/app/screens.rs` | model/effort row in the history card; menu label |
| `src/i18n_tables.rs` | EN and KO rows for the new strings |
| `src/tests.rs` | the eleven tests the spec names; struct literals gain the new field; call sites gain the launch argument |
| `README.md`, `README.ja.md`, `README.ko.md` | handoff options, reuse, expiry, budgets |
| `docs/sdlc/changes/180-…/` | state, approvals, review record |

## Order of work

1. **Contracts first.** Tests 1 (settings round trip and sanitiser), 6 (sweep
   leaves everything but expired compact archives), 4/5/7 (reuse, moved-on,
   expired) and the gone-snapshot fallback against the import, and 2 (command built from options). Watch
   each fail (missing symbols count as failing to compile only until stubs
   exist; each is then watched failing on its assertion).
2. `src/config.rs` constants; `src/agents/settings.rs` type, field, sanitiser,
   label helpers. Tree compiles; test 1 and test 10 pass.
3. `src/cli.rs` field and constant; `src/history.rs` import signature, lock,
   sweep, lookup, manifest v6. Update every existing call site and struct
   literal in `src/tests.rs`. Tests 2, 4, 5, 6, 7 and `a_reuse_whose_snapshot_is_gone_takes_a_fresh_one`
   pass; the existing handoff
   tests and `the_transcript_archive_is_readable_only_by_its_owner` stay green.
4. `src/app.rs`: background key/result, `ensure_handoff_archive_sweep_started`
   in `update`, options in `handoff_cli_session`, notice choice in
   `finish_full_history_import`. Tests 3, 8, 9 pass.
5. `src/app/screens.rs` card row and menu label; `src/i18n_tables.rs` rows.
   The i18n completeness tests pass.
6. READMEs in three languages.
7. Tidy; `make q.fast`; the named tests; the live `--ignored` tests if `codex`,
   `claude`, `agy`, and `tmux` are on `PATH`; run the app from the worktree build
   and compare the card, the menu, and the notice with `screen.md`.
8. Mutation sweep from `docs/sdlc/templates/mutations.py`: delete the
   fingerprint comparison, drop the restore-kind check in the sweep, remove the
   symlink check, drop the snapshot-readable check, invert the TTL comparison, make the reuse notice claim reuse
   on a fresh build, and drop the model from the command — each must turn a
   named test red.
9. Review (`scripts/check-review.sh` names the reviewers), close, commit, push,
   CI, merge, package.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The sweep deletes something that is not an expired compact archive | A full-history archive, a symlinked directory's target, or a fresh bundle disappears | Test 6 covers each kind; mutation of each guard in step 8 |
| A reuse races a sweep and launches with a deleted snapshot | The destination's bundle names a missing path | One process-wide lock over sweep, lookup, and rewrite (test 7 runs the sweep inside the import) |
| A manifest-only archive is chosen for reuse | The handoff fails, or the bundle names a snapshot that is not there | The lookup requires a regular snapshot file and the import falls back to a fresh snapshot when it reads no turns; `a_reuse_whose_snapshot_is_gone_takes_a_fresh_one` and its mutation in step 8 |
| A reused bundle is stale | The destination misses turns the source added | Fingerprint comparison; test 5 appends a turn and expects a fresh snapshot |
| A model string reaches tmux unvalidated | A shell metacharacter in the launch line | `build_agent_launch_command` runs `is_safe_agent_option`; test 2 and test 3 feed `;` |
| Per-frame cost in the card | Allocation or I/O in `ui_cli_sessions` | The options map is inserted once per target; the sweep is a background task behind a bool; `rust-reviewer` |
| An older build drops the handoff options on save | Options gone after a downgrade | Accepted in the spec; nothing else in the file is lost |
| Strings missing from a table | i18n completeness test red | Existing table tests |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked <name>` for each of the eleven new tests and the existing
  handoff, archive-permission, and i18n tests — `ok`.
- `cargo test --locked` in CI on the pushed branch — `test result: ok`.
- The eleven tests fail before their code and pass after; step 8's mutations are
  each caught.
- In the app built from the worktree: the card shows the model row; the menu
  shows `Codex CLI へ Handoff（gpt-5-mini · low）`; a second handoff of the same
  conversation shows the reuse notice; the Codex terminal command carries
  `--model gpt-5-mini`.

## Departures from the plan

None yet.
