# Plan: a session screen that gives its width to the terminal

- **Spec**: `./spec.md`
- **Approved**: 2026-09-27
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | `NOTICE_TOAST_SECONDS`, `FILE_FILTER_RESULT_LIMIT`, `TOUCHED_FILES_LIMIT` |
| `src/app.rs` | `brief_notice` + `notice_briefly` + `brief_notice_expired`; expiry in `update`; toast drawing whose ✕ clears both records; banner skips the brief notice; six success sites use `notice_briefly`; inspector fields replace the two column toggles; `verb_is_loud`; `resolved_paths_generation` bumped at every insert, `retain`, and `clear` of `resolved_paths` |
| `src/agents.rs` | `session_list_title` |
| `src/app/screens.rs` | `ui_terminal_session_tabs` (filter row, project row, two-line session row); `ui_terminal_panel` (one-line header, verb danger only when not idle, panel reopen icon, 最新へ overlay, two-column layout); `ui_prompt_timeline` loses its frame header |
| `src/ui/session_tree.rs` | `SessionColumnsLayout` two columns; `ui_session_inspector` with the three tabs; files tab rearranged; changes tab reads `git_changes_cache` (requests a load when absent, draws `Err` and empty states); `split_hidden_entries`; pure `touched_files(&HashMap<String, Option<PathBuf>>, &Path, usize)` (only `Some`, under root, sorted, capped); its inline tests rewritten for the new layout |
| `src/i18n_tables.rs` | rows for every new string in every table |
| `src/tests.rs` | tests that set `show_session_file_tree` / `show_prompt_timeline` move to the inspector fields; the four new tests in the spec |
| `src/glyphs.rs` | only if a fold glyph is missing (with `ICON_VOCABULARY`) |

## Order of work

1. Config constants and `brief_notice` with its pure helper and test; switch
   the six success sites; toast drawing. Compiles on its own.
2. `session_list_title` and its test.
3. Sidebar: filter row, project row, session row.
4. Inspector state fields, `SessionColumnsLayout`, `split_hidden_entries`,
   `ui_session_inspector`; header rewrite; 最新へ overlay. The tree does not
   compile between the field rename and the test updates — done in one step.
5. i18n rows; run the three gates; package and install; look at the screen.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A menu action is lost from the session row or the header | an item missing from a `···` | both `···` closures (row and header: Terminal.app, log copy, 今すぐ同期) are moved, not rewritten; an existing-string render test asserts the header menu button still draws; checked on screen |
| A dangerous-resume or pending-stop verb loses its red | a carried dangerous flag looks harmless | `only_an_idle_stop_loses_its_red`, inverted once to watch it fail |
| Mentioned-files list goes stale | a pruned path still listed | generation counter at every mutation site; `touched_files` test covers `None` entries and ordering |
| Changes tab runs git in the draw path | stutter | it reads `git_changes_cache` only; a missing entry requests the existing background load |
| Toast clears a failure that replaced it | an error disappears after 4 s | `brief_notice_expires_only_for_its_own_text` |
| Per-frame allocation in the new lists | stutter with a large tree | split and mentioned-files are cached on rebuild; rust-reviewer |
| Two scroll areas sharing an id in the inspector | scroll jumps between tabs | each tab's `ScrollArea` gets its own `id_salt` |
| New string missing from a table | i18n test fails | the existing all-tables test |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- The new tests named in the spec pass — including
  `only_an_idle_stop_loses_its_red` and a `touched_files` test — and each fails
  when the rule it pins is inverted.
- Installed app: the session screen matches `./screen.md`.

## Departures from the plan

Carried from the round-2 evaluation, recorded here rather than moving the
approved digests:

- The `src/tests.rs` row means six new tests: the five the spec names plus the
  `touched_files` test.
- `resolved_paths` is also cleared in `src/app/screens.rs` (session switch);
  the generation counter is bumped there too.
- Requirement 9 applies to the session-screen header only. The project
  overview's session rows also draw `session_verb` and are left as they are.

Found while building:

- The hidden-entries row reads 「{caret} 隠しファイル・フォルダ {count} 件」: a
  top-level dot entry can be a file as well as a folder.
- The file filter's results are cached on `session_filter_results`, keyed by
  root and query, so typing in the field does not rescan the tree each frame.
- `verb_is_loud` takes the status and the cancellation flag as well as the
  activity: 停止中… and a dangerous resume keep their red, which the spec
  requires and the activity alone cannot tell apart.
- The Stop test drives `AgentActivity` directly rather than a live session.
- The gates were red on `main` before this change: a refused `..` path set no
  notice (df0cd72), failing two R1.4 tests. That is change 089, fixed and
  committed on this branch ahead of this one.
- Review round 1 found the filter caching an empty answer while the scan was
  still loading. The filter now waits for a scan, both scan-arrival handlers
  clear its cache, and a seventh test watches the loading case (seen red on
  the old code). The inspector's no-project fallback to 会話 is now decided
  per draw instead of overwriting the tab chosen for other sessions.
