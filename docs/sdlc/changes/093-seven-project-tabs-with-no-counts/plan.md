# Plan: six project tabs with counts

- **Spec**: `./spec.md`
- **Approved**: 2026-09-27
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/app.rs` | `ProjectTab`: `Skills`, `Rules` → `AgentSettings`; `all()` → six; `label()`; `project_tab_count` |
| `src/app/screens.rs` | the tab row calls `tab_item_with_count`; the match arm for `AgentSettings`; `ui_agent_settings` |
| `src/ui/widgets.rs` | `tab_item_with_count`; `tab_item` delegates to it |
| `src/ui/session_tree.rs` | the Git-tab sentence |
| `src/i18n_tables.rs` | EN and KO rows for 「エージェント設定」 and the new sentence; the old sentence's rows removed |
| `src/tests.rs` | seven new tests |

## Order of work

1. `ProjectTab` and its callers; `ui_agent_settings`. Compiles.
2. `project_tab_count`, `tab_item_with_count`, the tab row.
3. The sentence; i18n rows.
4. Tests; gates; mutation check (one per new test).
5. rust-reviewer on the staged diff; commit; package; install; look at the
   screen.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| スキル or ルール becomes unreachable | no way to see instruction files | `agent_settings_shows_skills_and_rules` |
| The tab row does work per frame | stutter on the project page | `tab_counts_do_no_work`; reviewer |
| A count from the wrong project | 変更 N of another project | `project_tab_count` keys by `project.id`; `tab_counts_come_from_the_caches` seeds one project and asks another for `None` |
| An error read as zero | 「変更 0」 on a non-git folder | `Err` → `None`, tested |
| An orphaned message id | the old sentence's rows left behind | `the_old_tab_sentence_is_gone` |
| The accent number on a zero | 「変更 0」 highlighted | req 4: emphasis only when n > 0; `only_a_nonzero_change_count_is_emphasised` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- New tests watched failing with their rule inverted.
- Installed app: the tab row as in `./screen.md`.

## Departures from the plan

- `tab_item` is removed rather than kept as a wrapper (spec req 4): the
  project row was its only caller, so the wrapper was dead code and clippy
  refuses it. Its doc comment moved onto `tab_item_with_count`, and
  `only_a_nonzero_change_count_is_emphasised` checks the `None` case draws
  the bare label.
- `tab_counts_do_no_work` looks for `self.request_`, not `request_`:
  `pull_request_cache`, which the function reads, contains the shorter one.
