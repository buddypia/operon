# Spec: one ＋ per project header, and nothing else, starts a session

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. In the session column (`ui_terminal_session_tabs`), a project with no
   sessions and none pinned above draws its header line only: no
   `icon_text_button` 「セッションを開始」 under it, and not
   「セッションはまだありません」. The header's existing `small_icon_button`
   (`ICON_ADD`, hover 「セッションを開始」) opens the launch sheet with that
   project selected. Test: `an_empty_project_in_the_session_column_starts_a_session`.
2. In the history's 「プロジェクト別」 view (`ui_project_session_list`), every
   project's header ends in one `small_icon_button` with `ICON_ADD` and hover
   text 「セッションを開始」, in place of the `ICON_SHOW` glyph whose hover read
   「プロジェクトを開く」 and which already called `open_project_session_setup`.
   An empty project with no search draws nothing under its path line. Under a
   search with no match in the project, 「一致するセッションはありません。」 stays.
   Test: `an_empty_project_in_the_history_list_starts_a_session`.
3. In the history's 「グリッド」 view (`ui_session_grid`), every project's header
   gains an `icon_button` with `ICON_ADD` and hover 「セッションを開始」, at the
   right end of the line; the `ICON_SHOW` 「プロジェクトを開く」 beside the path
   keeps opening the project page. An empty project with no search draws nothing
   under its header. A search with no match keeps 「一致するセッションはありません。」.
   Test: `an_empty_project_in_the_history_grid_starts_a_session`.

In each test, the ＋ is found on the empty project's own header row (the painted
`ICON_ADD` level with its name), clicked, and the sheet must open with that
project selected; and no painted text ends in 「セッションを開始」 anywhere, which
is what the framed button painted.

## Behaviour

Japanese strings a person reads:

- 「セッションを開始」 — now only the hover text of each header ＋ (it already
  was, in the session column). The EN and KO rows from change 158 stay in use.
- 「一致するセッションはありません。」 — unchanged.
- 「プロジェクトを開く」 — no longer in 「プロジェクト別」; still the grid's
  project-page glyph.

States:

- **Empty project, no filter**: header only; the header's ＋ is the action.
- **Project with sessions**: the same ＋ in the same place, then its sessions.
- **Status chips filtering a project empty** (session column): skipped, as today.
- **Emptied by pinning**: header only, as today.
- **Click while tmux or the agent is missing**: the sheet opens and its footer
  says why launching is disabled, as from every other entry.

Failure reaches the person nowhere new: the click only opens the sheet.

## Design

Placement verdict: **EXTEND**. The three branches that change in
`src/app/screens.rs` are the ones change 158 wrote
(`git grep -n 'tr("セッションを開始")' src/app/screens.rs` finds them and the
header glyph). Route `modify`.

- `src/app/screens.rs`: the session column's empty branch drops its
  `icon_text_button`; `ui_project_session_list` swaps `ICON_SHOW` /
  「プロジェクトを開く」 for `ICON_ADD` / 「セッションを開始」 in the header and drops
  the empty-branch button; `ui_session_grid` adds a right-to-left `icon_button`
  (`ICON_ADD`) to its header and drops the empty-branch button.
- `src/tests.rs`: the three tests from change 158 change their click target
  from the framed button to the header ＋ on the empty project's row, and assert
  no framed button is painted. The pinned-project test is unchanged.
- No persisted-shape change, no external command, no background task. The draw
  path loses one button per empty project and the grid gains one per project.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry | No | No colour named. |
| Icons — a new mark means an `ICON_*` constant and an `ICON_VOCABULARY` entry | No | `ICON_ADD`, already the add-session mark in the session column header. |
| Controls — `DESIGN.md` *Controls* names the helper per button kind | Yes | `small_icon_button` in the dense list header (as the session column), `icon_button` in the grid header (as its `ICON_SHOW`). |
| Identifier SSOT | No | No identifier; labels are message ids through `tr`. |
| Durability | No | Nothing persisted changes. |
| Subprocess safety | No | No child spawned. |
| Documentation — user docs in three languages together | No | `grep -n "セッションを開始" README*` finds nothing; no table row added or removed. |
| Local-first | No | None added. |
| Budgets | No | No scan or output path. |

## Flagged concerns

- **Discoverability of a header glyph.** Change 158 kept the framed button
  because the header ＋ alone was "recognisable only on hover". The person has
  now seen both and called the worded button redundant, proposing a ＋. Answered:
  one ＋, always in the same place, in all three views; the empty project's
  header line, ending in 「0 件」 and ＋, is the whole group.

## Acceptance

- `cargo test --locked an_empty_project_in_the` — 3 passed;
  `cargo test --locked a_project_emptied_by_pinning` — 1 passed;
  `cargo test --locked every_` — all pass.
- `make q.fast` passes.
- In the packaged app, an empty project shows only its header with ＋; the ＋
  opens the launch sheet for it, in the column and both history views.

## Rejected alternatives

- An icon-only ＋ under the empty project's header — still two ＋ for one action
  in the session column.
- Hiding the header ＋ on an empty project and showing it in the body — the
  control would move between states.
- Keeping 「プロジェクトを開く」 beside a new ＋ in 「プロジェクト別」 — two header
  controls calling the same function.
