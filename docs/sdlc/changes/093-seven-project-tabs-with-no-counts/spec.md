# Spec: six project tabs with counts

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `ProjectTab` loses `Skills` and `Rules` and gains `AgentSettings`.
   `ProjectTab::all()` returns six, in this order: `Overview`, `Git`,
   `Worktrees`, `PullRequests`, `Files`, `AgentSettings`. The variant names
   `Git` and `Files` stay (every caller that opens a file sets
   `ProjectTab::Files`); only their labels change.
2. `ProjectTab::label` returns 「概要」, 「変更」, 「worktree」, 「PR」, 「ファイル」,
   「エージェント設定」 (through `tr` where the text is Japanese).
3. A new `OperonApp::project_tab_count(&self, tab, project_id) -> Option<usize>`
   reads only in-memory caches:
   - `Git`: `files.len()` of an `Ok` entry in `git_changes_cache`;
   - `Worktrees`: `len()` of an `Ok` entry in `worktree_cache`;
   - `PullRequests`: `len()` of an `Ok` entry in `pull_request_cache`;
   - every other tab, a missing entry, or an `Err`: `None`.
   It starts no background request.
4. `src/ui/widgets.rs` gains `tab_item_with_count`, taking `ui`, `palette`,
   `label`, `count: Option<usize>`, `emphasise: bool`, `selected: bool`, and
   returning the `Response`. With `None` it draws exactly what `tab_item` draws. With `Some(n)` the button's text is
   the label, a space, and `n`, the number in `palette.accent` when
   `emphasise && n > 0` and in the label's ink otherwise. The selected rule is drawn as
   in `tab_item`. `tab_item` stays and calls it with `None`.
5. The project page's tab row calls `tab_item_with_count` for every tab with
   `project_tab_count(tab, project.id)`, and `emphasise` true only for `Git`
   (so 変更 N is in the accent colour when N > 0; req 4 never emphasises
   an `n` of 0).
6. `ProjectTab::AgentSettings` draws `ui_agent_settings(ui, project)`:
   today's `ui_skills`, `SPACE_LG` of space, today's `ui_rules`, unchanged.
7. The session panel's 「worktree の変更はプロジェクトの Git タブで確認できます。」
   becomes 「worktree の変更はプロジェクトの「変更」タブで確認できます。」.
8. The old Git-tab sentence's EN and KO rows are removed — the only
   rows this change orphans (「エディタ」, 「スキル」, 「ルール」, 「概要」 have
   no rows today and fall back to Japanese). New EN and KO rows:
   「エージェント設定」 and the new sentence. 「変更」 and 「ファイル」 have
   rows already.
9. Nothing persisted changes. No new background request, child process, or
   file-system call.

## Behaviour

See `./screen.md`.

## Design

- `src/app.rs`: `ProjectTab`, its `all` and `label`; `project_tab_count`.
- `src/app/screens.rs`: the tab row; `ui_agent_settings`.
- `src/ui/widgets.rs`: `tab_item_with_count`.
- `src/ui/session_tree.rs`: the one sentence.
- `src/i18n_tables.rs`, `src/tests.rs`.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | yes | No new role: `accent`, `text_muted`, `text_strong` exist, and the accent rule already sits on this row. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | No icon added. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | no | No new identifier. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | `project_tab` is in-memory. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | The tab row starts nothing; PR is still requested only by its own tab. |
| Documentation — user-facing docs change in all three languages together | yes | README does not name the tabs; nothing to change. Checked with a search for スキル/ルール/エディタ in all three. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | no | UI only. |
| Budgets — any new scan or output path states its byte and item ceiling | no | No new scan. |
| Draw path — no I/O per frame | yes | Counts read three `HashMap`s; a test reads the function's source for request and process calls. |

## Flagged concerns

- **A count that never appears** — the tab row requests nothing, so 変更 and
  worktree show their counts only after 概要 (the default tab) or their own
  tab has loaded them. Resolved: 概要 is where a project page opens, and it
  requests both; PR is left to its tab on purpose (network).
- **Tests that name the old tabs** — none select `Skills` or `Rules`; the
  suite's `ProjectTab::Files` / `Worktrees` assertions are unaffected.

## Acceptance

- `cargo test --locked` passes, including new tests:
  `the_project_page_has_six_tabs` (`ProjectTab::all()` labels are exactly
  概要 / 変更 / worktree / PR / ファイル / エージェント設定),
  `tab_counts_come_from_the_caches` (with project A's caches seeded — 3
  changed files, 2 worktrees, 1 PR — A's counts are 3 / 2 / 1; project B,
  with nothing seeded, gets `None` for each; with an `Err`, `None`; 概要,
  ファイル, エージェント設定 `None`),
  `only_a_nonzero_change_count_is_emphasised` (`tab_item_with_count` drawn
  headless, its galley's section colours read: `Some(3)` with emphasis —
  the digit is `palette.accent`; `Some(0)` with emphasis — not accent;
  `None` — the same text as `tab_item`),
  `the_old_tab_sentence_is_gone` (no table row and no source string carries
  「プロジェクトの Git タブ」),
  `the_tab_row_draws_its_counts` (the project page drawn headless with those
  caches shows 「変更 3」's number, 「worktree 2」, 「PR 1」, and no 「スキル」
  tab), `agent_settings_shows_skills_and_rules` (drawing the tab draws both
  headings 「スキル」 and 「ルールと指示」), and
  `tab_counts_do_no_work` (the source of `project_tab_count` contains no
  `request_`, `spawn_background`, `Command::new`, or `fs::`).
- `cargo clippy --locked -- -D warnings` and `cargo fmt --check` are clean.
- In the installed app the tab row matches `./screen.md`.

## Rejected alternatives

- Requesting the PR list from the tab row so its count is always there —
  `gh pr list` reaches the network, and opening a project should not.
- A pill-shaped badge — a third kind of marker on a row that already has the
  accent rule; the mock draws a plain number.
